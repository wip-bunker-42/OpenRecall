use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FormDraft {
    #[serde(default)]
    pub question_title: Option<String>,
    pub id: Option<String>,
    pub front: String,
    pub back: String,
    #[serde(default)]
    pub notes: String,
    pub language: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct CardDraft {
    pub forms: Vec<FormDraft>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub page_size: u16,
}
