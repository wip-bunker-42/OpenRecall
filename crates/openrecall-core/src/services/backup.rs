use crate::{
    App, AppError, AppResult,
    backup::{
        BackupLimits, PreparedRestore,
        format::{BackupEnvelope, FORMAT_VERSION, digest},
    },
    contracts::backup::*,
};
use std::path::Path;
impl App {
    pub fn export_snapshot(&mut self, destination: &Path) -> AppResult<ExportReport> {
        self.export_snapshot_with_limits(destination, BackupLimits::default())
    }
    pub fn export_snapshot_with_limits(
        &mut self,
        destination: &Path,
        limits: BackupLimits,
    ) -> AppResult<ExportReport> {
        self.store.begin()?;
        let data = self.store.finish(self.store.snapshot())?;
        let envelope = BackupEnvelope {
            format_version: FORMAT_VERSION,
            schema_version: crate::storage::SCHEMA_VERSION,
            app_version: env!("CARGO_PKG_VERSION").into(),
            exported_at: self.clock.now().to_rfc3339(),
            payload_sha256: digest(&data)?,
            data,
        };
        let (path, bytes) = crate::backup::files::write(destination, &envelope, limits)?;
        Ok(ExportReport {
            path: path.to_string_lossy().into_owned(),
            bytes,
            payload_sha256: envelope.payload_sha256,
        })
    }
    pub fn preview_restore(&self, source: &Path) -> AppResult<RestorePreview> {
        self.preview_restore_with_limits(source, BackupLimits::default())
    }
    pub fn preview_restore_with_limits(
        &self,
        source: &Path,
        limits: BackupLimits,
    ) -> AppResult<RestorePreview> {
        let envelope = crate::backup::files::read(source, limits)?;
        let d = &envelope.data;
        let preview = RestorePreview {
            id: uuid::Uuid::new_v4().to_string(),
            expected_library_revision: self.store.revision()?,
            exported_at: envelope.exported_at.clone(),
            cards: d.cards.len() as u32,
            forms: d.cards.iter().map(|c| c.forms.len() as u32).sum(),
            decks: d.decks.len() as u32,
            reviews: d.reviews.len() as u32,
            profiles: d.profiles.len() as u32,
            format_version: envelope.format_version,
        };
        let mut previews = self.restore_previews.borrow_mut();
        previews.clear();
        previews.insert(
            preview.id.clone(),
            PreparedRestore {
                preview: preview.clone(),
                envelope,
            },
        );
        Ok(preview)
    }
    pub fn restore_snapshot(
        &mut self,
        preview_id: String,
        expected_library_revision: String,
        safety_destination: &Path,
    ) -> AppResult<RestoreReport> {
        let (preview, mut envelope) = {
            let p = self.restore_previews.borrow();
            let p = p
                .get(&preview_id)
                .ok_or_else(|| AppError::new("previewExpired"))?;
            (p.preview.clone(), p.envelope.clone())
        };
        if preview.expected_library_revision != expected_library_revision
            || self.store.revision()? != expected_library_revision
        {
            return Err(AppError::new("conflict"));
        }
        let safety_backup = self.export_snapshot(safety_destination)?;
        for job in &mut envelope.data.training_runs {
            if ["queued", "running"].contains(&job.status.as_str()) {
                job.status = "interrupted".into();
                job.finished_at = Some(self.clock.now().to_rfc3339());
                job.error = Some(AppError::new("trainingInterrupted"));
            }
        }
        self.store.begin()?;
        let result = self.store.replace_snapshot(&envelope.data);
        self.store.finish(result)?;
        self.jobs = Default::default();
        self.import_previews.clear();
        self.restore_previews.borrow_mut().clear();
        self.runtime_id = uuid::Uuid::new_v4().to_string();
        Ok(RestoreReport {
            safety_backup,
            cards: preview.cards,
            decks: preview.decks,
        })
    }
}
