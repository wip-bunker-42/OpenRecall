mod common;
use common::{spelling::*, *};
use serde_json::{Value, json};

fn config(f: &mut Fixture, deck: &str) -> Value {
    f.call("getSpellingConfig", json!({"deckId":deck}))
}
fn delete_word(f: &mut Fixture, deck: &str, text: &str) -> Value {
    let word = card(f, deck, text);
    f.call(
        "deleteCard",
        json!({"cardId":word["id"],"expectedRevision":word["revision"]}),
    );
    word
}

#[test]
fn manual_delete_prevents_immediate_recollection() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    let source_id = f.card(&source, "red blue", "معنى");
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    delete_word(&mut f, deck, "blue");
    let cfg = config(&mut f, deck);
    assert_ne!(cfg["revision"], result["config"]["revision"]);
    f.call(
        "rescanSpellingSource",
        json!({"deckId":deck,"expectedRevision":cfg["revision"]}),
    );
    assert_eq!(keys(&f, deck), ["red"]);
    let original = f.call("getCard", json!({"cardId":source_id}));
    let mut forms = original["forms"].clone();
    forms[0]["front"] = json!("red blue green");
    f.call(
        "updateCard",
        json!({"cardId":source_id,"expectedRevision":original["revision"],"draft":{"forms":forms}}),
    );
    assert_eq!(keys(&f, deck), ["green", "red"]);
    let page = f.call(
        "listSpellingExclusions",
        json!({"deckId":deck,"page":1,"pageSize":10}),
    );
    assert_eq!(page["total"], 1);
    assert_eq!(page["items"][0]["canonical"], "blue");
    assert!(f.app.execute_json(json!({"kind":"listSpellingExclusions","payload":{"deckId":deck,"page":0,"pageSize":10}})).is_err());
    f.reopen();
    assert_eq!(keys(&f, deck), ["green", "red"]);
    assert_eq!(
        snapshot(&mut f)["spelling"]["exclusions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn allow_word_readds_only_from_current_source_with_new_progress() {
    let mut f = Fixture::new();
    let one = f.deck("one");
    f.card(&one, "Alpha", "معنى");
    let two = f.deck("two");
    f.card(&two, "Beta", "معنى");
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
    let old = delete_word(&mut f, deck, "Alpha");
    assert_eq!(old["reviewCount"], 1);
    let cfg = config(&mut f, deck);
    let switched = f.call(
        "setSpellingSource",
        json!({"deckId":deck,"sourceDeckId":two,"expectedRevision":cfg["revision"]}),
    );
    let allowed=f.call("allowSpellingWord",json!({"deckId":deck,"canonical":"alpha","expectedRevision":switched["config"]["revision"]}));
    assert_eq!(allowed["addedWords"], 0);
    assert_eq!(keys(&f, deck), ["beta"]);
    f.card(&two, "Alpha", "معنى");
    let new = card(&mut f, deck, "Alpha");
    assert_ne!(new["id"], old["id"]);
    assert_eq!(new["reviewCount"], 0);
    assert!(new["memory"].is_null());
    delete_word(&mut f, deck, "Alpha");
    let cfg = config(&mut f, deck);
    let immediate = f.call(
        "allowSpellingWord",
        json!({"deckId":deck,"canonical":"alpha","expectedRevision":cfg["revision"]}),
    );
    assert_eq!(immediate["addedWords"], 1);
    assert_eq!(card(&mut f, deck, "Alpha")["reviewCount"], 0);
}

#[test]
fn managed_deck_rejects_generic_card_writes() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    f.card(&source, "Alpha", "معنى");
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    let word = card(&mut f, deck, "Alpha");
    let before = snapshot(&mut f);
    for request in [
        json!({"kind":"createCard","payload":{"deckId":deck,"draft":{"forms":[{"front":"manual","back":"manual"}]}}}),
        json!({"kind":"updateCard","payload":{"cardId":word["id"],"expectedRevision":word["revision"],"draft":{"forms":[{"id":word["forms"][0]["id"],"front":"manual","back":"manual"}]}}}),
        json!({"kind":"previewImport","payload":{"deckId":deck,"text":"[]"}}),
    ] {
        assert_eq!(
            f.app.execute_json(request).unwrap_err().code,
            "managedSpellingDeck"
        );
        assert_eq!(snapshot(&mut f), before);
    }
}

#[test]
fn normal_import_does_not_deduplicate_against_spelling_task() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    f.card(&source, "Word", "معنى");
    create(&mut f, &source);
    let target = f.deck("Target");
    let text = json!([{"front":"Word","back":"Word"}]).to_string();
    let p = f.call("previewImport", json!({"deckId":target,"text":text}));
    assert_eq!(p["counts"]["validCards"], 1);
    f.call("commitImport",json!({"previewId":p["previewId"],"libraryRevision":p["libraryRevision"],"selectedCardKeys":[p["cards"][0]["key"]],"excludedFormKeys":[]}));
    let duplicate = f.call("previewImport", json!({"deckId":target,"text":text}));
    assert_eq!(duplicate["cards"][0]["primary"]["duplicate"], true);
}

#[test]
fn stale_allow_and_whole_deck_deletion_preserve_scope() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    let source_id = f.card(&source, "red blue", "معنى");
    let result = create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    delete_word(&mut f, deck, "blue");
    let before = snapshot(&mut f);
    assert_eq!(f.app.execute_json(json!({"kind":"allowSpellingWord","payload":{"deckId":deck,"canonical":"blue","expectedRevision":result["config"]["revision"]}})).unwrap_err().code,"conflict");
    assert_eq!(snapshot(&mut f), before);
    let fresh = f.call("getDeck", json!({"deckId":deck}));
    let renamed = f.call(
        "renameDeck",
        json!({"deckId":deck,"name":"Personal name","expectedRevision":fresh["revision"]}),
    );
    f.call(
        "deleteDeck",
        json!({"deckId":deck,"expectedRevision":renamed["revision"]}),
    );
    assert_eq!(
        f.call("getCard", json!({"cardId":source_id}))["forms"][0]["front"],
        "red blue"
    );
    let data = snapshot(&mut f);
    for name in ["configs", "words", "snapshots", "origins", "exclusions"] {
        assert!(
            data["spelling"][name].as_array().unwrap().is_empty(),
            "{name}"
        );
    }
    let new = create(&mut f, &source);
    assert_ne!(new["deck"]["id"], deck);
    assert_eq!(new["addedWords"], 2);
}
