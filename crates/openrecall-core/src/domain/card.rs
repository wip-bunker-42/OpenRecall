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
    #[serde(default)]
    pub question_title: Option<String>,
    pub id: String,
    pub front: String,
    pub back: String,
    pub notes: String,
    pub language: Option<String>,
}
pub fn normalize_question_title(value: Option<String>) -> crate::AppResult<Option<String>> {
    let Some(text) = value else { return Ok(None) };
    if text
        .chars()
        .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
    {
        return Err(crate::AppError::field(
            "questionTitle",
            "invalidQuestionTitle",
        ));
    }
    let trimmed = text.trim();
    if trimmed.chars().count() > 160 {
        return Err(crate::AppError::field(
            "questionTitle",
            "invalidQuestionTitle",
        ));
    }
    Ok((!trimmed.is_empty()).then(|| trimmed.to_owned()))
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
