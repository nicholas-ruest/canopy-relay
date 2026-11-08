//! Canopy Relay application services and transactional effect boundary.
use chrono::Utc;
use cr_cedar_policy::CedarAuthorizer;
use cr_contracts::{validate_intervention_request, validate_observation_request};
use cr_dashboard::DashboardBuilder;
use cr_errors::{RelayError, Result};
use cr_repository::Repository;
use cr_types::{
    new_id, AuditEntry, Dashboard, Intervention, InterventionRequest, Observation,
    ObservationInput, UpstreamStatus,
};
use cr_upstream_compose::{CompositionInput, RuntimeClients};
use parking_lot::Mutex;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Default)]
struct DomainBooks {
    trust: cr_device_trust::DeviceTrust,
    trust_registry: cr_device_trust::TrustRegistry,
    ingest: cr_ingest::IngestBook,
    calibration: cr_calibration::CalibrationBook,
    habitat: cr_habitat_graph::HabitatGraph,
    provenance: cr_provenance::ProvenanceLedger,
    evidence_chain: cr_provenance::EvidenceChain,
    temporal: cr_temporal_anomaly::TemporalDetector,
    temporal_detectors: BTreeMap<String, cr_temporal_anomaly::RobustDetector>,
    mesh: cr_mesh_transport::MeshPlanner,
    forecast: cr_neural_forecast::ForecastBook,
    intervention: cr_intervention::InterventionPlanner,
    audit: cr_audit::AuditChain,
    traces: cr_observability::TraceBook,
}

pub struct CanopyRelay {
    repo: Arc<Repository>,
    policy: CedarAuthorizer,
    upstreams: RuntimeClients,
    seed: cr_seed::SeedWorld,
    books: Mutex<DomainBooks>,
}

impl CanopyRelay {
    pub fn new(repo: Arc<Repository>, policy: CedarAuthorizer, upstreams: RuntimeClients) -> Self {
        let seed = cr_seed::toronto_ravine();
        seed.validate().expect("static product seed is coherent");
        let mut books = DomainBooks::default();
        for sensor in &seed.sensors {
            books
                .trust_registry
                .provision(&sensor.device_id, "field-fw-v3", "rucelium-boundary-v1")
                .expect("static seed provisioning is valid");
        }
        Self {
            repo,
            policy,
            upstreams,
            seed,
            books: Mutex::new(books),
        }
    }

    pub async fn observe(
        &self,
        input: ObservationInput,
        trust_receipt: &str,
        correlation_id: &str,
    ) -> Result<Observation> {
        validate_observation_request(&input)?;
        if trust_receipt.len() != 64
            || !trust_receipt.bytes().all(|value| value.is_ascii_hexdigit())
        {
            return Err(RelayError::Unauthenticated(
                "source-bound device trust receipt required".into(),
            ));
        }
        let curve = cr_calibration::CalibrationCurve::new(
            input.calibration_version.clone(),
            vec![(0.0, 0.0), (1.0, 1.0)],
        )?;
        let calibrated_value = curve.apply(input.value)?;
        let evidence_payload =
            serde_json::to_vec(&input).map_err(|error| RelayError::Integrity(error.to_string()))?;
        let (provenance_hash, anomaly_score) = {
            let mut books = self.books.lock();
            books
                .trust_registry
                .admit_receipt(&input.device_id, trust_receipt)?;
            let evidence = books
                .evidence_chain
                .append(&input.device_id, &evidence_payload)?;
            books.evidence_chain.verify()?;
            let detector_key = format!("{}:{}", input.zone_id, input.metric);
            if !books.temporal_detectors.contains_key(&detector_key) {
                books.temporal_detectors.insert(
                    detector_key.clone(),
                    cr_temporal_anomaly::RobustDetector::new(12, 5.0, 2.0)?,
                );
            }
            let decision = books
                .temporal_detectors
                .get_mut(&detector_key)
                .expect("detector inserted")
                .observe(calibrated_value)?;
            (evidence.chain_hash, decision.score)
        };
        let observation = Observation {
            id: new_id("obs"),
            source: input.clone(),
            calibrated_value,
            confidence: (0.92_f64 - (anomaly_score / 100.0)).clamp(0.55, 0.92),
            provenance_hash,
            correlation_id: correlation_id.into(),
        };

        {
            let mut books = self.books.lock();
            books.trust.append(
                trust_receipt,
                &input.device_id,
                "verified-signature",
                1.0,
                vec![],
                BTreeMap::new(),
            )?;
            books.ingest.append(
                &observation.id,
                &input.zone_id,
                &input.metric,
                observation.calibrated_value,
                vec![],
                BTreeMap::new(),
            )?;
            books.calibration.append(
                &format!("cal-{}", observation.id),
                &input.device_id,
                &input.calibration_version,
                observation.confidence,
                vec![],
                BTreeMap::new(),
            )?;
        }

        self.repo.save_observation(observation.clone()).await?;
        self.append_audit(
            "observation-recorded",
            &observation.id,
            correlation_id,
            serde_json::to_value(&observation)
                .map_err(|error| RelayError::Integrity(error.to_string()))?,
        )
        .await?;
        Ok(observation)
    }

