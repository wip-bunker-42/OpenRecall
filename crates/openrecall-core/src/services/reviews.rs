use super::library::id;
use crate::{
    App, AppError, AppResult,
    contracts::{
        library::DeleteReport,
        review::{RateInput, ReviewResult, ReviewView},
    },
    domain::{
        card::CardDetail,
        review::{ActionRecord, ReviewLog, UndoRecord},
        session::Session,
    },
    scheduling::fsrs::Rating,
};
use rusqlite::OptionalExtension;
impl App {
    pub(crate) fn invalidate_card_undo(&self, card_id: &str) -> AppResult<()> {
        let mut invalid = std::collections::HashSet::new();
        for undo in self.store.all::<UndoRecord>("undo_records")? {
            if undo.card_before.id == card_id
                || undo
                    .next_card_before
                    .as_ref()
                    .is_some_and(|c| c.id == card_id)
            {
                self.store
                    .conn
                    .execute("DELETE FROM undo_records WHERE id=?1", [&undo.id])?;
                invalid.insert(undo.id);
            }
        }
        for mut session in self.store.all::<Session>("sessions")? {
            if session
                .undo_id
                .as_ref()
                .is_some_and(|id| invalid.contains(id))
            {
                session.undo_id = None;
                self.save_session(&session)?;
            }
        }
        Ok(())
    }
    pub(crate) fn refresh_session_counts(&self, deck_id: &str) -> AppResult<()> {
        let logs = self.store.all::<ReviewLog>("reviews")?;
        for mut session in self
            .store
            .all::<Session>("sessions")?
            .into_iter()
            .filter(|s| s.deck_id == deck_id)
        {
            session.attempts = 0;
            session.unique_card_ids.clear();
            session.new_card_ids.clear();
            session.ratings = [0; 4];
            session.response_ms = 0;
            for log in logs.iter().filter(|l| l.session_id == session.id) {
                session.attempts += 1;
                session.ratings[log.rating.number() as usize - 1] += 1;
                session.response_ms = session.response_ms.saturating_add(log.response_ms);
                if !session.unique_card_ids.contains(&log.card_id) {
                    session.unique_card_ids.push(log.card_id.clone());
                }
                if log.was_new && !session.new_card_ids.contains(&log.card_id) {
                    session.new_card_ids.push(log.card_id.clone());
                }
            }
            if session
                .current
                .as_ref()
                .is_some_and(|p| self.store.get::<CardDetail>("cards", &p.card_id).is_err())
            {
                session.current = None;
            }
            session
                .parked
                .retain(|p| self.store.get::<CardDetail>("cards", &p.card_id).is_ok());
            if session
                .last_review
                .as_ref()
                .is_some_and(|r| self.store.get::<CardDetail>("cards", &r.card_id).is_err())
            {
                session.last_review = None;
            }
            self.save_session(&session)?;
        }
        Ok(())
    }
    pub(crate) fn rate_card(&mut self, input: RateInput) -> AppResult<ReviewView> {
        if uuid::Uuid::parse_str(&input.action_id).is_err() {
            return Err(AppError::field("actionId", "invalidId"));
        }
        let signature = serde_json::to_string(&input)?;
        let prior: Option<String> = self
            .store
            .conn
            .query_row(
                "SELECT payload FROM actions WHERE id=?1",
                [&input.action_id],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(prior) = prior {
            let action: ActionRecord = serde_json::from_str(&prior)?;
            if action.signature != signature {
                return Err(AppError::new("actionConflict"));
            }
            if action.status == "undone" {
                return Err(AppError::new("actionUndone"));
            }
            return self.read_session(action.session_id);
        }
        let mut session: Session = self.store.get("sessions", &input.session_id)?;
        self.checkpoint(&mut session);
        if session.status != "active" {
            return Err(AppError::new("sessionFinished"));
        }
        let presentation = session
            .current
            .clone()
            .ok_or_else(|| AppError::new("noQuestion"))?;
        if presentation.id != input.presentation_id {
            return Err(AppError::new("invalidPresentation"));
        }
        if !presentation.answer_revealed {
            return Err(AppError::new("answerHidden"));
        }
        let mut card: CardDetail = self.store.get("cards", &presentation.card_id)?;
        if card.revision != input.expected_card_revision
            || card.revision != presentation.card_revision
        {
            return Err(AppError::new("conflict"));
        }
        let now = self.clock.now();
        let elapsed = if let Some(previous) = &card.reviewed_at {
            let previous: chrono::DateTime<chrono::Utc> = previous
                .parse()
                .map_err(|_| AppError::new("invalidHistory"))?;
            if now < previous {
                return Err(AppError::new("clockMovedBackward"));
            }
            crate::domain::time::elapsed_days(previous, now)
        } else {
            0.0
        };
        let (scheduler, policy, model_revision) = self.scheduler_for(&card)?;
        let previews = scheduler.preview(card.memory.clone(), elapsed, policy)?;
        let outcome = previews.chosen(input.rating).clone();
        if let Some(old) = session.undo_id.take() {
            self.store
                .conn
                .execute("DELETE FROM undo_records WHERE id=?1", [old])?;
        }
        let session_before = session.clone();
        let card_before = card.clone();
        let was_new = card.review_count == 0;
        crate::scheduling::forms::consume_form(&mut card.bag, &presentation.form_id)?;
        card.memory = Some(outcome.memory.clone());
        card.reviewed_at = Some(now.to_rfc3339());
        card.due_at = (now
            + chrono::Duration::milliseconds((outcome.interval_seconds * 1000.0).round() as i64))
        .to_rfc3339();
        if input.rating == Rating::Again {
            if !was_new {
                card.lapse_count = card.lapse_count.saturating_add(1);
            }
            card.phase = if card.has_graduated {
                "relearning"
            } else {
                "learning"
            }
            .into();
        } else if outcome.interval_seconds >= 86400.0 {
            card.has_graduated = true;
            card.phase = "review".into();
        } else if !card.has_graduated {
            card.phase = "learning".into();
        }
        card.review_count = card
            .review_count
            .checked_add(1)
            .ok_or_else(|| AppError::new("countLimit"))?;
        card.revision = id();
        self.save_card(&card)?;
        let log = ReviewLog {
            id: id(),
            card_id: card.id.clone(),
            deck_id: card.deck_id.clone(),
            session_id: session.id.clone(),
            form_id: presentation.form_id.clone(),
            rating: input.rating,
            reviewed_at: now.to_rfc3339(),
            elapsed_days: elapsed,
            response_ms: presentation.active_ms,
            previous_memory: card_before.memory.clone(),
            memory: outcome.memory,
            previous_due_at: card_before.due_at.clone(),
            next_due_at: card.due_at.clone(),
            previous_phase: card_before.phase.clone(),
            phase: card.phase.clone(),
            policy,
            model_revision: model_revision.clone(),
            was_new,
        };
        self.store.conn.execute("INSERT INTO reviews(id,card_id,deck_id,session_id,at_ms,model_revision,payload) VALUES(?1,?2,?3,?4,?5,?6,?7)",rusqlite::params![log.id,log.card_id,log.deck_id,log.session_id,now.timestamp_millis(),log.model_revision,serde_json::to_string(&log)?])?;
        let action = ActionRecord {
            id: input.action_id.clone(),
            session_id: session.id.clone(),
            signature,
            status: "applied".into(),
        };
        self.save_action(&action)?;
        session.attempts = session.attempts.saturating_add(1);
        session.ratings[input.rating.number() as usize - 1] += 1;
        session.response_ms = session.response_ms.saturating_add(presentation.active_ms);
        if !session.unique_card_ids.contains(&card.id) {
            session.unique_card_ids.push(card.id.clone());
        }
        if was_new && !session.new_card_ids.contains(&card.id) {
            session.new_card_ids.push(card.id.clone());
        }
        session.last_review = Some(ReviewResult {
            card_id: card.id.clone(),
            rating: input.rating,
            interval_seconds: outcome.interval_seconds,
            due_at: card.due_at.clone(),
        });
        session.current = None;
        session.parked.retain(|p| p.card_id != card.id);
        session.keep_empty_filter = false;
        let next_card_before = self
            .due_cards(&session.deck_id, session.filter)?
            .into_iter()
            .next()
            .filter(|c| c.id != card.id);
        let undo = UndoRecord {
            id: id(),
            session_id: session.id.clone(),
            session_before,
            card_before,
            next_card_before,
            log_id: log.id,
            action_id: input.action_id,
            model_revision,
        };
        self.store.conn.execute(
            "INSERT INTO undo_records(id,session_id,payload) VALUES(?1,?2,?3)",
            rusqlite::params![undo.id, undo.session_id, serde_json::to_string(&undo)?],
        )?;
        session.undo_id = Some(undo.id);
        self.touch_deck(&session.deck_id)?;
        self.session_view(&mut session)
    }
    fn save_action(&self, action: &ActionRecord) -> AppResult<()> {
        self.store.conn.execute("INSERT INTO actions(id,session_id,payload) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",rusqlite::params![action.id,action.session_id,serde_json::to_string(action)?])?;
        Ok(())
    }
    pub(crate) fn undo_review(&mut self, session_id: String) -> AppResult<ReviewView> {
        let current: Session = self.store.get("sessions", &session_id)?;
        let undo_id = current
            .undo_id
            .ok_or_else(|| AppError::new("undoUnavailable"))?;
        let undo: UndoRecord = self.store.get("undo_records", &undo_id)?;
        if undo.session_id != session_id {
            return Err(AppError::new("undoUnavailable"));
        }
        let (_, _, model) = self.scheduler_for(&undo.card_before)?;
        if model != undo.model_revision {
            return Err(AppError::new("undoUnavailable"));
        }
        let mut card = undo.card_before;
        card.revision = id();
        self.save_card(&card)?;
        if let Some(next) = undo.next_card_before {
            self.save_card(&next)?;
        }
        self.store
            .conn
            .execute("DELETE FROM reviews WHERE id=?1", [&undo.log_id])?;
        let mut action: ActionRecord = self.store.get("actions", &undo.action_id)?;
        action.status = "undone".into();
        self.save_action(&action)?;
        self.store
            .conn
            .execute("DELETE FROM undo_records WHERE id=?1", [&undo_id])?;
        let mut session = undo.session_before;
        session.clock_checkpoint = self.clock.monotonic_ms();
        session.runtime_id = self.runtime_id.clone();
        session.is_active = true;
        if let Some(p) = session.current.as_mut() {
            p.card_revision = card.revision;
        }
        self.touch_deck(&session.deck_id)?;
        self.session_view(&mut session)
    }
    pub(crate) fn close_session(&mut self, session_id: String) -> AppResult<DeleteReport> {
        let mut s: Session = self.store.get("sessions", &session_id)?;
        self.checkpoint(&mut s);
        s.is_active = false;
        s.undo_id = None;
        self.store.conn.execute(
            "DELETE FROM undo_records WHERE session_id=?1",
            [&session_id],
        )?;
        self.save_session(&s)?;
        Ok(DeleteReport { id: session_id })
    }
}
