use crate::{
    App, AppError, AppResult,
    contracts::library::{DeckDetail, DeleteReport},
    domain::profile::{ModelRevision, ProfileDetail, TrainingConfig},
};
pub(crate) fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
pub(crate) fn name(value: String) -> AppResult<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 256 {
        return Err(AppError::field("name", "requiredOrTooLong"));
    }
    Ok(value.into())
}
impl App {
    pub(crate) fn create_default_profile(&self, name: String) -> AppResult<ProfileDetail> {
        let model = ModelRevision {
            model_version: crate::domain::profile::model_version(),
            engine_revision: crate::domain::profile::engine_revision(),
            id: id(),
            parameters: fsrs::DEFAULT_PARAMETERS.to_vec(),
            origin: "default".into(),
            created_at: self.clock.now().to_rfc3339(),
            contributing_deck_ids: vec![],
            training_run_id: None,
        };
        self.store.put("models", &model.id, &model)?;
        let profile = ProfileDetail {
            model_revision: Some(model.clone()),
            id: id(),
            name,
            config: TrainingConfig::default(),
            parameters: model.parameters,
            active_model_revision: model.id,
            model_origin: "default".into(),
            member_deck_ids: vec![],
            revision: id(),
        };
        self.store.put("profiles", &profile.id, &profile)?;
        Ok(profile)
    }
    pub(crate) fn create_deck(
        &mut self,
        requested_name: String,
        profile_id: Option<String>,
    ) -> AppResult<DeckDetail> {
        let name = name(requested_name)?;
        let mut profile: ProfileDetail = match profile_id {
            Some(p) => self.store.get("profiles", &p)?,
            None => self.create_default_profile(format!("تحسين — {name}"))?,
        };
        let deck = DeckDetail {
            id: id(),
            name,
            profile_id: profile.id.clone(),
            created_at: self.clock.now().to_rfc3339(),
            revision: id(),
            data_revision: id(),
            logical_cards: 0,
            due_now: 0,
            new_cards: 0,
            desired_retention: None,
            min_interval_seconds: None,
            max_interval_days: None,
        };
        self.save_deck(&deck)?;
        profile.member_deck_ids.push(deck.id.clone());
        profile.revision = id();
        self.store.put("profiles", &profile.id, &profile)?;
        self.store.touch()?;
        Ok(deck)
    }
    pub(crate) fn save_deck(&self, deck: &DeckDetail) -> AppResult<()> {
        self.store.conn.execute("INSERT INTO decks(id,profile_id,payload) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET profile_id=excluded.profile_id,payload=excluded.payload",rusqlite::params![deck.id,deck.profile_id,serde_json::to_string(deck)?])?;
        Ok(())
    }
    pub(crate) fn rename_deck(
        &mut self,
        deck_id: String,
        new_name: String,
        expected: String,
    ) -> AppResult<DeckDetail> {
        let mut deck: DeckDetail = self.store.get("decks", &deck_id)?;
        if deck.revision != expected {
            return Err(AppError::new("conflict"));
        }
        deck.name = name(new_name)?;
        deck.revision = id();
        self.save_deck(&deck)?;
        self.store.touch()?;
        Ok(deck)
    }
    pub(crate) fn delete_deck(
        &mut self,
        deck_id: String,
        expected: String,
    ) -> AppResult<DeleteReport> {
        let deck: DeckDetail = self.store.get("decks", &deck_id)?;
        if deck.revision != expected {
            return Err(AppError::new("conflict"));
        }
        let mut profile: ProfileDetail = self.store.get("profiles", &deck.profile_id)?;
        profile.member_deck_ids.retain(|d| d != &deck_id);
        profile.revision = id();
        self.store.put("profiles", &profile.id, &profile)?;
        self.store
            .conn
            .execute("DELETE FROM decks WHERE id=?1", [&deck_id])?;
        self.store.touch()?;
        Ok(DeleteReport { id: deck_id })
    }
}
