use openrecall_core::{
    App, AppError, AppResult,
    contracts::{Request, Response},
};
use std::sync::atomic::{AtomicBool, Ordering};
#[derive(Default)]
pub struct Lifecycle {
    dirty: AtomicBool,
}
impl Lifecycle {
    pub fn set_dirty(&self, value: bool) {
        self.dirty.store(value, Ordering::SeqCst);
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }
    pub fn ensure_replace_allowed(&self, app: &mut App) -> AppResult<()> {
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
