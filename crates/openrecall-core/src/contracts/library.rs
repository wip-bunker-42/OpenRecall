use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
pub struct DeckDetail {
    #[serde(default)]
    pub kind: crate::domain::spelling::DeckKind,
    pub id: String,
    pub name: String,
    pub profile_id: String,
    pub created_at: String,
    pub revision: String,
    pub data_revision: String,
    pub logical_cards: u32,
    pub due_now: u32,
    pub new_cards: u32,
    pub desired_retention: Option<f64>,
    pub min_interval_seconds: Option<u32>,
    pub max_interval_days: Option<u32>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DeleteReport {
    pub id: String,
}
