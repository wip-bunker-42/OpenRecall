use crate::{
    App, AppResult, contracts::library::DeckDetail, domain::review::ReviewLog, training::dataset::*,
};
impl App {
    pub(crate) fn profile_dataset(&self, profile_id: &str) -> AppResult<TrainingDataset> {
        let profile = self.get_profile(profile_id)?;
        let settings = self.store.settings()?;
        let mut members = vec![];
        let mut observations = vec![];
        for deck_id in &profile.member_deck_ids {
            let deck: DeckDetail = self.store.get("decks", deck_id)?;
            members.push(TrainingMember {
                deck_id: deck.id,
                deck_name: deck.name,
                data_revision: deck.data_revision,
            });
            let mut stmt = self
                .store
                .conn
                .prepare("SELECT payload FROM reviews WHERE deck_id=?1 ORDER BY at_ms,rowid")?;
            for row in stmt.query_map([deck_id], |r| r.get::<_, String>(0))? {
                let log: ReviewLog = serde_json::from_str(&row?)?;
                observations.push(TrainingObservation {
                    card_id: log.card_id,
                    deck_id: log.deck_id,
                    reviewed_at: log.reviewed_at,
                    rating: log.rating,
                    undone: false,
                    deleted: false,
                });
            }
        }
        build_dataset(
            &observations,
            TrainingContext {
                profile_id: profile.id,
                members,
                config: profile.config,
                timezone: settings.timezone,
                day_start_minute: settings.day_start_minute,
                baseline_parameters: profile.parameters,
                model_revision: profile.active_model_revision,
            },
        )
    }
}
