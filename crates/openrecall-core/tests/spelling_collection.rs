mod common;
use common::spelling::*;
use common::*;
use serde_json::{Value, json};

#[test]
fn singleton_backfill_collects_all_fields_and_forms() {
    let mut f = Fixture::new();
    let source = f.deck("English");
    f.call(
        "createCard",
        json!({"deckId":source,"draft":{"forms":[
            {"questionTitle":"NotVocabulary","front":"Apple","back":"BLUE","notes":"OpenAI apple"},
            {"front":"blue","back":"Apple","notes":"OpenAI"}
        ]}}),
    );
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    assert_eq!(keys(&f, deck), ["apple", "blue", "openai"]);
    assert_eq!(result["deck"]["name"], "مجموعة التدرب على الكتابة");
    assert_eq!(result["deck"]["kind"], "spelling");
    assert_eq!(result["addedWords"], 3);
    let source_deck = f.call("getDeck", json!({"deckId":source}));
    assert_ne!(result["deck"]["profileId"], source_deck["profileId"]);
    let profile = f.call(
        "getProfile",
        json!({"profileId":result["deck"]["profileId"]}),
    );
    assert_eq!(profile["config"]["schedulePenalties"], true);
    let words = f.call(
        "listCards",
        json!({"deckId":deck,"query":"","page":1,"pageSize":100}),
    );
    for word in words["items"].as_array().unwrap() {
        assert_eq!(word["forms"].as_array().unwrap().len(), 1);
        assert_eq!(word["forms"][0]["front"], word["forms"][0]["back"]);
        assert_eq!(word["forms"][0]["language"], "en");
        assert_eq!(word["forms"][0]["notes"], "");
    }
    let before = snapshot(&mut f);
    let scanned = f.call(
        "rescanSpellingSource",
        json!({"deckId":deck,"expectedRevision":result["config"]["revision"]}),
    );
    assert_eq!(scanned["addedWords"], 0);
    assert_eq!(snapshot(&mut f), before);
    let duplicate = f
        .app
        .execute_json(json!({"kind":"createSpellingDeck","payload":{"sourceDeckId":source}}))
        .unwrap_err();
    assert_eq!(duplicate.code, "spellingDeckExists");
    assert_eq!(duplicate.context.as_deref(), Some(deck));
    f.reopen();
    assert_eq!(keys(&f, deck), ["apple", "blue", "openai"]);
    assert_eq!(
        f.call("getSpellingConfig", json!({"deckId":deck})),
        result["config"]
    );
    let data = snapshot(&mut f);
    assert_eq!(data["spelling"]["snapshots"].as_array().unwrap().len(), 2);
    assert_eq!(data["spelling"]["origins"].as_array().unwrap().len(), 6);
}

#[test]
fn source_switch_accumulates_without_resetting_words() {
    let mut f = Fixture::new();
    let one = f.deck("one");
    f.card(&one, "Alpha", "Alpha");
    let two = f.deck("two");
    f.card(&two, "alpha", "Beta");
    let result = create(&mut f, &one);
    let deck = result["deck"]["id"].as_str().unwrap();
    let view = f.call("startSession", json!({"deckId":deck}));
    let sid = view["payload"]["sessionId"].clone();
    let frame = if view["payload"]["canRate"] == true {
        view["payload"].clone()
    } else {
        f.call("revealAnswer", json!({"sessionId":sid}))["payload"].clone()
    };
    f.call("rateCard",json!({"sessionId":sid,"presentationId":frame["presentationId"],"rating":"good","actionId":uuid::Uuid::new_v4(),"expectedCardRevision":frame["cardRevision"]}));
    let old = card(&mut f, deck, "alpha");
    assert!(old["memory"].is_object());
    let changed = f.call(
        "setSpellingSource",
        json!({"deckId":deck,"sourceDeckId":two,"expectedRevision":result["config"]["revision"]}),
    );
    assert_eq!(changed["addedWords"], 1);
    assert_eq!(keys(&f, deck), ["alpha", "beta"]);
    let after = card(&mut f, deck, "alpha");
    assert_eq!(after, old);
    assert_eq!(
        snapshot(&mut f)["spelling"]["origins"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(changed["config"]["sourceDeckId"], two);
    let before = snapshot(&mut f);
    let stale=f.app.execute_json(json!({"kind":"setSpellingSource","payload":{"deckId":deck,"sourceDeckId":one,"expectedRevision":result["config"]["revision"]}})).unwrap_err();
    assert_eq!(stale.code, "conflict");
    assert_eq!(snapshot(&mut f), before);
}

#[test]
fn stale_binding_and_failed_backfill_roll_back() {
    let mut f = Fixture::new();
    let source = f.deck("source");
    f.card(&source, "Alpha", "Beta");
    let before = snapshot(&mut f);
    let db = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    db.execute_batch("CREATE TRIGGER fail_collection BEFORE INSERT ON spelling_words BEGIN SELECT RAISE(ABORT,'synthetic failure'); END").unwrap();
    let failed = f
        .app
        .execute_json(json!({"kind":"createSpellingDeck","payload":{"sourceDeckId":source}}));
    assert!(failed.is_err());
    assert_eq!(snapshot(&mut f), before);
    db.execute_batch("DROP TRIGGER fail_collection").unwrap();
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    let before = snapshot(&mut f);
    for other in [
        Value::String(deck.into()),
        Value::String(uuid::Uuid::new_v4().to_string()),
    ] {
        assert!(f.app.execute_json(json!({"kind":"setSpellingSource","payload":{"deckId":deck,"sourceDeckId":other,"expectedRevision":result["config"]["revision"]}})).is_err());
        assert_eq!(snapshot(&mut f), before);
    }
}

#[test]
fn empty_source_is_valid_and_later_failed_binding_preserves_the_old_library() {
    let mut f = Fixture::new();
    let empty = f.deck("empty");
    let result = create(&mut f, &empty);
    let deck = result["deck"]["id"].as_str().unwrap();
    assert_eq!(result["addedWords"], 0);
    assert!(keys(&f, deck).is_empty());
    let source = f.deck("populated");
    f.card(&source, "New", "Word");
    let before = snapshot(&mut f);
    let db = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    db.execute_batch("CREATE TRIGGER fail_collection BEFORE INSERT ON spelling_words BEGIN SELECT RAISE(ABORT,'synthetic failure'); END").unwrap();
    assert!(f.app.execute_json(json!({"kind":"setSpellingSource","payload":{"deckId":deck,"sourceDeckId":source,"expectedRevision":result["config"]["revision"]}})).is_err());
    assert_eq!(snapshot(&mut f), before);
}
