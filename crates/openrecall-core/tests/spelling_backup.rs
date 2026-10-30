mod common;
use common::{spelling::*, *};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn hash(value: &Value) -> String {
    let mut value = value.clone();
    value.sort_all_objects();
    Sha256::digest(serde_json::to_vec(&value).unwrap())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn save(path: &std::path::Path, raw: &mut Value) {
    raw["payloadSha256"] = json!(hash(&raw["data"]));
    std::fs::write(path, serde_json::to_vec(raw).unwrap()).unwrap();
}
fn rehash_snapshot(s: &mut Value) {
    let fields = [
        "spellingDeckId",
        "sourceDeckId",
        "sourceDeckName",
        "sourceCardId",
        "sourceFormId",
        "sourceFormIndex",
        "extractionPolicyVersion",
        "front",
        "back",
        "notes",
        "questionTitle",
        "language",
    ];
    s["contentSha256"] = json!(hash(&Value::Array(
        fields.iter().map(|key| s[key].clone()).collect()
    )));
}
fn export(f: &mut Fixture) -> Value {
    let report = f
        .app
        .export_snapshot(&f.dir.path().join("full.json"))
        .unwrap();
    serde_json::from_slice(&std::fs::read(report.path).unwrap()).unwrap()
}

#[test]
fn full_spelling_backup_preserves_deleted_sources_and_active_undo() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    f.card(&source, "red blue", "معنى");
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    let frame = f.call("startSession", json!({"deckId":deck}))["payload"].clone();
    f.call("rateCard",json!({"sessionId":frame["sessionId"],"presentationId":frame["presentationId"],"rating":"good","actionId":id(),"expectedCardRevision":frame["cardRevision"]}));
    let fresh = f.call("getDeck", json!({"deckId":source}));
    f.call(
        "deleteDeck",
        json!({"deckId":source,"expectedRevision":fresh["revision"]}),
    );
    let before = export(&mut f);
    let path = f.dir.path().join("portable.json");
    std::fs::write(&path, serde_json::to_vec(&before).unwrap()).unwrap();
    let fresh = f.call("getDeck", json!({"deckId":deck}));
    f.call(
        "deleteDeck",
        json!({"deckId":deck,"expectedRevision":fresh["revision"]}),
    );
    let preview = f.app.preview_restore(&path).unwrap();
    assert_eq!(preview.format_version, 2);
    f.app
        .restore_snapshot(
            preview.id,
            preview.expected_library_revision,
            &f.dir.path().join("safety.json"),
        )
        .unwrap();
    let after = export(&mut f);
    assert_eq!(after["data"], before["data"]);
    assert!(
        after["data"]["cards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["memory"].is_object())
    );
    let sources = f.call(
        "listSpellingSources",
        json!({"cardId":frame["card"]["id"],"page":1,"pageSize":20}),
    );
    assert_eq!(sources["items"][0]["snapshot"]["front"], "red blue");
    assert_eq!(sources["items"][0]["availability"], "deleted");
    let undo = f.call("undoReview", json!({"sessionId":frame["sessionId"]}));
    assert_eq!(undo["payload"]["presentationId"], frame["presentationId"]);
    assert_eq!(undo["payload"]["canRate"], true);
    assert_eq!(undo["payload"]["card"]["reviewCount"], 0);
    assert!(undo["payload"]["card"]["memory"].is_null());
}

#[test]
fn legacy_defaults_do_not_change_original_digest() {
    let bytes = include_bytes!("fixtures/backups/v0.1.0.json");
    let raw: Value = serde_json::from_slice(bytes).unwrap();
    let decoded = openrecall_core::backup::decode::decode(bytes).unwrap();
    assert_eq!(decoded.format_version, 1);
    assert_eq!(decoded.schema_version, 8);
    assert_eq!(decoded.payload_sha256, hash(&raw["data"]));
    assert_ne!(
        openrecall_core::backup::format::digest(&decoded.data).unwrap(),
        decoded.payload_sha256
    );
    assert_eq!(decoded.data.spelling, Default::default());
    assert!(
        decoded
            .data
            .decks
            .iter()
            .all(|deck| deck.kind == openrecall_core::domain::spelling::DeckKind::Standard)
    );
    for undo in &decoded.data.undo_records {
        assert!(
            undo.card_before
                .forms
                .iter()
                .all(|form| form.question_title.is_none())
        );
        assert!(
            undo.next_card_before
                .as_ref()
                .unwrap()
                .forms
                .iter()
                .all(|form| form.question_title.is_none())
        );
    }
    let mut f = Fixture::new();
    let path = f.dir.path().join("old.json");
    std::fs::write(&path, bytes).unwrap();
    let p = f.app.preview_restore(&path).unwrap();
    assert_eq!(p.format_version, 1);
    f.app
        .restore_snapshot(
            p.id,
            p.expected_library_revision,
            &f.dir.path().join("safety.json"),
        )
        .unwrap();
    let sid = &raw["data"]["undoRecords"][0]["sessionId"];
    assert_eq!(
        f.call("undoReview", json!({"sessionId":sid}))["payload"]["answerRevealed"],
        true
    );
    let mut changed = raw.clone();
    changed["data"]["decks"][0]["kind"] = json!("spelling");
    std::fs::write(&path, serde_json::to_vec(&changed).unwrap()).unwrap();
    assert_eq!(
        f.app.preview_restore(&path).unwrap_err().code,
        "backupDigest"
    );
    save(&path, &mut changed);
    assert_eq!(
        f.app.preview_restore(&path).unwrap_err().code,
        "invalidData"
    );
}

#[test]
fn recomputed_digest_does_not_authorize_inconsistent_spelling_data() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    f.card(&source, "red blue", "معنى");
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    let original = export(&mut f);
    let data_digest = hash(&original["data"]);
    let word_id = original["data"]["spelling"]["words"][0]["cardId"].clone();
    let card_index = original["data"]["cards"]
        .as_array()
        .unwrap()
        .iter()
        .position(|c| c["id"] == word_id)
        .unwrap();
    let cases = [
        "singleton",
        "word duplicate",
        "wrong owner",
        "wrong kind",
        "managed back",
        "extra form",
        "managed notes",
        "managed language",
        "managed heading",
        "canonical case",
        "display identity",
        "snapshot hash",
        "membership",
        "title is not vocabulary",
        "snapshot policy",
        "orphan origin",
        "orphan snapshot",
        "word without origin",
        "excluded active",
        "excluded canonical",
        "snapshot name",
        "live owner",
        "captured date",
        "source revision",
        "snapshot heading",
        "config revision",
        "duplicate content",
        "historical identifier",
    ];
    let mut failures = Vec::new();
    for name in cases {
        let mut bad = original.clone();
        let d = &mut bad["data"];
        match name {
            "singleton" => {
                let mut duplicate = d["decks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|v| v["id"] == deck)
                    .unwrap()
                    .clone();
                duplicate["id"] = json!(id());
                d["decks"].as_array_mut().unwrap().push(duplicate);
            }
            "word duplicate" => {
                let duplicate = d["spelling"]["words"][0].clone();
                d["spelling"]["words"]
                    .as_array_mut()
                    .unwrap()
                    .push(duplicate);
            }
            "wrong owner" => d["spelling"]["words"][0]["deckId"] = json!(source),
            "wrong kind" => {
                let row = d["decks"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|v| v["id"] == deck)
                    .unwrap();
                row["kind"] = json!("standard");
            }
            "managed back" => d["cards"][card_index]["forms"][0]["back"] = json!("different"),
            "extra form" => {
                let mut form = d["cards"][card_index]["forms"][0].clone();
                form["id"] = json!(id());
                d["cards"][card_index]["forms"]
                    .as_array_mut()
                    .unwrap()
                    .push(form);
            }
            "managed notes" => d["cards"][card_index]["forms"][0]["notes"] = json!("extra"),
            "managed language" => d["cards"][card_index]["forms"][0]["language"] = json!("ar"),
            "managed heading" => {
                d["cards"][card_index]["forms"][0]["questionTitle"] = json!("ترجم")
            }
            "canonical case" => d["spelling"]["words"][0]["canonical"] = json!("RED"),
            "display identity" => d["spelling"]["words"][0]["display"] = json!("green"),
            "snapshot hash" => {
                d["spelling"]["snapshots"][0]["contentSha256"] = json!("0".repeat(64))
            }
            "membership" | "title is not vocabulary" => {
                let s = &mut d["spelling"]["snapshots"][0];
                s["front"] = json!("غير إنجليزي");
                if name == "title is not vocabulary" {
                    s["questionTitle"] = json!("red blue");
                }
                rehash_snapshot(s);
            }
            "snapshot policy" => {
                let s = &mut d["spelling"]["snapshots"][0];
                s["extractionPolicyVersion"] = json!(2);
                rehash_snapshot(s);
            }
            "orphan origin" => d["spelling"]["origins"][0]["snapshotId"] = json!(id()),
            "orphan snapshot" => {
                let mut s = d["spelling"]["snapshots"][0].clone();
                s["id"] = json!(id());
                s["sourceCardId"] = json!(id());
                s["sourceFormId"] = json!(id());
                rehash_snapshot(&mut s);
                d["spelling"]["snapshots"].as_array_mut().unwrap().push(s);
            }
            "word without origin" => d["spelling"]["origins"]
                .as_array_mut()
                .unwrap()
                .retain(|o| o["spellingCardId"] != word_id),
            "excluded active" | "excluded canonical" => {
                let canonical = if name == "excluded active" {
                    d["spelling"]["words"][0]["canonical"].clone()
                } else {
                    json!("RED")
                };
                d["spelling"]["exclusions"].as_array_mut().unwrap().push(json!({"deckId":deck,"canonical":canonical,"excludedAt":original["exportedAt"]}));
            }
            "snapshot name" => {
                let s = &mut d["spelling"]["snapshots"][0];
                s["sourceDeckName"] = json!("");
                rehash_snapshot(s);
            }
            "live owner" => {
                let s = &mut d["spelling"]["snapshots"][0];
                s["sourceDeckId"] = json!(id());
                rehash_snapshot(s);
            }
            "captured date" => d["spelling"]["snapshots"][0]["capturedAt"] = json!("invalid"),
            "source revision" => d["spelling"]["snapshots"][0]["sourceCardRevision"] = json!(""),
            "snapshot heading" => {
                let s = &mut d["spelling"]["snapshots"][0];
                s["questionTitle"] = json!("bad\nheading");
                rehash_snapshot(s);
            }
            "config revision" => d["spelling"]["configs"][0]["revision"] = json!(""),
            "duplicate content" => {
                let mut s = d["spelling"]["snapshots"][0].clone();
                s["id"] = json!(id());
                d["spelling"]["origins"][0]["snapshotId"] = s["id"].clone();
                d["spelling"]["snapshots"].as_array_mut().unwrap().push(s);
            }
            "historical identifier" => {
                let s = &mut d["spelling"]["snapshots"][0];
                s["sourceCardId"] = json!("invalid");
                rehash_snapshot(s);
                for o in d["spelling"]["origins"].as_array_mut().unwrap() {
                    o["sourceCardId"] = json!("invalid");
                }
            }
            _ => unreachable!(),
        }
        let path = f.dir.path().join(format!("bad-{name}.json"));
        save(&path, &mut bad);
        match f.app.preview_restore(&path) {
            Err(error) if error.code == "invalidBackup" => {}
            other => failures.push(format!("{name}: {other:?}")),
        }
        assert_eq!(
            hash(&export(&mut f)["data"]),
            data_digest,
            "{name} wrote library state"
        );
    }
    assert!(
        failures.is_empty(),
        "semantic checks missing: {}",
        failures.join("; ")
    );
}

#[test]
fn opaque_revisions_do_not_require_an_identifier_format() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    f.card(&source, "Alpha", "معنى");
    create(&mut f, &source);
    let mut raw = export(&mut f);
    raw["data"]["spelling"]["configs"][0]["revision"] = json!("config-epoch-2");
    let fingerprint = raw["data"]["spelling"]["snapshots"][0]["contentSha256"].clone();
    raw["data"]["spelling"]["snapshots"][0]["sourceCardRevision"] =
        json!("opaque-source-version-17");
    let path = f.dir.path().join("opaque.json");
    save(&path, &mut raw);
    assert_eq!(
        raw["data"]["spelling"]["snapshots"][0]["contentSha256"],
        fingerprint
    );
    assert!(f.app.preview_restore(&path).is_ok());
}

#[test]
fn legacy_nested_shapes_cannot_absorb_new_product_fields() {
    let original: Value =
        serde_json::from_slice(include_bytes!("fixtures/backups/v0.1.0.json")).unwrap();
    let f = Fixture::new();
    for path in [
        "/data/settings",
        "/data/profiles/0/config",
        "/data/profiles/0",
        "/data/models/0",
        "/data/sessions/0",
        "/data/reviews/0",
        "/data/actions/0",
        "/data/cards/0/bag",
    ] {
        let mut bad = original.clone();
        bad.pointer_mut(path).unwrap()["futureProductField"] = json!(true);
        let file = f.dir.path().join("legacy-nested.json");
        save(&file, &mut bad);
        assert_eq!(
            f.app.preview_restore(&file).unwrap_err().code,
            "invalidData",
            "{path}"
        );
    }
}
