use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DeckContribution {
    pub deck_id: String,
    pub deck_name: String,
    pub observations: u32,
    pub cards: u32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TrainingEligibility {
    pub eligible: bool,
    pub usable_observations: u32,
    pub distinct_cards: u32,
    pub study_days: u32,
    pub again_count: u32,
    pub recalled_count: u32,
    pub train_count: u32,
    pub holdout_count: u32,
    pub holdout_cards: u32,
    pub unmet_conditions: Vec<String>,
    pub contributions: Vec<DeckContribution>,
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationMetrics {
    pub log_loss: f64,
    pub rmse: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DeckEvaluation {
    pub deck_id: String,
    pub observations: u32,
    pub baseline: Option<EvaluationMetrics>,
    pub candidate: Option<EvaluationMetrics>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct JobProgress {
    pub current: u64,
    pub total: u64,
    pub fraction: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TrainingJob {
    pub id: String,
    pub profile_id: String,
    pub status: String,
    pub progress: JobProgress,
    pub eligibility: TrainingEligibility,
    pub config: crate::domain::profile::TrainingConfig,
    pub source_revision: String,
    pub baseline_model_revision: String,
    pub member_deck_ids: Vec<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub error: Option<crate::AppError>,
    pub reason: Option<String>,
    pub cancel_requested: bool,
    pub baseline_metrics: Option<EvaluationMetrics>,
    pub candidate_metrics: Option<EvaluationMetrics>,
    pub per_deck: Vec<DeckEvaluation>,
    pub candidate_parameters: Option<Vec<f32>>,
}
