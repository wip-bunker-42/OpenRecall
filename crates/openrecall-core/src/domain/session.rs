use super::{card::CardForm, questions::QuestionType, spelling::DeckKind};
use crate::{contracts::review::FilterChange, scheduling::queue::ReviewFilter};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChoicePresentation {
    pub order: Vec<String>,
    #[serde(default)]
    pub selected_ids: Vec<String>,
    #[serde(default)]
    pub confirmed_correct: bool,
}

pub(crate) fn manual_grade_allowed(
    deck: DeckKind,
    form: &CardForm,
    presentation: &Presentation,
) -> bool {
    deck == DeckKind::Spelling
        || match form.question_type {
            QuestionType::Essay => true,
            QuestionType::Recall => presentation.answer_revealed,
            QuestionType::MultipleChoice => presentation
                .choices
                .as_ref()
                .is_some_and(|choices| choices.confirmed_correct),
        }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    #[serde(default)]
    pub choices: Option<ChoicePresentation>,
    pub id: String,
    pub card_id: String,
    pub form_id: String,
    pub answer_revealed: bool,
    pub card_revision: String,
    pub active_ms: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    #[serde(default)]
    pub choice_feedback: Option<ChoiceFeedback>,
    #[serde(default)]
    pub last_review: Option<crate::contracts::review::ReviewResult>,
    pub id: String,
    pub deck_id: String,
    pub filter: ReviewFilter,
    pub filter_changes: Vec<FilterChange>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: String,
    pub current: Option<Presentation>,
    pub parked: Vec<Presentation>,
    pub attempts: u32,
    pub unique_card_ids: Vec<String>,
    pub new_card_ids: Vec<String>,
    pub ratings: [u32; 4],
    pub active_ms: u64,
    pub response_ms: u64,
    pub clock_checkpoint: u64,
    pub is_active: bool,
    pub runtime_id: String,
    pub keep_empty_filter: bool,
    pub undo_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChoiceFeedback {
    pub id: String,
    pub review_id: String,
    pub action_id: String,
    pub card_id: String,
    pub card_revision: String,
    pub form: CardForm,
    pub choices: ChoicePresentation,
    pub unknown: bool,
}
