use crate::{AppError, AppResult, domain::settings::Settings};
use serde::{Deserialize, Serialize};
use std::path::Path;
use ts_rs::TS;
pub mod cards;
pub mod review;
use crate::scheduling::queue::ReviewFilter;
use review::{ResumableSession, ReviewView, SessionTiming};
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
    let config = ts_rs::Config::default().with_out_dir(output);
    Request::export_all(&config).map_err(|_| AppError::new("contractExport"))?;
    Response::export_all(&config).map_err(|_| AppError::new("contractExport"))?;
    AppError::export_all(&config).map_err(|_| AppError::new("contractExport"))?;
    Ok(())
}
