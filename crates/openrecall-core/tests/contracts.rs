mod common;
use common::*;
use serde_json::json;
#[test]
fn generated_duration_and_progress_fields_match_json_numbers() {
    let dir = tempfile::tempdir().unwrap();
    openrecall_core::contracts::export(dir.path()).unwrap();
    for file in std::fs::read_dir(dir.path()).unwrap() {
        let path = file.unwrap().path();
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(
            !source.contains("bigint"),
            "{} contains non-JSON bigint",
            path.display()
        );
    }
}
#[test]
fn contracts_round_trip_tagged_messages() {
    let mut f = Fixture::new();
    let response = f
        .app
        .execute_json(json!({"kind":"bootstrap","payload":{}}))
        .unwrap();
    assert_eq!(response["kind"], "bootstrap");
    assert!(response["payload"]["settings"]["revision"].is_string());
    assert!(
        f.app
            .execute_json(json!({"kind":"unexpectedCommand","payload":{}}))
            .is_err()
    );
}
