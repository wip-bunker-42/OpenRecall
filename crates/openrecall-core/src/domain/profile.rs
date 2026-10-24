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
impl TrainingConfig {
    pub fn validate(&self) -> crate::AppResult<()> {
        let fields = [
            (
                "numEpochs",
                self.num_epochs.is_some_and(|v| !(1..=100).contains(&v)),
            ),
            ("batchSize", !(16..=4096).contains(&self.batch_size)),
            (
                "learningRate",
                self.learning_rate
                    .is_some_and(|v| !v.is_finite() || !(0.00001..=0.1).contains(&v)),
            ),
            ("maxSeqLen", !(32..=4096).contains(&self.max_seq_len)),
            (
                "gamma",
                !self.gamma.is_finite() || !(0.1..=5.0).contains(&self.gamma),
            ),
        ];
        for (field, invalid) in fields {
            if invalid {
                return Err(crate::AppError::field(field, "range"));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDetail {
    #[serde(default)]
    pub model_revision: Option<ModelRevision>,
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
    #[serde(default = "model_version")]
    pub model_version: String,
    #[serde(default = "engine_revision")]
    pub engine_revision: String,
    pub id: String,
    pub parameters: Vec<f32>,
    pub origin: String,
    pub created_at: String,
    pub contributing_deck_ids: Vec<String>,
    pub training_run_id: Option<String>,
}
pub fn model_version() -> String {
    "FSRS-7".into()
}
pub fn engine_revision() -> String {
    "0fd59949e9168af7194418c2dba99836b5548e87".into()
}
