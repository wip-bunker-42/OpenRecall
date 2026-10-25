use crate::paths::NativePaths;
use openrecall_core::{
    App, AppError, AppResult,
    contracts::{Request, Response, system::SystemInfo},
};
use std::sync::{Arc, Mutex};
pub struct AppState {
    pub lifecycle: Arc<crate::lifecycle::Lifecycle>,
    pub backup_files: Arc<Mutex<crate::backup::BackupFiles>>,
    pub core: Arc<Mutex<App>>,
    pub paths: NativePaths,
}
#[tauri::command]
pub async fn execute(request: Request, state: tauri::State<'_, AppState>) -> AppResult<Response> {
    let core = state.core.clone();
    tauri::async_runtime::spawn_blocking(move || {
        core.lock()
            .map_err(|_| AppError::new("storageBusy"))?
            .execute(request)
    })
    .await
    .map_err(|_| AppError::new("nativeWorker"))?
}
#[tauri::command]
pub fn get_system_info(state: tauri::State<'_, AppState>) -> SystemInfo {
    SystemInfo {
        app_version: env!("CARGO_PKG_VERSION").into(),
        engine_version: "FSRS-7".into(),
        engine_revision: openrecall_core::domain::profile::engine_revision(),
        schema_version: openrecall_core::SCHEMA_VERSION,
        data_directory: state.paths.data.to_string_lossy().into_owned(),
    }
}
