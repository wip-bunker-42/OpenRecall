use crate::{
    App, AppError, AppResult,
    contracts::review::CopyQuestionInput,
    domain::{
        card::{CardDetail, CardForm},
        questions::QuestionType,
        session::Session,
    },
};

impl App {
    pub fn question_copy_text(&self, input: &CopyQuestionInput) -> AppResult<String> {
        let session: Session = self.store.get("sessions", &input.session_id)?;
        if session.status != "active" {
            return Err(AppError::new("sessionFinished"));
        }
        if let Some(feedback) = &session.choice_feedback {
            if feedback.id != input.display_id
                || feedback.card_revision != input.expected_card_revision
            {
                return Err(AppError::new("conflict"));
            }
            return format_question(&feedback.form, &feedback.choices.order);
        }
        let p = session
            .current
            .as_ref()
            .ok_or_else(|| AppError::new("noQuestion"))?;
        if p.id != input.display_id || p.card_revision != input.expected_card_revision {
            return Err(AppError::new("conflict"));
        }
        let card: CardDetail = self.store.get("cards", &p.card_id)?;
        if card.revision != p.card_revision {
            return Err(AppError::new("conflict"));
        }
        let form = card
            .forms
            .iter()
            .find(|f| f.id == p.form_id)
            .ok_or_else(|| AppError::new("noForms"))?;
        format_question(
            form,
            p.choices.as_ref().map_or(&[], |choices| &choices.order),
        )
    }
}

fn format_question(form: &CardForm, order: &[String]) -> AppResult<String> {
    match form.question_type {
        QuestionType::Essay => Ok(form.front.clone()),
        QuestionType::Recall => Err(AppError::new("copyNotApplicable")),
        QuestionType::MultipleChoice => {
            let set = form
                .multiple_choice
                .as_ref()
                .ok_or_else(|| AppError::new("noChoices"))?;
            if !crate::domain::questions::is_choice_permutation(set, order) {
                return Err(AppError::new("invalidChoiceSelection"));
            }
            let lines: Vec<_> = order
                .iter()
                .enumerate()
                .map(|(index, id)| {
                    let choice = set
                        .choices
                        .iter()
                        .find(|c| &c.id == id)
                        .expect("validated permutation");
                    format!("{}. {}", index + 1, choice.text)
                })
                .collect();
            Ok(format!("{}\n\n{}", form.front, lines.join("\n")))
        }
    }
}
