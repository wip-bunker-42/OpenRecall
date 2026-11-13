mod common;
use common::*;
use openrecall_core::App;
use rusqlite::{Connection, params};
use serde_json::{Value, json};

fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn ms(value: &Value) -> i64 {
    chrono::DateTime::parse_from_rfc3339(value.as_str().unwrap())
        .unwrap()
        .timestamp_millis()
}
fn rows(db: &Connection, table: &str) -> Vec<String> {
    db.prepare(&format!("SELECT payload FROM {table} ORDER BY id"))
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}
fn released_library(db: &Connection, raw: &Value) {
    for sql in [
        include_str!("../src/storage/migrations/001_settings.sql"),
        include_str!("../src/storage/migrations/002_library.sql"),
        include_str!("../src/storage/migrations/003_cards.sql"),
        include_str!("../src/storage/migrations/005_sessions.sql"),
        include_str!("../src/storage/migrations/006_reviews_undo.sql"),
        include_str!("../src/storage/migrations/008_training_runs.sql"),
    ] {
        db.execute_batch(sql).unwrap();
    }
    let version = raw["schemaVersion"].as_i64().unwrap();
    if version == 9 {
        db.execute_batch(include_str!("../src/storage/migrations/009_spelling.sql"))
            .unwrap();
    }
    let data = &raw["data"];
    db.execute(
        "INSERT INTO settings(id,payload) VALUES(1,?1)",
        [data["settings"].to_string()],
    )
    .unwrap();
    db.execute(
        "INSERT INTO metadata(key,value) VALUES('libraryRevision',?1)",
        [data["libraryRevision"].as_str().unwrap()],
    )
    .unwrap();
    for table in ["models", "profiles"] {
        for x in data[table].as_array().unwrap() {
            db.execute(
                &format!("INSERT INTO {table}(id,payload) VALUES(?1,?2)"),
                params![x["id"].as_str(), x.to_string()],
            )
            .unwrap();
        }
    }
    for x in data["decks"].as_array().unwrap() {
        if version == 9 {
            db.execute(
                "INSERT INTO decks(id,profile_id,kind,payload) VALUES(?1,?2,?3,?4)",
                params![
                    x["id"].as_str(),
                    x["profileId"].as_str(),
                    x["kind"].as_str(),
                    x.to_string()
                ],
            )
            .unwrap();
        } else {
            db.execute(
                "INSERT INTO decks(id,profile_id,payload) VALUES(?1,?2,?3)",
                params![x["id"].as_str(), x["profileId"].as_str(), x.to_string()],
            )
            .unwrap();
        }
    }
    for x in data["cards"].as_array().unwrap() {
        insert_card(db, x);
    }
    for x in data["sessions"].as_array().unwrap() {
        db.execute(
            "INSERT INTO sessions(id,deck_id,status,payload) VALUES(?1,?2,?3,?4)",
            params![
                x["id"].as_str(),
                x["deckId"].as_str(),
                x["status"].as_str(),
                x.to_string()
            ],
        )
        .unwrap();
    }
    for x in data["reviews"].as_array().unwrap() {
        db.execute("INSERT INTO reviews(id,card_id,deck_id,session_id,at_ms,model_revision,payload) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![x["id"].as_str(),x["cardId"].as_str(),x["deckId"].as_str(),x["sessionId"].as_str(),ms(&x["reviewedAt"]),x["modelRevision"].as_str(),x.to_string()]).unwrap();
    }
    for (table, key, wire_key) in [
        ("actions", "actions", "session_id"),
        ("undo_records", "undoRecords", "sessionId"),
    ] {
        for x in data[key].as_array().unwrap() {
            db.execute(
                &format!("INSERT INTO {table}(id,session_id,payload) VALUES(?1,?2,?3)"),
                params![x["id"].as_str(), x[wire_key].as_str(), x.to_string()],
            )
            .unwrap();
        }
    }
    if version == 9 {
        let spelling = &data["spelling"];
        for c in spelling["configs"].as_array().unwrap() {
            db.execute(
                "INSERT INTO spelling_configs VALUES(?1,?2,?3,?4,?5)",
                params![
                    c["deckId"].as_str(),
                    c["sourceDeckId"].as_str(),
                    c["sourceDeckName"].as_str(),
                    c["extractionPolicyVersion"].as_i64(),
                    c["revision"].as_str()
                ],
            )
            .unwrap();
        }
        for w in spelling["words"].as_array().unwrap() {
            db.execute(
                "INSERT INTO spelling_words VALUES(?1,?2,?3,?4)",
                params![
                    w["cardId"].as_str(),
                    w["deckId"].as_str(),
                    w["canonical"].as_str(),
                    w["display"].as_str()
                ],
            )
            .unwrap();
        }
        for s in spelling["snapshots"].as_array().unwrap() {
            db.execute(
                "INSERT INTO spelling_snapshots VALUES(?1,?2,?3,?4)",
                params![
                    s["id"].as_str(),
                    s["spellingDeckId"].as_str(),
                    s["contentSha256"].as_str(),
                    s.to_string()
                ],
            )
            .unwrap();
        }
        for o in spelling["origins"].as_array().unwrap() {
            db.execute(
                "INSERT INTO spelling_origins VALUES(?1,?2,?3,?4,?5)",
                params![
                    o["spellingCardId"].as_str(),
                    o["sourceCardId"].as_str(),
                    o["sourceFormId"].as_str(),
                    o["snapshotId"].as_str(),
                    o["firstSeenAt"].as_str()
                ],
            )
            .unwrap();
        }
        for e in spelling["exclusions"].as_array().unwrap() {
            db.execute(
                "INSERT INTO spelling_exclusions VALUES(?1,?2,?3)",
                params![
                    e["deckId"].as_str(),
                    e["canonical"].as_str(),
                    e["excludedAt"].as_str()
                ],
            )
            .unwrap();
        }
    }
    db.pragma_update(None, "user_version", version).unwrap();
}
fn insert_card(db: &Connection, x: &Value) {
    db.execute("INSERT INTO cards(id,deck_id,due_ms,created_ms,review_count,payload) VALUES(?1,?2,?3,?4,?5,?6)",params![x["id"].as_str(),x["deckId"].as_str(),ms(&x["dueAt"]),ms(&x["createdAt"]),x["reviewCount"].as_i64(),x.to_string()]).unwrap();
}

#[test]
fn schema8_upgrade_preserves_existing_payloads() {
    let f = Fixture::new();
    let path = f.dir.path().join("released.sqlite3");
    let db = Connection::open(&path).unwrap();
    let raw = serde_json::from_str(include_str!("fixtures/backups/v0.1.0.json")).unwrap();
    released_library(&db, &raw);
    let tables = [
        "cards",
        "decks",
        "sessions",
        "reviews",
        "undo_records",
        "actions",
        "profiles",
        "models",
    ];
    let before: Vec<_> = tables.iter().map(|table| rows(&db, table)).collect();
    let mut app = App::open(&path, f.clock.clone(), Box::new(SeededRandom::new(42))).unwrap();
    assert_eq!(
        db.pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
            .unwrap(),
        10
    );
    for (table, payloads) in tables.iter().zip(before) {
        assert_eq!(rows(&db, table), payloads, "{table}");
    }
    let decks = app
        .execute_json(json!({"kind":"listDecks","payload":{}}))
        .unwrap();
    assert_eq!(decks["payload"][0]["kind"], "standard");
    assert_eq!(
        db.query_row("SELECT kind FROM decks", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "standard"
    );
    assert!(
        app.export_snapshot(&f.dir.path().join("migrated.json"))
            .is_ok()
    );
}

fn mark_empty_deck_spelling(db: &Connection, deck: &Value) -> Value {
    let mut deck = deck.clone();
    deck["kind"] = json!("spelling");
    db.execute("DELETE FROM decks WHERE id=?1", [deck["id"].as_str()])
        .unwrap();
    db.execute(
        "INSERT INTO decks(id,profile_id,kind,payload) VALUES(?1,?2,'spelling',?3)",
        params![
            deck["id"].as_str(),
            deck["profileId"].as_str(),
            deck.to_string()
        ],
    )
    .unwrap();
    deck
}

#[test]
fn sqlite_prevents_a_second_spelling_kind() {
    let mut f = Fixture::new();
    let first = f.call("createDeck", json!({"name":"first"}));
    let db = Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    assert_eq!(
        db.pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
            .unwrap(),
        10
    );
    let first = mark_empty_deck_spelling(&db, &first);
    let mut other = first.clone();
    other["id"] = json!(uuid());
    assert!(
        db.execute(
            "INSERT INTO decks(id,profile_id,kind,payload) VALUES(?1,?2,'spelling',?3)",
            params![
                other["id"].as_str(),
                other["profileId"].as_str(),
                other.to_string()
            ]
        )
        .is_err()
    );
    assert!(db.execute("UPDATE decks SET kind='standard',payload=json_set(payload,'$.kind','standard') WHERE id=?1",[first["id"].as_str()]).is_err());
    assert!(
        db.execute(
            "UPDATE decks SET payload=json_set(payload,'$.kind','standard') WHERE id=?1",
            [first["id"].as_str()]
        )
        .is_err()
    );
    let renamed = f.call(
        "renameDeck",
        json!({"deckId":first["id"],"name":"renamed","expectedRevision":first["revision"]}),
    );
    assert_eq!(renamed["kind"], "spelling");
}

#[test]
fn format2_storage_round_trips_spelling_rows() {
    let mut f = Fixture::new();
    let source = f.call("createDeck", json!({"name":"Source"}));
    let source_card_id = f.card(source["id"].as_str().unwrap(), "Alpha", "beta");
    let original = f.call("getCard", json!({"cardId":source_card_id}));
    let deck = f.call("createDeck", json!({"name":"Spelling"}));
    let db = Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    db.execute_batch("PRAGMA foreign_keys=ON").unwrap();
    assert_eq!(
        db.pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
            .unwrap(),
        10
    );
    let deck = mark_empty_deck_spelling(&db, &deck);
    let mut word = original.clone();
    word["id"] = json!(uuid());
    word["deckId"] = deck["id"].clone();
    word["forms"][0]["id"] = json!(uuid());
    word["forms"][0]["back"] = json!("Alpha");
    word["forms"][0]["language"] = json!("en");
    insert_card(&db, &word);
    let snapshot_id = uuid();
    let revision = uuid();
    let at = original["createdAt"].as_str().unwrap();
    let mut snapshot = json!({"id":snapshot_id,"spellingDeckId":deck["id"],"sourceDeckId":source["id"],"sourceDeckName":"Source","sourceCardId":original["id"],"sourceFormId":original["forms"][0]["id"],"sourceFormIndex":0,"sourceCardRevision":original["revision"],"extractionPolicyVersion":1,"capturedAt":at,"front":"Alpha","back":"beta","notes":"","questionTitle":null,"language":null,"contentSha256":""});
    // The immutable content fingerprint omits capture time, opaque revision and ID.
    let content = json!([
        deck["id"],
        source["id"],
        "Source",
        original["id"],
        original["forms"][0]["id"],
        0,
        1,
        "Alpha",
        "beta",
        "",
        null,
        null
    ]);
    use sha2::{Digest, Sha256};
    snapshot["contentSha256"] = json!(
        Sha256::digest(serde_json::to_vec(&content).unwrap())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    let expected: openrecall_core::domain::spelling::SpellingBackupData = serde_json::from_value(json!({
        "configs":[{"deckId":deck["id"],"sourceDeckId":source["id"],"sourceDeckName":"Source","extractionPolicyVersion":1,"revision":revision}],
        "words":[{"cardId":word["id"],"deckId":deck["id"],"canonical":"alpha","display":"Alpha"}],
        "snapshots":[snapshot],
        "origins":[{"spellingCardId":word["id"],"sourceCardId":original["id"],"sourceFormId":original["forms"][0]["id"],"snapshotId":snapshot_id,"firstSeenAt":at}],
        "exclusions":[{"deckId":deck["id"],"canonical":"omitted","excludedAt":at}]
    })).unwrap();
    db.execute("INSERT INTO spelling_configs(deck_id,source_deck_id,source_deck_name,extraction_policy_version,revision) VALUES(?1,?2,'Source',1,?3)",params![deck["id"].as_str(),source["id"].as_str(),revision]).unwrap();
    db.execute("INSERT INTO spelling_words(card_id,deck_id,canonical,display) VALUES(?1,?2,'alpha','Alpha')",params![word["id"].as_str(),deck["id"].as_str()]).unwrap();
    db.execute("INSERT INTO spelling_snapshots(id,spelling_deck_id,content_sha256,payload) VALUES(?1,?2,?3,?4)",params![snapshot_id,deck["id"].as_str(),snapshot["contentSha256"].as_str(),snapshot.to_string()]).unwrap();
    db.execute("INSERT INTO spelling_origins(spelling_card_id,source_card_id,source_form_id,snapshot_id,first_seen_at) VALUES(?1,?2,?3,?4,?5)",params![word["id"].as_str(),original["id"].as_str(),original["forms"][0]["id"].as_str(),snapshot_id,at]).unwrap();
    db.execute(
        "INSERT INTO spelling_exclusions(deck_id,canonical,excluded_at) VALUES(?1,'omitted',?2)",
        params![deck["id"].as_str(), at],
    )
    .unwrap();
    let file = f.dir.path().join("spelling.json");
    f.app.export_snapshot(&file).unwrap();
    let before: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    assert_eq!(
        serde_json::from_value::<openrecall_core::domain::spelling::SpellingBackupData>(
            before["data"]["spelling"].clone()
        )
        .unwrap(),
        expected
    );
    snapshot["contentVersion"] = json!(1);
    snapshot["questionType"] = json!("recall");
    snapshot["multipleChoice"] = Value::Null;
    assert_eq!(before["data"]["spelling"]["snapshots"], json!([snapshot]));
    assert_eq!(before["data"]["spelling"]["origins"][0]["firstSeenAt"], at);
    let p = f.app.preview_restore(&file).unwrap();
    f.app
        .restore_snapshot(
            p.id,
            p.expected_library_revision,
            &f.dir.path().join("safety.json"),
        )
        .unwrap();
    let after = f.dir.path().join("after.json");
    f.app.export_snapshot(&after).unwrap();
    let restored: Value = serde_json::from_slice(&std::fs::read(&after).unwrap()).unwrap();
    assert_eq!(restored["data"]["spelling"], before["data"]["spelling"]);
    assert_eq!(
        db.query_row(
            "SELECT kind FROM decks WHERE id=?1",
            [deck["id"].as_str()],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "spelling"
    );
}

#[test]
fn schema9_upgrade_preserves_backup_payload_and_live_undo() {
    let f = Fixture::new();
    let path = f.dir.path().join("schema9.sqlite3");
    let db = Connection::open(&path).unwrap();
    let raw: Value = serde_json::from_str(include_str!("fixtures/backups/v0.2.0.json")).unwrap();
    released_library(&db, &raw);
    let tables = [
        "cards",
        "sessions",
        "reviews",
        "actions",
        "undo_records",
        "spelling_snapshots",
    ];
    let before: Vec<_> = tables.iter().map(|name| rows(&db, name)).collect();
    let mut app = App::open(&path, f.clock.clone(), Box::new(SeededRandom::new(42))).unwrap();
    assert_eq!(
        db.pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
            .unwrap(),
        10
    );
    for (name, payloads) in tables.iter().zip(before) {
        assert_eq!(rows(&db, name), payloads, "{name}");
    }
    let old = &raw["data"]["undoRecords"][0];
    let view = app
        .execute_json(json!({"kind":"undoReview","payload":{"sessionId":old["sessionId"]}}))
        .unwrap();
    assert_eq!(view["payload"]["payload"]["form"]["questionType"], "recall");
    assert_eq!(
        view["payload"]["payload"]["presentationId"],
        old["sessionBefore"]["current"]["id"]
    );
}
