use super::library::{id, name};
use crate::{
    App, AppError, AppResult,
    contracts::{
        library::{DeckDetail, DeleteReport},
        profiles::{AssignProfileInput, SaveDeckPolicyInput},
    },
    domain::{
        card::CardDetail,
        profile::{ModelRevision, ProfileDetail, TrainingConfig},
        review::ReviewLog,
        session::Session,
    },
    scheduling::fsrs::{FsrsScheduler, ReviewObservation},
};
impl App {
    pub(crate) fn get_profile(&self, profile_id: &str) -> AppResult<ProfileDetail> {
        let mut profile: ProfileDetail = self.store.get("profiles", profile_id)?;
        profile.model_revision = Some(self.store.get("models", &profile.active_model_revision)?);
        Ok(profile)
    }
    pub(crate) fn require_profile_revision(
        &self,
        id: &str,
        expected: &str,
    ) -> AppResult<ProfileDetail> {
        let p = self.get_profile(id)?;
        if p.revision != expected {
            return Err(AppError::new("conflict"));
        }
        Ok(p)
    }
    pub(crate) fn guard_decks_idle(&self, decks: &[String]) -> AppResult<()> {
        if self
            .store
            .all::<Session>("sessions")?
            .iter()
            .any(|s| s.status == "active" && decks.contains(&s.deck_id))
        {
            return Err(AppError::new("reviewInProgress"));
        }
        Ok(())
    }
    pub(crate) fn clear_deck_undos(&self, decks: &[String]) -> AppResult<()> {
        for mut s in self
            .store
            .all::<Session>("sessions")?
            .into_iter()
            .filter(|s| decks.contains(&s.deck_id))
        {
            s.undo_id = None;
            self.store
                .conn
                .execute("DELETE FROM undo_records WHERE session_id=?1", [&s.id])?;
            self.save_session(&s)?;
        }
        Ok(())
    }
    pub(crate) fn replay_decks(&self, decks: &[String], parameters: &[f32]) -> AppResult<()> {
        let scheduler = FsrsScheduler::new(parameters)?;
        for deck in decks {
            let mut stmt = self
                .store
                .conn
                .prepare("SELECT payload FROM cards WHERE deck_id=?1")?;
            let cards = stmt
                .query_map([deck], |r| r.get::<_, String>(0))?
                .map(|r| Ok(serde_json::from_str::<CardDetail>(&r?)?))
                .collect::<AppResult<Vec<_>>>()?;
            drop(stmt);
            for mut card in cards {
                let mut stmt = self
                    .store
                    .conn
                    .prepare("SELECT payload FROM reviews WHERE card_id=?1 ORDER BY at_ms,rowid")?;
                let history = stmt
                    .query_map([&card.id], |r| r.get::<_, String>(0))?
                    .map(|r| {
                        let log: ReviewLog = serde_json::from_str(&r?)?;
                        Ok(ReviewObservation {
                            rating: log.rating,
                            elapsed_days: log.elapsed_days,
                        })
                    })
                    .collect::<AppResult<Vec<_>>>()?;
                card.memory = scheduler.replay(&history)?;
                card.revision = id();
                self.save_card(&card)?;
            }
            self.touch_deck(deck)?;
        }
        self.clear_deck_undos(decks)
    }
    pub(crate) fn create_profile(&self, requested_name: String) -> AppResult<ProfileDetail> {
        let p = self.create_default_profile(name(requested_name)?)?;
        self.store.touch()?;
        Ok(p)
    }
    pub(crate) fn rename_profile(
        &self,
        profile_id: String,
        new_name: String,
        expected: String,
    ) -> AppResult<ProfileDetail> {
        let mut p = self.require_profile_revision(&profile_id, &expected)?;
        p.name = name(new_name)?;
        p.revision = id();
        self.store.put("profiles", &p.id, &p)?;
        self.store.touch()?;
        Ok(p)
    }
    pub(crate) fn save_training_config(
        &self,
        profile_id: String,
        config: TrainingConfig,
        expected: String,
    ) -> AppResult<ProfileDetail> {
        let mut p = self.require_profile_revision(&profile_id, &expected)?;
        config.validate()?;
        self.guard_decks_idle(&p.member_deck_ids)?;
        p.config = config;
        p.revision = id();
        self.store.put("profiles", &p.id, &p)?;
        self.store.touch()?;
        Ok(p)
    }
    pub(crate) fn assign_profile(&self, input: AssignProfileInput) -> AppResult<ProfileDetail> {
        let mut deck: DeckDetail = self.store.get("decks", &input.deck_id)?;
        if deck.revision != input.expected_deck_revision {
            return Err(AppError::new("conflict"));
        }
        let mut target =
            self.require_profile_revision(&input.profile_id, &input.expected_profile_revision)?;
        if deck.profile_id == target.id {
            return Ok(target);
        }
        let mut previous = self.get_profile(&deck.profile_id)?;
        let mut involved = previous.member_deck_ids.clone();
        involved.extend(target.member_deck_ids.clone());
        self.guard_decks_idle(&involved)?;
        previous.member_deck_ids.retain(|d| d != &deck.id);
        previous.revision = id();
        target.member_deck_ids.push(deck.id.clone());
        target.revision = id();
        deck.profile_id = target.id.clone();
        deck.revision = id();
        self.save_deck(&deck)?;
        self.store.put("profiles", &previous.id, &previous)?;
        self.store.put("profiles", &target.id, &target)?;
        self.replay_decks(&[deck.id], &target.parameters)?;
        self.store.touch()?;
        Ok(target)
    }
    pub(crate) fn detach_profile(
        &self,
        deck_id: String,
        new_name: String,
        expected: String,
    ) -> AppResult<ProfileDetail> {
        let mut deck: DeckDetail = self.store.get("decks", &deck_id)?;
        if deck.revision != expected {
            return Err(AppError::new("conflict"));
        }
        let mut previous = self.get_profile(&deck.profile_id)?;
        self.guard_decks_idle(&previous.member_deck_ids)?;
        let mut detached = previous.clone();
        detached.id = id();
        detached.name = name(new_name)?;
        detached.member_deck_ids = vec![deck_id.clone()];
        detached.revision = id();
        detached.model_origin = "inherited".into();
        self.store.put("profiles", &detached.id, &detached)?;
        previous.member_deck_ids.retain(|d| d != &deck_id);
        previous.revision = id();
        self.store.put("profiles", &previous.id, &previous)?;
        deck.profile_id = detached.id.clone();
        deck.revision = id();
        self.save_deck(&deck)?;
        self.clear_deck_undos(&[deck_id])?;
        self.store.touch()?;
        Ok(detached)
    }
    pub(crate) fn reset_profile(
        &self,
        profile_id: String,
        expected: String,
    ) -> AppResult<ProfileDetail> {
        let p = self.require_profile_revision(&profile_id, &expected)?;
        self.guard_decks_idle(&p.member_deck_ids)?;
        let model = ModelRevision {
            id: id(),
            parameters: fsrs::DEFAULT_PARAMETERS.to_vec(),
            origin: "default".into(),
            created_at: self.clock.now().to_rfc3339(),
            contributing_deck_ids: vec![],
            training_run_id: None,
            model_version: crate::domain::profile::model_version(),
            engine_revision: crate::domain::profile::engine_revision(),
        };
        self.publish_model(p, model)
    }
    pub(crate) fn publish_model(
        &self,
        mut p: ProfileDetail,
        model: ModelRevision,
    ) -> AppResult<ProfileDetail> {
        self.guard_decks_idle(&p.member_deck_ids)?;
        FsrsScheduler::new(&model.parameters)?;
        self.store.put("models", &model.id, &model)?;
        p.parameters = model.parameters.clone();
        p.active_model_revision = model.id.clone();
        p.model_origin = model.origin.clone();
        p.model_revision = Some(model);
        p.revision = id();
        self.store.put("profiles", &p.id, &p)?;
        self.replay_decks(&p.member_deck_ids, &p.parameters)?;
        self.store.touch()?;
        Ok(p)
    }
    pub(crate) fn delete_profile(
        &self,
        profile_id: String,
        expected: String,
    ) -> AppResult<DeleteReport> {
        let p = self.require_profile_revision(&profile_id, &expected)?;
        if !p.member_deck_ids.is_empty() {
            return Err(AppError::new("profileInUse"));
        }
        self.store
            .conn
            .execute("DELETE FROM profiles WHERE id=?1", [&profile_id])?;
        self.store.touch()?;
        Ok(DeleteReport { id: profile_id })
    }
    pub(crate) fn save_deck_policy(&self, input: SaveDeckPolicyInput) -> AppResult<DeckDetail> {
        let mut deck: DeckDetail = self.store.get("decks", &input.deck_id)?;
        if deck.revision != input.expected_revision {
            return Err(AppError::new("conflict"));
        }
        self.guard_decks_idle(&[deck.id.clone()])?;
        let mut effective = self.store.settings()?;
        effective.desired_retention = input
            .desired_retention
            .unwrap_or(effective.desired_retention);
        effective.min_interval_seconds = input
            .min_interval_seconds
            .unwrap_or(effective.min_interval_seconds);
        effective.max_interval_days = input
            .max_interval_days
            .unwrap_or(effective.max_interval_days);
        effective.validate()?;
        deck.desired_retention = input.desired_retention;
        deck.min_interval_seconds = input.min_interval_seconds;
        deck.max_interval_days = input.max_interval_days;
        deck.revision = id();
        self.save_deck(&deck)?;
        self.store.touch()?;
        self.deck_detail(&deck.id)
    }
}
