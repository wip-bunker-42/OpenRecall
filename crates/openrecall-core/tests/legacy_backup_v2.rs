mod common;
use common::{Fixture, spelling};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn digest_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

const FIXTURE: &[u8] = include_bytes!("fixtures/backups/v0.2.0.json");

#[test]
fn released_v2_preserves_spelling_and_undo() {
    assert_eq!(
        digest_bytes(FIXTURE),
        "8a1d49f6a8e88c6111df5cbbd60186ec16bb6350549bcaa522564cee8167da64"
    );
    let mut f = Fixture::new();
    let raw: Value = serde_json::from_slice(FIXTURE).unwrap();
    let path = f.dir.path().join("released.json");
    std::fs::write(&path, FIXTURE).unwrap();
    let preview = f.app.preview_restore(&path).unwrap();
    assert_eq!(preview.format_version, 2);
    f.app
        .restore_snapshot(
            preview.id,
            preview.expected_library_revision,
            &f.dir.path().join("safety.json"),
        )
        .unwrap();
    let restored = spelling::snapshot(&mut f);
    assert_eq!(restored["spelling"]["exclusions"][0]["canonical"], "green");
    assert_eq!(
        restored["spelling"]["snapshots"],
        raw["data"]["spelling"]["snapshots"]
    );
    assert_eq!(restored["reviews"], raw["data"]["reviews"]);
    assert!(restored["reviews"][0]["memory"]["stabilityFast"].is_number());
    assert!(
        restored["cards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|card| card["forms"][0]["questionTitle"] == "ترجم"
                && card["forms"].as_array().unwrap().len() == 2)
    );
    let undo = &raw["data"]["undoRecords"][0];
    let view = f.call("undoReview", json!({"sessionId":undo["sessionId"]}));
    assert_eq!(
        view["payload"]["presentationId"],
        undo["sessionBefore"]["current"]["id"]
    );
    assert_eq!(view["payload"]["mode"], "spelling");
    assert_eq!(view["payload"]["canRate"], true);
    assert_eq!(view["payload"]["card"]["reviewCount"], 0);
}

#[test]
fn released_v2_rejects_rehashed_new_nested_fields() {
    let mut f = Fixture::new();
    let before = spelling::snapshot(&mut f);
    let original: Value = serde_json::from_slice(FIXTURE).unwrap();
    for field in ["questionType", "choiceFeedback"] {
        let mut changed = original.clone();
        if field == "questionType" {
            let card = changed["data"]["cards"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|card| card["forms"][0]["questionTitle"] == "ترجم")
                .unwrap();
            card["forms"][0][field] = json!("essay");
        } else {
            changed["data"]["undoRecords"][0]["sessionBefore"][field] = json!({"unexpected":true});
        }
        let mut payload = changed["data"].clone();
        payload.sort_all_objects();
        changed["payloadSha256"] = json!(digest_bytes(&serde_json::to_vec(&payload).unwrap()));
        let path = f.dir.path().join("tampered.json");
        std::fs::write(&path, serde_json::to_vec(&changed).unwrap()).unwrap();
        let error = f.app.preview_restore(&path).expect_err(field);
        assert_ne!(error.code, "backupDigest");
        assert_eq!(spelling::snapshot(&mut f), before);
    }
}
