//! Cross-crate acceptance tests for the application transaction boundary.
//!
//! These tests exercise the same application service used by HTTP handlers. They
//! intentionally call the real selected upstream crates through `RuntimeClients`
//! and the OpenDAL-backed repository rather than replacing first-party behavior
//! with mocks.

use chrono::{TimeZone, Utc};
use cr_application::CanopyRelay;
use cr_cedar_policy::{CedarAuthorizer, DEFAULT_POLICY};
use cr_errors::RelayError;
use cr_opendal_store::ArtifactStore;
use cr_repository::Repository;
use cr_types::{GeoPoint, InterventionRequest, ObservationInput};
use cr_upstream_compose::RuntimeClients;
use std::{collections::BTreeMap, sync::Arc};

fn runtime_clients() -> RuntimeClients {
    RuntimeClients {
        rucelium: cr_rucelium::Client::new(),
        worldgraph: cr_worldgraph::Client::new(),
        midstream: cr_midstream::Client::new(),
        latentmesh: cr_latentmesh::Client::new(),
        ruvfann: cr_ruvfann::Client::new(),
    }
}

fn observation() -> ObservationInput {
    ObservationInput {
        device_id: "soil-7".into(),
        zone_id: "wetland-east".into(),
        species: "cattail".into(),
        metric: "soil-moisture".into(),
        value: 0.64,
        unit: "ratio".into(),
        captured_at: Utc
            .with_ymd_and_hms(2026, 11, 7, 12, 0, 0)
            .single()
            .expect("fixed acceptance time"),
        location: GeoPoint {
            lat: 43.642,
            lon: -79.374,
        },
        calibration_version: "soil-v3".into(),
        nonce: "watershed-seed-0001".into(),
        signature: "synthetic-source-signature".into(),
    }
}

fn intervention(action: &str) -> InterventionRequest {
    InterventionRequest {
        anomaly_id: "anomaly-mesh-loss".into(),
        action: action.into(),
        zone_id: "wetland-east".into(),
        rationale: "restore observation continuity with a reversible relay".into(),
        parameters: BTreeMap::from([
            ("duration_minutes".into(), "45".into()),
            ("max_nodes".into(), "2".into()),
        ]),
    }
}

async fn application(namespace: &str, store: ArtifactStore) -> CanopyRelay {
    let repository = Arc::new(
        Repository::restore(namespace, store)
            .await
            .expect("restore test repository"),
    );
    CanopyRelay::new(
        repository,
        CedarAuthorizer::parse(DEFAULT_POLICY).expect("compile selected Cedar policy"),
        runtime_clients(),
    )
}

#[tokio::test]
async fn primary_workflow_executes_all_selected_boundaries_and_commits_once() {
    let relay = application("primary-acceptance", ArtifactStore::memory().unwrap()).await;
    let created = relay
        .intervene(
            "field-lead",
            intervention("deploy-mesh-relay"),
            "primary-canopy-relay",
        )
        .await
        .expect("permitted intervention must execute");

    assert_eq!(created.state, "deployed");
    assert_eq!(created.policy_decision, "allow");
    assert_eq!(created.correlation_id, "primary-canopy-relay");
    assert!(!created.route.is_empty(), "LatentMesh must return a route");
    assert!((0.0..=1.0).contains(&created.predicted_effect));
    assert_eq!(created.evidence_hash.len(), 64);
    assert_eq!(relay.audit_count(), 1, "one transaction, one durable audit");

    let statuses = relay.upstream_status();
    assert_eq!(statuses.len(), 7);
    for required in [
        "rucelium",
        "worldgraph",
        "midstream",
        "latentmesh",
        "ruv-fann",
        "cedar-policy/cedar",
        "apache/opendal",
    ] {
        assert!(
            statuses.iter().any(|status| status.name == required),
            "missing selected dependency status: {required}"
        );
    }
}

#[tokio::test]
async fn restricted_action_is_denied_before_upstream_or_persistence_effects() {
    let relay = application("denial-acceptance", ArtifactStore::memory().unwrap()).await;
    let audit_before = relay.audit_count();
    let interventions_before = relay.interventions().len();

    let error = relay
        .intervene(
            "field-lead",
            intervention("controlled-burn"),
            "denial-recovery-canopy-relay",
        )
        .await
        .expect_err("irreversible intervention must be denied");

    assert!(matches!(error, RelayError::Denied(_)));
    assert_eq!(relay.audit_count(), audit_before);
    assert_eq!(relay.interventions().len(), interventions_before);

    let recovered = relay
        .intervene(
            "field-lead",
            intervention("deploy-mesh-relay"),
            "denial-recovery-canopy-relay",
        )
        .await
        .expect("corrected reversible action must succeed");
    assert_eq!(recovered.state, "deployed");
    assert_eq!(relay.audit_count(), audit_before + 1);
}

#[tokio::test]
async fn wrong_actor_is_denied_by_cedar_without_forbidden_effects() {
    let relay = application("actor-denial", ArtifactStore::memory().unwrap()).await;
    let result = relay
        .intervene(
            "observer",
            intervention("deploy-mesh-relay"),
            "actor-denied-canopy-relay",
        )
        .await;

    assert!(matches!(result, Err(RelayError::Denied(_))));
    assert!(relay.interventions().is_empty());
    assert!(relay.audit_entries().is_empty());
}

