use crate::{
    AppResult, Clock, RandomSource,
    contracts::{Bootstrap, Request, Response},
    storage::{SCHEMA_VERSION, Store},
};
use std::{path::Path, sync::Arc};
pub struct App {
    pub(crate) store: Store,
    #[allow(dead_code)] // Used by scheduling/session tasks, injected from the first migration.
    pub(crate) clock: Arc<dyn Clock>,
    #[allow(dead_code)]
    pub(crate) random: Box<dyn RandomSource>,
}
impl App {
    pub fn open(
        path: &Path,
        clock: Arc<dyn Clock>,
        random: Box<dyn RandomSource>,
    ) -> AppResult<Self> {
        Ok(Self {
            store: Store::open(path)?,
            clock,
            random,
        })
    }
    pub fn execute(&mut self, request: Request) -> AppResult<Response> {
        self.store.begin()?;
        let result = self.dispatch(request);
        self.store.finish(result)
    }
    pub fn execute_json(&mut self, request: serde_json::Value) -> AppResult<serde_json::Value> {
        let request = serde_json::from_value(request)?;
        Ok(serde_json::to_value(self.execute(request)?)?)
    }
    fn dispatch(&mut self, request: Request) -> AppResult<Response> {
        match request {
            Request::CreateDeck { name, profile_id } => {
                Ok(Response::CreateDeck(self.create_deck(name, profile_id)?))
            }
            Request::RenameDeck {
                deck_id,
                name,
                expected_revision,
            } => Ok(Response::RenameDeck(self.rename_deck(
                deck_id,
                name,
                expected_revision,
            )?)),
            Request::DeleteDeck {
                deck_id,
                expected_revision,
            } => Ok(Response::DeleteDeck(
                self.delete_deck(deck_id, expected_revision)?,
            )),
            Request::ListDecks {} => Ok(Response::ListDecks(self.store.all("decks")?)),
            Request::GetDeck { deck_id } => {
                Ok(Response::GetDeck(self.store.get("decks", &deck_id)?))
            }
            Request::SaveSettings {
                mut settings,
                expected_revision,
            } => {
                if self.store.settings()?.revision != expected_revision {
                    return Err(crate::AppError::new("conflict"));
                }
                settings.validate()?;
                settings.revision = uuid::Uuid::new_v4().to_string();
                self.store.conn.execute(
                    "UPDATE settings SET payload=?1 WHERE id=1",
                    [serde_json::to_string(&settings)?],
                )?;
                self.store.touch()?;
                Ok(Response::SaveSettings(settings))
            }
            Request::Bootstrap {} => Ok(Response::Bootstrap(Bootstrap {
                settings: self.store.settings()?,
                schema_version: SCHEMA_VERSION,
                decks: self.store.all("decks")?,
                profiles: self.store.all("profiles")?,
                library_revision: self.store.revision()?,
            })),
        }
    }
}
