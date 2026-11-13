use super::{
    extract::{EXTRACTION_POLICY_VERSION, extract_words},
    storage,
};
use crate::{
    App, AppError, AppResult,
    contracts::library::DeckDetail,
    domain::{
        card::{CardDetail, CardForm},
        questions::{ChoiceSet, QuestionType},
        spelling::*,
    },
};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub(crate) fn snapshot_fingerprint(s: &SourceSnapshot) -> AppResult<String> {
    let content = (
        &s.spelling_deck_id,
        &s.source_deck_id,
        &s.source_deck_name,
        &s.source_card_id,
        &s.source_form_id,
        s.source_form_index,
        s.extraction_policy_version,
        &s.front,
        &s.back,
        &s.notes,
        &s.question_title,
        &s.language,
    );
    let bytes = if s.content_version == 1 {
        serde_json::to_vec(&content)?
    } else {
        serde_json::to_vec(&(
            "openrecall-source-context-v2",
            content,
            s.question_type,
            &s.multiple_choice,
        ))?
    };
    Ok(Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

fn projected_texts<'a>(
    front: &'a str,
    back: &'a str,
    notes: &'a str,
    choices: Option<&'a ChoiceSet>,
) -> Vec<&'a str> {
    let mut texts = vec![front, back, notes];
    if let Some(set) = choices {
        texts.extend(
            set.choices
                .iter()
                .filter(|choice| set.correct_choice_ids.contains(&choice.id))
                .map(|choice| choice.text.as_str()),
        );
    }
    texts
}

pub(crate) fn eligible_texts(form: &CardForm) -> Vec<&str> {
    projected_texts(
        &form.front,
        &form.back,
        &form.notes,
        form.multiple_choice.as_ref(),
    )
}

pub(crate) fn snapshot_eligible_texts(snapshot: &SourceSnapshot) -> Vec<&str> {
    projected_texts(
        &snapshot.front,
        &snapshot.back,
        &snapshot.notes,
        snapshot.multiple_choice.as_ref(),
    )
}

impl App {
    /// Content-only collection; the caller owns the atomic command and final touch.
    pub(crate) fn collect_spelling_content(
        &mut self,
        source_card: &CardDetail,
        collect_new: bool,
    ) -> AppResult<u32> {
        let Some(config) = storage::config(&self.store.conn)? else {
            return Ok(0);
        };
        if config.extraction_policy_version != EXTRACTION_POLICY_VERSION {
            return Err(AppError::new("unsupportedExtractionPolicy"));
        }
        let collect_new =
            collect_new && config.source_deck_id.as_deref() == Some(source_card.deck_id.as_str());
        let source: DeckDetail = self.store.get("decks", &source_card.deck_id)?;
        let now = self.clock.now().to_rfc3339();
        let mut added = 0;
        for (index, form) in source_card.forms.iter().enumerate() {
            let mut seen = HashSet::new();
            let terms = eligible_texts(form)
                .into_iter()
                .flat_map(|text| extract_words(text))
                .filter(|word| seen.insert(word.canonical.clone()))
                .collect::<Vec<_>>();
            let mut snapshot: Option<SourceSnapshot> = None;
            for term in terms {
                if storage::is_excluded(&self.store.conn, &config.deck_id, &term.canonical)? {
                    continue;
                }
                let word = match storage::word(&self.store.conn, &config.deck_id, &term.canonical)?
                {
                    Some(word) => {
                        if !collect_new
                            && !storage::has_origin(
                                &self.store.conn,
                                &word.card_id,
                                &source_card.id,
                                &form.id,
                            )?
                        {
                            continue;
                        }
                        word
                    }
                    None if collect_new => {
                        let word = SpellingWord {
                            card_id: id(),
                            deck_id: config.deck_id.clone(),
                            canonical: term.canonical,
                            display: term.display,
                        };
                        let card = CardDetail {
                            previous_choice_orders: Default::default(),
                            bag: Default::default(),
                            id: word.card_id.clone(),
                            deck_id: word.deck_id.clone(),
                            forms: vec![CardForm {
                                question_type: QuestionType::Recall,
                                multiple_choice: None,
                                question_title: None,
                                id: id(),
                                front: word.display.clone(),
                                back: word.display.clone(),
                                notes: String::new(),
                                language: Some("en".into()),
                            }],
                            phase: "new".into(),
                            memory: None,
                            due_at: now.clone(),
                            created_at: now.clone(),
                            reviewed_at: None,
                            review_count: 0,
                            lapse_count: 0,
                            has_graduated: false,
                            revision: id(),
                        };
                        self.save_card(&card)?;
                        storage::save_word(&self.store.conn, &word)?;
                        added += 1;
                        word
                    }
                    None => continue,
                };
                if snapshot.is_none() {
                    let mut captured = SourceSnapshot {
                        content_version: if form.question_type == QuestionType::Recall {
                            1
                        } else {
                            2
                        },
                        question_type: form.question_type,
                        multiple_choice: form.multiple_choice.clone(),
                        id: id(),
                        spelling_deck_id: config.deck_id.clone(),
                        source_deck_id: source.id.clone(),
                        source_deck_name: source.name.clone(),
                        source_card_id: source_card.id.clone(),
                        source_form_id: form.id.clone(),
                        source_form_index: index as u32,
                        source_card_revision: source_card.revision.clone(),
                        extraction_policy_version: EXTRACTION_POLICY_VERSION,
                        captured_at: now.clone(),
                        front: form.front.clone(),
                        back: form.back.clone(),
                        notes: form.notes.clone(),
                        question_title: form.question_title.clone(),
                        language: form.language.clone(),
                        content_sha256: String::new(),
                    };
                    captured.content_sha256 = snapshot_fingerprint(&captured)?;
                    snapshot = Some(
                        match storage::snapshot_by_hash(
                            &self.store.conn,
                            &config.deck_id,
                            &captured.content_sha256,
                        )? {
                            Some(existing) => existing,
                            None => {
                                storage::save_snapshot(&self.store.conn, &captured)?;
                                captured
                            }
                        },
                    );
                }
                let captured = snapshot.as_ref().ok_or_else(|| AppError::new("storage"))?;
                storage::save_origin(
                    &self.store.conn,
                    &WordOrigin {
                        spelling_card_id: word.card_id,
                        source_card_id: source_card.id.clone(),
                        source_form_id: form.id.clone(),
                        snapshot_id: captured.id.clone(),
                        first_seen_at: now.clone(),
                    },
                )?;
            }
        }
        Ok(added)
    }
}
