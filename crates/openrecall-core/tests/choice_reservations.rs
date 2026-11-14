mod common;
use common::Fixture;
use serde_json::{Value, json};

fn card(f: &mut Fixture, deck: &str, shuffle: bool) -> Value {
    f.call("createCard",json!({"deckId":deck,"draft":{"forms":[{
        "questionType":"multipleChoice","front":"Choose one",
        "multipleChoice":{"choices":[{"key":"a","text":"Paris"},{"key":"b","text":"Rome"}],"correctChoiceKeys":["a"],"shuffleChoices":shuffle}
    }]}}))
}

#[test]
fn authored_order_survives_polling_filter_parking_and_restart() {
    let mut f = Fixture::new();
    let deck = f.deck("Choices");
    let saved = card(&mut f, &deck, false);
    let initial = f.call("startSession", json!({"deckId":deck}))["payload"].clone();
    let order = json!([
        saved["forms"][0]["multipleChoice"]["choices"][0]["id"],
        saved["forms"][0]["multipleChoice"]["choices"][1]["id"]
    ]);
    assert_eq!(initial["choices"]["order"], order);
    let session = &initial["sessionId"];
    let read = f.call("readSession", json!({"sessionId":session}));
    assert_eq!(read["payload"]["choices"]["order"], order);
    f.call(
        "setReviewFilter",
        json!({"sessionId":session,"filter":"previouslyReviewed"}),
    );
    f.reopen();
    let resumed = f.call(
        "setReviewFilter",
        json!({"sessionId":session,"filter":"all"}),
    );
    assert_eq!(
        resumed["payload"]["presentationId"],
        initial["presentationId"]
    );
    assert_eq!(resumed["payload"]["choices"]["order"], order);
    assert_eq!(resumed["payload"]["choices"]["selectedIds"], json!([]));
    let revision = f.call("getCard", json!({"cardId":saved["id"]}))["revision"].clone();
    f.call(
        "updateCard",
        json!({"cardId":saved["id"],"expectedRevision":revision,"draft":{"forms":[{
            "id":saved["forms"][0]["id"],"questionType":"essay","front":"Explain the choice"
        }]}}),
    );
    let edited = f.call("readSession", json!({"sessionId":session}));
    assert_eq!(edited["payload"]["form"]["questionType"], "essay");
    assert!(edited["payload"]["choices"].is_null());
    assert_eq!(edited["payload"]["answerRevealed"], false);
}

#[test]
fn shuffle_avoids_repeating_a_previous_permutation() {
    let mut f = Fixture::new();
    let deck = f.deck("Shuffled");
    let saved = card(&mut f, &deck, true);
    let ids = &saved["forms"][0]["multipleChoice"]["choices"];
    let first = ids[0]["id"].as_str().unwrap();
    let second = ids[1]["id"].as_str().unwrap();
    let db = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    let mut with_history = saved.clone();
    with_history["previousChoiceOrders"] =
        json!({saved["forms"][0]["id"].as_str().unwrap():[first,second]});
    db.execute(
        "UPDATE cards SET payload=?1 WHERE id=?2",
        rusqlite::params![with_history.to_string(), saved["id"].as_str()],
    )
    .unwrap();
    let frame = f.call("startSession", json!({"deckId":deck}));
    assert_eq!(frame["payload"]["choices"]["order"], json!([second, first]));
}
