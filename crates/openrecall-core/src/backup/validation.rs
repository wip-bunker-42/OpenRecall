use super::format::BackupData;
use crate::{
    AppError, AppResult,
    contracts::review::{ChoiceSubmission, ConfirmChoicesInput, RateInput},
    domain::review::ChoiceOutcome,
    domain::{
        card::{CardDetail, MemoryState},
        session::{Presentation, Session},
    },
    scheduling::fsrs::Rating,
    scheduling::fsrs::{FsrsScheduler, valid_memory},
};
use std::collections::{HashMap, HashSet};
fn check(valid: bool) -> AppResult<()> {
    if valid {
        Ok(())
    } else {
        Err(AppError::new("invalidBackup"))
    }
}
fn at(s: &str) -> AppResult<()> {
    check(chrono::DateTime::parse_from_rfc3339(s).is_ok())
}
fn ids<'a>(values: impl Iterator<Item = &'a str>) -> AppResult<HashSet<&'a str>> {
    let mut set = HashSet::new();
    for id in values {
        check(uuid::Uuid::parse_str(id).is_ok() && set.insert(id))?;
    }
    Ok(set)
}
fn memory(m: &MemoryState) -> AppResult<()> {
    check(valid_memory(m))
}
fn card(c: &CardDetail) -> AppResult<()> {
    at(&c.due_at)?;
    at(&c.created_at)?;
    if let Some(t) = &c.reviewed_at {
        at(t)?;
    }
    if let Some(m) = &c.memory {
        memory(m)?;
    }
    check(["new", "learning", "relearning", "review"].contains(&c.phase.as_str()))?;
    check(
        !c.forms.is_empty()
            && c.lapse_count <= c.review_count
            && (c.review_count == 0) == c.memory.is_none()
            && (c.review_count == 0) == c.reviewed_at.is_none(),
    )?;
    let form_ids = ids(c.forms.iter().map(|f| f.id.as_str()))?;
    for f in &c.forms {
        check(crate::domain::questions::validate_form(f).is_ok())?;
        check(
            crate::domain::card::normalize_question_title(f.question_title.clone())
                .is_ok_and(|title| title == f.question_title),
        )?;
    }
    for (id, order) in &c.previous_choice_orders {
        check(
            c.forms
                .iter()
                .find(|f| &f.id == id)
                .and_then(|f| f.multiple_choice.as_ref())
                .is_some_and(|set| crate::domain::questions::is_choice_permutation(set, order)),
        )?;
    }
    let remaining = ids(c.bag.remaining.iter().map(String::as_str))?;
    let consumed = ids(c.bag.consumed.iter().map(String::as_str))?;
    check(
        remaining.is_subset(&form_ids)
            && consumed.is_subset(&form_ids)
            && remaining.is_disjoint(&consumed),
    )?;
    if let Some(r) = &c.bag.reserved {
        check(c.bag.remaining.first() == Some(r))?;
    }
    Ok(())
}
fn presentation(p: &Presentation, deck: &str, cards: &HashMap<&str, &CardDetail>) -> AppResult<()> {
    let c = cards
        .get(p.card_id.as_str())
        .ok_or_else(|| AppError::new("invalidBackup"))?;
    let form = c
        .forms
        .iter()
        .find(|f| f.id == p.form_id)
        .ok_or_else(|| AppError::new("invalidBackup"))?;
    check(c.deck_id == deck)?;
    match (&form.multiple_choice, &p.choices) {
        (Some(set), Some(choices)) => check(
            !p.answer_revealed
                && ((!choices.confirmed_correct && choices.selected_ids.is_empty())
                    || crate::services::choice_reviews::validate_submission(
                        set,
                        &ChoiceSubmission::Selected {
                            choice_ids: choices.selected_ids.clone(),
                        },
                    )
                    .is_ok_and(|outcome| {
                        choices.confirmed_correct == (outcome == ChoiceOutcome::Correct)
                    }))
                && crate::domain::questions::is_choice_permutation(set, &choices.order)
                && (set.shuffle_choices
                    || choices.order.iter().eq(set.choices.iter().map(|c| &c.id))),
        ),
        (Some(_), None) => check(p.card_revision != c.revision),
        (None, None) => Ok(()),
        (None, Some(_)) => Err(AppError::new("invalidBackup")),
    }
}
fn confirmed_receipts(
    session: &Session,
    receipts: &HashSet<(String, String, Vec<String>)>,
) -> AppResult<()> {
    for p in session.current.iter().chain(&session.parked) {
        if let Some(choice) = p.choices.as_ref().filter(|choice| choice.confirmed_correct) {
            check(receipts.contains(&(
                session.id.clone(),
                p.id.clone(),
                choice.selected_ids.clone(),
            )))?;
        }
    }
    Ok(())
}
fn session(s: &Session, cards: &HashMap<&str, &CardDetail>) -> AppResult<()> {
    at(&s.started_at)?;
    if let Some(t) = &s.ended_at {
        at(t)?;
    }
    check(["active", "complete", "ended"].contains(&s.status.as_str()))?;
    if let Some(p) = &s.current {
        presentation(p, &s.deck_id, cards)?;
    }
    let mut seen = HashSet::new();
    for p in s.current.iter().chain(s.parked.iter()) {
        presentation(p, &s.deck_id, cards)?;
        check(seen.insert(&p.card_id))?;
    }
    for change in &s.filter_changes {
        at(&change.at)?;
    }
    Ok(())
}
fn aggregates(s: &Session, logs: &[&crate::domain::review::ReviewLog]) -> AppResult<()> {
    let mut counts = [0u32; 4];
    let mut total = 0u64;
    let mut unique = HashSet::new();
    let mut new = HashSet::new();
    for l in logs {
        counts[l.rating.number() as usize - 1] += 1;
        total = total.saturating_add(l.response_ms);
        unique.insert(l.card_id.as_str());
        if l.was_new {
            new.insert(l.card_id.as_str());
        }
    }
    check(
        s.attempts as usize == logs.len()
            && s.ratings == counts
            && s.response_ms == total
            && s.unique_card_ids
                .iter()
                .map(String::as_str)
                .collect::<HashSet<_>>()
                == unique
            && s.new_card_ids
                .iter()
                .map(String::as_str)
                .collect::<HashSet<_>>()
                == new,
    )
}
fn progress(c: &CardDetail, logs: &[&crate::domain::review::ReviewLog]) -> AppResult<()> {
    check(
        c.review_count as usize == logs.len()
            && c.lapse_count as usize
                == logs
                    .iter()
                    .filter(|l| l.rating.number() == 1 && !l.was_new)
                    .count(),
    )?;
    if let Some(last) = logs
        .iter()
        .max_by_key(|l| chrono::DateTime::parse_from_rfc3339(&l.reviewed_at).ok())
    {
        check(
            c.reviewed_at.as_deref() == Some(last.reviewed_at.as_str())
                && c.due_at == last.next_due_at
                && c.phase == last.phase,
        )
    } else {
        check(c.phase == "new" && c.reviewed_at.is_none() && c.memory.is_none())
    }
}
pub(crate) fn validate(d: &BackupData) -> AppResult<()> {
    crate::spelling::validate::validate(d)?;
    d.settings.validate()?;
    check(!d.library_revision.is_empty())?;
    let models = ids(d.models.iter().map(|m| m.id.as_str()))?;
    let profiles = ids(d.profiles.iter().map(|p| p.id.as_str()))?;
    let decks = ids(d.decks.iter().map(|v| v.id.as_str()))?;
    let cards = ids(d.cards.iter().map(|v| v.id.as_str()))?;
    let sessions = ids(d.sessions.iter().map(|v| v.id.as_str()))?;
    let reviews = ids(d.reviews.iter().map(|v| v.id.as_str()))?;
    let actions = ids(d.actions.iter().map(|v| v.id.as_str()))?;
    let undos = ids(d.undo_records.iter().map(|v| v.id.as_str()))?;
    ids(d.training_runs.iter().map(|v| v.id.as_str()))?;
    for m in &d.models {
        check(m.model_version == "FSRS-7" && m.parameters.len() == 34)?;
        FsrsScheduler::new(&m.parameters)?;
        at(&m.created_at)?;
    }
    for p in &d.profiles {
        p.config.validate()?;
        check(!p.name.trim().is_empty() && models.contains(p.active_model_revision.as_str()))?;
        FsrsScheduler::new(&p.parameters)?;
        let m = d
            .models
            .iter()
            .find(|m| m.id == p.active_model_revision)
            .ok_or_else(|| AppError::new("invalidBackup"))?;
        check(
            p.parameters == m.parameters
                && (p.model_origin == m.origin || p.model_origin == "inherited"),
        )?;
        let members = ids(p.member_deck_ids.iter().map(String::as_str))?;
        let actual = d
            .decks
            .iter()
            .filter(|v| v.profile_id == p.id)
            .map(|v| v.id.as_str())
            .collect::<HashSet<_>>();
        check(members == actual)?;
        if let Some(rev) = &p.model_revision {
            check(rev == m)?;
        }
    }
    for v in &d.decks {
        check(profiles.contains(v.profile_id.as_str()) && !v.name.trim().is_empty())?;
        at(&v.created_at)?;
        let mut settings = d.settings.clone();
        if let Some(x) = v.desired_retention {
            settings.desired_retention = x;
        }
        if let Some(x) = v.min_interval_seconds {
            settings.min_interval_seconds = x;
        }
        if let Some(x) = v.max_interval_days {
            settings.max_interval_days = x;
        }
        settings.validate()?;
    }
    let card_map = d
        .cards
        .iter()
        .map(|c| (c.id.as_str(), c))
        .collect::<HashMap<_, _>>();
    let session_map = d
        .sessions
        .iter()
        .map(|s| (s.id.as_str(), s))
        .collect::<HashMap<_, _>>();
    for c in &d.cards {
        check(decks.contains(c.deck_id.as_str()))?;
        card(c)?;
    }
    for s in &d.sessions {
        check(decks.contains(s.deck_id.as_str()))?;
        session(s, &card_map)?;
        if let Some(id) = &s.undo_id {
            check(undos.contains(id.as_str()))?;
        }
    }
    for l in &d.reviews {
        check(
            cards.contains(l.card_id.as_str())
                && decks.contains(l.deck_id.as_str())
                && sessions.contains(l.session_id.as_str())
                && models.contains(l.model_revision.as_str()),
        )?;
        check(
            card_map[l.card_id.as_str()].deck_id == l.deck_id
                && session_map[l.session_id.as_str()].deck_id == l.deck_id,
        )?;
        at(&l.reviewed_at)?;
        at(&l.previous_due_at)?;
        at(&l.next_due_at)?;
        memory(&l.memory)?;
        if let Some(m) = &l.previous_memory {
            memory(m)?;
        }
        check(l.elapsed_days.is_finite() && l.elapsed_days >= 0.0)?;
        let mut settings = d.settings.clone();
        settings.desired_retention = l.policy.desired_retention;
        settings.min_interval_seconds = l.policy.min_interval_seconds;
        settings.max_interval_days = l.policy.max_interval_days;
        settings.validate()?;
    }
    let mut by_card: HashMap<&str, Vec<&crate::domain::review::ReviewLog>> = HashMap::new();
    let mut by_session: HashMap<&str, Vec<&crate::domain::review::ReviewLog>> = HashMap::new();
    let logs_by_id = d
        .reviews
        .iter()
        .map(|l| (l.id.as_str(), l))
        .collect::<HashMap<_, _>>();
    let actions_by_id = d
        .actions
        .iter()
        .map(|a| (a.id.as_str(), a))
        .collect::<HashMap<_, _>>();
    for l in &d.reviews {
        by_card.entry(&l.card_id).or_default().push(l);
        by_session.entry(&l.session_id).or_default().push(l);
    }
    for c in &d.cards {
        progress(
            c,
            by_card
                .get(c.id.as_str())
                .map(Vec::as_slice)
                .unwrap_or_default(),
        )?;
    }
    // Session metrics are authoritative surviving-history aggregates, not caches.
    for s in &d.sessions {
        aggregates(
            s,
            by_session
                .get(s.id.as_str())
                .map(Vec::as_slice)
                .unwrap_or_default(),
        )?;
    }
    let mut receipts = HashSet::new();
    for a in &d.actions {
        check(
            sessions.contains(a.session_id.as_str())
                && ["applied", "undone"].contains(&a.status.as_str()),
        )?;
        if let Some(outcome) = a.choice_outcome {
            let input: ConfirmChoicesInput = serde_json::from_str(&a.signature)?;
            check(
                input.action_id == a.id
                    && input.session_id == a.session_id
                    && uuid::Uuid::parse_str(&input.presentation_id).is_ok()
                    && !input.expected_card_revision.is_empty()
                    && (outcome != ChoiceOutcome::Correct || a.status == "applied"),
            )?;
            match input.submission {
                ChoiceSubmission::Unknown => check(outcome == ChoiceOutcome::Unknown)?,
                ChoiceSubmission::Selected { choice_ids } => {
                    check(
                        outcome != ChoiceOutcome::Unknown
                            && !choice_ids.is_empty()
                            && choice_ids.len() <= 128
                            && choice_ids.windows(2).all(|pair| pair[0] < pair[1]),
                    )?;
                    ids(choice_ids.iter().map(String::as_str))?;
                    if outcome == ChoiceOutcome::Correct {
                        receipts.insert((input.session_id, input.presentation_id, choice_ids));
                    }
                }
            }
        } else {
            let input: RateInput = serde_json::from_str(&a.signature)?;
            check(input.action_id == a.id && input.session_id == a.session_id)?;
        }
    }
    for s in &d.sessions {
        confirmed_receipts(s, &receipts)?;
        let Some(f) = &s.choice_feedback else {
            continue;
        };
        let c = card_map
            .get(f.card_id.as_str())
            .ok_or_else(|| AppError::new("invalidBackup"))?;
        let log = logs_by_id
            .get(f.review_id.as_str())
            .ok_or_else(|| AppError::new("invalidBackup"))?;
        let action = actions_by_id
            .get(f.action_id.as_str())
            .ok_or_else(|| AppError::new("invalidBackup"))?;
        let set = f
            .form
            .multiple_choice
            .as_ref()
            .ok_or_else(|| AppError::new("invalidBackup"))?;
        let input: ConfirmChoicesInput = serde_json::from_str(&action.signature)?;
        let submission = if f.unknown {
            ChoiceSubmission::Unknown
        } else {
            ChoiceSubmission::Selected {
                choice_ids: f.choices.selected_ids.clone(),
            }
        };
        let outcome = crate::services::choice_reviews::validate_submission(set, &submission)?;
        check(
            s.status == "active"
                && s.current.is_none()
                && uuid::Uuid::parse_str(&f.id).is_ok()
                && c.deck_id == s.deck_id
                && c.revision == f.card_revision
                && c.forms.contains(&f.form)
                && c.previous_choice_orders.get(&f.form.id) == Some(&f.choices.order)
                && !f.choices.confirmed_correct
                && crate::domain::questions::is_choice_permutation(set, &f.choices.order)
                && (set.shuffle_choices
                    || f.choices.order.iter().eq(set.choices.iter().map(|c| &c.id)))
                && (!f.unknown || f.choices.selected_ids.is_empty())
                && log.card_id == c.id
                && log.form_id == f.form.id
                && log.session_id == s.id
                && log.rating == Rating::Again
                && log.next_due_at == c.due_at
                && action.status == "applied"
                && action.session_id == s.id
                && action.choice_outcome == Some(outcome)
                && outcome != ChoiceOutcome::Correct
                && input.submission == submission
                && s.last_review.as_ref().is_some_and(|last| {
                    last.card_id == c.id
                        && last.rating == Rating::Again
                        && last.due_at == log.next_due_at
                }),
        )?;
    }
    for u in &d.undo_records {
        check(
            sessions.contains(u.session_id.as_str())
                && reviews.contains(u.log_id.as_str())
                && actions.contains(u.action_id.as_str())
                && models.contains(u.model_revision.as_str()),
        )?;
        check(
            u.session_before.id == u.session_id
                && u.session_before.choice_feedback.is_none()
                && session_map[u.session_id.as_str()].undo_id.as_deref() == Some(u.id.as_str()),
        )?;
        if let Some(feedback) = &session_map[u.session_id.as_str()].choice_feedback {
            check(
                feedback.review_id == u.log_id
                    && feedback.action_id == u.action_id
                    && u.session_before
                        .current
                        .as_ref()
                        .and_then(|p| p.choices.as_ref())
                        == Some(&feedback.choices),
            )?;
        }
        card(&u.card_before)?;
        check(cards.contains(u.card_before.id.as_str()))?;
        let log = logs_by_id
            .get(u.log_id.as_str())
            .ok_or_else(|| AppError::new("invalidBackup"))?;
        let current = card_map[u.card_before.id.as_str()];
        check(
            log.card_id == current.id
                && log.session_id == u.session_id
                && log.model_revision == u.model_revision
                && u.card_before.deck_id == current.deck_id
                && u.card_before.forms == current.forms
                && u.card_before.created_at == current.created_at
                && u.card_before.review_count.checked_add(1) == Some(current.review_count)
                && u.card_before.memory == log.previous_memory
                && u.card_before.due_at == log.previous_due_at
                && u.card_before.phase == log.previous_phase
                && u.session_before.status == "active"
                && u.session_before.deck_id == current.deck_id,
        )?;
        progress(
            &u.card_before,
            &by_card
                .get(current.id.as_str())
                .into_iter()
                .flatten()
                .copied()
                .filter(|l| l.id != u.log_id)
                .collect::<Vec<_>>(),
        )?;
        aggregates(
            &u.session_before,
            &by_session
                .get(u.session_id.as_str())
                .into_iter()
                .flatten()
                .copied()
                .filter(|l| l.id != u.log_id)
                .collect::<Vec<_>>(),
        )?;
        if let Some(c) = &u.next_card_before {
            card(c)?;
            check(cards.contains(c.id.as_str()))?;
            let current = card_map[c.id.as_str()];
            let mut learning_before = c.clone();
            // Reserving the next presentation may shuffle its bag, but cannot
            // change its content, owner or learning state.
            learning_before.bag = current.bag.clone();
            learning_before.revision = current.revision.clone();
            check(
                learning_before == *current
                    && c.deck_id == u.card_before.deck_id
                    && c.id != u.card_before.id,
            )?;
        }
        let mut before = card_map.clone();
        before.insert(u.card_before.id.as_str(), &u.card_before);
        if let Some(c) = &u.next_card_before {
            before.insert(c.id.as_str(), c);
        }
        session(&u.session_before, &before)?;
        confirmed_receipts(&u.session_before, &receipts)?;
        let action = actions_by_id
            .get(u.action_id.as_str())
            .ok_or_else(|| AppError::new("invalidBackup"))?;
        check(action.status == "applied" && action.session_id == u.session_id)?;
        let p = u
            .session_before
            .current
            .as_ref()
            .ok_or_else(|| AppError::new("invalidBackup"))?;
        check(
            p.card_id == log.card_id
                && p.form_id == log.form_id
                && p.card_revision == u.card_before.revision
                && u.card_before.bag.reserved.as_deref() == Some(p.form_id.as_str()),
        )?;
        let form = u
            .card_before
            .forms
            .iter()
            .find(|form| form.id == p.form_id)
            .ok_or_else(|| AppError::new("invalidBackup"))?;
        if let Some(outcome) = action.choice_outcome {
            let input: ConfirmChoicesInput = serde_json::from_str(&action.signature)?;
            let set = form
                .multiple_choice
                .as_ref()
                .ok_or_else(|| AppError::new("invalidBackup"))?;
            let choices = p
                .choices
                .as_ref()
                .ok_or_else(|| AppError::new("invalidBackup"))?;
            check(
                input.presentation_id == p.id
                    && input.expected_card_revision == u.card_before.revision
                    && log.rating == Rating::Again
                    && u.next_card_before.is_none()
                    && outcome != ChoiceOutcome::Correct
                    && !choices.confirmed_correct
                    && outcome
                        == crate::services::choice_reviews::validate_submission(
                            set,
                            &input.submission,
                        )?
                    && match input.submission {
                        ChoiceSubmission::Selected { choice_ids } => {
                            choice_ids == choices.selected_ids
                        }
                        ChoiceSubmission::Unknown => choices.selected_ids.is_empty(),
                    },
            )?;
        } else {
            let input: RateInput = serde_json::from_str(&action.signature)?;
            check(
                input.presentation_id == p.id
                    && input.expected_card_revision == u.card_before.revision
                    && input.rating == log.rating
                    && d.decks
                        .iter()
                        .find(|deck| deck.id == u.card_before.deck_id)
                        .is_some_and(|deck| {
                            crate::domain::session::manual_grade_allowed(deck.kind, form, p)
                        }),
            )?;
            check(u.deferred_card_befores.is_empty())?;
        }
        ids(u.deferred_card_befores.iter().map(|card| card.id.as_str()))?;
        for deferred in &u.deferred_card_befores {
            card(deferred)?;
            let current = card_map
                .get(deferred.id.as_str())
                .ok_or_else(|| AppError::new("invalidBackup"))?;
            let mut before = deferred.clone();
            before.bag = current.bag.clone();
            check(
                deferred.id != u.card_before.id
                    && deferred.deck_id == u.card_before.deck_id
                    && before == **current,
            )?;
        }
    }
    for j in &d.training_runs {
        check(
            profiles.contains(j.profile_id.as_str())
                && models.contains(j.baseline_model_revision.as_str()),
        )?;
        j.config.validate()?;
        at(&j.started_at)?;
        if let Some(t) = &j.finished_at {
            at(t)?;
        }
        check(
            [
                "queued",
                "running",
                "cancelled",
                "interrupted",
                "failed",
                "noImprovement",
                "ready",
                "applied",
            ]
            .contains(&j.status.as_str())
                && j.progress.fraction.is_finite()
                && (0.0..=1.0).contains(&j.progress.fraction),
        )?;
        if let Some(parameters) = &j.candidate_parameters {
            check(parameters.len() == 34)?;
            FsrsScheduler::new(parameters)?;
        }
        for metric in j.baseline_metrics.iter().chain(j.candidate_metrics.iter()) {
            check(
                metric.log_loss.is_finite()
                    && metric.log_loss >= 0.0
                    && metric.rmse.is_finite()
                    && (0.0..=1.0).contains(&metric.rmse),
            )?;
        }
    }
    Ok(())
}