#[tokio::test]
async fn durable_intervention_and_audit_restore_from_a_new_repository_instance() {
    let store = ArtifactStore::memory().expect("OpenDAL memory service");
    let relay = application("restart-acceptance", store.clone()).await;
    let original = relay
        .intervene(
            "field-lead",
            intervention("deploy-mesh-relay"),
            "durable-retrieval-canopy-relay",
        )
        .await
        .expect("create durable intervention");
    let original_audit = relay.audit_entries();
    assert_eq!(original_audit.len(), 1);

    drop(relay);
    let restarted = application("restart-acceptance", store).await;
    let restored = restarted.interventions();
    let restored_audit = restarted.audit_entries();

    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].id, original.id);
    assert_eq!(restored[0].evidence_hash, original.evidence_hash);
    assert_eq!(restored_audit, original_audit);
    assert_eq!(
        restored_audit[0].correlation_id,
        "durable-retrieval-canopy-relay"
    );
}

#[tokio::test]
async fn trusted_observation_is_persisted_and_hash_linked_into_audit() {
    let relay = application("observation-acceptance", ArtifactStore::memory().unwrap()).await;
    let trust_receipt = "a".repeat(64);
    let accepted = relay
        .observe(
            observation(),
            &trust_receipt,
            "observation-primary-canopy-relay",
        )
        .await
        .expect("trusted observation must be accepted");

    assert!(accepted.id.starts_with("obs-"));
    assert_eq!(accepted.provenance_hash.len(), 64);
    assert_eq!(accepted.source.zone_id, "wetland-east");
    let audit = relay.audit_entries();
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0].event_type, "observation-recorded");
    assert_eq!(audit[0].subject_id, accepted.id);
    assert_eq!(audit[0].previous_hash, "GENESIS");
    assert_eq!(audit[0].hash.len(), 64);
}

#[tokio::test]
async fn malformed_device_receipt_is_rejected_before_observation_write() {
    let relay = application("bad-receipt", ArtifactStore::memory().unwrap()).await;
    let result = relay
        .observe(
            observation(),
            "not-a-source-bound-receipt",
            "bad-device-canopy-relay",
        )
        .await;

    assert!(matches!(result, Err(RelayError::Unauthenticated(_))));
    assert_eq!(relay.audit_count(), 0);
}

#[tokio::test]
async fn replayed_device_receipt_is_rejected_before_a_second_write() {
    let relay = application("replayed-receipt", ArtifactStore::memory().unwrap()).await;
    let receipt = "c".repeat(64);
    relay
        .observe(observation(), &receipt, "receipt-first-use")
        .await
        .expect("first signed-ingest receipt is accepted");
    let before = relay.audit_count();
    let replay = relay
        .observe(observation(), &receipt, "receipt-replay")
        .await;
    assert!(matches!(replay, Err(RelayError::Conflict(_))));
    assert_eq!(relay.audit_count(), before);
}

#[tokio::test]
async fn invalid_location_is_rejected_by_typed_contract_before_domain_mutation() {
    let relay = application("invalid-location", ArtifactStore::memory().unwrap()).await;
    let mut invalid = observation();
    invalid.location.lat = 143.0;
    let result = relay
        .observe(invalid, &"b".repeat(64), "invalid-location-canopy-relay")
        .await;

    assert!(matches!(result, Err(RelayError::Validation(_))));
    assert_eq!(relay.audit_count(), 0);
}

#[tokio::test]
async fn dashboard_is_a_coherent_projection_of_seed_runtime_and_durable_state() {
    let relay = application("dashboard-projection", ArtifactStore::memory().unwrap()).await;
    let empty = relay.dashboard().expect("initial dashboard");
    assert_eq!(empty.zones.len(), 3);
    assert_eq!(empty.sensors.len(), 2);
    assert_eq!(empty.anomalies.len(), 1);
    assert!(empty.interventions.is_empty());
    assert!(empty.audit.is_empty());
    assert_eq!(empty.upstreams.len(), 7);

    relay
        .intervene(
            "field-lead",
            intervention("schedule-calibration"),
            "dashboard-after-effect",
        )
        .await
        .expect("calibration action is policy permitted");
    let updated = relay.dashboard().expect("updated dashboard");
    assert_eq!(updated.interventions.len(), 1);
    assert_eq!(updated.audit.len(), 1);
    assert_eq!(
        updated.interventions[0].request.action,
        "schedule-calibration"
    );
}

#[test]
fn malformed_cedar_policy_fails_service_construction_closed() {
    let malformed = r#"permit(principal, action, resource) when {"#;
    assert!(CedarAuthorizer::parse(malformed).is_err());
}

#[tokio::test]
async fn selected_composition_rejects_missing_correlation_before_any_upstream() {
    let seed = cr_seed::toronto_ravine();
    let result = runtime_clients()
        .execute(cr_upstream_compose::CompositionInput {
            correlation_id: String::new(),
            zone_id: "wetland-east".into(),
            observations: vec![],
            anomalies: seed.anomalies,
            links: seed.links,
        })
        .await;
    assert!(matches!(result, Err(RelayError::Validation(_))));
}