    pub async fn intervene(
        &self,
        actor: &str,
        request: InterventionRequest,
        correlation_id: &str,
    ) -> Result<Intervention> {
        validate_intervention_request(&request)?;
        let audit_before = self.repo.audit_count();
        // These high-impact actions are outside the prototype's ecological safety envelope.
        if matches!(
            request.action.as_str(),
            "controlled-burn" | "release-species"
        ) {
            return Err(RelayError::Denied(
                "restricted ecological intervention".into(),
            ));
        }
        self.policy.authorize(actor, &request)?;
        if self.repo.audit_count() != audit_before {
            return Err(RelayError::Integrity(
                "policy evaluation changed durable state".into(),
            ));
        }

        let result = self
            .upstreams
            .execute(CompositionInput {
                correlation_id: correlation_id.into(),
                zone_id: request.zone_id.clone(),
                observations: vec![],
                anomalies: self
                    .seed
                    .anomalies
                    .iter()
                    .filter(|item| item.id == request.anomaly_id)
                    .cloned()
                    .collect(),
                links: self.seed.links.clone(),
            })
            .await?;
        let predicted_effect = result.combined_score()?;
        if !(0.0..=1.0).contains(&predicted_effect) {
            return Err(RelayError::Integrity(
                "composed intervention score outside bounds".into(),
            ));
        }

        let evidence_hash = self.record_domain_evidence(
            &request,
            correlation_id,
            predicted_effect,
            &result.route,
            result.forecasts.len(),
        )?;
        let intervention = Intervention {
            id: new_id("int"),
            request,
            state: "deployed".into(),
            route: result.route,
            predicted_effect,
            policy_decision: "allow".into(),
            correlation_id: correlation_id.into(),
            created_at: Utc::now(),
            evidence_hash,
        };
        self.repo.save_intervention(intervention.clone()).await?;
        self.append_audit(
            "intervention-deployed",
            &intervention.id,
            correlation_id,
            serde_json::to_value(&intervention)
                .map_err(|error| RelayError::Integrity(error.to_string()))?,
        )
        .await?;
        if self.repo.audit_count() != audit_before + 1 {
            return Err(RelayError::Integrity(
                "intervention commit must append exactly one audit event".into(),
            ));
        }
        Ok(intervention)
    }

