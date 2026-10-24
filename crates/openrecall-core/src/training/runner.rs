use super::dataset::TrainingDataset;
use crate::{
    AppError, AppResult,
    contracts::training::{DeckEvaluation, EvaluationMetrics},
    domain::profile::TrainingConfig,
};
use std::sync::{Arc, Mutex};
#[derive(Clone, Debug)]
pub struct TrainingCandidate {
    pub parameters: Vec<f32>,
    pub baseline: EvaluationMetrics,
    pub candidate: EvaluationMetrics,
    pub per_deck: Vec<DeckEvaluation>,
    pub acceptable: bool,
    pub changed: bool,
}
pub fn train_snapshot(
    dataset: TrainingDataset,
    config: TrainingConfig,
    progress: Arc<Mutex<fsrs::CombinedProgressState>>,
) -> AppResult<TrainingCandidate> {
    if !dataset.eligibility.eligible {
        return Err(AppError::new("insufficientData"));
    }
    config.validate()?;
    if config != dataset.context.config {
        return Err(AppError::new("staleConfiguration"));
    }
    if progress
        .lock()
        .map_err(|_| AppError::new("trainingFailed"))?
        .want_abort
    {
        return Err(AppError::new("cancelled"));
    }
    let baseline = super::evaluate::score(
        &dataset.context.baseline_parameters,
        &dataset.holdout_targets,
        Some(&progress),
    )?;
    let result = fsrs::compute_parameters(fsrs::ComputeParametersInput {
        train_set: dataset.train_items,
        card_ids: Some(dataset.train_card_ids),
        progress: Some(progress.clone()),
        enable_short_term: true,
        enable_sched_penalties: config.schedule_penalties,
        model_version: fsrs::ComputeParametersVersion::Fsrs7,
        num_relearning_steps: None,
        training_config: Some(fsrs::TrainingConfig {
            num_epochs: config.num_epochs.unwrap_or(0) as usize,
            batch_size: config.batch_size as usize,
            seed: config.seed as u64,
            learning_rate: config.learning_rate.unwrap_or(0.0),
            max_seq_len: config.max_seq_len as usize,
            gamma: config.gamma,
        }),
    });
    if progress
        .lock()
        .map_err(|_| AppError::new("trainingFailed"))?
        .want_abort
    {
        return Err(AppError::new("cancelled"));
    }
    let parameters = result.map_err(|_| AppError::new("trainingFailed"))?;
    crate::scheduling::fsrs::FsrsScheduler::new(&parameters)?;
    let candidate = super::evaluate::score(&parameters, &dataset.holdout_targets, Some(&progress))?;
    let mut per_deck = vec![];
    for member in &dataset.context.members {
        let targets = dataset
            .holdout_targets
            .iter()
            .filter(|t| t.deck_id == member.deck_id)
            .cloned()
            .collect::<Vec<_>>();
        per_deck.push(DeckEvaluation {
            deck_id: member.deck_id.clone(),
            observations: targets.len() as u32,
            baseline: if targets.is_empty() {
                None
            } else {
                Some(super::evaluate::score(
                    &dataset.context.baseline_parameters,
                    &targets,
                    Some(&progress),
                )?)
            },
            candidate: if targets.is_empty() {
                None
            } else {
                Some(super::evaluate::score(
                    &parameters,
                    &targets,
                    Some(&progress),
                )?)
            },
        });
    }
    let changed =
        parameters != dataset.context.baseline_parameters && parameters != fsrs::DEFAULT_PARAMETERS;
    let acceptable = changed && super::evaluate::candidate_acceptable(baseline, candidate);
    Ok(TrainingCandidate {
        parameters,
        baseline,
        candidate,
        per_deck,
        acceptable,
        changed,
    })
}
