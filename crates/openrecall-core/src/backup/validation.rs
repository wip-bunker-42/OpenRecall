use super::format::BackupData;
use crate::{
    AppError, AppResult,
    domain::{
        card::{CardDetail, MemoryState},
        session::{Presentation, Session},
    },
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
        check(!f.front.trim().is_empty() && !f.back.trim().is_empty())?;
        check(
            crate::domain::card::normalize_question_title(f.question_title.clone())
                .is_ok_and(|title| title == f.question_title),
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
    check(c.deck_id == deck && c.forms.iter().any(|f| f.id == p.form_id))
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
    spelling_ownership(d)?;
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
    for a in &d.actions {
        check(
            sessions.contains(a.session_id.as_str())
                && ["applied", "undone"].contains(&a.status.as_str()),
        )?;
        let input: crate::contracts::review::RateInput = serde_json::from_str(&a.signature)?;
        check(input.action_id == a.id && input.session_id == a.session_id)?;
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
                && session_map[u.session_id.as_str()].undo_id.as_deref() == Some(u.id.as_str()),
        )?;
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
        let action = actions_by_id
            .get(u.action_id.as_str())
            .ok_or_else(|| AppError::new("invalidBackup"))?;
        check(action.status == "applied" && action.session_id == u.session_id)?;
        let input: crate::contracts::review::RateInput = serde_json::from_str(&action.signature)?;
        let p = u
            .session_before
            .current
            .as_ref()
            .ok_or_else(|| AppError::new("invalidBackup"))?;
        check(
            input.presentation_id == p.id
                && input.expected_card_revision == u.card_before.revision
                && p.card_id == log.card_id
                && p.form_id == log.form_id
                && p.answer_revealed
                && input.rating == log.rating,
        )?;
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

fn spelling_ownership(d: &BackupData) -> AppResult<()> {
    use crate::domain::spelling::DeckKind;
    let owned: HashSet<_> = d
        .decks
        .iter()
        .filter(|v| v.kind == DeckKind::Spelling)
        .map(|v| v.id.as_str())
        .collect();
    check(owned.len() <= 1)?;
    let data = &d.spelling;
    let configs = ids(data.configs.iter().map(|v| v.deck_id.as_str()))?;
    check(configs == owned)?;
    for c in &data.configs {
        check(c.extraction_policy_version == crate::spelling::extract::EXTRACTION_POLICY_VERSION)?;
        if let Some(source) = &c.source_deck_id {
            check(
                d.decks
                    .iter()
                    .any(|v| v.id == *source && v.kind == DeckKind::Standard),
            )?;
        }
    }
    let words = ids(data.words.iter().map(|v| v.card_id.as_str()))?;
    let actual: HashSet<_> = d
        .cards
        .iter()
        .filter(|v| owned.contains(v.deck_id.as_str()))
        .map(|v| v.id.as_str())
        .collect();
    check(words == actual)?;
    let mut keys = HashSet::new();
    for w in &data.words {
        check(
            owned.contains(w.deck_id.as_str())
                && d.cards
                    .iter()
                    .any(|v| v.id == w.card_id && v.deck_id == w.deck_id)
                && keys.insert((w.deck_id.as_str(), w.canonical.as_str()))
                && !w.canonical.is_empty()
                && !w.display.trim().is_empty(),
        )?;
    }
    ids(data.snapshots.iter().map(|v| v.id.as_str()))?;
    for s in &data.snapshots {
        check(owned.contains(s.spelling_deck_id.as_str()))?;
    }
    let mut edges = HashSet::new();
    for o in &data.origins {
        let word = data
            .words
            .iter()
            .find(|v| v.card_id == o.spelling_card_id)
            .ok_or_else(|| AppError::new("invalidBackup"))?;
        check(
            data.snapshots.iter().any(|s| {
                s.id == o.snapshot_id
                    && s.spelling_deck_id == word.deck_id
                    && s.source_card_id == o.source_card_id
                    && s.source_form_id == o.source_form_id
            }) && edges.insert((&o.spelling_card_id, &o.source_card_id, &o.source_form_id)),
        )?;
        at(&o.first_seen_at)?;
    }
    let mut excluded = HashSet::new();
    for x in &data.exclusions {
        check(
            owned.contains(x.deck_id.as_str())
                && !keys.contains(&(x.deck_id.as_str(), x.canonical.as_str()))
                && excluded.insert((&x.deck_id, &x.canonical)),
        )?;
        at(&x.excluded_at)?;
    }
    Ok(())
}
