use openrecall_core::{
    App, AppError, AppResult,
    contracts::{Request, Response},
};
use std::sync::atomic::{AtomicBool, Ordering};
#[derive(Default)]
pub struct Lifecycle {
    dirty: AtomicBool,
    installing: AtomicBool,
    close_prompt: AtomicBool,
}
impl Lifecycle {
    pub fn set_dirty(&self, value: bool) {
        self.dirty.store(value, Ordering::SeqCst);
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }
    pub fn ensure_available(&self) -> AppResult<()> {
        if self.installing.load(Ordering::SeqCst) {
            Err(AppError::new("updateInProgress"))
        } else {
            Ok(())
        }
    }
    pub fn begin_install(&self) -> AppResult<()> {
        self.installing
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map(|_| ())
            .map_err(|_| AppError::new("updateInProgress"))
    }
    pub fn finish_install(&self) {
        self.installing.store(false, Ordering::SeqCst);
    }
    pub fn begin_close_prompt(&self) -> bool {
        !self.close_prompt.swap(true, Ordering::SeqCst)
    }
    pub fn finish_close_prompt(&self) {
        self.close_prompt.store(false, Ordering::SeqCst);
    }
    pub fn ensure_replace_allowed(&self, app: &mut App) -> AppResult<()> {
        self.ensure_available()?;
        if self.is_dirty() {
            return Err(AppError::new("unsavedChanges"));
        }
        if let Response::Bootstrap(b) = app.execute(Request::Bootstrap {})?
            && !b.resumable_sessions.is_empty()
        {
            return Err(AppError::new("reviewInProgress"));
        }
        Ok(())
    }
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditState {
    pub unsaved_changes: bool,
}
#[tauri::command]
pub fn set_edit_state(input: EditState, state: tauri::State<'_, crate::commands::AppState>) {
    state.lifecycle.set_dirty(input.unsaved_changes);
}
