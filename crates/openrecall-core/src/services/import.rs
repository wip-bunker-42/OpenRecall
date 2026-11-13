use super::library::id;
use crate::{
    App, AppError, AppResult, FieldError,
    contracts::{
        cards::{CardDraft, FormDraft},
        import::*,
    },
    domain::{
        card::CardForm,
        questions::{
            ChoiceSetDraft, QuestionType, SelectionMode, normalized, validate_choice_draft,
        },
    },
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
pub(crate) fn fingerprint(front: &str, back: &str) -> String {
    let front = normalized(front);
    let back = normalized(back);
    let mut hash = Sha256::new();
    for s in [&front, &back] {
        hash.update((s.len() as u64).to_be_bytes());
        hash.update(s.as_bytes());
    }
    hash.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

fn typed_fingerprint(
    kind: QuestionType,
    front: &str,
    back: &str,
    mode: SelectionMode,
    mut options: Vec<(String, bool)>,
) -> String {
    if kind == QuestionType::Recall {
        return fingerprint(front, back);
    }
    options.sort_unstable();
    let content = serde_json::json!([kind, normalized(front), mode, options]).to_string();
    Sha256::digest(content.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn form_fingerprint(form: &CardForm) -> String {
    let (mode, options) = form
        .multiple_choice
        .as_ref()
        .map(|set| {
            (
                set.selection_mode,
                set.choices
                    .iter()
                    .map(|c| (normalized(&c.text), set.correct_choice_ids.contains(&c.id)))
                    .collect(),
            )
        })
        .unwrap_or_default();
    typed_fingerprint(form.question_type, &form.front, &form.back, mode, options)
}

fn preview_fingerprint(form: &ImportForm) -> String {
    let (mode, options) = form
        .multiple_choice
        .as_ref()
        .map(|set| {
            (
                set.selection_mode,
                set.choices
                    .iter()
                    .map(|c| {
                        (
                            normalized(&c.text),
                            set.correct_choice_keys.contains(&c.key),
                        )
                    })
                    .collect(),
            )
        })
        .unwrap_or_default();
    typed_fingerprint(form.question_type, &form.front, &form.back, mode, options)
}
fn issue(field: String, code: &str) -> FieldError {
    FieldError {
        field,
        code: code.into(),
    }
}
fn parse_form(
    value: &Value,
    path: &str,
    key: String,
    inherited: Option<QuestionType>,
) -> ImportForm {
    let mut form = ImportForm {
        question_type: inherited.unwrap_or_default(),
        multiple_choice: None,
        question_title: None,
        key,
        front: String::new(),
        back: String::new(),
        notes: String::new(),
        valid: true,
        duplicate: false,
        duplicate_of: None,
        issues: vec![],
        warnings: vec![],
    };
    let Some(object) = value.as_object() else {
        form.valid = false;
        form.issues.push(issue(path.into(), "objectExpected"));
        return form;
    };
    if let Some(kind) = object.get("questionType") {
        match serde_json::from_value(kind.clone()) {
            Ok(kind) => form.question_type = kind,
            Err(_) => form
                .issues
                .push(issue(format!("{path}.questionType"), "invalidQuestionType")),
        }
    }
    let recall = form.question_type == QuestionType::Recall;
    for (name, destination, required) in [
        ("front", &mut form.front, true),
        ("back", &mut form.back, recall),
        ("notes", &mut form.notes, false),
    ] {
        match object.get(name) {
            Some(Value::String(text)) => {
                *destination = text.clone();
                if required && text.trim().is_empty() {
                    form.issues
                        .push(issue(format!("{path}.{name}"), "required"));
                }
            }
            None if !required => {}
            None => form
                .issues
                .push(issue(format!("{path}.{name}"), "required")),
            _ => form
                .issues
                .push(issue(format!("{path}.{name}"), "stringExpected")),
        }
    }
    if !recall {
        if form.back.trim().is_empty() {
            form.back.clear();
        } else {
            form.issues
                .push(issue(format!("{path}.back"), "unexpectedAnswer"));
        }
    }
    let choice_fields = [
        "choices",
        "correctChoices",
        "selectionMode",
        "shuffleChoices",
    ];
    if form.question_type == QuestionType::MultipleChoice {
        let mut draft = serde_json::json!({
            "choices": object.get("choices"),
            "correctChoiceKeys": object.get("correctChoices")
        });
        for name in ["selectionMode", "shuffleChoices"] {
            if let Some(field) = object.get(name) {
                draft[name] = field.clone();
            }
        }
        match serde_json::from_value::<ChoiceSetDraft>(draft) {
            Ok(set) => {
                if set.choices.iter().any(|c| c.id.is_some()) {
                    form.issues
                        .push(issue(format!("{path}.choices"), "invalidOwner"));
                }
                if let Err(error) = validate_choice_draft(&set) {
                    form.issues.extend(error.fields.into_iter().map(|field| {
                        let name = field.field.trim_start_matches("multipleChoice.");
                        let name = if name == "correctChoiceKeys" {
                            "correctChoices"
                        } else {
                            name
                        };
                        issue(format!("{path}.{name}"), &field.code)
                    }));
                }
                form.multiple_choice = Some(set);
            }
            Err(_) => form
                .issues
                .push(issue(format!("{path}.choices"), "invalidChoices")),
        }
    } else if choice_fields.iter().any(|name| object.contains_key(*name)) {
        form.issues
            .push(issue(format!("{path}.choices"), "unexpectedChoices"));
    }
    if inherited.is_some() && object.contains_key("variants") {
        form.issues
            .push(issue(format!("{path}.variants"), "nestedVariants"));
    }
    let title = match object.get("questionTitle") {
        Some(Value::String(text)) => {
            crate::domain::card::normalize_question_title(Some(text.clone()))
                .map_err(|_| "invalidQuestionTitle")
        }
        None | Some(Value::Null) => Ok(None),
        _ => Err("stringExpected"),
    };
    match title {
        Ok(title) => form.question_title = title,
        Err(code) => form
            .issues
            .push(issue(format!("{path}.questionTitle"), code)),
    }
    for name in object.keys() {
        if ![
            "front",
            "back",
            "notes",
            "variants",
            "questionTitle",
            "questionType",
        ]
        .contains(&name.as_str())
            && !choice_fields.contains(&name.as_str())
        {
            form.warnings
                .push(issue(format!("{path}.{name}"), "unknownField"));
        }
    }
    form.valid = form.issues.is_empty();
    form
}
fn mark_duplicate(form: &mut ImportForm, seen: &mut HashMap<String, DuplicateRef>, add: bool) {
    if !form.valid {
        return;
    }
    let key = preview_fingerprint(form);
    if let Some(reference) = seen.get(&key) {
        form.duplicate = true;
        form.duplicate_of = Some(reference.clone());
    } else if add {
        seen.insert(
            key,
            DuplicateRef {
                scope: "file".into(),
                card_id: None,
                deck_id: None,
                form_id: None,
                source_key: Some(form.key.clone()),
            },
        );
    }
}
impl App {
    fn existing_forms(&self) -> AppResult<HashMap<String, DuplicateRef>> {
        let mut result = HashMap::new();
        let mut stmt=self.store.conn.prepare("SELECT c.id,c.deck_id,f.value FROM cards c JOIN decks d ON d.id=c.deck_id,json_each(c.payload,'$.forms') f WHERE d.kind='standard'")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;
        for row in rows {
            let (card, deck, raw) = row?;
            let form: CardForm = serde_json::from_str(&raw)?;
            result
                .entry(form_fingerprint(&form))
                .or_insert(DuplicateRef {
                    scope: "library".into(),
                    card_id: Some(card),
                    deck_id: Some(deck),
                    form_id: Some(form.id),
                    source_key: None,
                });
        }
        Ok(result)
    }
    pub(crate) fn preview_import(
        &mut self,
        deck_id: String,
        text: String,
    ) -> AppResult<ImportPreview> {
        self.require_manual_card_deck(&deck_id)?;
        if text.len() > 50 * 1024 * 1024 {
            return Err(AppError::new("importTooLarge"));
        }
        let parsed: Value = serde_json::from_str(text.strip_prefix('\u{feff}').unwrap_or(&text))
            .map_err(|e| {
                let mut error = AppError::new("invalidJson");
                error.context = Some(format!("{}:{}", e.line(), e.column()));
                error
            })?;
        let rows = parsed
            .as_array()
            .ok_or_else(|| AppError::new("arrayExpected"))?;
        let mut seen = self.existing_forms()?;
        let mut cards = vec![];
        let mut warnings = vec![];
        let mut counts = ImportCounts::default();
        for (i, row) in rows.iter().enumerate() {
            let key = format!("card-{i}");
            let path = format!("$[{i}]");
            let mut primary = parse_form(row, &path, format!("{key}-primary"), None);
            let mut issues = vec![];
            if row
                .as_object()
                .is_some_and(|o| o.get("variants").is_some_and(|v| !v.is_array()))
            {
                issues.push(issue(format!("{path}.variants"), "arrayExpected"));
            }
            mark_duplicate(&mut primary, &mut seen, issues.is_empty());
            let valid = primary.valid && !primary.duplicate && issues.is_empty();
            let mut variants = vec![];
            if let Some(items) = row.get("variants").and_then(Value::as_array) {
                for (v, item) in items.iter().enumerate() {
                    let mut form = parse_form(
                        item,
                        &format!("{path}.variants[{v}]"),
                        format!("{key}-variant-{v}"),
                        Some(primary.question_type),
                    );
                    if item
                        .as_object()
                        .is_some_and(|o| !o.contains_key("questionTitle"))
                    {
                        form.question_title = primary.question_title.clone();
                    }
                    mark_duplicate(&mut form, &mut seen, valid);
                    variants.push(form);
                }
            }
            if valid {
                counts.valid_cards += 1;
                counts.selected_cards += 1;
            } else if primary.duplicate {
                counts.duplicate_cards += 1;
            } else {
                counts.invalid_cards += 1;
            }
            for form in std::iter::once(&primary).chain(variants.iter()) {
                if !form.valid {
                    counts.invalid_forms += 1;
                } else if form.duplicate {
                    counts.duplicate_forms += 1;
                } else {
                    counts.valid_forms += 1;
                }
                warnings.extend(form.warnings.clone());
            }
            cards.push(ImportCard {
                key,
                primary,
                variants,
                valid,
                issues,
            });
        }
        counts.warnings = warnings.len() as u32;
        let preview = ImportPreview {
            preview_id: id(),
            deck_id,
            library_revision: self.store.revision()?,
            cards,
            counts,
            warnings,
        };
        self.import_previews.clear();
        self.import_previews
            .insert(preview.preview_id.clone(), preview.clone());
        Ok(preview)
    }
    pub(crate) fn commit_import(&mut self, input: CommitImportInput) -> AppResult<ImportReport> {
        let preview = self
            .import_previews
            .get(&input.preview_id)
            .cloned()
            .ok_or_else(|| AppError::new("previewExpired"))?;
        self.require_manual_card_deck(&preview.deck_id)?;
        if input.library_revision != preview.library_revision
            || self.store.revision()? != preview.library_revision
        {
            return Err(AppError::new("conflict"));
        }
        let selected: HashSet<_> = input.selected_card_keys.iter().cloned().collect();
        let excluded: HashSet<_> = input.excluded_form_keys.iter().cloned().collect();
        if selected.is_empty() {
            return Err(AppError::new("noSelection"));
        }
        if selected.len() != input.selected_card_keys.len()
            || selected
                .iter()
                .any(|key| !preview.cards.iter().any(|c| &c.key == key && c.valid))
            || excluded.iter().any(|key| {
                !preview
                    .cards
                    .iter()
                    .any(|c| c.variants.iter().any(|v| &v.key == key))
            })
        {
            return Err(AppError::new("invalidSelection"));
        }
        let mut seen = self.existing_forms()?;
        let mut drafts = vec![];
        let mut inserted_forms = 0u32;
        let mut excluded_forms = 0u32;
        for card in &preview.cards {
            if !selected.contains(&card.key) {
                continue;
            }
            let mut forms = vec![];
            for form in std::iter::once(&card.primary).chain(card.variants.iter()) {
                if excluded.contains(&form.key) {
                    excluded_forms += 1;
                    continue;
                }
                if !form.valid || form.duplicate {
                    return Err(AppError::new("invalidSelection"));
                }
                let key = preview_fingerprint(form);
                if seen.contains_key(&key) {
                    return Err(AppError::new("conflict"));
                }
                seen.insert(
                    key,
                    DuplicateRef {
                        scope: "selected".into(),
                        card_id: None,
                        deck_id: None,
                        form_id: None,
                        source_key: Some(form.key.clone()),
                    },
                );
                forms.push(FormDraft {
                    question_type: form.question_type,
                    multiple_choice: form.multiple_choice.clone(),
                    question_title: form.question_title.clone(),
                    id: None,
                    front: form.front.clone(),
                    back: form.back.clone(),
                    notes: form.notes.clone(),
                    language: None,
                });
                inserted_forms += 1;
            }
            drafts.push(CardDraft { forms });
        }
        let now = self.clock.now();
        let inserted_cards = drafts.len() as u32;
        let spelling = crate::spelling::storage::config(&self.store.conn)?
            .filter(|config| config.source_deck_id.as_deref() == Some(preview.deck_id.as_str()));
        let before = self.store.conn.total_changes();
        let mut collected = 0;
        for draft in drafts {
            let card = self.create_card_at(preview.deck_id.clone(), draft, now)?;
            if spelling.is_some() {
                collected += self.collect_spelling_content(&card, true)?;
            }
        }
        if let Some(config) = spelling {
            self.finish_spelling_collection(&config.deck_id, collected, before)?;
        }
        self.import_previews.remove(&input.preview_id);
        Ok(ImportReport {
            inserted_cards,
            inserted_forms,
            excluded_cards: preview.cards.len() as u32 - inserted_cards,
            excluded_forms,
        })
    }
}
