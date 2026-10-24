use crate::{
    AppError, AppResult,
    training::{
        dataset::TrainingDataset,
        runner::{TrainingCandidate, train_snapshot},
    },
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
pub(crate) struct Worker {
    pub progress: Arc<Mutex<fsrs::CombinedProgressState>>,
    pub result: Arc<Mutex<Option<AppResult<TrainingCandidate>>>>,
}
#[derive(Default)]
pub(crate) struct JobRegistry {
    pub running: HashMap<String, Worker>,
    pub pending: Vec<(String, TrainingDataset)>,
}
impl JobRegistry {
    pub fn launch(&mut self) {
        for (id, dataset) in self.pending.drain(..) {
            let progress = fsrs::CombinedProgressState::new_shared();
            let result = Arc::new(Mutex::new(None));
            let thread_progress = progress.clone();
            let thread_result = result.clone();
            let spawned = std::thread::Builder::new()
                .name("openrecall-training".into())
                .spawn(move || {
                    let config = dataset.context.config.clone();
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        train_snapshot(dataset, config, thread_progress)
                    }))
                    .unwrap_or_else(|_| Err(AppError::new("trainingFailed")));
                    if let Ok(mut slot) = thread_result.lock() {
                        *slot = Some(outcome);
                    }
                });
            if spawned.is_err()
                && let Ok(mut slot) = result.lock()
            {
                *slot = Some(Err(AppError::new("trainingFailed")));
            }
            self.running.insert(id, Worker { progress, result });
        }
    }
    pub fn cancel_all(&self) {
        for worker in self.running.values() {
            if let Ok(mut progress) = worker.progress.lock() {
                progress.want_abort = true;
            }
        }
    }
}
impl Drop for JobRegistry {
    fn drop(&mut self) {
        self.cancel_all();
    }
}
