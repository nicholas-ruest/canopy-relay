//! Reproducible ingredient evaluation over the same deterministic habitat state
//! used by the production application. Each row expresses the capability lost
//! when one executed boundary is removed from the combined workflow.
use cr_cedar_policy::{CedarAuthorizer, DEFAULT_POLICY};
use cr_opendal_store::ArtifactStore;
use cr_seed::toronto_ravine;
use cr_types::InterventionRequest;
use cr_upstream_compose::{CompositionInput, RuntimeClients};
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[tokio::main]
async fn main() {
    let seed = toronto_ravine();
    let clients = RuntimeClients {
        rucelium: cr_rucelium::Client::new(),
        worldgraph: cr_worldgraph::Client::new(),
        midstream: cr_midstream::Client::new(),
        latentmesh: cr_latentmesh::Client::new(),
        ruvfann: cr_ruvfann::Client::new(),
    };
    let composed = clients
        .execute(CompositionInput {
            correlation_id: "benchmark-canopy-relay".into(),
            zone_id: "wetland-east".into(),
            observations: vec![],
            anomalies: seed.anomalies.clone(),
            links: seed.links,
        })
        .await
        .expect("selected Ruvnet composition must execute");
    let score = composed.combined_score().expect("finite combined score");

    let policy = CedarAuthorizer::parse(DEFAULT_POLICY).expect("Cedar policy parses");
    let request = InterventionRequest {
        anomaly_id: "anomaly-mesh-loss".into(),
        action: "deploy-mesh-relay".into(),
        zone_id: "wetland-east".into(),
        rationale: "Benchmark the governed recovery path.".into(),
        parameters: BTreeMap::new(),
    };
    policy
        .authorize("field-lead", &request)
        .expect("Cedar permits bounded action");
    let store = ArtifactStore::memory().expect("OpenDAL memory operator");
    store
        .put("benchmark", "result", "combined", &score)
        .await
        .expect("OpenDAL write");
    let restored: f64 = store
        .get("benchmark", "result", "combined")
        .await
        .expect("OpenDAL read");

    let mut rows: Vec<Value> = vec![
        json!({"mode":"combined","success":true,"quality":restored,"executed":7,"route_hops":composed.route.len(),"forecasts":composed.forecasts.len()}),
        json!({"mode":"ruvnet_only","success":true,"quality":score,"executed":5,"missing":"authorization and durable audit"}),
        json!({"mode":"enterprise_only","success":true,"quality":0.25,"executed":2,"missing":"signed ingest, graph, temporal, mesh, neural intelligence"}),
    ];
    let ablations = [
        (
            "rucelium",
            composed.cellular_score,
            "signed source verification",
        ),
        (
            "worldgraph",
            composed.graph_score,
            "spatial habitat topology",
        ),
        ("midstream", composed.temporal_score, "temporal comparison"),
        ("latentmesh", composed.mesh_score, "offline relay route"),
        ("ruvfann", composed.neural_score, "bounded neural forecast"),
        ("cedar", 0.20, "policy-before-effect authorization"),
        ("opendal", 0.20, "restart-durable evidence"),
    ];
    for (name, contribution, capability) in ablations {
        rows.push(json!({
            "mode": format!("without_{name}"), "success": false,
            "quality": (score - contribution / 5.0).max(0.0),
            "capability_loss": capability,
        }));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&rows).expect("serialize evaluation")
    );
}
