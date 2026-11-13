use super::card::CardForm;
use crate::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use ts_rs::TS;
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum QuestionType {
    #[default]
    Recall,
    Essay,
    MultipleChoice,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum SelectionMode {
    #[default]
    Single,
    Multiple,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Choice {
    pub id: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChoiceSet {
    pub choices: Vec<Choice>,
    pub correct_choice_ids: Vec<String>,
    pub selection_mode: SelectionMode,
    pub shuffle_choices: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChoiceDraft {
    pub id: Option<String>,
    pub key: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChoiceSetDraft {
    pub choices: Vec<ChoiceDraft>,
    pub correct_choice_keys: Vec<String>,
    #[serde(default)]
    pub selection_mode: SelectionMode,
    #[serde(default = "default_shuffle")]
    pub shuffle_choices: bool,
}

fn default_shuffle() -> bool {
    true
}

pub(crate) fn normalized(text: &str) -> String {
    text.nfc()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn validate_choices<'a>(
    choices: impl ExactSizeIterator<Item = (&'a str, &'a str)>,
    correct: &[String],
    selection_mode: SelectionMode,
) -> AppResult<()> {
    if !(2..=128).contains(&choices.len()) {
        return Err(AppError::field("multipleChoice.choices", "choiceCount"));
    }
    let mut keys = HashSet::new();
    let mut texts = HashSet::new();
    for (index, (key, text)) in choices.enumerate() {
        if key.trim().is_empty()
            || key.len() > 128
            || key.chars().any(char::is_control)
            || !keys.insert(key)
        {
            return Err(AppError::field(
                &format!("multipleChoice.choices.{index}.key"),
                "invalidChoiceKey",
            ));
        }
        if text.trim().is_empty() || text.len() > 16 * 1024 || !texts.insert(normalized(text)) {
            return Err(AppError::field(
                &format!("multipleChoice.choices.{index}.text"),
                "invalidChoiceText",
            ));
        }
    }
    let mut correct_keys = HashSet::new();
    if correct.is_empty()
        || (selection_mode == SelectionMode::Single && correct.len() != 1)
        || correct
            .iter()
            .any(|key| !keys.contains(key.as_str()) || !correct_keys.insert(key))
    {
        return Err(AppError::field(
            "multipleChoice.correctChoiceKeys",
            "invalidChoiceAnswer",
        ));
    }
    Ok(())
}

pub(crate) fn validate_choice_draft(draft: &ChoiceSetDraft) -> AppResult<()> {
    validate_choices(
        draft
            .choices
            .iter()
            .map(|c| (c.key.as_str(), c.text.as_str())),
        &draft.correct_choice_keys,
        draft.selection_mode,
    )
}

pub(crate) fn resolve_choices(
    draft: ChoiceSetDraft,
    previous: Option<&ChoiceSet>,
) -> AppResult<ChoiceSet> {
    validate_choice_draft(&draft)?;
    let mut used_ids = HashSet::new();
    let mut ids_by_key = HashMap::new();
    let mut choices = Vec::with_capacity(draft.choices.len());
    for choice in draft.choices {
        let id = match choice.id {
            None => uuid::Uuid::new_v4().to_string(),
            Some(id) if previous.is_some_and(|set| set.choices.iter().any(|c| c.id == id)) => id,
            Some(_) => return Err(AppError::field("multipleChoice.choices", "invalidOwner")),
        };
        if !used_ids.insert(id.clone()) {
            return Err(AppError::field("multipleChoice.choices", "duplicateId"));
        }
        ids_by_key.insert(choice.key, id.clone());
        choices.push(Choice {
            id,
            text: choice.text,
        });
    }
    let correct_choice_ids = draft
        .correct_choice_keys
        .iter()
        .map(|key| ids_by_key[key].clone())
        .collect();
    Ok(ChoiceSet {
        choices,
        correct_choice_ids,
        selection_mode: draft.selection_mode,
        shuffle_choices: draft.shuffle_choices,
    })
}

pub(crate) fn validate_form(form: &CardForm) -> AppResult<()> {
    validate_content(
        form.question_type,
        &form.front,
        &form.back,
        form.multiple_choice.as_ref(),
    )
}

pub(crate) fn validate_content(
    kind: QuestionType,
    front: &str,
    back: &str,
    choices: Option<&ChoiceSet>,
) -> AppResult<()> {
    if front.trim().is_empty() {
        return Err(AppError::field("front", "required"));
    }
    match kind {
        QuestionType::Recall if back.trim().is_empty() => Err(AppError::field("back", "required")),
        QuestionType::Essay | QuestionType::MultipleChoice if !back.is_empty() => {
            Err(AppError::field("back", "unexpectedAnswer"))
        }
        QuestionType::MultipleChoice => {
            let set = choices.ok_or_else(|| AppError::field("multipleChoice", "required"))?;
            if set
                .choices
                .iter()
                .any(|c| uuid::Uuid::parse_str(&c.id).is_err())
            {
                return Err(AppError::field("multipleChoice.choices", "invalidId"));
            }
            validate_choices(
                set.choices.iter().map(|c| (c.id.as_str(), c.text.as_str())),
                &set.correct_choice_ids,
                set.selection_mode,
            )
        }
        _ if choices.is_some() => Err(AppError::field("multipleChoice", "unexpectedChoices")),
        _ => Ok(()),
    }
}
