use crate::{
    AppResult, Clock, RandomSource,
    contracts::{Bootstrap, Request, Response},
    storage::{SCHEMA_VERSION, Store},
};
use std::{path::Path, sync::Arc};
pub struct App {
    pub(crate) runtime_id: String,
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
            runtime_id: uuid::Uuid::new_v4().to_string(),
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
            Request::StartSession { deck_id } => {
                Ok(Response::StartSession(self.start_session(deck_id)?))
            }
            Request::ReadSession { session_id } => {
                Ok(Response::ReadSession(self.read_session(session_id)?))
            }
            Request::ResumeSession { session_id } => {
                Ok(Response::ResumeSession(self.read_session(session_id)?))
            }
            Request::SetReviewFilter { session_id, filter } => Ok(Response::SetReviewFilter(
                self.set_filter(session_id, filter)?,
            )),
            Request::RevealAnswer { session_id } => {
                Ok(Response::RevealAnswer(self.reveal_answer(session_id)?))
            }
            Request::EndSession { session_id } => {
                Ok(Response::EndSession(self.end_session(session_id)?))
            }
            Request::SetSessionActivity { session_id, active } => Ok(Response::SetSessionActivity(
                self.session_activity(session_id, active)?,
            )),
            Request::CreateCard { deck_id, draft } => {
                Ok(Response::CreateCard(self.create_card(deck_id, draft)?))
            }
            Request::UpdateCard {
                card_id,
                draft,
                expected_revision,
            } => Ok(Response::UpdateCard(self.update_card(
                card_id,
                draft,
                expected_revision,
            )?)),
            Request::DeleteCard {
                card_id,
                expected_revision,
            } => Ok(Response::DeleteCard(
                self.delete_card(card_id, expected_revision)?,
            )),
            Request::GetCard { card_id } => {
                Ok(Response::GetCard(self.store.get("cards", &card_id)?))
            }
            Request::ListCards {
                deck_id,
                query,
                page,
                page_size,
            } => Ok(Response::ListCards(
                self.list_cards(deck_id, query, page, page_size)?,
            )),
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
            Request::ListDecks {} => Ok(Response::ListDecks(self.deck_list()?)),
            Request::GetDeck { deck_id } => Ok(Response::GetDeck(self.deck_detail(&deck_id)?)),
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
                resumable_sessions: self.resumable_sessions()?,
                decks: self.deck_list()?,
                profiles: self.store.all("profiles")?,
                library_revision: self.store.revision()?,
            })),
        }
    }
}
