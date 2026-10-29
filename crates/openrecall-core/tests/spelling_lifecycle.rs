mod common;
use common::{spelling::*, *};
use serde_json::{Value, json};

fn edit(f: &mut Fixture, card: &Value, front: &str, notes: &str) -> Value {
    let mut forms = card["forms"].clone();
    forms[0]["front"] = json!(front);
    forms[0]["notes"] = json!(notes);
    f.call(
        "updateCard",
        json!({"cardId":card["id"],"expectedRevision":card["revision"],"draft":{"forms":forms}}),
    )
}
fn contexts(data: &Value, key: &str) -> Vec<Value> {
    let registry = &data["spelling"];
    let word = registry["words"]
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["canonical"] == key)
        .unwrap();
    registry["origins"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["spellingCardId"] == word["cardId"])
        .map(|o| {
            registry["snapshots"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["id"] == o["snapshotId"])
                .unwrap()
                .clone()
        })
        .collect()
}
fn reserve_after_one_rating(f: &mut Fixture, deck: &str) -> (Value, Value) {
    let view = f.call("startSession", json!({"deckId":deck}));
    let sid = view["payload"]["sessionId"].clone();
    let frame = if view["payload"]["canRate"] == true {
        view["payload"].clone()
    } else {
        f.call("revealAnswer", json!({"sessionId":sid}))["payload"].clone()
    };
    f.call("rateCard",json!({"sessionId":sid,"presentationId":frame["presentationId"],"rating":"good","actionId":uuid::Uuid::new_v4(),"expectedCardRevision":frame["cardRevision"]}));
    (
        sid.clone(),
        f.call("readSession", json!({"sessionId":sid}))["payload"].clone(),
    )
}

#[test]
fn red_blue_edit_freezes_red_and_updates_blue() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    let source_id = f.card(&source, "red blue", "معنى");
    let source_card = f.call("getCard", json!({"cardId":source_id}));
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    let (sid, reserved) = reserve_after_one_rating(&mut f, deck);
    let before = snapshot(&mut f);
    edit(&mut f, &source_card, "blue green", "");
    assert_eq!(keys(&f, deck), ["blue", "green", "red"]);
    let after = snapshot(&mut f);
    assert_eq!(contexts(&after, "red")[0]["front"], "red blue");
    assert_eq!(contexts(&after, "blue")[0]["front"], "blue green");
    assert_eq!(
        contexts(&after, "green")[0]["id"],
        contexts(&after, "blue")[0]["id"]
    );
    assert_eq!(after["spelling"]["snapshots"].as_array().unwrap().len(), 2);
    for word in before["cards"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|w| w["deckId"] == deck)
    {
        assert_eq!(
            after["cards"]
                .as_array()
                .unwrap()
                .iter()
                .find(|w| w["id"] == word["id"])
                .unwrap(),
            word
        );
    }
    let read = f.call("readSession", json!({"sessionId":sid}));
    assert_eq!(
        read["payload"]["presentationId"],
        reserved["presentationId"]
    );
    assert_eq!(read["payload"]["cardRevision"], reserved["cardRevision"]);
}

#[test]
fn shared_snapshot_refresh_preserves_learning_revision_and_undo() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    let source_id = f.card(&source, "red blue", "معنى");
    let original = f.call("getCard", json!({"cardId":source_id}));
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    let (sid, reserved) = reserve_after_one_rating(&mut f, deck);
    let before = f.call("getDeck", json!({"deckId":deck}));
    let before_data = snapshot(&mut f);
    edit(&mut f, &original, "red blue", "ملاحظة جديدة");
    let after = f.call("getDeck", json!({"deckId":deck}));
    assert_eq!(after["dataRevision"], before["dataRevision"]);
    let data = snapshot(&mut f);
    assert_ne!(data["libraryRevision"], before_data["libraryRevision"]);
    assert_eq!(data["spelling"]["snapshots"].as_array().unwrap().len(), 1);
    assert_eq!(
        contexts(&data, "red")[0]["id"],
        contexts(&data, "blue")[0]["id"]
    );
    assert_eq!(contexts(&data, "red")[0]["notes"], "ملاحظة جديدة");
    assert_eq!(data["undoRecords"], before_data["undoRecords"]);
    let view = f.call("readSession", json!({"sessionId":sid}));
    assert_eq!(
        view["payload"]["presentationId"],
        reserved["presentationId"]
    );
    assert_eq!(view["payload"]["cardRevision"], reserved["cardRevision"]);
    assert!(
        f.call("undoReview", json!({"sessionId":sid}))["payload"]["presentationId"].is_string()
    );
}

