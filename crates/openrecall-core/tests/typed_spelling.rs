mod common;
use common::{Fixture, spelling};
use serde_json::{Value, json};

fn draft(form_id: Option<&Value>, correct: &str) -> Value {
    json!({"forms":[{"id":form_id,"questionType":"multipleChoice","front":"اختر اللون",
        "multipleChoice":{"choices":[{"key":"b","text":"blue"},{"key":"g","text":"green"},{"key":"x","text":"wrongword"}],"correctChoiceKeys":[correct]}}]})
}

#[test]
fn former_correct_word_keeps_its_eligible_context_when_it_becomes_a_distractor() {
    let mut f = Fixture::new();
    let source = f.deck("Colors");
    let source_card = f.call(
        "createCard",
        json!({"deckId":source,"draft":draft(None,"b")}),
    );
    let result = spelling::create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    assert_eq!(spelling::keys(&f, deck), vec!["blue"]);
    let p = f.call("startSession", json!({"deckId":deck}))["payload"].clone();
    f.call("rateCard",json!({"sessionId":p["sessionId"],"presentationId":p["presentationId"],"expectedCardRevision":p["cardRevision"],"rating":"good","actionId":uuid::Uuid::new_v4().to_string()}));
    let blue = f.call("getCard", json!({"cardId":p["card"]["id"]}));
    f.call("updateCard",json!({"cardId":source_card["id"],"expectedRevision":source_card["revision"],"draft":draft(Some(&source_card["forms"][0]["id"]),"g")}));
    assert_eq!(spelling::keys(&f, deck), vec!["blue", "green"]);
    let after = f.call("getCard", json!({"cardId":blue["id"]}));
    assert_eq!(after, blue);
    let context = f.call(
        "listSpellingSources",
        json!({"cardId":blue["id"],"page":1,"pageSize":20}),
    );
    let old = &context["items"][0]["snapshot"];
    assert_eq!(
        old["multipleChoice"]["correctChoiceIds"][0],
        old["multipleChoice"]["choices"][0]["id"]
    );
    assert_eq!(context["items"][0]["availability"], "changed");
    let d = f.call("getDeck", json!({"deckId":source}));
    f.call(
        "deleteDeck",
        json!({"deckId":source,"expectedRevision":d["revision"]}),
    );
    let deleted = f.call(
        "listSpellingSources",
        json!({"cardId":blue["id"],"page":1,"pageSize":20}),
    );
    assert_eq!(deleted["items"][0]["availability"], "deleted");
    assert_eq!(deleted["items"][0]["snapshot"], *old);
}
