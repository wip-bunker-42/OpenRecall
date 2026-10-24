use crate::{
    contracts::{library::DeckDetail, training::TrainingJob},
    domain::{
        card::CardDetail,
        profile::{ModelRevision, ProfileDetail},
        review::{ActionRecord, ReviewLog, UndoRecord},
        session::Session,
        settings::Settings,
    },
};
use serde::{Deserialize, Serialize};
pub const FORMAT_VERSION: u32 = 1;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupEnvelope {
    pub format_version: u32,
    pub schema_version: u32,
    pub app_version: String,
    pub exported_at: String,
    pub payload_sha256: String,
    pub data: BackupData,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupData {
    pub settings: Settings,
    pub library_revision: String,
    pub models: Vec<ModelRevision>,
    pub profiles: Vec<ProfileDetail>,
    pub decks: Vec<DeckDetail>,
    pub cards: Vec<CardDetail>,
    pub sessions: Vec<Session>,
    pub reviews: Vec<ReviewLog>,
    pub actions: Vec<ActionRecord>,
    pub undo_records: Vec<UndoRecord>,
    pub training_runs: Vec<TrainingJob>,
}
pub fn digest(data: &BackupData) -> crate::AppResult<String> {
    use sha2::{Digest, Sha256};
    // Value uses sorted object keys. Digest is independent of struct field order.
    Ok(
        Sha256::digest(serde_json::to_vec(&serde_json::to_value(data)?)?)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
    )
}
