use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
pub struct MemoryState {
    pub stability: f32,
    pub stability_fast: f32,
    pub difficulty: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
pub struct CardForm {
    pub id: String,
    pub front: String,
    pub back: String,
    pub notes: String,
    pub language: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
pub struct CardDetail {
    #[serde(default)]
    pub bag: crate::scheduling::forms::FormBag,
    pub id: String,
    pub deck_id: String,
    pub forms: Vec<CardForm>,
    pub phase: String,
    pub memory: Option<MemoryState>,
    pub due_at: String,
    pub created_at: String,
    pub reviewed_at: Option<String>,
    pub review_count: u32,
    pub lapse_count: u32,
    pub has_graduated: bool,
    pub revision: String,
}
