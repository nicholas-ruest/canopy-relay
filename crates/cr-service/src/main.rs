use axum::{
    http::StatusCode,
    middleware::from_fn,
    routing::{get, post},
    Router,
};
use cr_api::ApiState;
use cr_application::CanopyRelay;
use cr_cedar_policy::{CedarAuthorizer, DEFAULT_POLICY};
use cr_opendal_store::ArtifactStore;
use cr_repository::Repository;
use cr_upstream_compose::RuntimeClients;
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};

#[tokio::main]
async fn main() {
    if let Some(address) = healthcheck_address() {
        if let Err(error) = check_readiness(&address) {
            eprintln!("canopy-relay readiness check failed: {error}");
            std::process::exit(1);
        }
        return;
    }
    if let Err(error) = run().await {
        eprintln!("canopy-relay failed: {error}");
        std::process::exit(1);
    }
}

fn healthcheck_address() -> Option<String> {
    let mut arguments = std::env::args().skip(1);
    match (
        arguments.next().as_deref(),
        arguments.next(),
        arguments.next(),
    ) {
        (Some("--healthcheck"), Some(address), None) => Some(address),
        _ => None,
    }
}

fn check_readiness(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect_timeout(&address.parse()?, Duration::from_secs(2))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    stream.write_all(b"GET /readyz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")?;
    let mut response = Vec::with_capacity(256);
    stream.take(4096).read_to_end(&mut response)?;
    let status = response
        .split(|byte| *byte == b'\n')
        .next()
        .ok_or("readiness response did not contain a status line")?;
    if !status.windows(3).any(|window| window == b"200") {
        return Err(format!(
            "readiness endpoint returned {:?}",
            String::from_utf8_lossy(status)
        )
        .into());
    }
    Ok(())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let data = std::env::var("CANOPY_RELAY_DATA").unwrap_or_else(|_| ".canopy-relay".into());
    let store = ArtifactStore::filesystem(PathBuf::from(data).as_path())?;
    let repository = Arc::new(Repository::restore("canopy-relay", store).await?);
    let upstreams = RuntimeClients {
        rucelium: cr_rucelium::Client::new(),
        worldgraph: cr_worldgraph::Client::new(),
        midstream: cr_midstream::Client::new(),
        latentmesh: cr_latentmesh::Client::new(),
        ruvfann: cr_ruvfann::Client::new(),
    };
    let relay = Arc::new(CanopyRelay::new(
        repository,
        CedarAuthorizer::parse(DEFAULT_POLICY)?,
        upstreams,
    ));
    let state = ApiState { relay };
    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(|| async { "ready" }))
        .route("/api/v1/dashboard", get(cr_api::dashboard))
        .route("/api/v1/observations", post(cr_api::create_observation))
        .route(
            "/api/v1/interventions",
            post(cr_api::create_intervention).get(cr_api::list_interventions),
        )
        .route("/api/v1/audit", get(cr_api::list_audit))
        .route("/api/v1/audit/{id}", get(cr_api::get_audit))
        .with_state(state)
        .layer(from_fn(cr_middleware::product_controls))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(30),
        ))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());
    let address: SocketAddr = std::env::var("CANOPY_RELAY_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "canopy relay ready");
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod healthcheck_tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;

    fn receive_probe(stream: &mut TcpStream) {
        let mut request = Vec::with_capacity(128);
        let mut buffer = [0_u8; 128];
        loop {
            let count = stream.read(&mut buffer).expect("read readiness request");
            assert!(count > 0, "client closed before completing the request");
            request.extend_from_slice(&buffer[..count]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
            assert!(request.len() < 4096, "readiness request exceeded limit");
        }
        assert!(request.starts_with(b"GET /readyz "));
    }

    #[test]
    fn readiness_probe_accepts_http_200() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test listener");
        let address = listener.local_addr().expect("local address");
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept probe");
            receive_probe(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nready")
                .expect("write response");
        });
        check_readiness(&address.to_string()).expect("probe should pass");
        server.join().expect("server thread");
    }

    #[test]
    fn readiness_probe_rejects_non_success_status() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test listener");
        let address = listener.local_addr().expect("local address");
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept probe");
            receive_probe(&mut stream);
            stream
                .write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n")
                .expect("write response");
        });
        assert!(check_readiness(&address.to_string()).is_err());
        server.join().expect("server thread");
    }
}
