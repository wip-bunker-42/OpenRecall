use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ExportReport {
    pub path: String,
    pub bytes: u64,
    pub payload_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RestorePreview {
    pub id: String,
    pub expected_library_revision: String,
    pub exported_at: String,
    pub cards: u32,
    pub forms: u32,
    pub decks: u32,
    pub reviews: u32,
    pub profiles: u32,
    pub format_version: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReport {
    pub safety_backup: ExportReport,
    pub cards: u32,
    pub decks: u32,
}
