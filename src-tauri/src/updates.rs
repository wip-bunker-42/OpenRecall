use openrecall_core::{AppError, AppResult, contracts::updates::UpdateStatus};
use std::{
    future::Future,
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::Emitter;
use tauri_plugin_updater::{Update, UpdaterExt};
pub type StatusSink = Arc<dyn Fn(UpdateStatus) + Send + Sync>;
pub type DownloadProgress = Arc<dyn Fn(usize, Option<u64>) + Send + Sync>;
pub struct Candidate<C> {
    pub version: String,
    pub notes: Option<String>,
    pub value: C,
}
struct UpdateState<C> {
    status: UpdateStatus,
    candidate: Option<C>,
    bytes: Option<Vec<u8>>,
    busy: bool,
}
pub struct UpdateManager<C> {
    state: Arc<Mutex<UpdateState<C>>>,
}
impl<C: Clone + Send + 'static> UpdateManager<C> {
    pub fn new(current_version: &str) -> Self {
        Self {
            state: Arc::new(Mutex::new(UpdateState {
                status: UpdateStatus {
                    phase: "idle".into(),
                    current_version: current_version.into(),
                    available_version: None,
                    notes: None,
                    downloaded_bytes: 0,
                    total_bytes: None,
                    error_code: None,
                },
                candidate: None,
                bytes: None,
                busy: false,
            })),
        }
    }
    pub fn status(&self) -> AppResult<UpdateStatus> {
        Ok(self
            .state
            .lock()
            .map_err(|_| AppError::new("updateBusy"))?
            .status
            .clone())
    }
    fn fail(&self, error: AppError, emit: &StatusSink) -> AppError {
        if let Ok(mut s) = self.state.lock() {
            s.status.phase = "failed".into();
            s.status.error_code = Some(error.code.clone());
            s.busy = false;
            s.bytes = None;
        }
        if let Ok(s) = self.status() {
            emit(s)
        }
        error
    }
    pub async fn check<F: Future<Output = AppResult<Option<Candidate<C>>>>>(
        &self,
        check: impl FnOnce() -> F,
        emit: StatusSink,
    ) -> AppResult<UpdateStatus> {
        {
            let mut s = self.state.lock().map_err(|_| AppError::new("updateBusy"))?;
            if s.busy {
                return Err(AppError::new("updateBusy"));
            }
            s.busy = true;
            s.candidate = None;
            s.bytes = None;
            s.status.phase = "checking".into();
            s.status.error_code = None;
            s.status.available_version = None;
            s.status.notes = None;
            s.status.downloaded_bytes = 0;
            s.status.total_bytes = None;
        }
        emit(self.status()?);
        let candidate = check().await.map_err(|e| self.fail(e, &emit))?;
        {
            let mut s = self.state.lock().map_err(|_| AppError::new("updateBusy"))?;
            s.busy = false;
            if let Some(c) = candidate {
                s.status.phase = "available".into();
                s.status.available_version = Some(c.version);
                s.status.notes = c.notes;
                s.candidate = Some(c.value);
            } else {
                s.status.phase = "upToDate".into();
            }
        }
        let status = self.status()?;
        emit(status.clone());
        Ok(status)
    }
    pub async fn download<F: Future<Output = AppResult<Vec<u8>>>>(
        &self,
        download: impl FnOnce(C, DownloadProgress) -> F,
        verify: impl FnOnce(&C, &[u8]) -> AppResult<()>,
        emit: StatusSink,
    ) -> AppResult<UpdateStatus> {
        let candidate = {
            let mut s = self.state.lock().map_err(|_| AppError::new("updateBusy"))?;
            if s.busy {
                return Err(AppError::new("updateBusy"));
            }
            let c = s
                .candidate
                .clone()
                .ok_or_else(|| AppError::new("updateNotReady"))?;
            s.busy = true;
            s.bytes = None;
            s.status.phase = "downloading".into();
            s.status.downloaded_bytes = 0;
            s.status.total_bytes = None;
            s.status.error_code = None;
            c
        };
        emit(self.status()?);
        let state = self.state.clone();
        let events = emit.clone();
        let progress: DownloadProgress = Arc::new(move |chunk, total| {
            if let Ok(mut s) = state.lock() {
                s.status.downloaded_bytes = s.status.downloaded_bytes.saturating_add(chunk as u64);
                s.status.total_bytes = total;
                let status = s.status.clone();
                drop(s);
                events(status)
            }
        });
        let bytes = download(candidate.clone(), progress)
            .await
            .map_err(|e| self.fail(e, &emit))?;
        verify(&candidate, &bytes).map_err(|e| self.fail(e, &emit))?;
        {
            let mut s = self.state.lock().map_err(|_| AppError::new("updateBusy"))?;
            s.status.downloaded_bytes = bytes.len() as u64;
            s.status.phase = "ready".into();
            s.bytes = Some(bytes);
            s.busy = false;
        }
        let status = self.status()?;
        emit(status.clone());
        Ok(status)
    }
    pub fn install(
        &self,
        prepare: impl FnOnce() -> AppResult<()>,
        install: impl FnOnce(C, Vec<u8>) -> AppResult<()>,
    ) -> AppResult<()> {
        let (candidate, bytes) = {
            let mut s = self.state.lock().map_err(|_| AppError::new("updateBusy"))?;
            if s.busy {
                return Err(AppError::new("updateBusy"));
            }
            if s.status.phase != "ready" {
                return Err(AppError::new("updateNotReady"));
            }
            let c = s
                .candidate
                .clone()
                .ok_or_else(|| AppError::new("updateNotReady"))?;
            let bytes = s
                .bytes
                .clone()
                .ok_or_else(|| AppError::new("updateNotReady"))?;
            s.busy = true;
            s.status.phase = "installing".into();
            (c, bytes)
        };
        if let Err(e) = prepare() {
            let mut s = self.state.lock().map_err(|_| AppError::new("updateBusy"))?;
            s.busy = false;
            s.status.phase = "ready".into();
            return Err(e);
        }
        if let Err(e) = install(candidate, bytes) {
            let mut s = self.state.lock().map_err(|_| AppError::new("updateBusy"))?;
            s.busy = false;
            s.status.phase = "ready".into();
            return Err(e);
        }
        Ok(())
    }
}
fn sink(app: tauri::AppHandle) -> StatusSink {
    Arc::new(move |status| {
        let _ = app.emit_to("main", "openrecall-update", status);
    })
}
#[tauri::command]
pub fn get_update_status(
    state: tauri::State<'_, crate::commands::AppState>,
) -> AppResult<UpdateStatus> {
    state.updates.status()
}
#[tauri::command]
pub async fn check_for_update(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::commands::AppState>,
) -> AppResult<UpdateStatus> {
    let events = sink(app.clone());
    state
        .updates
        .check(
            || async move {
                let update = app
                    .updater_builder()
                    .timeout(Duration::from_secs(30))
                    .build()
                    .map_err(|_| AppError::new("updateCheckFailed"))?
                    .check()
                    .await
                    .map_err(|_| AppError::new("updateCheckFailed"))?;
                Ok(update.map(|mut u| {
                    u.timeout = Some(Duration::from_secs(600));
                    Candidate {
                        version: u.version.clone(),
                        notes: u.body.clone(),
                        value: u,
                    }
                }))
            },
            events,
        )
        .await
}
#[tauri::command]
pub async fn download_update(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::commands::AppState>,
) -> AppResult<UpdateStatus> {
    state
        .updates
        .download(
            |update, progress| async move {
                update
                    .download(move |chunk, total| progress(chunk, total), || {})
                    .await
                    .map_err(|e| match e {
                        tauri_plugin_updater::Error::Minisign(_)
                        | tauri_plugin_updater::Error::MissingSignedVersion
                        | tauri_plugin_updater::Error::SignedVersionMismatch { .. } => {
                            AppError::new("updateVerificationFailed")
                        }
                        _ => AppError::new("updateDownloadFailed"),
                    })
            },
            |update, bytes| {
                update
                    .verify(bytes)
                    .map_err(|_| AppError::new("updateVerificationFailed"))
            },
            sink(app),
        )
        .await
}
#[tauri::command]
pub async fn install_update(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::commands::AppState>,
) -> AppResult<()> {
    let updates = state.updates.clone();
    let core = state.core.clone();
    let lifecycle = state.lifecycle.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut core = core.lock().map_err(|_| AppError::new("storageBusy"))?;
        let result = updates.install(
            || {
                lifecycle.ensure_replace_allowed(&mut core)?;
                core.prepare_for_restart()?;
                lifecycle.begin_install()
            },
            |update: Update, bytes| {
                update
                    .install(bytes)
                    .map_err(|_| AppError::new("updateInstallFailed"))
            },
        );
        if result.is_err() {
            lifecycle.finish_install();
        } else {
            #[cfg(not(windows))]
            app.restart();
        }
        #[cfg(windows)]
        let _ = app;
        result
    })
    .await
    .map_err(|_| AppError::new("nativeWorker"))?
}
