mod common;
use common::*;
use serde_json::{Value, json};

fn preview(f: &mut Fixture, deck: &str, rows: Value) -> Value {
    f.call(
        "previewImport",
        json!({"deckId":deck,"text":rows.to_string()}),
    )
}

#[test]
fn title_import_inherits_only_when_omitted() {
    let mut f = Fixture::new();
    let deck = f.deck("Titles");
    let p = preview(
        &mut f,
        &deck,
        json!([
            {"questionTitle":" ترجم ","front":"one","back":"a","variants":[
                {"front":"two","back":"b"},
                {"questionTitle":null,"front":"three","back":"c"},
                {"questionTitle":"   ","front":"four","back":"d"},
                {"questionTitle":"<b>أكمل</b>","front":"five","back":"e"}
            ]},
            {"front":"old JSON","back":"default"}
        ]),
    );
    assert_eq!(p["cards"][0]["primary"]["questionTitle"], "ترجم");
    assert_eq!(p["cards"][0]["variants"][0]["questionTitle"], "ترجم");
    assert!(p["cards"][0]["variants"][1]["questionTitle"].is_null());
    assert!(p["cards"][0]["variants"][2]["questionTitle"].is_null());
    assert_eq!(p["cards"][0]["variants"][3]["questionTitle"], "<b>أكمل</b>");
    assert_eq!(p["counts"]["warnings"], 0);
    f.call("commitImport", json!({"previewId":p["previewId"],"libraryRevision":p["libraryRevision"],"selectedCardKeys":[p["cards"][0]["key"],p["cards"][1]["key"]],"excludedFormKeys":[]}));
    let cards = f.call(
        "listCards",
        json!({"deckId":deck,"query":"one","page":1,"pageSize":20}),
    );
    let forms = &cards["items"][0]["forms"];
    assert_eq!(forms[1]["questionTitle"], "ترجم");
    assert!(forms[2]["questionTitle"].is_null());
    assert_eq!(forms[4]["questionTitle"], "<b>أكمل</b>");
    let duplicate = preview(
        &mut f,
        &deck,
        json!([{"questionTitle":"Different title","front":"one","back":"a"}]),
    );
    assert_eq!(duplicate["cards"][0]["primary"]["duplicate"], true);
}

#[test]
fn invalid_titles_are_excludable_and_cannot_enter_cards_or_backups() {
    let mut f = Fixture::new();
    let deck = f.deck("Validation");
    for bad in [
        json!("a".repeat(161)),
        json!(42),
        json!("line\nbreak"),
        json!("\ttrimmed control"),
        json!("nul\u{0}"),
    ] {
        let p = preview(
            &mut f,
            &deck,
            json!([{"front":"one","back":"a","variants":[{"questionTitle":bad,"front":"two","back":"b"}]}]),
        );
        assert_eq!(p["cards"][0]["variants"][0]["valid"], false, "{bad}");
        assert_eq!(
            p["cards"][0]["variants"][0]["issues"][0]["field"],
            "$[0].variants[0].questionTitle"
        );
        assert!(f.app.execute_json(json!({"kind":"createCard","payload":{"deckId":deck,"draft":{"forms":[{"front":"q","back":"a","questionTitle":bad}]}}})).is_err());
    }
    let valid = f.call("createCard", json!({"deckId":deck,"draft":{"forms":[{"front":"q","back":"a","questionTitle":"أ".repeat(160)}]}}));
    assert_eq!(valid["forms"][0]["questionTitle"], "أ".repeat(160));
}

#[test]
fn title_edit_retains_memory_and_form_identity() {
    let mut f = Fixture::new();
    let deck = f.deck("Progress");
    f.card(&deck, "question", "answer");
    let start = f.call("startSession", json!({"deckId":deck}));
    let sid = &start["payload"]["sessionId"];
    let frame = f.call("revealAnswer", json!({"sessionId":sid}))["payload"].clone();
    f.call("rateCard",json!({"sessionId":sid,"presentationId":frame["presentationId"],"rating":"good","actionId":uuid::Uuid::new_v4(),"expectedCardRevision":frame["cardRevision"]}));
    let cards = f.call(
        "listCards",
        json!({"deckId":deck,"query":"","page":1,"pageSize":20}),
    );
    let before = &cards["items"][0];
    let mut forms = before["forms"].clone();
    forms[0]["questionTitle"] = json!(" أكمل ");
    let edited = f.call("updateCard",json!({"cardId":before["id"],"expectedRevision":before["revision"],"draft":{"forms":forms}}));
    assert_eq!(edited["forms"][0]["id"], before["forms"][0]["id"]);
    assert_eq!(edited["forms"][0]["questionTitle"], "أكمل");
    for key in [
        "memory",
        "dueAt",
        "reviewedAt",
        "reviewCount",
        "lapseCount",
        "phase",
        "bag",
    ] {
        assert_eq!(edited[key], before[key], "{key}");
    }
    let path = f.dir.path().join("titles.json");
    f.app.export_snapshot(&path).unwrap();
    let p = f.app.preview_restore(&path).unwrap();
    f.app
        .restore_snapshot(
            p.id,
            p.expected_library_revision,
            &f.dir.path().join("safety.json"),
        )
        .unwrap();
    let after = f.call(
        "listCards",
        json!({"deckId":deck,"query":"","page":1,"pageSize":20}),
    );
    assert_eq!(after["items"][0], edited);
    let mut corrupt: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    corrupt["data"]["cards"][0]["forms"][0]["questionTitle"] = json!("invalid\nheading");
    use openrecall_core::backup::format::{BackupEnvelope, digest};
    let mut typed: BackupEnvelope = serde_json::from_value(corrupt).unwrap();
    typed.payload_sha256 = digest(&typed.data).unwrap();
    std::fs::write(
        &path,
        serde_json::to_vec(&serde_json::to_value(&typed).unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        f.app.preview_restore(&path).unwrap_err().code,
        "invalidBackup"
    );
}
