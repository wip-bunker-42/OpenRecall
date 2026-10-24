use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AssignProfileInput {
    pub deck_id: String,
    pub profile_id: String,
    pub expected_deck_revision: String,
    pub expected_profile_revision: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SaveDeckPolicyInput {
    pub deck_id: String,
    pub desired_retention: Option<f64>,
    pub min_interval_seconds: Option<u32>,
    pub max_interval_days: Option<u32>,
    pub expected_revision: String,
}
