use crate::FieldError;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateRef {
    pub scope: String,
    pub card_id: Option<String>,
    pub deck_id: Option<String>,
    pub form_id: Option<String>,
    pub source_key: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ImportForm {
    pub key: String,
    pub front: String,
    pub back: String,
    pub notes: String,
    pub valid: bool,
    pub duplicate: bool,
    pub duplicate_of: Option<DuplicateRef>,
    pub issues: Vec<FieldError>,
    pub warnings: Vec<FieldError>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ImportCard {
    pub key: String,
    pub primary: ImportForm,
    pub variants: Vec<ImportForm>,
    pub valid: bool,
    pub issues: Vec<FieldError>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ImportCounts {
    pub valid_cards: u32,
    pub invalid_cards: u32,
    pub duplicate_cards: u32,
    pub selected_cards: u32,
    pub valid_forms: u32,
    pub invalid_forms: u32,
    pub duplicate_forms: u32,
    pub warnings: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub preview_id: String,
    pub deck_id: String,
    pub library_revision: String,
    pub cards: Vec<ImportCard>,
    pub counts: ImportCounts,
    pub warnings: Vec<FieldError>,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub inserted_cards: u32,
    pub inserted_forms: u32,
    pub excluded_cards: u32,
    pub excluded_forms: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CommitImportInput {
    pub preview_id: String,
    pub library_revision: String,
    pub selected_card_keys: Vec<String>,
    pub excluded_form_keys: Vec<String>,
}
