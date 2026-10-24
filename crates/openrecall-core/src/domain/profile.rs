use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
pub struct TrainingConfig {
    pub num_epochs: Option<u16>,
    pub batch_size: u16,
    pub learning_rate: Option<f64>,
    pub seed: u32,
    pub max_seq_len: u16,
    pub gamma: f64,
    pub schedule_penalties: bool,
}
impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            num_epochs: None,
            batch_size: 512,
            learning_rate: None,
            seed: 2023,
            max_seq_len: 256,
            gamma: 1.0,
            schedule_penalties: true,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDetail {
    pub id: String,
    pub name: String,
    pub config: TrainingConfig,
    pub parameters: Vec<f32>,
    pub active_model_revision: String,
    pub model_origin: String,
    pub member_deck_ids: Vec<String>,
    pub revision: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
pub struct ModelRevision {
    pub id: String,
    pub parameters: Vec<f32>,
    pub origin: String,
    pub created_at: String,
    pub contributing_deck_ids: Vec<String>,
    pub training_run_id: Option<String>,
}