#[test]
fn deleted_sources_leave_words_contexts_and_history() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    let original=f.call("createCard",json!({"deckId":source,"draft":{"forms":[{"front":"red blue","back":"معنى"},{"front":"red","back":"معنى"}]}}));
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    let (sid, reserved) = reserve_after_one_rating(&mut f, deck);
    let before = snapshot(&mut f);
    let edited=f.call("updateCard",json!({"cardId":original["id"],"expectedRevision":original["revision"],"draft":{"forms":[original["forms"][1]]}}));
    f.call(
        "deleteCard",
        json!({"cardId":edited["id"],"expectedRevision":edited["revision"]}),
    );
    let source_deck = f.call("getDeck", json!({"deckId":source}));
    f.call(
        "deleteDeck",
        json!({"deckId":source,"expectedRevision":source_deck["revision"]}),
    );
    assert_eq!(keys(&f, deck), ["blue", "red"]);
    let config = f.call("getSpellingConfig", json!({"deckId":deck}));
    assert!(config["sourceDeckId"].is_null());
    assert_eq!(config["sourceDeckName"], "Source");
    assert_ne!(config["revision"], result["config"]["revision"]);
    let after = snapshot(&mut f);
    assert_eq!(contexts(&after, "blue")[0]["front"], "red blue");
    assert_eq!(after["reviews"], before["reviews"]);
    assert_eq!(after["undoRecords"], before["undoRecords"]);
    assert_eq!(
        f.call("readSession", json!({"sessionId":sid}))["payload"]["presentationId"],
        reserved["presentationId"]
    );
    let report = f
        .app
        .export_snapshot(&f.dir.path().join("deleted.json"))
        .unwrap();
    assert!(
        f.app
            .preview_restore(std::path::Path::new(&report.path))
            .is_ok()
    );
}

#[test]
fn former_binding_updates_context_without_collecting_new_words() {
    let mut f = Fixture::new();
    let one = f.deck("one");
    let source_id = f.card(&one, "red blue", "معنى");
    let original = f.call("getCard", json!({"cardId":source_id}));
    let two = f.deck("two");
    f.card(&two, "gold", "معنى");
    let result = create(&mut f, &one);
    let deck = result["deck"]["id"].as_str().unwrap();
    f.call(
        "setSpellingSource",
        json!({"deckId":deck,"sourceDeckId":two,"expectedRevision":result["config"]["revision"]}),
    );
    edit(&mut f, &original, "blue green", "");
    f.card(&one, "violet", "معنى");
    assert_eq!(keys(&f, deck), ["blue", "gold", "red"]);
    let data = snapshot(&mut f);
    assert_eq!(contexts(&data, "blue")[0]["front"], "blue green");
    assert_eq!(contexts(&data, "red")[0]["front"], "red blue");
    f.card(&two, "silver gold", "معنى");
    assert_eq!(keys(&f, deck), ["blue", "gold", "red", "silver"]);
}

#[test]
fn committed_imports_collect_but_preview_and_failed_content_writes_do_not() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    let source_id = f.card(&source, "alpha", "معنى");
    let original = f.call("getCard", json!({"cardId":source_id}));
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    let before = snapshot(&mut f);
    let preview=f.call("previewImport",json!({"deckId":source,"text":json!([{"front":"new","back":"معنى"},{"front":"crash","back":"معنى","variants":[{"front":"delta","back":"معنى"}]}]).to_string()}));
    assert_eq!(snapshot(&mut f), before);
    let db = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    db.execute_batch("CREATE TRIGGER fail_incremental BEFORE INSERT ON spelling_words WHEN NEW.canonical='crash' BEGIN SELECT RAISE(ABORT,'synthetic failure'); END").unwrap();
    for request in [
        json!({"kind":"createCard","payload":{"deckId":source,"draft":{"forms":[{"front":"crash","back":"معنى"}]}}}),
        json!({"kind":"updateCard","payload":{"cardId":original["id"],"expectedRevision":original["revision"],"draft":{"forms":[{"id":original["forms"][0]["id"],"front":"alpha crash","back":"معنى"}]}}}),
        json!({"kind":"commitImport","payload":{"previewId":preview["previewId"],"libraryRevision":preview["libraryRevision"],"selectedCardKeys":[preview["cards"][0]["key"],preview["cards"][1]["key"]],"excludedFormKeys":[]}}),
    ] {
        assert!(f.app.execute_json(request).is_err());
        assert_eq!(snapshot(&mut f), before);
    }
    db.execute_batch("DROP TRIGGER fail_incremental").unwrap();
    f.call("commitImport",json!({"previewId":preview["previewId"],"libraryRevision":preview["libraryRevision"],"selectedCardKeys":[preview["cards"][0]["key"],preview["cards"][1]["key"]],"excludedFormKeys":[]}));
    assert_eq!(keys(&f, deck), ["alpha", "crash", "delta", "new"]);
}
