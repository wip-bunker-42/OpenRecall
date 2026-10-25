use crate::{
    App, AppError, AppResult,
    contracts::{Request, Response, training::TrainingJob},
};
impl App {
    pub fn prepare_for_restart(&mut self) -> AppResult<()> {
        let mut active = false;
        for job in self.store.all::<TrainingJob>("training_runs")? {
            if ["queued", "running"].contains(&job.status.as_str())
                && let Response::GetTrainingJob(latest) =
                    self.execute(Request::GetTrainingJob { job_id: job.id })?
            {
                active |= ["queued", "running"].contains(&latest.status.as_str());
            }
        }
        if active {
            return Err(AppError::new("trainingInProgress"));
        }
        let busy: i64 = self
            .store
            .conn
            .query_row("PRAGMA wal_checkpoint(FULL)", [], |row| row.get(0))?;
        if busy != 0 {
            return Err(AppError::new("storageBusy"));
        }
        Ok(())
    }
}
