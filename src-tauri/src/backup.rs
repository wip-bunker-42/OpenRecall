use crate::{commands::AppState, paths::NativePaths};
use openrecall_core::{
    App, AppError, AppResult,
    contracts::backup::{ExportReport, RestorePreview, RestoreReport},
};
use std::{collections::HashMap, path::PathBuf};
use tauri_plugin_dialog::DialogExt;
#[derive(Default)]
pub struct BackupFiles {
    exports: HashMap<String, PathBuf>,
}
impl BackupFiles {
    fn remember(&mut self, report: &ExportReport) {
        self.exports
            .insert(report.export_id.clone(), PathBuf::from(&report.path));
    }
    pub fn resolve_export(&self, id: &str) -> AppResult<PathBuf> {
        self.exports
            .get(id)
            .filter(|p| p.is_file())
            .cloned()
            .ok_or_else(|| AppError::new("unknownExport"))
    }
    pub fn export(
        &mut self,
        core: &mut App,
        paths: &NativePaths,
        name: &str,
        fallback: impl FnOnce() -> AppResult<Option<PathBuf>>,
    ) -> AppResult<ExportReport> {
        let report = with_destination(paths, name, fallback, |p| core.export_snapshot(p))?;
        self.remember(&report);
        Ok(report)
    }
    pub fn restore(
        &mut self,
        core: &mut App,
        paths: &NativePaths,
        id: String,
        revision: String,
        name: &str,
        fallback: impl FnOnce() -> AppResult<Option<PathBuf>>,
    ) -> AppResult<RestoreReport> {
        let report = with_destination(paths, name, fallback, |p| {
            core.restore_snapshot(id.clone(), revision.clone(), p)
        })?;
        self.remember(&report.safety_backup);
        Ok(report)
    }
}
fn with_destination<T>(
    paths: &NativePaths,
    name: &str,
    fallback: impl FnOnce() -> AppResult<Option<PathBuf>>,
    mut write: impl FnMut(&std::path::Path) -> AppResult<T>,
) -> AppResult<T> {
    if let Some(downloads) = &paths.downloads {
        match write(&downloads.join(name)) {
            Ok(report) => return Ok(report),
            Err(e) if e.code == "fileAccess" => {}
            Err(e) => return Err(e),
        }
    }
    let mut path = fallback()?.ok_or_else(|| AppError::new("cancelled"))?;
    if path.extension().is_none() {
        path.set_extension("json");
    }
    write(&path)
}
fn name(prefix: &str) -> String {
    format!(
        "OpenRecall-{prefix}-{}.json",
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    )
}
fn save_dialog(app: &tauri::AppHandle, name: &str) -> AppResult<Option<PathBuf>> {
    app.dialog()
        .file()
        .set_title("اختيار مكان حفظ النسخة الاحتياطية")
        .set_file_name(name)
        .add_filter("JSON", &["json"])
        .blocking_save_file()
        .map(|p| p.into_path().map_err(|_| AppError::new("fileAccess")))
        .transpose()
}
#[tauri::command]
pub async fn export_backup(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<ExportReport> {
    let core = state.core.clone();
    let paths = state.paths.clone();
    let files = state.backup_files.clone();
    let lifecycle = state.lifecycle.clone();
    tauri::async_runtime::spawn_blocking(move || {
        if lifecycle.is_dirty() {
            return Err(AppError::new("unsavedChanges"));
        }
        let mut core = core.lock().map_err(|_| AppError::new("storageBusy"))?;
        lifecycle.ensure_available()?;
        let mut files = files.lock().map_err(|_| AppError::new("storageBusy"))?;
        let filename = name("backup");
        files.export(&mut core, &paths, &filename, || {
            save_dialog(&app, &filename)
        })
    })
    .await
    .map_err(|_| AppError::new("nativeWorker"))?
}
#[tauri::command]
pub async fn pick_backup_for_preview(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<Option<RestorePreview>> {
    let core = state.core.clone();
    let lifecycle = state.lifecycle.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut core = core.lock().map_err(|_| AppError::new("storageBusy"))?;
        lifecycle.ensure_replace_allowed(&mut core)?;
        let Some(path) = app
            .dialog()
            .file()
            .set_title("اختيار نسخة OpenRecall الاحتياطية")
            .add_filter("JSON", &["json"])
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        let path = path.into_path().map_err(|_| AppError::new("fileAccess"))?;
        Ok(Some(core.preview_restore(&path)?))
    })
    .await
    .map_err(|_| AppError::new("nativeWorker"))?
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RestoreInput {
    pub preview_id: String,
    pub expected_library_revision: String,
}
#[tauri::command]
pub async fn restore_backup(
    input: RestoreInput,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<RestoreReport> {
    let core = state.core.clone();
    let paths = state.paths.clone();
    let files = state.backup_files.clone();
    let lifecycle = state.lifecycle.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut core = core.lock().map_err(|_| AppError::new("storageBusy"))?;
        lifecycle.ensure_replace_allowed(&mut core)?;
        let filename = name("before-restore");
        files
            .lock()
            .map_err(|_| AppError::new("storageBusy"))?
            .restore(
                &mut core,
                &paths,
                input.preview_id,
                input.expected_library_revision,
                &filename,
                || save_dialog(&app, &filename),
            )
    })
    .await
    .map_err(|_| AppError::new("nativeWorker"))?
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RevealInput {
    pub export_id: String,
}
#[tauri::command]
pub async fn reveal_export(input: RevealInput, state: tauri::State<'_, AppState>) -> AppResult<()> {
    let files = state.backup_files.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let path = files
            .lock()
            .map_err(|_| AppError::new("storageBusy"))?
            .resolve_export(&input.export_id)?;
        tauri_plugin_opener::reveal_item_in_dir(path).map_err(|_| AppError::new("fileAccess"))
    })
    .await
    .map_err(|_| AppError::new("nativeWorker"))?
}
