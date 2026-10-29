//! Released format-1/schema-8 shapes. Never add new product fields here.
use crate::{
    AppResult,
    contracts::training::TrainingJob,
    domain::{
        card::MemoryState,
        profile::{ModelRevision, ProfileDetail},
        review::{ActionRecord, ReviewLog},
        session::Session,
        settings::Settings,
    },
    scheduling::forms::FormBag,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyData {
    settings: Settings,
    library_revision: String,
    models: Vec<ModelRevision>,
    profiles: Vec<ProfileDetail>,
    decks: Vec<LegacyDeck>,
    cards: Vec<LegacyCard>,
    sessions: Vec<Session>,
    reviews: Vec<ReviewLog>,
    actions: Vec<ActionRecord>,
    undo_records: Vec<LegacyUndo>,
    training_runs: Vec<TrainingJob>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyDeck {
    id: String,
    name: String,
    profile_id: String,
    created_at: String,
    revision: String,
    data_revision: String,
    logical_cards: u32,
    due_now: u32,
    new_cards: u32,
    desired_retention: Option<f64>,
    min_interval_seconds: Option<u32>,
    max_interval_days: Option<u32>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyForm {
    id: String,
    front: String,
    back: String,
    notes: String,
    language: Option<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyCard {
    #[serde(default)]
    bag: FormBag,
    id: String,
    deck_id: String,
    forms: Vec<LegacyForm>,
    phase: String,
    memory: Option<MemoryState>,
    due_at: String,
    created_at: String,
    reviewed_at: Option<String>,
    review_count: u32,
    lapse_count: u32,
    has_graduated: bool,
    revision: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyUndo {
    id: String,
    session_id: String,
    session_before: Session,
    card_before: LegacyCard,
    next_card_before: Option<LegacyCard>,
    log_id: String,
    action_id: String,
    model_revision: String,
}

pub(super) fn upgrade(raw: serde_json::Value) -> AppResult<super::format::BackupData> {
    let old: LegacyData = serde_json::from_value(raw)?;
    // Only frozen fields survive this boundary. New optional fields receive their
    // current documented defaults after the original checksum has been verified.
    let mut upgraded = serde_json::to_value(old)?;
    upgraded["spelling"] =
        serde_json::to_value(crate::domain::spelling::SpellingBackupData::default())?;
    Ok(serde_json::from_value(upgraded)?)
}
