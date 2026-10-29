use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum DeckKind {
    #[default]
    Standard,
    Spelling,
}
impl DeckKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Spelling => "spelling",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpellingConfig {
    pub deck_id: String,
    pub source_deck_id: Option<String>,
    pub source_deck_name: Option<String>,
    pub extraction_policy_version: u16,
    pub revision: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpellingWord {
    pub card_id: String,
    pub deck_id: String,
    pub canonical: String,
    pub display: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceSnapshot {
    pub id: String,
    pub spelling_deck_id: String,
    pub source_deck_id: String,
    pub source_deck_name: String,
    pub source_card_id: String,
    pub source_form_id: String,
    pub source_form_index: u32,
    pub source_card_revision: String,
    pub extraction_policy_version: u16,
    pub captured_at: String,
    pub front: String,
    pub back: String,
    pub notes: String,
    pub question_title: Option<String>,
    pub language: Option<String>,
    pub content_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WordOrigin {
    pub spelling_card_id: String,
    pub source_card_id: String,
    pub source_form_id: String,
    pub snapshot_id: String,
    pub first_seen_at: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpellingExclusion {
    pub deck_id: String,
    pub canonical: String,
    pub excluded_at: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpellingBackupData {
    pub configs: Vec<SpellingConfig>,
    pub words: Vec<SpellingWord>,
    pub snapshots: Vec<SourceSnapshot>,
    pub origins: Vec<WordOrigin>,
    pub exclusions: Vec<SpellingExclusion>,
}
