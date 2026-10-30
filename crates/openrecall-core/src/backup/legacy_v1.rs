//! Released format-1/schema-8 shapes. Never add new product fields here.
use crate::AppResult;
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

// Frozen recursively from the released v0.1.0 wire types. These copies
// deliberately do not share mutable product DTOs or behavior.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    pub timezone: String,
    pub day_start_minute: u16,
    pub desired_retention: f64,
    pub min_interval_seconds: u32,
    pub max_interval_days: u32,
    pub shortcuts_enabled: bool,
    pub text_scale: f64,
    pub reduced_motion: String,
    pub page_size: u16,
    pub revision: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainingConfig {
    pub num_epochs: Option<u16>,
    pub batch_size: u16,
    pub learning_rate: Option<f64>,
    pub seed: u32,
    pub max_seq_len: u16,
    pub gamma: f64,
    pub schedule_penalties: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileDetail {
    #[serde(default)]
    pub model_revision: Option<ModelRevision>,
    pub id: String,
    pub name: String,
    pub config: TrainingConfig,
    pub parameters: Vec<f32>,
    pub active_model_revision: String,
    pub model_origin: String,
    pub member_deck_ids: Vec<String>,
    pub revision: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelRevision {
    #[serde(default = "model_version")]
    pub model_version: String,
    #[serde(default = "engine_revision")]
    pub engine_revision: String,
    pub id: String,
    pub parameters: Vec<f32>,
    pub origin: String,
    pub created_at: String,
    pub contributing_deck_ids: Vec<String>,
    pub training_run_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemoryState {
    pub stability: f32,
    pub stability_fast: f32,
    pub difficulty: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FormBag {
    pub remaining: Vec<String>,
    pub consumed: Vec<String>,
    pub last_consumed: Option<String>,
    pub reserved: Option<String>,
    pub cycle: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Presentation {
    pub id: String,
    pub card_id: String,
    pub form_id: String,
    pub answer_revealed: bool,
    pub card_revision: String,
    pub active_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Session {
    #[serde(default)]
    pub last_review: Option<ReviewResult>,
    pub id: String,
    pub deck_id: String,
    pub filter: ReviewFilter,
    pub filter_changes: Vec<FilterChange>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: String,
    pub current: Option<Presentation>,
    pub parked: Vec<Presentation>,
    pub attempts: u32,
    pub unique_card_ids: Vec<String>,
    pub new_card_ids: Vec<String>,
    pub ratings: [u32; 4],
    pub active_ms: u64,
    pub response_ms: u64,
    pub clock_checkpoint: u64,
    pub is_active: bool,
    pub runtime_id: String,
    pub keep_empty_filter: bool,
    pub undo_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewResult {
    pub card_id: String,
    pub rating: Rating,
    pub interval_seconds: f64,
    pub due_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FilterChange {
    pub at: String,
    pub from: ReviewFilter,
    pub to: ReviewFilter,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ReviewFilter {
    #[default]
    All,
    New,
    PreviouslyReviewed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewLog {
    pub id: String,
    pub card_id: String,
    pub deck_id: String,
    pub session_id: String,
    pub form_id: String,
    pub rating: Rating,
    pub reviewed_at: String,
    pub elapsed_days: f32,
    pub response_ms: u64,
    pub previous_memory: Option<MemoryState>,
    pub memory: MemoryState,
    pub previous_due_at: String,
    pub next_due_at: String,
    pub previous_phase: String,
    pub phase: String,
    pub policy: SchedulePolicy,
    pub model_revision: String,
    pub was_new: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionRecord {
    pub id: String,
    pub session_id: String,
    pub signature: String,
    pub status: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Rating {
    Again,
    Hard,
    Good,
    Easy,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SchedulePolicy {
    pub desired_retention: f64,
    pub min_interval_seconds: u32,
    pub max_interval_days: u32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeckContribution {
    pub deck_id: String,
    pub deck_name: String,
    pub observations: u32,
    pub cards: u32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainingEligibility {
    pub eligible: bool,
    pub usable_observations: u32,
    pub distinct_cards: u32,
    pub study_days: u32,
    pub again_count: u32,
    pub recalled_count: u32,
    pub train_count: u32,
    pub holdout_count: u32,
    pub holdout_cards: u32,
    pub unmet_conditions: Vec<String>,
    pub contributions: Vec<DeckContribution>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvaluationMetrics {
    pub log_loss: f64,
    pub rmse: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeckEvaluation {
    pub deck_id: String,
    pub observations: u32,
    pub baseline: Option<EvaluationMetrics>,
    pub candidate: Option<EvaluationMetrics>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JobProgress {
    pub current: u64,
    pub total: u64,
    pub fraction: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainingJob {
    pub id: String,
    pub profile_id: String,
    pub status: String,
    pub progress: JobProgress,
    pub eligibility: TrainingEligibility,
    pub config: TrainingConfig,
    pub source_revision: String,
    pub baseline_model_revision: String,
    pub member_deck_ids: Vec<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub error: Option<AppError>,
    pub reason: Option<String>,
    pub cancel_requested: bool,
    pub baseline_metrics: Option<EvaluationMetrics>,
    pub candidate_metrics: Option<EvaluationMetrics>,
    pub per_deck: Vec<DeckEvaluation>,
    pub candidate_parameters: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FieldError {
    pub field: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppError {
    pub code: String,
    pub fields: Vec<FieldError>,
    pub context: Option<String>,
}

fn model_version() -> String {
    "FSRS-7".into()
}
fn engine_revision() -> String {
    "0fd59949e9168af7194418c2dba99836b5548e87".into()
}
