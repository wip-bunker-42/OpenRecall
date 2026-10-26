pub mod backup;
mod commands;
pub mod files;
pub mod lifecycle;
pub mod paths;
mod startup;
pub mod updates;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
pub fn run() {
    let builder = tauri::Builder::default();
    #[cfg(feature = "automation")]
    let builder = builder
        .plugin(tauri_plugin_wdio::init())
        .plugin(tauri_plugin_wdio_webdriver::init());
    builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let opened = (|| -> openrecall_core::AppResult<_> {
                let data = app
                    .path()
                    .app_data_dir()
                    .map_err(|_| openrecall_core::AppError::new("fileAccess"))?;
                #[cfg(feature = "automation")]
                let data = std::env::var_os("OPENRECALL_TEST_DATA")
                    .map(std::path::PathBuf::from)
                    .unwrap_or(data);
                let downloads = app.path().download_dir().ok();
                #[cfg(feature = "automation")]
                let downloads = std::env::var_os("OPENRECALL_TEST_DOWNLOADS")
                    .map(std::path::PathBuf::from)
                    .or(downloads);
                let paths = paths::NativePaths::from_roots(data, downloads)?;
                let core = openrecall_core::App::open(
                    &paths.database,
                    Arc::new(openrecall_core::SystemClock::default()),
                    Box::new(openrecall_core::SystemRandom::default()),
                )?;
                Ok((paths, core))
            })();
            let (paths, core) = match opened {
                Ok(opened) => opened,
                Err(error) => {
                    // No AppState is installed: study/file/update commands cannot
                    // accidentally operate on a substitute empty library.
                    startup::show_recovery(app.handle(), &error);
                    return Ok(());
                }
            };
            app.manage(commands::AppState {
                updates: Arc::new(updates::UpdateManager::new(env!("CARGO_PKG_VERSION"))),
                lifecycle: Arc::new(Default::default()),
                backup_files: Arc::new(Mutex::new(Default::default())),
                core: Arc::new(Mutex::new(core)),
                paths,
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let Some(state) = window.try_state::<commands::AppState>() else {
                    return;
                };
                if state.lifecycle.ensure_available().is_err() {
                    api.prevent_close();
                    return;
                }
                if state.lifecycle.is_dirty() {
                    api.prevent_close();
                    if state.lifecycle.begin_close_prompt() {
                        let window = window.clone();
                        let lifecycle = state.lifecycle.clone();
                        window
                            .app_handle()
                            .dialog()
                            .message("لديك تغييرات غير محفوظة. هل تريد تجاهلها وإغلاق التطبيق؟")
                            .title("تغييرات غير محفوظة")
                            .buttons(tauri_plugin_dialog::MessageDialogButtons::OkCancelCustom(
                                "تجاهل وإغلاق".into(),
                                "متابعة التعديل".into(),
                            ))
                            .show(move |discard| {
                                lifecycle.finish_close_prompt();
                                if discard {
                                    lifecycle.set_dirty(false);
                                    let _ = window.close();
                                }
                            });
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::execute,
            commands::get_system_info,
            files::pick_import_file,
            lifecycle::set_edit_state,
            backup::export_backup,
            backup::pick_backup_for_preview,
            backup::restore_backup,
            backup::reveal_export,
            updates::get_update_status,
            updates::check_for_update,
            updates::download_update,
            updates::install_update
        ])
        .run(tauri::generate_context!())
        .expect("Unable to start OpenRecall");
}
