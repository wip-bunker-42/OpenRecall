use crate::{AppError, AppResult, domain::settings::Settings};
use serde::{Deserialize, Serialize};
use std::path::Path;
use ts_rs::TS;
pub mod backup;
pub mod cards;
pub mod import;
pub mod spelling;
pub mod statistics;
pub mod system;
pub mod updates;
use import::{CommitImportInput, ImportPreview, ImportReport};
pub mod profiles;
pub mod training;
use training::{TrainingEligibility, TrainingJob};
pub mod review;
use crate::domain::profile::TrainingConfig;
use crate::scheduling::queue::ReviewFilter;
use profiles::{AssignProfileInput, SaveDeckPolicyInput};
use review::{RateInput, ResumableSession, ReviewView, SessionTiming};
pub mod library;
use crate::domain::card::CardDetail;
use crate::domain::profile::ProfileDetail;
use cards::{CardDraft, Page};
use library::{DeckDetail, DeleteReport};
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub resumable_sessions: Vec<ResumableSession>,
    pub settings: Settings,
    pub schema_version: u32,
    pub decks: Vec<DeckDetail>,
    pub profiles: Vec<ProfileDetail>,
    pub library_revision: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    content = "payload",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Request {
    GetDefaultSettings {},
    GetDeckStatistics {
        deck_id: String,
    },
    StartTraining {
        profile_id: String,
    },
    GetTrainingJob {
        job_id: String,
    },
    CancelTraining {
        job_id: String,
    },
    ApplyTraining {
        job_id: String,
        expected_profile_revision: String,
    },
    ListTrainingRuns {
        profile_id: String,
    },
    GetTrainingEligibility {
        profile_id: String,
    },
    ListProfiles {},
    GetProfile {
        profile_id: String,
    },
    CreateProfile {
        name: String,
    },
    RenameProfile {
        profile_id: String,
        name: String,
        expected_revision: String,
    },
    SaveTrainingConfig {
        profile_id: String,
        config: TrainingConfig,
        expected_revision: String,
    },
    AssignProfile(AssignProfileInput),
    DetachProfile {
        deck_id: String,
        new_name: String,
        expected_deck_revision: String,
    },
    ResetProfile {
        profile_id: String,
        expected_revision: String,
    },
    DeleteProfile {
        profile_id: String,
        expected_revision: String,
    },
    SaveDeckPolicy(SaveDeckPolicyInput),
    PreviewImport {
        deck_id: String,
        text: String,
    },
    CommitImport(CommitImportInput),
    RateCard(RateInput),
    UndoReview {
        session_id: String,
    },
    CloseSession {
        session_id: String,
    },
    StartSession {
        deck_id: String,
    },
    ReadSession {
        session_id: String,
    },
    ResumeSession {
        session_id: String,
    },
    SetReviewFilter {
        session_id: String,
        filter: ReviewFilter,
    },
    RevealAnswer {
        session_id: String,
    },
    EndSession {
        session_id: String,
    },
    SetSessionActivity {
        session_id: String,
        active: bool,
    },
    CreateCard {
        deck_id: String,
        draft: CardDraft,
    },
    UpdateCard {
        card_id: String,
        draft: CardDraft,
        expected_revision: String,
    },
    DeleteCard {
        card_id: String,
        expected_revision: String,
    },
    GetCard {
        card_id: String,
    },
    ListCards {
        deck_id: String,
        query: String,
        page: u32,
        page_size: u16,
    },
    Bootstrap {},
    CreateDeck {
        name: String,
        profile_id: Option<String>,
    },
    RenameDeck {
        deck_id: String,
        name: String,
        expected_revision: String,
    },
    DeleteDeck {
        deck_id: String,
        expected_revision: String,
    },
    ListDecks {},
    GetDeck {
        deck_id: String,
    },
    SaveSettings {
        settings: Settings,
        expected_revision: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "payload", rename_all = "camelCase")]
pub enum Response {
    GetDefaultSettings(Settings),
    GetDeckStatistics(statistics::DeckStatistics),
    StartTraining(TrainingJob),
    GetTrainingJob(TrainingJob),
    CancelTraining(TrainingJob),
    ApplyTraining(ProfileDetail),
    ListTrainingRuns(Vec<TrainingJob>),
    GetTrainingEligibility(TrainingEligibility),
    ListProfiles(Vec<ProfileDetail>),
    GetProfile(ProfileDetail),
    CreateProfile(ProfileDetail),
    RenameProfile(ProfileDetail),
    SaveTrainingConfig(ProfileDetail),
    AssignProfile(ProfileDetail),
    DetachProfile(ProfileDetail),
    ResetProfile(ProfileDetail),
    DeleteProfile(DeleteReport),
    SaveDeckPolicy(DeckDetail),
    PreviewImport(ImportPreview),
    CommitImport(ImportReport),
    RateCard(ReviewView),
    UndoReview(ReviewView),
    CloseSession(DeleteReport),
    StartSession(ReviewView),
    ReadSession(ReviewView),
    ResumeSession(ReviewView),
    SetReviewFilter(ReviewView),
    RevealAnswer(ReviewView),
    EndSession(ReviewView),
    SetSessionActivity(SessionTiming),
    CreateCard(CardDetail),
    UpdateCard(CardDetail),
    DeleteCard(DeleteReport),
    GetCard(CardDetail),
    ListCards(Page<CardDetail>),
    Bootstrap(Bootstrap),
    CreateDeck(DeckDetail),
    RenameDeck(DeckDetail),
    DeleteDeck(DeleteReport),
    ListDecks(Vec<DeckDetail>),
    GetDeck(DeckDetail),
    SaveSettings(Settings),
}
pub fn export(output: &Path) -> AppResult<()> {
    let config = ts_rs::Config::default()
        .with_large_int("number")
        .with_out_dir(output);
    Request::export_all(&config).map_err(|_| AppError::new("contractExport"))?;
    Response::export_all(&config).map_err(|_| AppError::new("contractExport"))?;
    AppError::export_all(&config).map_err(|_| AppError::new("contractExport"))?;
    system::SystemInfo::export_all(&config).map_err(|_| AppError::new("contractExport"))?;
    system::PickedImportFile::export_all(&config).map_err(|_| AppError::new("contractExport"))?;
    backup::ExportReport::export_all(&config).map_err(|_| AppError::new("contractExport"))?;
    backup::RestorePreview::export_all(&config).map_err(|_| AppError::new("contractExport"))?;
    backup::RestoreReport::export_all(&config).map_err(|_| AppError::new("contractExport"))?;
    updates::UpdateStatus::export_all(&config).map_err(|_| AppError::new("contractExport"))?;
    spelling::SpellingDeckResult::export_all(&config)
        .map_err(|_| AppError::new("contractExport"))?;
    spelling::SpellingSourceView::export_all(&config)
        .map_err(|_| AppError::new("contractExport"))?;
    crate::domain::spelling::SpellingWord::export_all(&config)
        .map_err(|_| AppError::new("contractExport"))?;
    crate::domain::spelling::WordOrigin::export_all(&config)
        .map_err(|_| AppError::new("contractExport"))?;
    crate::domain::spelling::SpellingExclusion::export_all(&config)
        .map_err(|_| AppError::new("contractExport"))?;
    Ok(())
}
