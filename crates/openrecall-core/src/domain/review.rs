use crate::{
    domain::{
        card::{CardDetail, MemoryState},
        session::Session,
    },
    scheduling::fsrs::{Rating, SchedulePolicy},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewLog {
    pub id: String,
    pub card_id: String,
    pub deck_id: String,
    pub session_id: String,
    pub form_id: String,
    pub rating: Rating,
    pub reviewed_at: String,
    pub elapsed_days: f32,
    pub response_ms: u64,
    pub previous_memory: Option<MemoryState>,
    pub memory: MemoryState,
    pub previous_due_at: String,
    pub next_due_at: String,
    pub previous_phase: String,
    pub phase: String,
    pub policy: SchedulePolicy,
    pub model_revision: String,
    pub was_new: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoRecord {
    pub id: String,
    pub session_id: String,
    pub session_before: Session,
    pub card_before: CardDetail,
    pub next_card_before: Option<CardDetail>,
    pub log_id: String,
    pub action_id: String,
    pub model_revision: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionRecord {
    pub id: String,
    pub session_id: String,
    pub signature: String,
    pub status: String,
}
