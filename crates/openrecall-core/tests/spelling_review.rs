mod common;
use common::{spelling::*, *};
use serde_json::{Value, json};

fn rate(f: &mut Fixture, frame: &Value, rating: &str, action: &str) -> Value {
    f.call("rateCard",json!({"sessionId":frame["sessionId"],"presentationId":frame["presentationId"],"rating":rating,"actionId":action,"expectedCardRevision":frame["cardRevision"]}))
}
fn sources(f: &mut Fixture, word: &str, page: u32) -> Value {
    f.call(
        "listSpellingSources",
        json!({"cardId":word,"page":page,"pageSize":10}),
    )
}

#[test]
fn spelling_rates_without_reveal_and_normal_still_requires_it() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    let source_id = f.card(&source, "Alpha", "معنى");
    let original = f.call("getCard", json!({"cardId":source_id}));
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    let frame = f.call("startSession", json!({"deckId":deck}))["payload"].clone();
    assert_eq!(frame["mode"], "spelling");
    assert_eq!(frame["canRate"], true);
    assert_eq!(frame["answerRevealed"], false);
    assert!(frame["previews"].is_object());
    assert!(frame["previewDueAt"].is_object());
    assert_eq!(
        f.app
            .execute_json(json!({"kind":"revealAnswer","payload":{"sessionId":frame["sessionId"]}}))
            .unwrap_err()
            .code,
        "spellingRevealNotApplicable"
    );
    let action = uuid::Uuid::new_v4().to_string();
    let graded = rate(&mut f, &frame, "good", &action);
    assert_eq!(graded["state"], "summary");
    assert_eq!(graded["payload"]["attempts"], 1);
    assert_eq!(
        rate(&mut f, &frame, "good", &action)["payload"]["attempts"],
        1
    );
    assert_eq!(f.call("getCard", json!({"cardId":source_id})), original);
    let db = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    let diagnostic:(String,bool)=db.query_row("SELECT d.kind,json_extract(u.payload,'$.sessionBefore.current.answerRevealed') FROM undo_records u JOIN sessions s ON s.id=u.session_id JOIN decks d ON d.id=s.deck_id",[],|row|Ok((row.get(0)?,row.get(1)?))).unwrap();
    assert_eq!(diagnostic, ("spelling".into(), false));
    assert_eq!(snapshot(&mut f)["reviews"].as_array().unwrap().len(), 1);
    let undone = f.call("undoReview", json!({"sessionId":frame["sessionId"]}));
    assert_eq!(undone["payload"]["presentationId"], frame["presentationId"]);
    assert_eq!(undone["payload"]["canRate"], true);
    assert_eq!(undone["payload"]["answerRevealed"], false);
    assert!(undone["payload"]["card"]["memory"].is_null());
    assert_eq!(undone["payload"]["card"]["reviewCount"], 0);
    let normal = f.call("startSession", json!({"deckId":source}))["payload"].clone();
    assert_eq!(normal["mode"], "standard");
    assert_eq!(normal["canRate"], false);
    assert!(normal["previews"].is_null());
    let error=f.app.execute_json(json!({"kind":"rateCard","payload":{"sessionId":normal["sessionId"],"presentationId":normal["presentationId"],"rating":"good","actionId":uuid::Uuid::new_v4(),"expectedCardRevision":normal["cardRevision"]}})).unwrap_err();
    assert_eq!(error.code, "answerHidden");
    let revealed =
        f.call("revealAnswer", json!({"sessionId":normal["sessionId"]}))["payload"].clone();
    assert_eq!(revealed["canRate"], true);
    rate(&mut f, &revealed, "good", &uuid::Uuid::new_v4().to_string());
    let path = f.dir.path().join("ordinary-undo.json");
    f.app.export_snapshot(&path).unwrap();
    use openrecall_core::backup::format::{BackupEnvelope, digest};
    let mut forged: BackupEnvelope =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let ordinary = forged
        .data
        .undo_records
        .iter_mut()
        .find(|undo| undo.card_before.deck_id == source)
        .unwrap();
    ordinary
        .session_before
        .current
        .as_mut()
        .unwrap()
        .answer_revealed = false;
    forged.payload_sha256 = digest(&forged.data).unwrap();
    std::fs::write(
        &path,
        serde_json::to_vec(&serde_json::to_value(&forged).unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        f.app.preview_restore(&path).unwrap_err().code,
        "invalidBackup"
    );
}

#[test]
fn context_refresh_during_rating_does_not_stale_word() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    let source_id = f.card(&source, "Alpha", "معنى");
    let original = f.call("getCard", json!({"cardId":source_id}));
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    let frame = f.call("startSession", json!({"deckId":deck}))["payload"].clone();
    let mut forms = original["forms"].clone();
    forms[0]["notes"] = json!("سياق جديد");
    f.call(
        "updateCard",
        json!({"cardId":source_id,"expectedRevision":original["revision"],"draft":{"forms":forms}}),
    );
    let context = sources(&mut f, frame["card"]["id"].as_str().unwrap(), 1);
    assert_eq!(context["items"][0]["snapshot"]["notes"], "سياق جديد");
    assert_eq!(context["items"][0]["availability"], "current");
    let fresh = f.call("getDeck", json!({"deckId":source}));
    f.call(
        "deleteDeck",
        json!({"deckId":source,"expectedRevision":fresh["revision"]}),
    );
    assert_eq!(
        rate(&mut f, &frame, "easy", &uuid::Uuid::new_v4().to_string())["state"],
        "summary"
    );
    assert_eq!(
        sources(&mut f, frame["card"]["id"].as_str().unwrap(), 1)["items"][0]["availability"],
        "deleted"
    );
}

