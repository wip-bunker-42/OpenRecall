pub mod backup;
mod commands;
pub mod files;
pub mod lifecycle;
pub mod paths;
use std::sync::{Arc, Mutex};
use tauri::Manager;
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let data = app.path().app_data_dir()?;
            #[cfg(feature = "automation")]
            let data = std::env::var_os("OPENRECALL_TEST_DATA")
                .map(std::path::PathBuf::from)
                .unwrap_or(data);
            let paths = paths::NativePaths::from_roots(data, app.path().download_dir().ok())?;
            let core = openrecall_core::App::open(
                &paths.database,
                Arc::new(openrecall_core::SystemClock::default()),
                Box::new(openrecall_core::SystemRandom::default()),
            )?;
            app.manage(commands::AppState {
                lifecycle: Arc::new(Default::default()),
                backup_files: Arc::new(Mutex::new(Default::default())),
                core: Arc::new(Mutex::new(core)),
                paths,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::execute,
            commands::get_system_info,
            files::pick_import_file,
            lifecycle::set_edit_state,
            backup::export_backup,
            backup::pick_backup_for_preview,
            backup::restore_backup,
            backup::reveal_export
        ])
        .run(tauri::generate_context!())
        .expect("Unable to start OpenRecall");
}
