use crate::{
    App, AppResult, contracts::library::DeckDetail, domain::review::ReviewLog, training::dataset::*,
};
impl App {
    pub(crate) fn save_training_job(
        &self,
        job: &crate::contracts::training::TrainingJob,
    ) -> AppResult<()> {
        self.store.conn.execute("INSERT INTO training_runs(id,profile_id,payload) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",rusqlite::params![job.id,job.profile_id,serde_json::to_string(job)?])?;
        self.store.touch()
    }
    pub(crate) fn recover_training_jobs(&self) -> AppResult<()> {
        for mut job in self
            .store
            .all::<crate::contracts::training::TrainingJob>("training_runs")?
        {
            if ["queued", "running"].contains(&job.status.as_str()) {
                job.status = "interrupted".into();
                job.finished_at = Some(self.clock.now().to_rfc3339());
                job.error = Some(crate::AppError::new("trainingInterrupted"));
                self.save_training_job(&job)?;
            }
        }
        Ok(())
    }
    pub(crate) fn start_training(
        &mut self,
        profile_id: String,
    ) -> AppResult<crate::contracts::training::TrainingJob> {
        use crate::contracts::training::{JobProgress, TrainingJob};
        let profile = self.get_profile(&profile_id)?;
        self.guard_decks_idle(&profile.member_deck_ids)?;
        for existing in self.store.all::<TrainingJob>("training_runs")? {
            if existing.profile_id == profile_id
                && ["queued", "running"].contains(&existing.status.as_str())
            {
                return self.poll_training(existing.id);
            }
        }
        let dataset = self.profile_dataset(&profile_id)?;
        if !dataset.eligibility.eligible {
            return Err(crate::AppError::new("insufficientData"));
        }
        let job = TrainingJob {
            id: uuid::Uuid::new_v4().to_string(),
            profile_id,
            status: "queued".into(),
            progress: JobProgress::default(),
            eligibility: dataset.eligibility.clone(),
            config: dataset.context.config.clone(),
            source_revision: dataset.revision_token.clone(),
            baseline_model_revision: dataset.context.model_revision.clone(),
            member_deck_ids: dataset
                .context
                .members
                .iter()
                .map(|m| m.deck_id.clone())
                .collect(),
            started_at: self.clock.now().to_rfc3339(),
            finished_at: None,
            error: None,
            reason: None,
            cancel_requested: false,
            baseline_metrics: None,
            candidate_metrics: None,
            per_deck: vec![],
            candidate_parameters: None,
        };
        self.save_training_job(&job)?;
        self.jobs.pending.push((job.id.clone(), dataset));
        Ok(job)
    }
    pub(crate) fn poll_training(
        &mut self,
        job_id: String,
    ) -> AppResult<crate::contracts::training::TrainingJob> {
        use crate::contracts::training::TrainingJob;
        let mut job: TrainingJob = self.store.get("training_runs", &job_id)?;
        if ["queued", "running"].contains(&job.status.as_str()) {
            if let Some(worker) = self.jobs.running.get(&job_id) {
                let p = worker
                    .progress
                    .lock()
                    .map_err(|_| crate::AppError::new("trainingFailed"))?;
                job.progress.current = p
                    .splits
                    .iter()
                    .map(|s| s.current() as u64)
                    .fold(0, u64::saturating_add);
                job.progress.total = p
                    .splits
                    .iter()
                    .map(|s| s.total() as u64)
                    .fold(0, u64::saturating_add);
                job.progress.fraction = if job.progress.total == 0 {
                    0.0
                } else {
                    (job.progress.current as f64 / job.progress.total as f64).min(1.0)
                };
                let abort = p.want_abort;
                drop(p);
                let result = worker
                    .result
                    .lock()
                    .map_err(|_| crate::AppError::new("trainingFailed"))?
                    .clone();
                match result {
                    Some(Ok(result)) if !abort => {
                        job.status = if result.acceptable {
                            "ready"
                        } else {
                            "noImprovement"
                        }
                        .into();
                        job.reason = if !result.changed {
                            Some("unchangedParameters".into())
                        } else if !result.acceptable {
                            Some("heldoutDidNotImprove".into())
                        } else {
                            None
                        };
                        job.baseline_metrics = Some(result.baseline);
                        job.candidate_metrics = Some(result.candidate);
                        job.per_deck = result.per_deck;
                        job.candidate_parameters = Some(result.parameters);
                    }
                    Some(Err(error)) => {
                        job.status = if error.code == "cancelled" {
                            "cancelled"
                        } else {
                            "failed"
                        }
                        .into();
                        job.error = Some(error);
                    }
                    Some(Ok(_)) => {
                        job.status = "cancelled".into();
                        job.error = Some(crate::AppError::new("cancelled"));
                    }
                    None => job.status = "running".into(),
                }
                if job.status != "running" {
                    job.finished_at = Some(self.clock.now().to_rfc3339());
                    job.progress.fraction = 1.0;
                }
            } else {
                job.status = "interrupted".into();
                job.error = Some(crate::AppError::new("trainingInterrupted"));
                job.finished_at = Some(self.clock.now().to_rfc3339());
            }
        }
        if job.status == "ready"
            && self
                .profile_dataset(&job.profile_id)
                .is_ok_and(|data| data.revision_token != job.source_revision)
        {
            job.status = "failed".into();
            job.error = Some(crate::AppError::new("staleCandidate"));
        }
        self.save_training_job(&job)?;
        Ok(job)
    }
    pub(crate) fn cancel_training(
        &mut self,
        job_id: String,
    ) -> AppResult<crate::contracts::training::TrainingJob> {
        let mut job: crate::contracts::training::TrainingJob =
            self.store.get("training_runs", &job_id)?;
        if ["queued", "running"].contains(&job.status.as_str()) {
            if let Some(worker) = self.jobs.running.get(&job_id) {
                worker
                    .progress
                    .lock()
                    .map_err(|_| crate::AppError::new("trainingFailed"))?
                    .want_abort = true;
            }
            job.cancel_requested = true;
            self.save_training_job(&job)?;
        }
        self.poll_training(job_id)
    }
    pub(crate) fn apply_training(
        &mut self,
        job_id: String,
        expected: String,
    ) -> AppResult<crate::domain::profile::ProfileDetail> {
        use crate::{AppError, domain::profile::ModelRevision};
        let job: crate::contracts::training::TrainingJob =
            self.store.get("training_runs", &job_id)?;
        let profile = self.require_profile_revision(&job.profile_id, &expected)?;
        let data = self.profile_dataset(&profile.id)?;
        if data.revision_token != job.source_revision {
            return Err(AppError::new("staleCandidate"));
        }
        if job.status != "ready" {
            return Err(AppError::new("trainingNotReady"));
        }
        let parameters = job
            .candidate_parameters
            .clone()
            .ok_or_else(|| AppError::new("trainingNotReady"))?;
        let baseline =
            crate::training::evaluate::score_targets(&profile.parameters, &data.holdout_targets)?;
        let candidate =
            crate::training::evaluate::score_targets(&parameters, &data.holdout_targets)?;
        if parameters == profile.parameters
            || parameters == fsrs::DEFAULT_PARAMETERS
            || !crate::training::evaluate::candidate_acceptable(baseline, candidate)
        {
            return Err(AppError::new("trainingNotReady"));
        }
        let model = ModelRevision {
            id: uuid::Uuid::new_v4().to_string(),
            parameters,
            origin: "trained".into(),
            created_at: self.clock.now().to_rfc3339(),
            contributing_deck_ids: profile.member_deck_ids.clone(),
            training_run_id: Some(job.id.clone()),
            model_version: crate::domain::profile::model_version(),
            engine_revision: crate::domain::profile::engine_revision(),
        };
        let profile = self.publish_model(profile, model)?;
        let mut job = job;
        job.status = "applied".into();
        self.save_training_job(&job)?;
        Ok(profile)
    }
    pub(crate) fn list_training_runs(
        &self,
        profile_id: String,
    ) -> AppResult<Vec<crate::contracts::training::TrainingJob>> {
        self.get_profile(&profile_id)?;
        Ok(self
            .store
            .all::<crate::contracts::training::TrainingJob>("training_runs")?
            .into_iter()
            .filter(|j| j.profile_id == profile_id)
            .collect())
    }
    pub(crate) fn profile_dataset(&self, profile_id: &str) -> AppResult<TrainingDataset> {
        let profile = self.get_profile(profile_id)?;
        let settings = self.store.settings()?;
        let mut members = vec![];
        let mut observations = vec![];
        for deck_id in &profile.member_deck_ids {
            let deck: DeckDetail = self.store.get("decks", deck_id)?;
            members.push(TrainingMember {
                deck_id: deck.id,
                deck_name: deck.name,
                data_revision: deck.data_revision,
            });
            let mut stmt = self
                .store
                .conn
                .prepare("SELECT payload FROM reviews WHERE deck_id=?1 ORDER BY at_ms,rowid")?;
            for row in stmt.query_map([deck_id], |r| r.get::<_, String>(0))? {
                let log: ReviewLog = serde_json::from_str(&row?)?;
                observations.push(TrainingObservation {
                    card_id: log.card_id,
                    deck_id: log.deck_id,
                    reviewed_at: log.reviewed_at,
                    rating: log.rating,
                    undone: false,
                    deleted: false,
                });
            }
        }
        build_dataset(
            &observations,
            TrainingContext {
                profile_id: profile.id,
                members,
                config: profile.config,
                timezone: settings.timezone,
                day_start_minute: settings.day_start_minute,
                baseline_parameters: profile.parameters,
                model_revision: profile.active_model_revision,
            },
        )
    }
}