#[test]
fn ordinary_source_rating_does_not_mark_unchanged_context_as_changed() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    f.card(&source, "Alpha", "معنى");
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    let word = card(&mut f, deck, "Alpha");
    let frame = f.call("startSession", json!({"deckId":source}))["payload"].clone();
    let revealed =
        f.call("revealAnswer", json!({"sessionId":frame["sessionId"]}))["payload"].clone();
    rate(&mut f, &revealed, "good", &uuid::Uuid::new_v4().to_string());
    let fresh = f.call("getDeck", json!({"deckId":source}));
    f.call(
        "renameDeck",
        json!({"deckId":source,"name":"renamed","expectedRevision":fresh["revision"]}),
    );
    let page = sources(&mut f, word["id"].as_str().unwrap(), 1);
    assert_eq!(page["items"][0]["availability"], "current");
    let current = f.call("getCard", json!({"cardId":frame["card"]["id"]}));
    assert_ne!(
        current["revision"],
        page["items"][0]["snapshot"]["sourceCardRevision"]
    );
}

#[test]
fn retained_sources_page_after_every_live_source_is_gone() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    for i in 0..11 {
        f.card(&source, "Alpha", &format!("معنى {i}"));
        f.advance(1);
    }
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    let word = card(&mut f, deck, "Alpha");
    let word_id = word["id"].as_str().unwrap();
    let first = sources(&mut f, word_id, 1);
    let second = sources(&mut f, word_id, 2);
    assert_eq!(first["total"], 11);
    assert_eq!(first["items"].as_array().unwrap().len(), 10);
    assert_eq!(second["items"].as_array().unwrap().len(), 1);
    assert!(
        first["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v["availability"] == "current")
    );
    let fresh = f.call("getDeck", json!({"deckId":source}));
    f.call(
        "deleteDeck",
        json!({"deckId":source,"expectedRevision":fresh["revision"]}),
    );
    let after = sources(&mut f, word_id, 1);
    assert_eq!(after["total"], 11);
    for (old, new) in first["items"]
        .as_array()
        .unwrap()
        .iter()
        .zip(after["items"].as_array().unwrap())
    {
        assert_eq!(old["snapshot"], new["snapshot"]);
        assert_eq!(new["availability"], "deleted");
    }
    assert!(f.app.execute_json(json!({"kind":"listSpellingSources","payload":{"cardId":word_id,"page":0,"pageSize":10}})).is_err());
}

#[test]
fn spelling_filters_due_arrivals_and_restart_keep_native_presentations() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    f.card(&source, "Alpha Beta", "معنى");
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    let first = f.call("startSession", json!({"deckId":deck}))["payload"].clone();
    assert_eq!(first["canRate"], true);
    let second =
        rate(&mut f, &first, "again", &uuid::Uuid::new_v4().to_string())["payload"].clone();
    f.advance(61);
    let polled = f.call("readSession", json!({"sessionId":first["sessionId"]}));
    assert_eq!(
        polled["payload"]["presentationId"],
        second["presentationId"]
    );
    let again = rate(&mut f, &second, "good", &uuid::Uuid::new_v4().to_string())["payload"].clone();
    assert_eq!(again["card"]["id"], first["card"]["id"]);
    let empty = f.call(
        "setReviewFilter",
        json!({"sessionId":first["sessionId"],"filter":"new"}),
    );
    assert_eq!(empty["state"], "emptyFilter");
    let restored = f.call(
        "setReviewFilter",
        json!({"sessionId":first["sessionId"],"filter":"all"}),
    );
    assert_eq!(
        restored["payload"]["presentationId"],
        again["presentationId"]
    );
    f.reopen();
    let resumed = f.call("resumeSession", json!({"sessionId":first["sessionId"]}));
    assert_eq!(resumed["payload"]["mode"], "spelling");
    assert_eq!(resumed["payload"]["canRate"], true);
    assert_eq!(
        resumed["payload"]["presentationId"],
        again["presentationId"]
    );
}

#[test]
fn removed_form_is_changed_while_its_card_still_exists() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    let original=f.call("createCard",json!({"deckId":source,"draft":{"forms":[{"front":"Alpha","back":"معنى"},{"front":"Beta","back":"معنى"}]}}));
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    let word = card(&mut f, deck, "Alpha");
    f.call("updateCard",json!({"cardId":original["id"],"expectedRevision":original["revision"],"draft":{"forms":[original["forms"][1]]}}));
    assert_eq!(
        sources(&mut f, word["id"].as_str().unwrap(), 1)["items"][0]["availability"],
        "changed"
    );
}
