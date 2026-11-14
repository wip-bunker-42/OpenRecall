//! Frozen format-2/schema-9 wire types from the v0.2.0 release.
use super::{
    format::BackupData,
    legacy_v1::{
        ActionRecord, FormBag, MemoryState, ModelRevision, ProfileDetail, ReviewLog, Session,
        Settings, TrainingJob,
    },
};
use crate::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyData {
    spelling: SpellingBackupData,
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
    #[serde(default)]
    kind: DeckKind,
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
    #[serde(default)]
    question_title: Option<String>,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum DeckKind {
    #[default]
    Standard,
    Spelling,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SpellingConfig {
    deck_id: String,
    source_deck_id: Option<String>,
    source_deck_name: Option<String>,
    extraction_policy_version: u16,
    revision: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SpellingWord {
    card_id: String,
    deck_id: String,
    canonical: String,
    display: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SourceSnapshot {
    id: String,
    spelling_deck_id: String,
    source_deck_id: String,
    source_deck_name: String,
    source_card_id: String,
    source_form_id: String,
    source_form_index: u32,
    source_card_revision: String,
    extraction_policy_version: u16,
    captured_at: String,
    front: String,
    back: String,
    notes: String,
    question_title: Option<String>,
    language: Option<String>,
    content_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WordOrigin {
    spelling_card_id: String,
    source_card_id: String,
    source_form_id: String,
    snapshot_id: String,
    first_seen_at: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SpellingExclusion {
    deck_id: String,
    canonical: String,
    excluded_at: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SpellingBackupData {
    configs: Vec<SpellingConfig>,
    words: Vec<SpellingWord>,
    snapshots: Vec<SourceSnapshot>,
    origins: Vec<WordOrigin>,
    exclusions: Vec<SpellingExclusion>,
}

pub(super) fn upgrade(raw: serde_json::Value) -> AppResult<BackupData> {
    let released: LegacyData = serde_json::from_value(raw)?;
    Ok(serde_json::from_value(serde_json::to_value(released)?)?)
}
