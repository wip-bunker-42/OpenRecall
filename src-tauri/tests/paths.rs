use openrecall_desktop_lib::paths::NativePaths;
#[test]
fn database_and_download_paths_use_native_roots_and_survive_restart() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("app-data");
    let downloads = dir.path().join("Downloads");
    std::fs::create_dir_all(&downloads).unwrap();
    let paths = NativePaths::from_roots(data.clone(), Some(downloads.clone())).unwrap();
    assert_eq!(paths.database, data.join("openrecall.sqlite3"));
    assert_eq!(paths.downloads, Some(downloads));
    let mut app = openrecall_core::App::open(
        &paths.database,
        std::sync::Arc::new(openrecall_core::SystemClock::default()),
        Box::new(openrecall_core::SystemRandom::default()),
    )
    .unwrap();
    let created = app
        .execute_json(
            serde_json::json!({"kind":"createDeck","payload":{"name":"native","profileId":null}}),
        )
        .unwrap();
    drop(app);
    let mut app = openrecall_core::App::open(
        &paths.database,
        std::sync::Arc::new(openrecall_core::SystemClock::default()),
        Box::new(openrecall_core::SystemRandom::default()),
    )
    .unwrap();
    let d = &created["payload"];
    let renamed=app.execute_json(serde_json::json!({"kind":"renameDeck","payload":{"deckId":d["id"],"name":"renamed","expectedRevision":d["revision"]}})).unwrap();
    app.execute_json(serde_json::json!({"kind":"deleteDeck","payload":{"deckId":d["id"],"expectedRevision":renamed["payload"]["revision"]}})).unwrap();
    assert!(paths.database.exists());
}
