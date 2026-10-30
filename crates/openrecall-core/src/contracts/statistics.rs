use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PeriodStatistics {
    pub completed_for_day: u32,
    pub attempts: u32,
    pub unique_cards: u32,
    pub active_ms: u64,
    pub mean_response_ms: Option<f64>,
    pub rating_counts: [u32; 4],
    pub observed_recall_rate: Option<f64>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PhaseCounts {
    pub new: u32,
    pub learning: u32,
    pub relearning: u32,
    pub review: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ForecastDay {
    pub date: String,
    pub cards: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct HistoryDay {
    pub date: String,
    pub statistics: PeriodStatistics,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LibraryStatistics {
    pub today: PeriodStatistics,
    pub daily_history: Vec<HistoryDay>,
    pub spelling_completed_today: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProfileStatus {
    pub id: String,
    pub name: String,
    pub model_origin: String,
    pub model_revision: String,
    pub eligibility: super::training::TrainingEligibility,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DeckStatistics {
    pub cards: u32,
    pub forms: u32,
    pub phases: PhaseCounts,
    pub due_now: u32,
    pub due_later_today: u32,
    pub next_due_at: Option<String>,
    pub forecast_7_days: Vec<ForecastDay>,
    pub forecast_30_days: Vec<ForecastDay>,
    pub today: PeriodStatistics,
    pub last_7_days: PeriodStatistics,
    pub last_30_days: PeriodStatistics,
    pub all_time: PeriodStatistics,
    pub daily_history: Vec<HistoryDay>,
    pub profile_status: ProfileStatus,
}
