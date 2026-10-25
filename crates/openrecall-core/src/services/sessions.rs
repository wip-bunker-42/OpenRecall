use super::library::id;
use crate::{
    App, AppError, AppResult,
    contracts::{library::DeckDetail, review::*},
    domain::{
        card::CardDetail,
        profile::ProfileDetail,
        session::{Presentation, Session},
    },
    scheduling::{
        forms::reserve_form,
        fsrs::{FsrsScheduler, RatingPreviews, SchedulePolicy},
        queue::ReviewFilter,
    },
};
use chrono::{DateTime, Utc};
impl App {
    pub(crate) fn save_session(&self, s: &Session) -> AppResult<()> {
        self.store.conn.execute("INSERT INTO sessions(id,deck_id,status,payload) VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET status=excluded.status,payload=excluded.payload",rusqlite::params![s.id,s.deck_id,s.status,serde_json::to_string(s)?])?;
        self.store.touch()
    }
    pub(crate) fn checkpoint(&self, s: &mut Session) {
        let now = self.clock.monotonic_ms();
        if s.runtime_id == self.runtime_id && s.is_active && s.status == "active" {
            let delta = now.saturating_sub(s.clock_checkpoint);
            // An active-window heartbeat bounds gaps. Suspended/closed time is
            // never mistaken for hours spent answering a question.
            if delta <= 30_000 {
                s.active_ms = s.active_ms.saturating_add(delta);
                if let Some(p) = s.current.as_mut() {
                    p.active_ms = p.active_ms.saturating_add(delta);
                }
            }
        }
        s.clock_checkpoint = now;
        s.runtime_id = self.runtime_id.clone();
    }
    pub(crate) fn due_cards(&self, deck: &str, filter: ReviewFilter) -> AppResult<Vec<CardDetail>> {
        let mut stmt=self.store.conn.prepare(&format!("SELECT payload FROM cards WHERE deck_id=?1 AND due_ms<=?2 AND {} ORDER BY due_ms,created_ms,id",filter.sql()))?;
        stmt.query_map(
            rusqlite::params![deck, self.clock.now().timestamp_millis()],
            |r| r.get::<_, String>(0),
        )?
        .map(|r| Ok(serde_json::from_str(&r?)?))
        .collect()
    }
    pub(crate) fn next_due(&self, deck: &str, filter: ReviewFilter) -> AppResult<Option<String>> {
        let value: Option<i64> = self.store.conn.query_row(
            &format!(
                "SELECT min(due_ms) FROM cards WHERE deck_id=?1 AND due_ms>?2 AND {}",
                filter.sql()
            ),
            rusqlite::params![deck, self.clock.now().timestamp_millis()],
            |r| r.get(0),
        )?;
        Ok(value
            .and_then(DateTime::<Utc>::from_timestamp_millis)
            .map(|d| d.to_rfc3339()))
    }
    pub(crate) fn scheduler_for(
        &self,
        card: &CardDetail,
    ) -> AppResult<(FsrsScheduler, SchedulePolicy, String)> {
        let deck: DeckDetail = self.store.get("decks", &card.deck_id)?;
        let profile: ProfileDetail = self.store.get("profiles", &deck.profile_id)?;
        let settings = self.store.settings()?;
        Ok((
            FsrsScheduler::new(&profile.parameters)?,
            SchedulePolicy {
                desired_retention: deck.desired_retention.unwrap_or(settings.desired_retention),
                min_interval_seconds: deck
                    .min_interval_seconds
                    .unwrap_or(settings.min_interval_seconds),
                max_interval_days: deck.max_interval_days.unwrap_or(settings.max_interval_days),
            },
            profile.active_model_revision,
        ))
    }
    pub(crate) fn card_previews(
        &self,
        card: &CardDetail,
        now: DateTime<Utc>,
    ) -> AppResult<RatingPreviews> {
        let (scheduler, policy, _) = self.scheduler_for(card)?;
        let elapsed = match card.reviewed_at.as_ref() {
            Some(t) => crate::domain::time::elapsed_days(
                t.parse().map_err(|_| AppError::new("invalidHistory"))?,
                now,
            ),
            None => 0.0,
        };
        scheduler.preview(card.memory.clone(), elapsed, policy)
    }
    pub(crate) fn start_session(&mut self, deck_id: String) -> AppResult<ReviewView> {
        let _: DeckDetail = self.store.get("decks", &deck_id)?;
        if let Some(existing) = self
            .store
            .all::<Session>("sessions")?
            .into_iter()
            .find(|s| s.deck_id == deck_id && s.status == "active")
        {
            return self.read_session(existing.id);
        }
        let mut s = Session {
            last_review: None,
            id: id(),
            deck_id,
            filter: ReviewFilter::All,
            filter_changes: vec![],
            started_at: self.clock.now().to_rfc3339(),
            ended_at: None,
            status: "active".into(),
            current: None,
            parked: vec![],
            attempts: 0,
            unique_card_ids: vec![],
            new_card_ids: vec![],
            ratings: [0; 4],
            active_ms: 0,
            response_ms: 0,
            clock_checkpoint: self.clock.monotonic_ms(),
            is_active: true,
            runtime_id: self.runtime_id.clone(),
            keep_empty_filter: false,
            undo_id: None,
        };
        self.session_view(&mut s)
    }
    pub(crate) fn read_session(&mut self, session_id: String) -> AppResult<ReviewView> {
        let mut s: Session = self.store.get("sessions", &session_id)?;
        self.checkpoint(&mut s);
        self.session_view(&mut s)
    }
    pub(crate) fn set_filter(
        &mut self,
        session_id: String,
        filter: ReviewFilter,
    ) -> AppResult<ReviewView> {
        let mut s: Session = self.store.get("sessions", &session_id)?;
        self.checkpoint(&mut s);
        if s.status != "active" {
            return Err(AppError::new("sessionFinished"));
        }
        if s.filter != filter {
            s.filter_changes.push(FilterChange {
                at: self.clock.now().to_rfc3339(),
                from: s.filter,
                to: filter,
            });
            s.filter = filter;
        }
        if let Some(current) = s.current.take() {
            let card: CardDetail = self.store.get("cards", &current.card_id)?;
            if filter.matches(card.review_count) {
                s.current = Some(current);
            } else {
                s.parked.retain(|p| p.card_id != current.card_id);
                s.parked.push(current);
            }
        }
        s.keep_empty_filter = true;
        self.session_view(&mut s)
    }
    pub(crate) fn reveal_answer(&mut self, session_id: String) -> AppResult<ReviewView> {
        let mut s: Session = self.store.get("sessions", &session_id)?;
        self.checkpoint(&mut s);
        s.current
            .as_mut()
            .ok_or_else(|| AppError::new("noQuestion"))?
            .answer_revealed = true;
        self.session_view(&mut s)
    }
    pub(crate) fn end_session(&mut self, session_id: String) -> AppResult<ReviewView> {
        let mut s: Session = self.store.get("sessions", &session_id)?;
        self.checkpoint(&mut s);
        s.status = "ended".into();
        s.ended_at = Some(self.clock.now().to_rfc3339());
        self.save_session(&s)?;
        Ok(ReviewView::Summary(self.session_summary(&s)?))
    }
    pub(crate) fn session_activity(
        &mut self,
        session_id: String,
        active: bool,
    ) -> AppResult<SessionTiming> {
        let mut s: Session = self.store.get("sessions", &session_id)?;
        self.checkpoint(&mut s);
        s.is_active = active;
        self.save_session(&s)?;
        Ok(SessionTiming {
            active_ms: s.active_ms,
            question_ms: s.current.as_ref().map_or(0, |p| p.active_ms),
        })
    }
    pub(crate) fn resumable_sessions(&self) -> AppResult<Vec<ResumableSession>> {
        Ok(self
            .store
            .all::<Session>("sessions")?
            .into_iter()
            .filter(|s| s.status == "active")
            .map(|s| ResumableSession {
                session_id: s.id,
                deck_id: s.deck_id,
                filter: s.filter,
                started_at: s.started_at,
                attempts: s.attempts,
            })
            .collect())
    }
    pub(crate) fn session_view(&mut self, s: &mut Session) -> AppResult<ReviewView> {
        if s.status != "active" {
            return Ok(ReviewView::Summary(self.session_summary(s)?));
        }
        let due = self.due_cards(&s.deck_id, s.filter)?;
        if s.current
            .as_ref()
            .is_some_and(|p| self.store.get::<CardDetail>("cards", &p.card_id).is_err())
        {
            s.current = None;
        }
        if s.current.is_none() {
            if let Some(next) = due.first() {
                s.current = if let Some(at) = s.parked.iter().position(|p| p.card_id == next.id) {
                    Some(s.parked.remove(at))
                } else {
                    Some(Presentation {
                        id: id(),
                        card_id: next.id.clone(),
                        form_id: String::new(),
                        answer_revealed: false,
                        card_revision: next.revision.clone(),
                        active_ms: 0,
                    })
                };
            } else if !s.keep_empty_filter {
                s.status = "complete".into();
                s.ended_at = Some(self.clock.now().to_rfc3339());
                self.save_session(s)?;
                return Ok(ReviewView::Summary(self.session_summary(s)?));
            }
        }
        let next_due_at = self.next_due(&s.deck_id, s.filter)?;
        let progress = Progress {
            current: s.attempts + u32::from(s.current.is_some()),
            total: s.attempts + (due.len() as u32).max(u32::from(s.current.is_some())),
            completed: s.attempts,
        };
        if let Some(p) = s.current.as_mut() {
            let mut card: CardDetail = self.store.get("cards", &p.card_id)?;
            let old_bag = card.bag.clone();
            let ids = card.forms.iter().map(|f| f.id.clone()).collect::<Vec<_>>();
            let selected = reserve_form(&mut card.bag, &ids, self.random.as_mut())?;
            if card.bag != old_bag {
                self.save_card(&card)?;
            }
            if !p.form_id.is_empty() && (p.form_id != selected || p.card_revision != card.revision)
            {
                p.answer_revealed = false;
            }
            p.form_id = selected;
            p.card_revision = card.revision.clone();
            let form = card
                .forms
                .iter()
                .find(|f| f.id == p.form_id)
                .cloned()
                .ok_or_else(|| AppError::new("noForms"))?;
            let preview_time = self.clock.now();
            let previews = if p.answer_revealed {
                Some(self.card_previews(&card, preview_time)?)
            } else {
                None
            };
            let frame = ReviewFrame {
                preview_due_at: previews.as_ref().map(|previews| {
                    let due = |seconds: f64| {
                        (preview_time
                            + chrono::Duration::milliseconds((seconds * 1000.0).round() as i64))
                        .to_rfc3339()
                    };
                    RatingDueDates {
                        again: due(previews.again.interval_seconds),
                        hard: due(previews.hard.interval_seconds),
                        good: due(previews.good.interval_seconds),
                        easy: due(previews.easy.interval_seconds),
                    }
                }),
                last_review: s.last_review.clone(),
                session_id: s.id.clone(),
                filter: s.filter,
                presentation_id: p.id.clone(),
                card_revision: card.revision.clone(),
                card,
                form,
                answer_revealed: p.answer_revealed,
                previews,
                progress,
                next_due_at,
                can_undo: s.undo_id.is_some(),
            };
            self.save_session(s)?;
            Ok(ReviewView::Question(Box::new(frame)))
        } else {
            let empty = EmptyFilterView {
                last_review: s.last_review.clone(),
                session_id: s.id.clone(),
                filter: s.filter,
                next_due_at,
                progress,
                can_undo: s.undo_id.is_some(),
            };
            self.save_session(s)?;
            Ok(ReviewView::EmptyFilter(empty))
        }
    }
    pub(crate) fn session_summary(&self, s: &Session) -> AppResult<SessionSummary> {
        let ended = s
            .ended_at
            .clone()
            .unwrap_or_else(|| self.clock.now().to_rfc3339());
        let start: DateTime<Utc> = s
            .started_at
            .parse()
            .map_err(|_| AppError::new("invalidHistory"))?;
        let end: DateTime<Utc> = ended.parse().map_err(|_| AppError::new("invalidHistory"))?;
        Ok(SessionSummary {
            last_review: s.last_review.clone(),
            session_id: s.id.clone(),
            filter: s.filter,
            filter_changes: s.filter_changes.clone(),
            started_at: s.started_at.clone(),
            ended_at: ended,
            wall_ms: (end - start).num_milliseconds().max(0) as u64,
            active_ms: s.active_ms,
            attempts: s.attempts,
            unique_cards: s.unique_card_ids.len() as u32,
            new_cards: s.new_card_ids.len() as u32,
            ratings: s.ratings,
            mean_response_ms: if s.attempts == 0 {
                None
            } else {
                Some(s.response_ms as f64 / s.attempts as f64)
            },
            matching_due_remaining: self.due_cards(&s.deck_id, s.filter)?.len() as u32,
            all_due_remaining: self.due_cards(&s.deck_id, ReviewFilter::All)?.len() as u32,
            next_due_at: self.next_due(&s.deck_id, s.filter)?,
            can_undo: s.undo_id.is_some(),
        })
    }
}
