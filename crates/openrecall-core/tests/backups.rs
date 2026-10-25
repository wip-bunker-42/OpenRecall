mod common;
use common::*;
use serde_json::{Value, json};
#[test]
fn exported_digest_matches_the_canonical_json_payload_actually_written() {
    use sha2::{Digest, Sha256};
    let mut f = Fixture::new();
    let d = f.deck("digest");
    f.card(&d, "q", "a");
    let file = f.dir.path().join("digest.json");
    f.app.export_snapshot(&file).unwrap();
    let raw = read(&file);
    let mut canonical = raw["data"].clone();
    canonical.sort_all_objects();
    let hash: String = Sha256::digest(serde_json::to_vec(&canonical).unwrap())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(raw["payloadSha256"], hash);
}
#[test]
fn corrupt_card_progress_and_undo_before_images_are_rejected_semantically() {
    use openrecall_core::backup::format::{BackupEnvelope, digest};
    let mut f = Fixture::new();
    let deck = f.deck("progress integrity");
    f.card(&deck, "q", "a");
    let start = f.call("startSession", json!({"deckId":deck}));
    let sid = start["payload"]["sessionId"].clone();
    let reveal = f.call("revealAnswer", json!({"sessionId":sid}));
    let frame = &reveal["payload"];
    f.call("rateCard",json!({"sessionId":sid,"presentationId":frame["presentationId"],"rating":"easy","actionId":uuid::Uuid::new_v4(),"expectedCardRevision":frame["cardRevision"]}));
    let file = f.dir.path().join("progress.json");
    f.app.export_snapshot(&file).unwrap();
    let original: BackupEnvelope = serde_json::from_value(read(&file)).unwrap();
    for mode in 0..7 {
        let mut bad = original.clone();
        match mode {
            0 => bad.data.cards[0].review_count += 1,
            1 => bad.data.cards[0].lapse_count = 1,
            2 => bad.data.cards[0].reviewed_at = Some("2020-01-01T00:00:00Z".into()),
            3 => bad.data.cards[0].due_at = "2030-01-01T00:00:00Z".into(),
            4 => {
                bad.data.undo_records[0].card_before = bad.data.cards[0].clone();
            }
            5 => bad.data.undo_records[0].session_before.attempts = 99,
            _ => bad.data.undo_records[0].card_before.forms[0].front = "independent text".into(),
        };
        bad.payload_sha256 = digest(&bad.data).unwrap();
        let target = f.dir.path().join(format!("bad-progress-{mode}.json"));
        std::fs::write(
            &target,
            serde_json::to_vec(&serde_json::to_value(&bad).unwrap()).unwrap(),
        )
        .unwrap();
        let error = f.app.preview_restore(&target).unwrap_err();
        assert_eq!(error.code, "invalidBackup", "mode {mode}");
    }
}
fn read(path: &std::path::Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
#[test]
fn invalid_header_dates_and_corrupted_next_card_undo_snapshots_are_rejected() {
    use openrecall_core::backup::format::{BackupEnvelope, digest};
    let mut f = Fixture::new();
    let deck = f.deck("next undo");
    f.card(&deck, "one", "answer");
    f.card(&deck, "two", "answer");
    let start = f.call("startSession", json!({"deckId":deck}));
    let sid = start["payload"]["sessionId"].clone();
    let frame = f.call("revealAnswer", json!({"sessionId":sid}))["payload"].clone();
    f.call("rateCard",json!({"sessionId":sid,"presentationId":frame["presentationId"],"rating":"easy","actionId":uuid::Uuid::new_v4(),"expectedCardRevision":frame["cardRevision"]}));
    let source = f.dir.path().join("next.json");
    f.app.export_snapshot(&source).unwrap();
    let original: BackupEnvelope = serde_json::from_value(read(&source)).unwrap();
    for mode in 0..4 {
        let mut bad = original.clone();
        if mode == 0 {
            bad.exported_at = "invalid timestamp".into()
        } else {
            let next = bad.data.undo_records[0].next_card_before.as_mut().unwrap();
            match mode {
                1 => next.due_at = "2030-01-01T00:00:00Z".into(),
                2 => next.forms[0].front = "changed question".into(),
                _ => next.deck_id = uuid::Uuid::new_v4().to_string(),
            }
        }
        bad.payload_sha256 = digest(&bad.data).unwrap();
        let file = f.dir.path().join(format!("invalid-next-{mode}.json"));
        std::fs::write(
            &file,
            serde_json::to_vec(&serde_json::to_value(bad).unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(
            f.app.preview_restore(&file).unwrap_err().code,
            "invalidBackup",
            "mode {mode}"
        );
    }
    assert!(f.app.preview_restore(&source).is_ok());
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
