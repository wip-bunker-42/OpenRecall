use openrecall_core::{AppError, AppResult, contracts::system::PickedImportFile};
use std::{io::Read, path::Path};
use tauri_plugin_dialog::DialogExt;
pub fn read_import_file_with_limit(path: &Path, max_bytes: u64) -> AppResult<PickedImportFile> {
    let file = std::fs::File::open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() {
        return Err(AppError::new("fileAccess"));
    }
    if meta.len() > max_bytes {
        return Err(AppError::new("importTooLarge"));
    }
    let mut bytes = Vec::new();
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(AppError::new("importTooLarge"));
    }
    let text = String::from_utf8(bytes).map_err(|_| AppError::new("invalidUtf8"))?;
    Ok(PickedImportFile {
        name: path
            .file_name()
            .ok_or_else(|| AppError::new("fileAccess"))?
            .to_string_lossy()
            .into_owned(),
        text: text.strip_prefix('\u{feff}').unwrap_or(&text).to_owned(),
    })
}
#[tauri::command]
pub async fn pick_import_file(app: tauri::AppHandle) -> AppResult<Option<PickedImportFile>> {
    tauri::async_runtime::spawn_blocking(move || {
        let Some(path) = app
            .dialog()
            .file()
            .set_title("اختيار ملف بطاقات JSON")
            .add_filter("JSON", &["json"])
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        let path = path.into_path().map_err(|_| AppError::new("fileAccess"))?;
        Ok(Some(read_import_file_with_limit(&path, 50 * 1024 * 1024)?))
    })
    .await
    .map_err(|_| AppError::new("nativeWorker"))?
}
