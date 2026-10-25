use openrecall_core::{App, AppError, SystemClock, SystemRandom};
use openrecall_desktop_lib::{backup::BackupFiles, lifecycle::Lifecycle, paths::NativePaths};
use std::sync::Arc;
fn core(path: &std::path::Path) -> App {
    App::open(
        path,
        Arc::new(SystemClock::default()),
        Box::new(SystemRandom::default()),
    )
    .unwrap()
}
#[test]
fn downloads_collision_fallback_cancellation_and_known_export_reveal() {
    let dir = tempfile::tempdir().unwrap();
    let downloads = dir.path().join("Downloads");
    std::fs::create_dir(&downloads).unwrap();
    let paths = NativePaths::from_roots(dir.path().join("data"), Some(downloads.clone())).unwrap();
    let mut app = core(&paths.database);
    let mut files = BackupFiles::default();
    let a = files
        .export(&mut app, &paths, "library.json", || {
            panic!("Downloads available")
        })
        .unwrap();
    let b = files
        .export(&mut app, &paths, "library.json", || {
            panic!("Downloads available")
        })
        .unwrap();
    assert_eq!(
        std::path::Path::new(&a.path),
        downloads.join("library.json")
    );
    assert_eq!(
        std::path::Path::new(&b.path),
        downloads.join("library-1.json")
    );
    assert_eq!(
        files.resolve_export(&a.export_id).unwrap(),
        std::path::PathBuf::from(&a.path)
    );
    assert_eq!(
        files.resolve_export("arbitrary-path").unwrap_err().code,
        "unknownExport"
    );
    let mut fallback_paths = paths.clone();
    fallback_paths.downloads = Some(dir.path().join("missing"));
    let fallback = dir.path().join("chosen.json");
    let saved = files
        .export(&mut app, &fallback_paths, "library.json", || {
            Ok(Some(fallback.clone()))
        })
        .unwrap();
    assert_eq!(saved.path, fallback.to_string_lossy());
    assert!(fallback.is_file());
    assert_eq!(
        files
            .export(&mut app, &fallback_paths, "library.json", || Ok(None))
            .unwrap_err()
            .code,
        "cancelled"
    );
    assert!(
        files
            .export(&mut app, &fallback_paths, "library.json", || Err(
                AppError::new("fileAccess")
            ))
            .is_err()
    );
}
#[test]
fn confirmed_restore_is_guarded_and_preserves_reserved_question_filter_and_undo() {
    use serde_json::json;
    let dir = tempfile::tempdir().unwrap();
    let paths =
        NativePaths::from_roots(dir.path().join("data"), Some(dir.path().to_path_buf())).unwrap();
    let mut app = core(&paths.database);
    let call = |app: &mut App, kind: &str, payload| {
        app.execute_json(json!({"kind":kind,"payload":payload}))
            .unwrap()["payload"]
            .clone()
    };
    let deck = call(&mut app, "createDeck", json!({"name":"Restore"}))["id"].clone();
    for q in ["one", "two"] {
        call(
            &mut app,
            "createCard",
            json!({"deckId":deck,"draft":{"forms":[{"front":q,"back":"answer","notes":""}]}}),
        );
    }
    let start = call(&mut app, "startSession", json!({"deckId":deck}));
    let sid = start["payload"]["sessionId"].clone();
    let reveal = call(&mut app, "revealAnswer", json!({"sessionId":sid}));
    let frame = &reveal["payload"];
    let next = call(
        &mut app,
        "rateCard",
        json!({"sessionId":sid,"presentationId":frame["presentationId"],"rating":"easy","actionId":"12345678-1234-4234-8234-123456789012","expectedCardRevision":frame["cardRevision"]}),
    );
    let lifecycle = Lifecycle::default();
    let mut files = BackupFiles::default();
    assert_eq!(
        lifecycle.ensure_replace_allowed(&mut app).unwrap_err().code,
        "reviewInProgress"
    );
    let report = files
        .export(&mut app, &paths, "live.json", || panic!())
        .unwrap();
    call(&mut app, "endSession", json!({"sessionId":sid}));
    lifecycle.set_dirty(true);
    assert_eq!(
        lifecycle.ensure_replace_allowed(&mut app).unwrap_err().code,
        "unsavedChanges"
    );
    lifecycle.set_dirty(false);
    lifecycle.ensure_replace_allowed(&mut app).unwrap();
    let preview = app
        .preview_restore(std::path::Path::new(&report.path))
        .unwrap();
    assert_eq!((preview.cards, preview.decks, preview.reviews), (2, 1, 1));
    let restored = files
        .restore(
            &mut app,
            &paths,
            preview.id,
            preview.expected_library_revision,
            "safety.json",
            || panic!(),
        )
        .unwrap();
    assert!(std::path::Path::new(&restored.safety_backup.path).is_file());
    assert!(
        files
            .resolve_export(&restored.safety_backup.export_id)
            .is_ok()
    );
    let resumed = call(&mut app, "readSession", json!({"sessionId":sid}));
    assert_eq!(resumed["payload"]["form"], next["payload"]["form"]);
    assert_eq!(resumed["payload"]["filter"], next["payload"]["filter"]);
    assert_eq!(resumed["payload"]["canUndo"], true);
    let undo = call(&mut app, "undoReview", json!({"sessionId":sid}));
    assert_eq!(undo["payload"]["form"], reveal["payload"]["form"]);
}
