use openrecall_desktop_lib::files::read_import_file_with_limit;
#[test]
fn selected_native_file_round_trips_through_real_core_import_and_persistence() {
    use serde_json::json;
    let dir = tempfile::tempdir().unwrap();
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/fixtures/native-smoke-cards.json");
    let picked = read_import_file_with_limit(&source, 50 * 1024 * 1024).unwrap();
    let mut app = openrecall_core::App::open(
        &dir.path().join("native.sqlite3"),
        std::sync::Arc::new(openrecall_core::SystemClock::default()),
        Box::new(openrecall_core::SystemRandom::default()),
    )
    .unwrap();
    let deck = app
        .execute_json(
            json!({"kind":"createDeck","payload":{"name":"native import","profileId":null}}),
        )
        .unwrap()["payload"]["id"]
        .clone();
    let p = app
        .execute_json(json!({"kind":"previewImport","payload":{"deckId":deck,"text":picked.text}}))
        .unwrap()["payload"]
        .clone();
    let keys: Vec<_> = p["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["key"].clone())
        .collect();
    let result=app.execute_json(json!({"kind":"commitImport","payload":{"previewId":p["previewId"],"libraryRevision":p["libraryRevision"],"selectedCardKeys":keys,"excludedFormKeys":[]}})).unwrap();
    assert_eq!(result["payload"]["insertedCards"], 2);
    assert_eq!(result["payload"]["insertedForms"], 3);
    drop(app);
    let mut app = openrecall_core::App::open(
        &dir.path().join("native.sqlite3"),
        std::sync::Arc::new(openrecall_core::SystemClock::default()),
        Box::new(openrecall_core::SystemRandom::default()),
    )
    .unwrap();
    let result = app
        .execute_json(
            json!({"kind":"listCards","payload":{"deckId":deck,"query":"","page":1,"pageSize":20}}),
        )
        .unwrap();
    assert_eq!(result["payload"]["total"], 2);
    assert!(
        result["payload"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["forms"][0]["front"].as_str().unwrap().contains("<h1>"))
    );
}
#[test]
fn native_picker_read_checks_size_utf8_and_bom_and_returns_no_path() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("بطاقات.json");
    std::fs::write(&file, "\u{feff}[{\"front\":\"سؤال\",\"back\":\"answer\"}]").unwrap();
    let result = read_import_file_with_limit(&file, 1000).unwrap();
    assert_eq!(result.name, "بطاقات.json");
    assert!(result.text.starts_with('['));
    assert!(
        !serde_json::to_string(&result)
            .unwrap()
            .contains(&dir.path().to_string_lossy().to_string())
    );
    assert_eq!(
        read_import_file_with_limit(&file, 10).unwrap_err().code,
        "importTooLarge"
    );
    std::fs::write(&file, [0xff, 0xfe]).unwrap();
    assert_eq!(
        read_import_file_with_limit(&file, 1000).unwrap_err().code,
        "invalidUtf8"
    );
}
