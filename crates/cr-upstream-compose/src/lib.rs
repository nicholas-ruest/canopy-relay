//! Primary workflow composition across all five selected Ruvnet projects.
use cr_errors::{RelayError, Result};
use cr_types::{Anomaly, Forecast, MeshLink, Observation, UpstreamStatus};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct RuntimeClients {
    pub rucelium: cr_rucelium::Client,
    pub worldgraph: cr_worldgraph::Client,
    pub midstream: cr_midstream::Client,
    pub latentmesh: cr_latentmesh::Client,
    pub ruvfann: cr_ruvfann::Client,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompositionInput {
    pub correlation_id: String,
    pub zone_id: String,
    pub observations: Vec<Observation>,
    pub anomalies: Vec<Anomaly>,
    pub links: Vec<MeshLink>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompositionResult {
    pub route: Vec<String>,
    pub forecasts: Vec<Forecast>,
    pub cellular_score: f64,
    pub graph_score: f64,
    pub temporal_score: f64,
    pub mesh_score: f64,
    pub neural_score: f64,
    pub evidence: Vec<serde_json::Value>,
}

impl CompositionResult {
    pub fn combined_score(&self) -> Result<f64> {
        let scores = [
            self.cellular_score,
            self.graph_score,
            self.temporal_score,
            self.mesh_score,
            self.neural_score,
        ];
        if scores.iter().any(|score| !score.is_finite()) {
            return Err(RelayError::Integrity("non-finite upstream score".into()));
        }
        Ok(scores.iter().sum::<f64>() / scores.len() as f64)
    }
}

impl RuntimeClients {
    pub async fn execute(&self, input: CompositionInput) -> Result<CompositionResult> {
        if input.correlation_id.trim().is_empty() || input.zone_id.trim().is_empty() {
            return Err(RelayError::Validation(
                "composition requires correlation and zone".into(),
            ));
        }
        let common = cr_rucelium::Request {
            correlation_id: input.correlation_id.clone(),
            zone_id: input.zone_id.clone(),
            observations: input.observations.clone(),
            anomalies: input.anomalies.clone(),
            links: input.links.clone(),
        };
        // Deliberately ordered: each stage consumes a validated product state and a
        // later failure cannot be mistaken for a successful intervention commit.
        let cellular = self.rucelium.execute(&common).await?;
        let graph = self
            .worldgraph
            .execute(&cr_worldgraph::Request {
                correlation_id: input.correlation_id.clone(),
                zone_id: input.zone_id.clone(),
                observations: input.observations.clone(),
                anomalies: input.anomalies.clone(),
                links: input.links.clone(),
            })
            .await?;
        let temporal = self
            .midstream
            .execute(&cr_midstream::Request {
                correlation_id: input.correlation_id.clone(),
                zone_id: input.zone_id.clone(),
                observations: input.observations.clone(),
                anomalies: input.anomalies.clone(),
                links: input.links.clone(),
            })
            .await?;
        let mesh = self
            .latentmesh
            .execute(&cr_latentmesh::Request {
                correlation_id: input.correlation_id.clone(),
                zone_id: input.zone_id.clone(),
                observations: input.observations.clone(),
                anomalies: input.anomalies.clone(),
                links: input.links.clone(),
            })
            .await?;
        if mesh.route.is_empty() {
            return Err(RelayError::Integrity(
                "LatentMesh returned no admissible relay route".into(),
            ));
        }
        let neural = self
            .ruvfann
            .execute(&cr_ruvfann::Request {
                correlation_id: input.correlation_id,
                zone_id: input.zone_id,
                observations: input.observations,
                anomalies: input.anomalies,
                links: input.links,
            })
            .await?;
        if neural.forecasts.is_empty() {
            return Err(RelayError::Integrity(
                "ruv-FANN returned no bounded forecast".into(),
            ));
        }
        let result = CompositionResult {
            route: mesh.route,
            forecasts: neural.forecasts,
            cellular_score: cellular.score,
            graph_score: graph.score,
            temporal_score: temporal.score,
            mesh_score: mesh.score,
            neural_score: neural.score,
            evidence: vec![
                cellular.evidence,
                graph.evidence,
                temporal.evidence,
                mesh.evidence,
                neural.evidence,
            ],
        };
        result.combined_score()?;
        Ok(result)
    }

    pub fn statuses(&self) -> Vec<UpstreamStatus> {
        vec![
            self.rucelium.status(),
            self.worldgraph.status(),
            self.midstream.status(),
            self.latentmesh.status(),
            self.ruvfann.status(),
        ]
    }
}
