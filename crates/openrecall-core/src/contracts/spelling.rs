use crate::{
    contracts::library::DeckDetail,
    domain::spelling::{SourceSnapshot, SpellingConfig},
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SpellingDeckResult {
    pub deck: DeckDetail,
    pub config: SpellingConfig,
    pub added_words: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum SourceAvailability {
    Current,
    Changed,
    Deleted,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SpellingSourceView {
    pub snapshot: SourceSnapshot,
    pub availability: SourceAvailability,
}
