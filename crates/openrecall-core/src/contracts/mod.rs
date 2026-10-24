use crate::{AppError, AppResult, domain::settings::Settings};
use serde::{Deserialize, Serialize};
use std::path::Path;
use ts_rs::TS;
pub mod library;
use crate::domain::profile::ProfileDetail;
use library::{DeckDetail, DeleteReport};
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
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
