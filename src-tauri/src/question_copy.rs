use crate::commands::AppState;
use openrecall_core::{App, AppError, AppResult, contracts::review::CopyQuestionInput};
use tauri_plugin_clipboard_manager::ClipboardExt;

fn copy_text(
    core: &App,
    input: &CopyQuestionInput,
    write: impl FnOnce(String) -> AppResult<()>,
) -> AppResult<()> {
    write(core.question_copy_text(input)?)
}

#[tauri::command]
pub async fn copy_question(
    input: CopyQuestionInput,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<()> {
    let core = state.core.clone();
    let lifecycle = state.lifecycle.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let core = core.lock().map_err(|_| AppError::new("storageBusy"))?;
        lifecycle.ensure_available()?;
        copy_text(&core, &input, |text| {
            app.clipboard()
                .write_text(text)
                .map_err(|_| AppError::new("clipboardWrite"))
        })
    })
    .await
    .map_err(|_| AppError::new("nativeWorker"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use openrecall_core::{SystemClock, SystemRandom};
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn writes_only_validated_text_and_surfaces_writer_failure() {
        let dir = tempfile::tempdir().unwrap();
        let mut core = App::open(
            &dir.path().join("library.sqlite"),
            Arc::new(SystemClock::default()),
            Box::new(SystemRandom::default()),
        )
        .unwrap();
        let deck = core
            .execute_json(json!({"kind":"createDeck","payload":{"name":"Copy test"}}))
            .unwrap();
        let deck_id = &deck["payload"]["id"];
        core.execute_json(json!({"kind":"createCard","payload":{"deckId":deck_id,
            "draft":{"forms":[{"questionType":"essay","front":"Explain this.\nSecond line"}]}}}))
            .unwrap();
        let session = core
            .execute_json(json!({"kind":"startSession","payload":{"deckId":deck_id}}))
            .unwrap();
        let frame = &session["payload"]["payload"];
        let input: CopyQuestionInput = serde_json::from_value(json!({
            "sessionId":frame["sessionId"],"displayId":frame["presentationId"],
            "expectedCardRevision":frame["cardRevision"]
        }))
        .unwrap();
        let mut writes = Vec::new();
        copy_text(&core, &input, |text| {
            writes.push(text);
            Ok(())
        })
        .unwrap();
        assert_eq!(writes, ["Explain this.\nSecond line"]);
        let mut stale = input.clone();
        stale.display_id.clear();
        assert!(
            copy_text(&core, &stale, |_| panic!(
                "stale display wrote to clipboard"
            ))
            .is_err()
        );
        assert_eq!(
            copy_text(&core, &input, |_| Err(AppError::new("clipboardWrite")))
                .unwrap_err()
                .code,
            "clipboardWrite"
        );
        assert_eq!(core.question_copy_text(&input).unwrap(), writes[0]);
    }
}
