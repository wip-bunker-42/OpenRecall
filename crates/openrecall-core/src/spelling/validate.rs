use super::{
    collect::snapshot_fingerprint,
    extract::{EXTRACTION_POLICY_VERSION, canonical_word, extract_words},
};
use crate::{
    AppError, AppResult,
    backup::format::BackupData,
    domain::{
        card::normalize_question_title,
        spelling::{DeckKind, SpellingBackupData},
    },
};
use std::collections::{HashMap, HashSet};
use unicode_normalization::UnicodeNormalization;

fn check(valid: bool) -> AppResult<()> {
    if valid {
        Ok(())
    } else {
        Err(AppError::new("invalidBackup"))
    }
}
fn identifier(value: &str) -> AppResult<()> {
    check(uuid::Uuid::parse_str(value).is_ok())
}
fn at(value: &str) -> AppResult<()> {
    check(chrono::DateTime::parse_from_rfc3339(value).is_ok())
}
fn name(value: &str) -> bool {
    !value.trim().is_empty() && value.chars().count() <= 256
}
fn unique_ids<'a>(values: impl Iterator<Item = &'a str>) -> AppResult<HashSet<&'a str>> {
    let mut set = HashSet::new();
    for id in values {
        identifier(id)?;
        check(set.insert(id))?;
    }
    Ok(set)
}

/// Pure format-2 registry validation. Historical source IDs need not remain live.
pub fn validate(d: &BackupData) -> AppResult<()> {
    let data = &d.spelling;
    let owned: Vec<_> = d
        .decks
        .iter()
        .filter(|deck| deck.kind == DeckKind::Spelling)
        .collect();
    check(owned.len() <= 1)?;
    let Some(owner) = owned.first() else {
        return check(data == &SpellingBackupData::default());
    };
    let configs = unique_ids(data.configs.iter().map(|c| c.deck_id.as_str()))?;
    check(configs.len() == 1 && configs.contains(owner.id.as_str()))?;
    let config = data
        .configs
        .first()
        .ok_or_else(|| AppError::new("invalidBackup"))?;
    check(!config.revision.is_empty())?;
    check(config.extraction_policy_version == EXTRACTION_POLICY_VERSION)?;
    let decks: HashMap<_, _> = d
        .decks
        .iter()
        .map(|deck| (deck.id.as_str(), deck))
        .collect();
    let cards: HashMap<_, _> = d
        .cards
        .iter()
        .map(|card| (card.id.as_str(), card))
        .collect();
    if let Some(source) = &config.source_deck_id {
        identifier(source)?;
        check(
            decks
                .get(source.as_str())
                .is_some_and(|deck| deck.kind == DeckKind::Standard),
        )?;
    }
    check(config.source_deck_name.as_deref().is_none_or(name))?;

    let word_ids = unique_ids(data.words.iter().map(|word| word.card_id.as_str()))?;
    let actual: HashSet<_> = d
        .cards
        .iter()
        .filter(|card| card.deck_id == owner.id)
        .map(|card| card.id.as_str())
        .collect();
    check(word_ids == actual)?;
    let mut keys = HashSet::new();
    for word in &data.words {
        check(
            word.deck_id == owner.id
                && keys.insert(word.canonical.as_str())
                && canonical_word(&word.canonical).as_deref() == Some(word.canonical.as_str())
                && canonical_word(&word.display).as_deref() == Some(word.canonical.as_str())
                && word.display.nfc().eq(word.display.chars())
                && (word.canonical != "i" || word.display == "I"),
        )?;
        let card = cards
            .get(word.card_id.as_str())
            .ok_or_else(|| AppError::new("invalidBackup"))?;
        check(card.deck_id == word.deck_id)?;
        let [form] = card.forms.as_slice() else {
            return Err(AppError::new("invalidBackup"));
        };
        check(
            form.front == word.display
                && form.back == word.display
                && form.notes.is_empty()
                && form.language.as_deref() == Some("en")
                && form.question_title.is_none(),
        )?;
    }
    let words: HashMap<_, _> = data
        .words
        .iter()
        .map(|word| (word.card_id.as_str(), word))
        .collect();
    unique_ids(data.snapshots.iter().map(|snapshot| snapshot.id.as_str()))?;
    let mut fingerprints = HashSet::new();
    let mut membership = HashMap::new();
    for s in &data.snapshots {
        check(s.spelling_deck_id == owner.id && s.source_deck_id != owner.id)?;
        for id in [&s.source_deck_id, &s.source_card_id, &s.source_form_id] {
            identifier(id)?;
        }
        check(!s.source_card_revision.is_empty())?;
        at(&s.captured_at)?;
        check(
            name(&s.source_deck_name)
                && !s.front.trim().is_empty()
                && !s.back.trim().is_empty()
                && s.language
                    .as_ref()
                    .is_none_or(|language| !language.trim().is_empty())
                && normalize_question_title(s.question_title.clone())
                    .is_ok_and(|title| title == s.question_title)
                && s.extraction_policy_version == EXTRACTION_POLICY_VERSION
                && s.content_sha256 == snapshot_fingerprint(s)?
                && fingerprints.insert(s.content_sha256.as_str()),
        )?;
        if let Some(deck) = decks.get(s.source_deck_id.as_str()) {
            check(deck.kind == DeckKind::Standard)?;
        }
        if let Some(card) = cards.get(s.source_card_id.as_str()) {
            check(card.deck_id == s.source_deck_id)?;
        }
        // Extract once per shared snapshot, then prove each edge by set membership.
        let terms: HashSet<_> = [&s.front, &s.back, &s.notes]
            .into_iter()
            .flat_map(|text| extract_words(text))
            .map(|word| word.canonical)
            .collect();
        membership.insert(s.id.as_str(), terms);
    }
    let snapshots: HashMap<_, _> = data.snapshots.iter().map(|s| (s.id.as_str(), s)).collect();
    let mut edges = HashSet::new();
    let mut linked_words = HashSet::new();
    let mut linked_snapshots = HashSet::new();
    for o in &data.origins {
        for id in [
            &o.spelling_card_id,
            &o.source_card_id,
            &o.source_form_id,
            &o.snapshot_id,
        ] {
            identifier(id)?;
        }
        at(&o.first_seen_at)?;
        let word = words
            .get(o.spelling_card_id.as_str())
            .ok_or_else(|| AppError::new("invalidBackup"))?;
        let snapshot = snapshots
            .get(o.snapshot_id.as_str())
            .ok_or_else(|| AppError::new("invalidBackup"))?;
        check(
            snapshot.spelling_deck_id == word.deck_id
                && snapshot.source_card_id == o.source_card_id
                && snapshot.source_form_id == o.source_form_id
                && edges.insert((&o.spelling_card_id, &o.source_card_id, &o.source_form_id))
                && membership
                    .get(o.snapshot_id.as_str())
                    .is_some_and(|terms| terms.contains(&word.canonical)),
        )?;
        linked_words.insert(o.spelling_card_id.as_str());
        linked_snapshots.insert(o.snapshot_id.as_str());
    }
    check(linked_words == word_ids && linked_snapshots.len() == snapshots.len())?;
    let mut excluded = HashSet::new();
    for x in &data.exclusions {
        check(
            x.deck_id == owner.id
                && canonical_word(&x.canonical).as_deref() == Some(x.canonical.as_str())
                && !keys.contains(x.canonical.as_str())
                && excluded.insert(x.canonical.as_str()),
        )?;
        at(&x.excluded_at)?;
    }
    Ok(())
}
