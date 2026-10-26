mod common;
use common::*;
use serde_json::json;
#[test]
fn forms_are_one_card_and_edits_preserve_progress_and_due_instants() {
    let mut f = Fixture::new();
    let deck = f.call("createDeck", json!({"name":"اختبار","profileId":null}));
    let card=f.call("createCard",json!({"deckId":deck["id"],"draft":{"forms":[
        {"front":"سؤال عربي 100%_ ' 🌍","back":"English answer","notes":"ملاحظة"},
        {"front":"日本語","back":"الإجابة","notes":""}, {"front":"Alternative","back":"جواب","notes":""}]}}));
    assert_eq!(card["forms"].as_array().unwrap().len(), 3);
    assert!(card["memory"].is_null());
    assert_eq!(card["phase"], "new");
    let detail = f.call("getDeck", json!({"deckId":deck["id"]}));
    assert_eq!(detail["logicalCards"], 1);
    assert_eq!(detail["newCards"], 1);
    assert_eq!(detail["dueNow"], 1);
    let mut forms = card["forms"].clone();
    forms[0]["front"] = json!("Edited سؤال");
    let edited = f.call(
        "updateCard",
        json!({"cardId":card["id"],"expectedRevision":card["revision"],"draft":{"forms":forms}}),
    );
    assert_eq!(edited["dueAt"], card["dueAt"]);
    assert_eq!(edited["memory"], card["memory"]);
    assert_eq!(edited["forms"][1]["id"], card["forms"][1]["id"]);
    let mut settings = f.call("bootstrap", json!({}))["settings"].clone();
    let rev = settings["revision"].clone();
    settings["timezone"] = json!("Asia/Tokyo");
    f.call(
        "saveSettings",
        json!({"settings":settings,"expectedRevision":rev}),
    );
    f.reopen();
    assert_eq!(
        f.call("getCard", json!({"cardId":card["id"]}))["dueAt"],
        card["dueAt"]
    );
    let found = f.call(
        "listCards",
        json!({"deckId":deck["id"],"query":"日本語","page":1,"pageSize":10}),
    );
    assert_eq!(found["total"], 1);
    let not_all = f.call(
        "listCards",
        json!({"deckId":deck["id"],"query":"%_","page":1,"pageSize":10}),
    );
    assert_eq!(not_all["total"], 0);
    f.call(
        "deleteCard",
        json!({"cardId":edited["id"],"expectedRevision":edited["revision"]}),
    );
    assert_eq!(
        f.call("getDeck", json!({"deckId":deck["id"]}))["logicalCards"],
        0
    );
}
#[test]
fn card_validation_and_pagination_guard_ownership() {
    let mut f = Fixture::new();
    let deck = f.call("createDeck", json!({"name":"deck","profileId":null}));
    for forms in [
        json!([]),
        json!([{"front":" ","back":"x"}]),
        json!([{"front":"x","back":""}]),
    ] {
        assert!(f.app.execute_json(json!({"kind":"createCard","payload":{"deckId":deck["id"],"draft":{"forms":forms}}})).is_err());
    }
    let mut cards = vec![];
    for n in 0..12 {
        cards.push(f.call(
            "createCard",
            json!({"deckId":deck["id"],"draft":{"forms":[{"front":format!("q{n}"),"back":"a"}]}}),
        ));
    }
    let page = f.call(
        "listCards",
        json!({"deckId":deck["id"],"query":"","page":2,"pageSize":10}),
    );
    assert_eq!(page["total"], 12);
    assert_eq!(page["items"].as_array().unwrap().len(), 2);
    assert!(f.app.execute_json(json!({"kind":"updateCard","payload":{"cardId":cards[0]["id"],"expectedRevision":cards[0]["revision"],"draft":{"forms":cards[1]["forms"]}}})).is_err());
    let latest = f.call("getDeck", json!({"deckId":deck["id"]}));
    f.call(
        "deleteDeck",
        json!({"deckId":deck["id"],"expectedRevision":latest["revision"]}),
    );
    assert!(
        f.app
            .execute_json(json!({"kind":"getCard","payload":{"cardId":cards[0]["id"]}}))
            .is_err()
    );
}
