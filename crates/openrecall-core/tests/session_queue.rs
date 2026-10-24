mod common;
use chrono::{DateTime, Utc};
use common::*;
use openrecall_core::scheduling::queue::*;
use serde_json::{Value, json};
#[test]
fn due_order_filters_and_future_exclusion_are_explicit() {
    let now: DateTime<Utc> = "2026-10-24T00:00:00Z".parse().unwrap();
    let entries = vec![
        QueueEntry {
            card_id: "later".into(),
            due_at: now,
            created_at: now,
            review_count: 0,
        },
        QueueEntry {
            card_id: "oldest".into(),
            due_at: now - chrono::Duration::hours(1),
            created_at: now,
            review_count: 2,
        },
        QueueEntry {
            card_id: "future".into(),
            due_at: now + chrono::Duration::seconds(1),
            created_at: now,
            review_count: 0,
        },
    ];
    let ids = |filter| {
        select_due(&entries, filter, now)
            .into_iter()
            .map(|c| c.card_id)
            .collect::<Vec<_>>()
    };
    assert_eq!(ids(ReviewFilter::All), vec!["oldest", "later"]);
    assert_eq!(ids(ReviewFilter::New), vec!["later"]);
    assert_eq!(ids(ReviewFilter::PreviouslyReviewed), vec!["oldest"]);
}
fn future_card(f: &mut Fixture, id: &str, after: i64) {
    use openrecall_core::Clock;
    let mut card = f.call("getCard", json!({"cardId":id}));
    let due = f.clock.now() + chrono::Duration::seconds(after);
    card["dueAt"] = json!(due.to_rfc3339());
    let c = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    c.execute(
        "UPDATE cards SET due_ms=?1,payload=?2 WHERE id=?3",
        rusqlite::params![due.timestamp_millis(), card.to_string(), id],
    )
    .unwrap();
}
#[test]
fn live_arrivals_do_not_preempt_question_and_filters_park_revealed_form() {
    let mut f = Fixture::new();
    let deck = f.deck("مراجعة");
    let a = f.card(&deck, "سؤال", "جواب");
    f.advance(1);
    let b = f.card(&deck, "ثانٍ", "جواب");
    future_card(&mut f, &b, 20);
    let view = f.call("startSession", json!({"deckId":deck}));
    assert_eq!(view["state"], "question");
    let frame = &view["payload"];
    let sid = frame["sessionId"].clone();
    assert_eq!(frame["filter"], "all");
    assert_eq!(frame["card"]["id"], a);
    assert_eq!(frame["progress"]["total"], 1);
    let reveal = f.call("revealAnswer", json!({"sessionId":sid}));
    let form = reveal["payload"]["form"]["id"].clone();
    assert_eq!(reveal["payload"]["answerRevealed"], true);
    let empty = f.call(
        "setReviewFilter",
        json!({"sessionId":sid,"filter":"previouslyReviewed"}),
    );
    assert_eq!(empty["state"], "emptyFilter");
    let restored = f.call("setReviewFilter", json!({"sessionId":sid,"filter":"all"}));
    assert_eq!(restored["payload"]["form"]["id"], form);
    assert_eq!(restored["payload"]["answerRevealed"], true);
    f.advance(21);
    let arrived = f.call("readSession", json!({"sessionId":sid}));
    assert_eq!(arrived["payload"]["card"]["id"], a);
    assert_eq!(arrived["payload"]["progress"]["total"], 2);
    assert_eq!(
        f.call("getCard", json!({"cardId":a}))["bag"]["consumed"],
        json!([])
    );
    f.reopen();
    let resumed = f.call("resumeSession", json!({"sessionId":sid}));
    assert_eq!(resumed["payload"]["form"]["id"], form);
    assert_eq!(resumed["payload"]["answerRevealed"], true);
}
#[test]
fn active_time_excludes_inactive_sleep_and_closed_app_gaps() {
    let mut f = Fixture::new();
    let deck = f.deck("وقت");
    f.card(&deck, "q", "a");
    let view = f.call("startSession", json!({"deckId":deck}));
    let sid = view["payload"]["sessionId"].clone();
    f.advance(5);
    let timing = f.call(
        "setSessionActivity",
        json!({"sessionId":sid,"active":false}),
    );
    assert_eq!(timing["activeMs"], 5000);
    f.advance(3600);
    let timing = f.call("setSessionActivity", json!({"sessionId":sid,"active":true}));
    assert_eq!(timing["activeMs"], 5000);
    f.advance(7200);
    f.reopen();
    f.call("resumeSession", json!({"sessionId":sid}));
    let summary = f.call("endSession", json!({"sessionId":sid}));
    assert_eq!(summary["state"], "summary");
    assert_eq!(summary["payload"]["activeMs"], 5000);
    assert_eq!(summary["payload"]["attempts"], 0);
    assert!(summary["payload"]["meanResponseMs"].is_null());
    let empty_deck = f.deck("empty");
    let empty: Value = f.call("startSession", json!({"deckId":empty_deck}));
    assert_eq!(empty["state"], "summary");
    assert_eq!(empty["payload"]["allDueRemaining"], 0);
}
