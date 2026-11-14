use super::reviews::GradeDestination;
use crate::{
    App, AppError, AppResult,
    contracts::review::{ChoiceSubmission, ConfirmChoicesInput, ReviewView},
    domain::{
        card::CardDetail,
        questions::{ChoiceSet, SelectionMode},
        review::{ActionRecord, ChoiceOutcome},
        session::Session,
    },
    scheduling::fsrs::Rating,
};
use std::collections::HashSet;

pub(crate) fn validate_submission(
    set: &ChoiceSet,
    submission: &ChoiceSubmission,
) -> AppResult<ChoiceOutcome> {
    let ChoiceSubmission::Selected { choice_ids } = submission else {
        return Ok(ChoiceOutcome::Unknown);
    };
    let chosen: HashSet<_> = choice_ids.iter().collect();
    if chosen.is_empty()
        || chosen.len() != choice_ids.len()
        || (set.selection_mode == SelectionMode::Single && chosen.len() != 1)
        || chosen
            .iter()
            .any(|id| !set.choices.iter().any(|choice| &choice.id == *id))
    {
        return Err(AppError::new("invalidChoiceSelection"));
    }
    Ok(if chosen == set.correct_choice_ids.iter().collect() {
        ChoiceOutcome::Correct
    } else {
        ChoiceOutcome::Incorrect
    })
}

impl App {
    pub(crate) fn confirm_choices(
        &mut self,
        mut input: ConfirmChoicesInput,
    ) -> AppResult<ReviewView> {
        if uuid::Uuid::parse_str(&input.action_id).is_err() {
            return Err(AppError::field("actionId", "invalidId"));
        }
        if let ChoiceSubmission::Selected { choice_ids } = &mut input.submission {
            choice_ids.sort_unstable();
        }
        let signature = serde_json::to_string(&input)?;
        if let Some(view) = self.retry_action(&input.action_id, &signature)? {
            return Ok(view);
        }
        let mut session: Session = self.store.get("sessions", &input.session_id)?;
        self.checkpoint(&mut session);
        if session.status != "active" {
            return Err(AppError::new("sessionFinished"));
        }
        let p = session
            .current
            .as_mut()
            .ok_or_else(|| AppError::new("noQuestion"))?;
        if p.id != input.presentation_id {
            return Err(AppError::new("invalidPresentation"));
        }
        let card: CardDetail = self.store.get("cards", &p.card_id)?;
        if card.revision != input.expected_card_revision || card.revision != p.card_revision {
            return Err(AppError::new("conflict"));
        }
        let set = card
            .forms
            .iter()
            .find(|f| f.id == p.form_id)
            .and_then(|f| f.multiple_choice.as_ref())
            .ok_or_else(|| AppError::new("noChoices"))?;
        let choices = p
            .choices
            .as_mut()
            .ok_or_else(|| AppError::new("noChoices"))?;
        if choices.confirmed_correct {
            return Err(AppError::new("choiceAlreadyConfirmed"));
        }
        let outcome = validate_submission(set, &input.submission)?;
        choices.selected_ids = match input.submission {
            ChoiceSubmission::Selected { choice_ids } => choice_ids,
            ChoiceSubmission::Unknown => vec![],
        };
        let action = ActionRecord {
            id: input.action_id,
            session_id: session.id.clone(),
            signature,
            status: "applied".into(),
            choice_outcome: Some(outcome),
        };
        if outcome == ChoiceOutcome::Correct {
            choices.confirmed_correct = true;
            self.save_action(&action)?;
            self.session_view(&mut session)
        } else {
            self.apply_rating(
                session,
                card,
                Rating::Again,
                action,
                GradeDestination::Feedback {
                    unknown: outcome == ChoiceOutcome::Unknown,
                },
            )
        }
    }

    pub(crate) fn continue_choice_feedback(
        &mut self,
        session_id: String,
        feedback_id: String,
    ) -> AppResult<ReviewView> {
        let mut session: Session = self.store.get("sessions", &session_id)?;
        if let Some(feedback) = &session.choice_feedback {
            if feedback.id != feedback_id {
                return Err(AppError::new("invalidFeedback"));
            }
            session.choice_feedback = None;
        }
        self.checkpoint(&mut session);
        self.session_view(&mut session)
    }
}
