use crate::{
    domain::card::{CardDetail, CardForm},
    scheduling::{fsrs::RatingPreviews, queue::ReviewFilter},
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
pub struct FilterChange {
    pub at: String,
    pub from: ReviewFilter,
    pub to: ReviewFilter,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub current: u32,
    pub total: u32,
    pub completed: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFrame {
    pub session_id: String,
    pub filter: ReviewFilter,
    pub presentation_id: String,
    pub card: CardDetail,
    pub form: CardForm,
    pub answer_revealed: bool,
    pub previews: Option<RatingPreviews>,
    pub progress: Progress,
    pub next_due_at: Option<String>,
    pub card_revision: String,
    pub can_undo: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct EmptyFilterView {
    pub session_id: String,
    pub filter: ReviewFilter,
    pub next_due_at: Option<String>,
    pub progress: Progress,
    pub can_undo: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub session_id: String,
    pub filter: ReviewFilter,
    pub filter_changes: Vec<FilterChange>,
    pub started_at: String,
    pub ended_at: String,
    pub wall_ms: u64,
    pub active_ms: u64,
    pub attempts: u32,
    pub unique_cards: u32,
    pub new_cards: u32,
    pub ratings: [u32; 4],
    pub mean_response_ms: Option<f64>,
    pub matching_due_remaining: u32,
    pub all_due_remaining: u32,
    pub next_due_at: Option<String>,
    pub can_undo: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "state", content = "payload", rename_all = "camelCase")]
pub enum ReviewView {
    Question(Box<ReviewFrame>),
    EmptyFilter(EmptyFilterView),
    Summary(SessionSummary),
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SessionTiming {
    pub active_ms: u64,
    pub question_ms: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ResumableSession {
    pub session_id: String,
    pub deck_id: String,
    pub filter: ReviewFilter,
    pub started_at: String,
    pub attempts: u32,
}
