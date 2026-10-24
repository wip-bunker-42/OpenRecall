mod common;
use common::*;
use serde_json::{Value, json};
fn read(path: &std::path::Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
#[test]
fn trained_shared_profiles_detached_models_reports_cycles_and_live_undo_round_trip() {
    let mut f = Fixture::new();
    let (profile, decks, _) = common::training::seed_profile(&mut f);
    let job = f.call("startTraining", json!({"profileId":profile}));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        let j = f.call("getTrainingJob", json!({"jobId":job["id"]}));
        if j["status"] == "ready" {
            break;
        }
        assert!(["running", "queued"].contains(&j["status"].as_str().unwrap()));
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let p = f.call("getProfile", json!({"profileId":profile}));
    f.call(
        "applyTraining",
        json!({"jobId":job["id"],"expectedProfileRevision":p["revision"]}),
    );
    let third = f.call("createDeck", json!({"name":"detach","profileId":profile}));
    f.call("detachProfile",json!({"deckId":third["id"],"newName":"independent","expectedDeckRevision":third["revision"]}));
    let start = f.call("startSession", json!({"deckId":decks[0]}));
    let sid = start["payload"]["sessionId"].clone();
    let revealed = f.call("revealAnswer", json!({"sessionId":sid}));
    let frame = &revealed["payload"];
    f.call("rateCard",json!({"sessionId":sid,"presentationId":frame["presentationId"],"rating":"easy","actionId":uuid::Uuid::new_v4(),"expectedCardRevision":frame["cardRevision"]}));
    let file = f.dir.path().join("trained.json");
    f.app.export_snapshot(&file).unwrap();
    let original = read(&file);
    let preview = f.app.preview_restore(&file).unwrap();
    f.app
        .restore_snapshot(
            preview.id,
            preview.expected_library_revision,
            &f.dir.path().join("safety-trained.json"),
        )
        .unwrap();
    let after = f.dir.path().join("trained-after.json");
    f.app.export_snapshot(&after).unwrap();
    assert_eq!(read(&after)["data"], original["data"]);
    let undone = f.call("undoReview", json!({"sessionId":sid}));
    assert_eq!(
        undone["payload"]["presentationId"],
        revealed["payload"]["presentationId"]
    );
    assert_eq!(undone["payload"]["answerRevealed"], true);
}
#[test]
fn restore_preserves_full_user_state_and_fails_atomically() {
    let mut f = Fixture::new();
    let deck = f.deck("portable");
    let card = f.card(&deck, "عربي English 日本語", "answer");
    let start = f.call("startSession", json!({"deckId":deck}));
    let sid = start["payload"]["sessionId"].clone();
    let revealed = f.call("revealAnswer", json!({"sessionId":sid}));
    let frame = &revealed["payload"];
    f.call("rateCard",json!({"sessionId":sid,"presentationId":frame["presentationId"],"rating":"easy","actionId":uuid::Uuid::new_v4(),"expectedCardRevision":frame["cardRevision"]}));
    f.call("undoReview", json!({"sessionId":sid}));
    f.call(
        "setReviewFilter",
        json!({"sessionId":sid,"filter":"previouslyReviewed"}),
    );
    let file = f.dir.path().join("backup.json");
    let report = f.app.export_snapshot(&file).unwrap();
    let exported = read(std::path::Path::new(&report.path));
    assert!(!exported.to_string().contains("test.sqlite3"));
    let preview = f.app.preview_restore(&file).unwrap();
    let before = f.call("bootstrap", json!({}));
    assert_eq!(preview.cards, 1);
    f.deck("later");
    let current = f.call("bootstrap", json!({}));
    assert_eq!(
        f.app
            .restore_snapshot(
                preview.id.clone(),
                before["libraryRevision"].as_str().unwrap().into(),
                &f.dir.path().join("safety.json")
            )
            .unwrap_err()
            .code,
        "conflict"
    );
    let preview = f.app.preview_restore(&file).unwrap();
    assert!(
        f.app
            .restore_snapshot(
                preview.id.clone(),
                current["libraryRevision"].as_str().unwrap().into(),
                &f.dir.path().join("missing/safety.json")
            )
            .is_err()
    );
    assert_eq!(f.call("bootstrap", json!({})), current);
    f.app
        .restore_snapshot(
            preview.id,
            current["libraryRevision"].as_str().unwrap().into(),
            &f.dir.path().join("safety.json"),
        )
        .unwrap();
    assert_eq!(
        f.call("getCard", json!({"cardId":card}))["bag"],
        exported["data"]["cards"][0]["bag"]
    );
    let after = f.dir.path().join("after.json");
    f.app.export_snapshot(&after).unwrap();
    assert_eq!(read(&after)["data"], exported["data"]);
    let view = f.call("resumeSession", json!({"sessionId":sid}));
    assert_eq!(view["state"], "emptyFilter");
    let view = f.call("setReviewFilter", json!({"sessionId":sid,"filter":"all"}));
    assert_eq!(view["payload"]["answerRevealed"], true);
    let frame = &view["payload"];
    f.call("rateCard",json!({"sessionId":sid,"presentationId":frame["presentationId"],"rating":"good","actionId":uuid::Uuid::new_v4(),"expectedCardRevision":frame["cardRevision"]}));
    f.call("undoReview", json!({"sessionId":sid}));
}
#[test]
fn rejects_corruption_future_versions_invalid_models_and_dangling_references_without_writes() {
    use sha2::{Digest, Sha256};
    let mut f = Fixture::new();
    let deck = f.deck("validation");
    f.card(&deck, "q", "a");
    let file = f.dir.path().join("backup.json");
    f.app.export_snapshot(&file).unwrap();
    let original = read(&file);
    let before = f.call("bootstrap", json!({}));
    for mode in 0..5 {
        let mut value = original.clone();
        match mode {
            0 => value["payloadSha256"] = json!("bad"),
            1 => value["formatVersion"] = json!(999),
            2 => value["data"]["cards"][0]["deckId"] = json!("missing"),
            3 => value["data"]["models"][0]["parameters"] = json!([1, 2]),
            _ => {
                value["data"]["cards"][0]["memory"] =
                    json!({"stability":-1,"stabilityFast":0,"difficulty":20})
            }
        };
        if mode >= 2 {
            value["payloadSha256"] = json!(
                Sha256::digest(serde_json::to_vec(&value["data"]).unwrap())
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            );
        }
        let bad = f.dir.path().join(format!("bad{mode}.json"));
        std::fs::write(&bad, value.to_string()).unwrap();
        assert!(f.app.preview_restore(&bad).is_err(), "mode {mode}");
        assert_eq!(f.call("bootstrap", json!({})), before);
    }
    let second = f.app.export_snapshot(&file).unwrap();
    assert_ne!(second.path, file.to_string_lossy());
    assert_eq!(read(&file), original);
}
#[test]
fn restore_transaction_failure_rolls_back_and_limits_are_symmetric() {
    use openrecall_core::backup::BackupLimits;
    let mut f = Fixture::new();
    let deck = f.deck("rollback");
    f.card(&deck, "q", "a");
    let file = f.dir.path().join("backup.json");
    assert_eq!(
        f.app
            .export_snapshot_with_limits(&file, BackupLimits { max_bytes: 10 })
            .unwrap_err()
            .code,
        "backupTooLarge"
    );
    assert!(!file.exists());
    f.app.export_snapshot(&file).unwrap();
    let len = std::fs::metadata(&file).unwrap().len();
    assert!(
        f.app
            .preview_restore_with_limits(&file, BackupLimits { max_bytes: len })
            .is_ok()
    );
    assert!(
        f.app
            .preview_restore_with_limits(&file, BackupLimits { max_bytes: len - 1 })
            .is_err()
    );
    let preview = f.app.preview_restore(&file).unwrap();
    let before = f.call("bootstrap", json!({}));
    let conn = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    conn.execute_batch("CREATE TRIGGER fail_restore BEFORE INSERT ON cards BEGIN SELECT RAISE(ABORT,'injected'); END").unwrap();
    assert!(
        f.app
            .restore_snapshot(
                preview.id,
                before["libraryRevision"].as_str().unwrap().into(),
                &f.dir.path().join("safety.json")
            )
            .is_err()
    );
    assert_eq!(f.call("bootstrap", json!({})), before);
}
