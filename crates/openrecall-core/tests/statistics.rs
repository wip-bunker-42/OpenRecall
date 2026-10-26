mod common;
use common::*;
use openrecall_core::Clock;
use serde_json::json;
use std::sync::atomic::Ordering;
#[test]
fn all_phases_and_calendar_windows_are_explicit_and_pooled_counts_stay_separate() {
    let mut f = Fixture::new();
    let deck = f.deck("windows");
    let c = f.card(&deck, "q", "a");
    let start = f.call("startSession", json!({"deckId":deck}));
    let sid = start["payload"]["sessionId"].clone();
    let r = f.call("revealAnswer", json!({"sessionId":sid}));
    f.call("rateCard",json!({"sessionId":sid,"presentationId":r["payload"]["presentationId"],"rating":"easy","actionId":uuid::Uuid::new_v4(),"expectedCardRevision":r["payload"]["cardRevision"]}));
    let conn = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    let text: String = conn
        .query_row("SELECT payload FROM reviews", [], |r| r.get(0))
        .unwrap();
    let original: serde_json::Value = serde_json::from_str(&text).unwrap();
    for days in [6, 7, 29, 30] {
        let mut log = original.clone();
        let id = uuid::Uuid::new_v4().to_string();
        log["id"] = json!(id);
        let at = f.clock.now() - chrono::Duration::days(days);
        log["reviewedAt"] = json!(at.to_rfc3339());
        conn.execute("INSERT INTO reviews(id,card_id,deck_id,session_id,at_ms,model_revision,payload) VALUES(?1,?2,?3,?4,?5,?6,?7)",rusqlite::params![id,c,deck,sid.as_str().unwrap(),at.timestamp_millis(),original["modelRevision"].as_str().unwrap(),log.to_string()]).unwrap();
    }
    for phase in ["new", "learning", "relearning"] {
        let id = f.card(&deck, phase, "answer");
        conn.execute(
            "UPDATE cards SET payload=json_set(payload,'$.phase',?1) WHERE id=?2",
            rusqlite::params![phase, id],
        )
        .unwrap();
    }
    let s = f.call("getDeckStatistics", json!({"deckId":deck}));
    assert_eq!(
        s["phases"],
        json!({"new":1,"learning":1,"relearning":1,"review":1})
    );
    assert_eq!(s["today"]["attempts"], 1);
    assert_eq!(s["last7Days"]["attempts"], 2);
    assert_eq!(s["last30Days"]["attempts"], 4);
    assert_eq!(s["allTime"]["attempts"], 5);
    assert_eq!(s["dailyHistory"].as_array().unwrap().len(), 5);
    assert_eq!(s["forecast30Days"].as_array().unwrap().len(), 30);
    let d = f.call("getDeck", json!({"deckId":deck}));
    let other = f.call(
        "createDeck",
        json!({"name":"shared","profileId":d["profileId"]}),
    );
    f.card(other["id"].as_str().unwrap(), "other", "a");
    let s = f.call("getDeckStatistics", json!({"deckId":deck}));
    assert_eq!(s["cards"], 4);
    assert_eq!(
        s["profileStatus"]["eligibility"]["contributions"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}
#[test]
fn statistics_count_logical_cards_and_surviving_reviews() {
    let mut f = Fixture::new();
    let deck = f.deck("statistics");
    let card=f.call("createCard",json!({"deckId":deck,"draft":{"forms":[{"front":"1","back":"a","notes":""},{"front":"2","back":"b","notes":""},{"front":"3","back":"c","notes":""}]}}));
    let mut sid = json!(null);
    for rating in ["again", "hard", "good", "easy"] {
        let c = f.call("getCard", json!({"cardId":card["id"]}));
        let due: chrono::DateTime<chrono::Utc> = c["dueAt"].as_str().unwrap().parse().unwrap();
        if due > f.clock.now() {
            f.clock
                .utc
                .store(due.timestamp_millis() + 1, Ordering::SeqCst);
        }
        let start = f.call("startSession", json!({"deckId":deck}));
        sid = start["payload"]["sessionId"].clone();
        f.advance(5);
        let revealed = f.call("revealAnswer", json!({"sessionId":sid}));
        let frame = &revealed["payload"];
        f.call("rateCard",json!({"sessionId":sid,"presentationId":frame["presentationId"],"rating":rating,"actionId":uuid::Uuid::new_v4(),"expectedCardRevision":frame["cardRevision"]}));
    }
    let s = f.call("getDeckStatistics", json!({"deckId":deck}));
    assert_eq!(s["cards"], 1);
    assert_eq!(s["forms"], 3);
    assert_eq!(s["allTime"]["attempts"], 4);
    assert_eq!(s["allTime"]["uniqueCards"], 1);
    assert_eq!(s["allTime"]["ratingCounts"], json!([1, 1, 1, 1]));
    assert_eq!(s["allTime"]["activeMs"], 20000);
    assert_eq!(s["allTime"]["meanResponseMs"], 5000.0);
    assert_eq!(s["allTime"]["observedRecallRate"], 0.75);
    assert_eq!(s["phases"]["learning"], 1);
    assert_eq!(s["profileStatus"]["eligibility"]["eligible"], false);
    f.call("undoReview", json!({"sessionId":sid}));
    let s = f.call("getDeckStatistics", json!({"deckId":deck}));
    assert_eq!(s["allTime"]["attempts"], 3);
    assert_eq!(s["allTime"]["observedRecallRate"], 2.0 / 3.0);
    let c = f.call("getCard", json!({"cardId":card["id"]}));
    f.call(
        "deleteCard",
        json!({"cardId":card["id"],"expectedRevision":c["revision"]}),
    );
    let s = f.call("getDeckStatistics", json!({"deckId":deck}));
    assert_eq!(s["cards"], 0);
    assert_eq!(s["allTime"]["attempts"], 0);
    assert!(s["allTime"]["observedRecallRate"].is_null());
    assert!(s["allTime"]["meanResponseMs"].is_null());
}
#[test]
fn forecasts_respect_cairo_fall_back_and_configured_rollover_without_changing_due_instants() {
    let mut f = Fixture::new();
    let deck = f.deck("DST");
    f.clock.utc.store(
        "2026-10-29T22:30:00Z"
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap()
            .timestamp_millis(),
        Ordering::SeqCst,
    );
    let conn = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    for (i, due) in [
        "2026-10-29T20:00:00Z",
        "2026-10-29T22:50:00Z",
        "2026-10-29T23:10:00Z",
    ]
    .iter()
    .enumerate()
    {
        let id = f.card(&deck, &format!("q{i}"), "a");
        let ms = due
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap()
            .timestamp_millis();
        conn.execute(
            "UPDATE cards SET due_ms=?1,payload=json_set(payload,'$.dueAt',?2) WHERE id=?3",
            rusqlite::params![ms, due, id],
        )
        .unwrap();
    }
    let s = f.call("getDeckStatistics", json!({"deckId":deck}));
    assert_eq!(s["dueNow"], 1);
    assert_eq!(s["dueLaterToday"], 1);
    assert_eq!(s["nextDueAt"], "2026-10-29T22:50:00Z");
    assert_eq!(
        s["forecast7Days"][0],
        json!({"date":"2026-10-29","cards":2})
    );
    assert_eq!(s["forecast7Days"][1]["cards"], 1);
    assert_eq!(s["phases"]["new"], 3);
    let mut settings = f.call("bootstrap", json!({}))["settings"].clone();
    let rev = settings["revision"].clone();
    settings["timezone"] = json!("Asia/Tokyo");
    f.call(
        "saveSettings",
        json!({"settings":settings,"expectedRevision":rev}),
    );
    let after = f.call("getDeckStatistics", json!({"deckId":deck}));
    assert_eq!(after["nextDueAt"], s["nextDueAt"]);
    assert_eq!(after["forecast7Days"][0]["date"], "2026-10-30");
    assert_eq!(after["dueLaterToday"], 2);
}
