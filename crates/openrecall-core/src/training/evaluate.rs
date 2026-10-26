use super::dataset::HeldOutTarget;
use crate::{AppError, AppResult, contracts::training::EvaluationMetrics};
pub fn score_targets(
    parameters: &[f32],
    targets: &[HeldOutTarget],
) -> AppResult<EvaluationMetrics> {
    score(parameters, targets, None)
}
pub(crate) fn score(
    parameters: &[f32],
    targets: &[HeldOutTarget],
    abort: Option<&std::sync::Arc<std::sync::Mutex<fsrs::CombinedProgressState>>>,
) -> AppResult<EvaluationMetrics> {
    if targets.is_empty() {
        return Err(AppError::new("insufficientData"));
    }
    let scheduler = crate::scheduling::fsrs::FsrsScheduler::new(parameters)?;
    let mut log_loss = 0.0;
    let mut square_error = 0.0;
    for target in targets {
        if let Some(progress) = abort
            && progress
                .lock()
                .map_err(|_| AppError::new("trainingFailed"))?
                .want_abort
        {
            return Err(AppError::new("cancelled"));
        }
        if !target.elapsed_days.is_finite() || target.elapsed_days < 0.0 {
            return Err(AppError::new("invalidHistory"));
        }
        let memory = scheduler
            .replay(&target.history)?
            .ok_or_else(|| AppError::new("invalidHistory"))?;
        let raw = scheduler.engine.current_retrievability(
            fsrs::MemoryState {
                stability: memory.stability,
                stability_fast: memory.stability_fast,
                difficulty: memory.difficulty,
            },
            target.elapsed_days,
        ) as f64;
        if !raw.is_finite() || !(0.0..=1.0).contains(&raw) {
            return Err(AppError::new("invalidPrediction"));
        }
        let outcome = if target.rating == crate::scheduling::fsrs::Rating::Again {
            0.0
        } else {
            1.0
        };
        let p = raw.clamp(1e-7, 1.0 - 1e-7);
        log_loss -= outcome * p.ln() + (1.0 - outcome) * (1.0 - p).ln();
        square_error += (outcome - raw).powi(2);
    }
    Ok(EvaluationMetrics {
        log_loss: log_loss / targets.len() as f64,
        rmse: (square_error / targets.len() as f64).sqrt(),
    })
}
pub fn candidate_acceptable(baseline: EvaluationMetrics, candidate: EvaluationMetrics) -> bool {
    [
        baseline.log_loss,
        candidate.log_loss,
        baseline.rmse,
        candidate.rmse,
    ]
    .iter()
    .all(|v| v.is_finite() && *v >= 0.0)
        && baseline.rmse <= 1.0
        && candidate.rmse <= 1.0
        && baseline.log_loss - candidate.log_loss >= 0.0001 - 1e-12
        && candidate.rmse - baseline.rmse <= 0.0001 + 1e-12
}