    fn record_domain_evidence(
        &self,
        request: &InterventionRequest,
        correlation_id: &str,
        score: f64,
        route: &[String],
        forecast_count: usize,
    ) -> Result<String> {
        let mut metadata = BTreeMap::new();
        metadata.insert("correlation_id".into(), correlation_id.into());
        metadata.insert("action".into(), request.action.clone());
        metadata.insert("route".into(), route.join("->"));
        metadata.insert("forecast_count".into(), forecast_count.to_string());
        let mut books = self.books.lock();
        let graph = books.habitat.append(
            &format!("graph-{correlation_id}"),
            &request.zone_id,
            "spatial-enrichment",
            score,
            vec![],
            metadata.clone(),
        )?;
        let provenance = books.provenance.append(
            &format!("prov-{correlation_id}"),
            &request.anomaly_id,
            "seven-upstream-composition",
            score,
            vec![],
            metadata.clone(),
        )?;
        books.temporal.append(
            &format!("temporal-{correlation_id}"),
            &request.zone_id,
            "anomaly-window",
            score,
            vec![],
            metadata.clone(),
        )?;
        books.mesh.append(
            &format!("mesh-{correlation_id}"),
            &request.zone_id,
            "relay-route",
            route.len() as f64,
            vec![],
            metadata.clone(),
        )?;
        books.forecast.append(
            &format!("forecast-{correlation_id}"),
            &request.zone_id,
            "neural-horizon",
            forecast_count as f64,
            vec![],
            metadata.clone(),
        )?;
        books.intervention.append(
            &format!("plan-{correlation_id}"),
            &request.zone_id,
            &request.action,
            score,
            vec![],
            metadata.clone(),
        )?;
        books.traces.append(
            &format!("trace-{correlation_id}"),
            correlation_id,
            "primary-workflow",
            7.0,
            vec![],
            metadata.clone(),
        )?;
        books.audit.append(
            &format!("effect-{correlation_id}"),
            &request.zone_id,
            "authorized-effect",
            score,
            vec![],
            metadata,
        )?;
        let mut digest = Sha256::new();
        digest.update(graph.digest);
        digest.update(provenance.digest);
        Ok(hex::encode(digest.finalize()))
    }

    async fn append_audit(
        &self,
        event_type: &str,
        subject_id: &str,
        correlation_id: &str,
        payload: serde_json::Value,
    ) -> Result<()> {
        let previous_hash = self
            .repo
            .audit()
            .last()
            .map(|entry| entry.hash.clone())
            .unwrap_or_else(|| "GENESIS".into());
        let sequence = self.repo.audit_count() as u64 + 1;
        let mut digest = Sha256::new();
        digest.update(sequence.to_le_bytes());
        digest.update(event_type.as_bytes());
        digest.update(subject_id.as_bytes());
        digest.update(correlation_id.as_bytes());
        digest.update(previous_hash.as_bytes());
        digest.update(
            serde_json::to_vec(&payload)
                .map_err(|error| RelayError::Integrity(error.to_string()))?,
        );
        self.repo
            .append_audit(AuditEntry {
                id: new_id("audit"),
                sequence,
                event_type: event_type.into(),
                subject_id: subject_id.into(),
                correlation_id: correlation_id.into(),
                payload,
                previous_hash,
                hash: hex::encode(digest.finalize()),
                created_at: Utc::now(),
            })
            .await
    }

    pub fn dashboard(&self) -> Result<Dashboard> {
        DashboardBuilder::default()
            .topology(self.seed.zones.clone(), self.seed.links.clone())
            .sensor_state(self.seed.sensors.clone())
            .operational_state(self.seed.anomalies.clone(), self.repo.interventions())
            .evidence(self.repo.audit())
            .dependencies(self.upstream_status())
            .build()
    }
    pub fn interventions(&self) -> Vec<Intervention> {
        self.repo.interventions()
    }
    pub fn audit_entries(&self) -> Vec<AuditEntry> {
        self.repo.audit()
    }
    pub fn audit_count(&self) -> usize {
        self.repo.audit_count()
    }
    pub fn upstream_status(&self) -> Vec<UpstreamStatus> {
        let mut statuses = self.upstreams.statuses();
        statuses.push(UpstreamStatus {
            name: "cedar-policy/cedar".into(),
            revision: "5af7b8c".into(),
            healthy: true,
            last_operation: "authorize-intervention".into(),
        });
        statuses.push(UpstreamStatus {
            name: "apache/opendal".into(),
            revision: "532050a".into(),
            healthy: true,
            last_operation: "persist-audit".into(),
        });
        statuses
    }
}
