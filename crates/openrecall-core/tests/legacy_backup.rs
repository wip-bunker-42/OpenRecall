mod common;
use common::*;
use serde_json::{Value, json};

fn legacy() -> Value {
    serde_json::from_str(include_str!("fixtures/backups/v0.1.0.json")).unwrap()
}

#[test]
fn new_exports_are_format_two() {
    let mut f = Fixture::new();
    let path = f.dir.path().join("new.json");
    f.app.export_snapshot(&path).unwrap();
    let raw: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(raw["formatVersion"], 2);
}

#[test]
fn legacy_checksum_precedes_defaults_and_nested_undo_survives() {
    let mut f = Fixture::new();
    let raw = legacy();
    assert_eq!(raw["formatVersion"], 1);
    assert_eq!(raw["schemaVersion"], 8);
    let path = f.dir.path().join("old.json");
    std::fs::write(&path, serde_json::to_vec(&raw).unwrap()).unwrap();
    let preview = f.app.preview_restore(&path).unwrap();
    assert_eq!(preview.format_version, 1);
    f.app
        .restore_snapshot(
            preview.id,
            preview.expected_library_revision,
            &f.dir.path().join("safety.json"),
        )
        .unwrap();
    let after = f.dir.path().join("after.json");
    f.app.export_snapshot(&after).unwrap();
    let restored: Value = serde_json::from_slice(&std::fs::read(after).unwrap()).unwrap();
    assert_eq!(
        restored["data"]["cards"][0]["dueAt"],
        raw["data"]["cards"][0]["dueAt"]
    );
    assert_eq!(restored["data"]["undoRecords"], raw["data"]["undoRecords"]);
    let undo = &raw["data"]["undoRecords"][0];
    let view = f.call("undoReview", json!({"sessionId":undo["sessionId"]}));
    assert_eq!(
        view["payload"]["presentationId"],
        undo["sessionBefore"]["current"]["id"]
    );
    assert_eq!(view["payload"]["answerRevealed"], true);

    let mut corrupt = raw;
    corrupt["data"]["cards"][0]["forms"][0]["questionTitle"] = json!("smuggled");
    std::fs::write(&path, serde_json::to_vec(&corrupt).unwrap()).unwrap();
    assert_eq!(
        f.app.preview_restore(&path).unwrap_err().code,
        "backupDigest"
    );
    use sha2::{Digest, Sha256};
    let mut canonical = corrupt["data"].clone();
    canonical.sort_all_objects();
    corrupt["payloadSha256"] = json!(
        Sha256::digest(serde_json::to_vec(&canonical).unwrap())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    std::fs::write(&path, serde_json::to_vec(&corrupt).unwrap()).unwrap();
    assert_eq!(
        f.app.preview_restore(&path).unwrap_err().code,
        "invalidData"
    );
}
