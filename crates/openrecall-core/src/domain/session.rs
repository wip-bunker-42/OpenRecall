use crate::{contracts::review::FilterChange, scheduling::queue::ReviewFilter};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    pub id: String,
    pub card_id: String,
    pub form_id: String,
    pub answer_revealed: bool,
    pub card_revision: String,
    pub active_ms: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub deck_id: String,
    pub filter: ReviewFilter,
    pub filter_changes: Vec<FilterChange>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: String,
    pub current: Option<Presentation>,
    pub parked: Vec<Presentation>,
    pub attempts: u32,
    pub unique_card_ids: Vec<String>,
    pub new_card_ids: Vec<String>,
    pub ratings: [u32; 4],
    pub active_ms: u64,
    pub response_ms: u64,
    pub clock_checkpoint: u64,
    pub is_active: bool,
    pub runtime_id: String,
    pub keep_empty_filter: bool,
    pub undo_id: Option<String>,
}
