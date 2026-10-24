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
