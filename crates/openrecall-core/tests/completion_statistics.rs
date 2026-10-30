mod common;
use chrono::{NaiveDate, Utc};
use common::*;
use openrecall_core::{
    Clock,
    domain::{card::MemoryState, review::ReviewLog},
    scheduling::fsrs::{Rating, SchedulePolicy},
    statistics::completion::daily_completions,
};
use serde_json::{Value, json};
use std::sync::atomic::Ordering;
fn log(card: &str, at: &str, due: &str) -> ReviewLog {
    ReviewLog {
        id: uuid::Uuid::new_v4().to_string(),
        card_id: card.into(),
        deck_id: uuid::Uuid::new_v4().to_string(),
        session_id: uuid::Uuid::new_v4().to_string(),
        form_id: uuid::Uuid::new_v4().to_string(),
        rating: Rating::Again,
        reviewed_at: at.into(),
        elapsed_days: 0.0,
        response_ms: 1000,
        previous_memory: None,
        memory: MemoryState {
            stability: 1.0,
            stability_fast: 0.5,
            difficulty: 5.0,
        },
        previous_due_at: at.into(),
        next_due_at: due.into(),
        previous_phase: "new".into(),
        phase: "learning".into(),
        policy: SchedulePolicy {
            desired_retention: 0.9,
            min_interval_seconds: 60,
            max_interval_days: 36500,
        },
        model_revision: uuid::Uuid::new_v4().to_string(),
        was_new: true,
    }
}
fn day(s: &str) -> NaiveDate {
    s.parse().unwrap()
}
fn count(logs: &[ReviewLog], date: &str) -> u32 {
    *daily_completions(logs, chrono_tz::Africa::Cairo, 60)
        .unwrap()
        .get(&day(date))
        .unwrap_or(&0)
}
#[test]
fn completion_is_last_effective_review_per_card_and_study_day() {
    let card = uuid::Uuid::new_v4().to_string();
    let first = log(&card, "2026-01-15T22:59:30Z", "2026-01-15T23:00:30Z");
    assert_eq!(count(std::slice::from_ref(&first), "2026-01-15"), 1);
    let mut later = log(&card, "2026-01-15T22:59:40Z", "2026-01-15T22:59:55Z");
    assert_eq!(count(&[first.clone(), later.clone()], "2026-01-15"), 0);
    later.reviewed_at = first.reviewed_at.clone();
    later.id = "00000000-0000-4000-8000-000000000001".into();
    let mut first = first;
    first.id = "ffffffff-ffff-4fff-8fff-ffffffffffff".into();
    assert_eq!(count(&[first.clone(), later.clone()], "2026-01-15"), 0);
    assert_eq!(count(&[later, first], "2026-01-15"), 1);
}
#[test]
fn next_day_review_does_not_erase_yesterday() {
    let card = uuid::Uuid::new_v4().to_string();
    let yesterday = log(&card, "2026-01-15T22:59:30Z", "2026-01-15T23:00:30Z");
    let tomorrow = log(&card, "2026-01-15T23:00:30Z", "2026-01-15T23:01:30Z");
    let counts = daily_completions(&[yesterday, tomorrow], chrono_tz::Africa::Cairo, 60).unwrap();
    assert_eq!(counts[&day("2026-01-15")], 1);
    assert_eq!(counts[&day("2026-01-16")], 0);
}
#[test]
fn calendar_boundaries_use_current_zone_start_and_cairo_dst() {
    let card = uuid::Uuid::new_v4().to_string();
    let winter = log(&card, "2026-01-15T22:59:30Z", "2026-01-15T23:00:30Z");
    let same_day = log(&card, "2026-01-15T22:58:00Z", "2026-01-15T22:59:00Z");
    assert_eq!(count(&[same_day], "2026-01-15"), 0);
    assert_eq!(
        daily_completions(std::slice::from_ref(&winter), chrono_tz::UTC, 60).unwrap()
            [&day("2026-01-15")],
        0
    );
    assert_eq!(
        daily_completions(&[winter], chrono_tz::Africa::Cairo, 0).unwrap()[&day("2026-01-16")],
        0
    );
    assert_eq!(
        count(
            &[log(&card, "2026-07-15T21:59:30Z", "2026-07-15T22:00:30Z")],
            "2026-07-15"
        ),
        1
    );
    assert_eq!(
        count(
            &[log(&card, "2026-04-23T21:59:30Z", "2026-04-23T22:00:30Z")],
            "2026-04-23"
        ),
        1
    );
    assert!(
        daily_completions(&[], chrono_tz::Africa::Cairo, 60)
            .unwrap()
            .is_empty()
    );
}
fn rate(f: &mut Fixture, deck: &str, spelling: bool) -> Value {
    let start = f.call("startSession", json!({"deckId":deck}));
    let frame = if spelling {
        start["payload"].clone()
    } else {
        f.call(
            "revealAnswer",
            json!({"sessionId":start["payload"]["sessionId"]}),
        )["payload"]
            .clone()
    };
    f.call("rateCard",json!({"sessionId":frame["sessionId"],"presentationId":frame["presentationId"],"rating":"easy","actionId":uuid::Uuid::new_v4(),"expectedCardRevision":frame["cardRevision"]}));
    frame
}
#[test]
fn undo_and_deletion_recompute_only_surviving_history() {
    let mut f = Fixture::new();
    let deck = f.deck("Source");
    f.card(&deck, "red", "معنى");
    f.card(&deck, "unreviewed", "معنى");
    assert_eq!(
        f.call("getDeckStatistics", json!({"deckId":deck}))["today"]["completedForDay"],
        0
    );
    let frame = rate(&mut f, &deck, false);
    assert_eq!(
        f.call("getDeckStatistics", json!({"deckId":deck}))["today"]["completedForDay"],
        1
    );
    f.call("undoReview", json!({"sessionId":frame["sessionId"]}));
    assert_eq!(
        f.call("getDeckStatistics", json!({"deckId":deck}))["today"]["completedForDay"],
        0
    );
    let frame = rate(&mut f, &deck, false);
    let detail = f.call("getCard", json!({"cardId":frame["card"]["id"]}));
    f.call(
        "deleteCard",
        json!({"cardId":frame["card"]["id"],"expectedRevision":detail["revision"]}),
    );
    let stats = f.call("getDeckStatistics", json!({"deckId":deck}));
    assert_eq!(stats["allTime"]["completedForDay"], 0);
    assert!(stats["dailyHistory"].as_array().unwrap().is_empty());
}
#[test]
fn library_counts_source_and_spelling_as_independent_tasks() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    f.card(&source, "red", "معنى");
    let spelling = f.call("createSpellingDeck", json!({"sourceDeckId":source}));
    let deck = spelling["deck"]["id"].as_str().unwrap();
    assert_eq!(
        f.call("getLibraryStatistics", json!({}))["today"]["completedForDay"],
        0
    );
    rate(&mut f, &source, false);
    rate(&mut f, deck, true);
    let stats = f.call("getLibraryStatistics", json!({}));
    assert_eq!(stats["today"]["completedForDay"], 2);
    assert_eq!(stats["spellingCompletedToday"], 1);
    assert_eq!(stats["dailyHistory"][0]["statistics"]["completedForDay"], 2);
    let fresh = f.call("getDeck", json!({"deckId":source}));
    f.call(
        "deleteDeck",
        json!({"deckId":source,"expectedRevision":fresh["revision"]}),
    );
    let stats = f.call("getLibraryStatistics", json!({}));
    assert_eq!(stats["today"]["completedForDay"], 1);
    assert_eq!(stats["spellingCompletedToday"], 1);
}
#[test]
fn future_reviews_preserve_historical_completions_and_period_sums() {
    let mut f = Fixture::new();
    let deck = f.deck("History");
    let card = f.card(&deck, "red", "معنى");
    rate(&mut f, &deck, false);
    let before = f.call("getDeckStatistics", json!({"deckId":deck}));
    let yesterday = before["dailyHistory"][0]["date"].clone();
    let detail = f.call("getCard", json!({"cardId":card}));
    let due: chrono::DateTime<Utc> = detail["dueAt"].as_str().unwrap().parse().unwrap();
    assert!(due > f.clock.now());
    f.clock
        .utc
        .store(due.timestamp_millis() + 1, Ordering::SeqCst);
    rate(&mut f, &deck, false);
    let after = f.call("getDeckStatistics", json!({"deckId":deck}));
    assert_eq!(after["dailyHistory"][0]["date"], yesterday);
    assert_eq!(after["dailyHistory"][0]["statistics"]["completedForDay"], 1);
    assert_eq!(after["allTime"]["completedForDay"], 2);
    assert_eq!(after["today"]["completedForDay"], 1);
}
