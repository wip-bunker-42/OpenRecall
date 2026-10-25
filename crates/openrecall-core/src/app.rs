use crate::{
    AppResult, Clock, RandomSource,
    contracts::{Bootstrap, Request, Response},
    storage::{SCHEMA_VERSION, Store},
};
use std::{path::Path, sync::Arc};
pub struct App {
    pub(crate) restore_previews:
        std::cell::RefCell<std::collections::HashMap<String, crate::backup::PreparedRestore>>,
    pub(crate) jobs: crate::jobs::JobRegistry,
    pub(crate) import_previews:
        std::collections::HashMap<String, crate::contracts::import::ImportPreview>,
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
        let app = Self {
            restore_previews: Default::default(),
            jobs: Default::default(),
            import_previews: Default::default(),
            runtime_id: uuid::Uuid::new_v4().to_string(),
            store: Store::open(path)?,
            clock,
            random,
        };
        app.store.begin()?;
        let recovered = app.recover_training_jobs();
        app.store.finish(recovered)?;
        Ok(app)
    }
    pub fn execute(&mut self, request: Request) -> AppResult<Response> {
        self.store.begin()?;
        let pending_before = self.jobs.pending.len();
        let result = self.dispatch(request);
        let result = self.store.finish(result);
        if result.is_ok() {
            self.jobs.launch();
        } else {
            self.jobs.pending.truncate(pending_before);
        }
        result
    }
    pub fn execute_json(&mut self, request: serde_json::Value) -> AppResult<serde_json::Value> {
        let request = serde_json::from_value(request)?;
        Ok(serde_json::to_value(self.execute(request)?)?)
    }
    fn dispatch(&mut self, request: Request) -> AppResult<Response> {
        match request {
            Request::GetDefaultSettings {} => Ok(Response::GetDefaultSettings(
                crate::domain::settings::Settings::default(),
            )),
            Request::GetDeckStatistics { deck_id } => {
                Ok(Response::GetDeckStatistics(self.deck_statistics(&deck_id)?))
            }
            Request::StartTraining { profile_id } => {
                Ok(Response::StartTraining(self.start_training(profile_id)?))
            }
            Request::GetTrainingJob { job_id } => {
                Ok(Response::GetTrainingJob(self.poll_training(job_id)?))
            }
            Request::CancelTraining { job_id } => {
                Ok(Response::CancelTraining(self.cancel_training(job_id)?))
            }
            Request::ApplyTraining {
                job_id,
                expected_profile_revision,
            } => Ok(Response::ApplyTraining(
                self.apply_training(job_id, expected_profile_revision)?,
            )),
            Request::ListTrainingRuns { profile_id } => Ok(Response::ListTrainingRuns(
                self.list_training_runs(profile_id)?,
            )),
            Request::GetTrainingEligibility { profile_id } => Ok(Response::GetTrainingEligibility(
                self.profile_dataset(&profile_id)?.eligibility,
            )),
            Request::ListProfiles {} => Ok(Response::ListProfiles(self.store.all("profiles")?)),
            Request::GetProfile { profile_id } => {
                Ok(Response::GetProfile(self.get_profile(&profile_id)?))
            }
            Request::CreateProfile { name } => {
                Ok(Response::CreateProfile(self.create_profile(name)?))
            }
            Request::RenameProfile {
                profile_id,
                name,
                expected_revision,
            } => Ok(Response::RenameProfile(self.rename_profile(
                profile_id,
                name,
                expected_revision,
            )?)),
            Request::SaveTrainingConfig {
                profile_id,
                config,
                expected_revision,
            } => Ok(Response::SaveTrainingConfig(self.save_training_config(
                profile_id,
                config,
                expected_revision,
            )?)),
            Request::AssignProfile(input) => {
                Ok(Response::AssignProfile(self.assign_profile(input)?))
            }
            Request::DetachProfile {
                deck_id,
                new_name,
                expected_deck_revision,
            } => Ok(Response::DetachProfile(self.detach_profile(
                deck_id,
                new_name,
                expected_deck_revision,
            )?)),
            Request::ResetProfile {
                profile_id,
                expected_revision,
            } => Ok(Response::ResetProfile(
                self.reset_profile(profile_id, expected_revision)?,
            )),
            Request::DeleteProfile {
                profile_id,
                expected_revision,
            } => Ok(Response::DeleteProfile(
                self.delete_profile(profile_id, expected_revision)?,
            )),
            Request::SaveDeckPolicy(input) => {
                Ok(Response::SaveDeckPolicy(self.save_deck_policy(input)?))
            }
            Request::PreviewImport { deck_id, text } => {
                Ok(Response::PreviewImport(self.preview_import(deck_id, text)?))
            }
            Request::CommitImport(input) => Ok(Response::CommitImport(self.commit_import(input)?)),
            Request::RateCard(input) => Ok(Response::RateCard(self.rate_card(input)?)),
            Request::UndoReview { session_id } => {
                Ok(Response::UndoReview(self.undo_review(session_id)?))
            }
            Request::CloseSession { session_id } => {
                Ok(Response::CloseSession(self.close_session(session_id)?))
            }
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
