mod common;
use common::*;
use openrecall_core::Clock;
use serde_json::{Value, json};
#[test]
fn a_new_session_cannot_rewind_a_previous_completed_session() {
    let mut f = Fixture::new();
    let d = f.deck("undo ownership");
    let cid = f.card(&d, "q", "a");
    let start = f.call("startSession", json!({"deckId":d}));
    let old = start["payload"]["sessionId"].clone();
    let reveal = f.call("revealAnswer", json!({"sessionId":old}));
    f.app
        .execute_json(request(&reveal["payload"], "easy", "old grade"))
        .unwrap();
    f.call("startSession", json!({"deckId":d}));
    assert_eq!(
        f.app
            .execute_json(json!({"kind":"undoReview","payload":{"sessionId":old}}))
            .unwrap_err()
            .code,
        "undoUnavailable"
    );
    assert_eq!(f.call("getCard", json!({"cardId":cid}))["reviewCount"], 1);
}
#[test]
fn deleting_other_session_history_invalidates_stale_undo_aggregates() {
    let mut f = Fixture::new();
    let d = f.deck("delete history");
    f.card(&d, "a", "1");
    f.card(&d, "b", "2");
    let start = f.call("startSession", json!({"deckId":d}));
    let sid = start["payload"]["sessionId"].clone();
    let first = start["payload"]["card"]["id"].clone();
    for action in ["first", "second"] {
        let reveal = f.call("revealAnswer", json!({"sessionId":sid}));
        f.app
            .execute_json(request(&reveal["payload"], "easy", action))
            .unwrap();
    }
    let card = f.call("getCard", json!({"cardId":first}));
    f.call(
        "deleteCard",
        json!({"cardId":first,"expectedRevision":card["revision"]}),
    );
    let view = f.call("readSession", json!({"sessionId":sid}));
    assert_eq!(view["payload"]["attempts"], 1);
    assert_eq!(view["payload"]["canUndo"], false);
}
fn request(frame: &Value, rating: &str, action: &str) -> Value {
    let id = uuid::Uuid::from_u128(
        action
            .bytes()
            .fold(0u128, |n, b| n.wrapping_mul(131).wrapping_add(b as u128)),
    )
    .to_string();
    json!({"kind":"rateCard","payload":{"sessionId":frame["sessionId"],"presentationId":frame["presentationId"],"rating":rating,"actionId":id,"expectedCardRevision":frame["cardRevision"]}})
}
fn count(f: &Fixture, table: &str) -> u32 {
    rusqlite::Connection::open(f.dir.path().join("test.sqlite3"))
        .unwrap()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
#[test]
fn retried_and_undone_actions_never_double_rate_and_summary_undo_restores_everything() {
    let mut f = Fixture::new();
    let deck = f.deck("تراجع");
    let card=f.call("createCard",json!({"deckId":deck,"draft":{"forms":[{"front":"q1","back":"a1","notes":"n1"},{"front":"q2","back":"a2","notes":"n2"}]}}));
    let start = f.call("startSession", json!({"deckId":deck}));
    let sid = start["payload"]["sessionId"].clone();
    f.advance(7);
    let reveal = f.call("revealAnswer", json!({"sessionId":sid}));
    f.advance(3);
    let before = f.call("getCard", json!({"cardId":card["id"]}));
    let rate = request(&reveal["payload"], "easy", "action-one");
    let rated = f.app.execute_json(rate.clone()).unwrap()["payload"].clone();
    assert_eq!(rated["state"], "summary");
    assert_eq!(rated["payload"]["attempts"], 1);
    assert_eq!(rated["payload"]["uniqueCards"], 1);
    assert_eq!(rated["payload"]["newCards"], 1);
    assert_eq!(rated["payload"]["meanResponseMs"], 10000.0);
    let saved = f.call("getCard", json!({"cardId":card["id"]}));
    assert_eq!(saved["reviewCount"], 1);
    assert_eq!(saved["phase"], "review");
    assert_eq!(saved["bag"]["consumed"].as_array().unwrap().len(), 1);
    let due: chrono::DateTime<chrono::Utc> = saved["dueAt"].as_str().unwrap().parse().unwrap();
    let seconds = reveal["payload"]["previews"]["easy"]["intervalSeconds"]
        .as_f64()
        .unwrap();
    assert!(((due - f.clock.now()).num_milliseconds() as f64 / 1000.0 - seconds).abs() < 0.002);
    f.app.execute_json(rate.clone()).unwrap();
    assert_eq!(count(&f, "reviews"), 1);
    let undone = f.call("undoReview", json!({"sessionId":sid}));
    assert_eq!(undone["state"], "question");
    assert_eq!(undone["payload"]["answerRevealed"], true);
    assert_eq!(
        undone["payload"]["form"]["id"],
        reveal["payload"]["form"]["id"]
    );
    let restored = f.call("getCard", json!({"cardId":card["id"]}));
    for field in [
        "memory",
        "dueAt",
        "reviewedAt",
        "reviewCount",
        "lapseCount",
        "phase",
        "bag",
    ] {
        assert_eq!(restored[field], before[field], "{field}");
    }
    assert_eq!(count(&f, "reviews"), 0);
    assert_eq!(f.app.execute_json(rate).unwrap_err().code, "actionUndone");
    assert_eq!(count(&f, "reviews"), 0);
}
#[test]
fn a_failed_write_rolls_back_memory_bag_session_and_action_token() {
    let mut f = Fixture::new();
    let deck = f.deck("atomic");
    let cid = f.card(&deck, "question", "answer");
    let start = f.call("startSession", json!({"deckId":deck}));
    let sid = start["payload"]["sessionId"].clone();
    let reveal = f.call("revealAnswer", json!({"sessionId":sid}));
    let before = f.call("getCard", json!({"cardId":cid}));
    let conn = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    conn.execute_batch("CREATE TRIGGER reject_review BEFORE INSERT ON reviews BEGIN SELECT RAISE(ABORT,'injected failure'); END").unwrap();
    assert!(
        f.app
            .execute_json(request(&reveal["payload"], "good", "failed-action"))
            .is_err()
    );
    assert_eq!(f.call("getCard", json!({"cardId":cid})), before);
    assert_eq!(count(&f, "reviews"), 0);
    assert_eq!(count(&f, "actions"), 0);
    let view = f.call("readSession", json!({"sessionId":sid}));
    assert_eq!(view["payload"]["progress"]["completed"], 0);
    assert_eq!(view["payload"]["answerRevealed"], true);
}
#[test]
fn new_filter_membership_changes_on_rating_and_undo_restores_it() {
    let mut f = Fixture::new();
    let deck = f.deck("new");
    f.card(&deck, "q", "a");
    let start = f.call("startSession", json!({"deckId":deck}));
    let sid = start["payload"]["sessionId"].clone();
    f.call("setReviewFilter", json!({"sessionId":sid,"filter":"new"}));
    let reveal = f.call("revealAnswer", json!({"sessionId":sid}));
    let rated = f
        .app
        .execute_json(request(&reveal["payload"], "again", "new-again"))
        .unwrap()["payload"]
        .clone();
    assert_eq!(rated["state"], "summary");
    assert_eq!(rated["payload"]["filter"], "new");
    f.advance(61);
    let summary = f.call("readSession", json!({"sessionId":sid}));
    assert_eq!(summary["payload"]["allDueRemaining"], 1);
    assert_eq!(summary["payload"]["matchingDueRemaining"], 0);
    let undo = f.call("undoReview", json!({"sessionId":sid}));
    assert_eq!(undo["payload"]["filter"], "new");
    assert_eq!(undo["payload"]["card"]["reviewCount"], 0);
}
#[test]
fn due_learning_card_joins_queue_and_undo_restores_prior_filter_and_next_bag() {
    let mut f = Fixture::new();
    let deck = f.deck("queue");
    let a=f.call("createCard",json!({"deckId":deck,"draft":{"forms":[{"front":"A1","back":"a"},{"front":"A2","back":"a"}]}}));
    f.advance(1);
    let b = f.card(&deck, "B", "b");
    let start = f.call("startSession", json!({"deckId":deck}));
    let sid = start["payload"]["sessionId"].clone();
    let reveal = f.call("revealAnswer", json!({"sessionId":sid}));
    let aform = reveal["payload"]["form"]["id"].clone();
    let after = f
        .app
        .execute_json(request(&reveal["payload"], "again", "queue-a"))
        .unwrap()["payload"]
        .clone();
    assert_eq!(after["payload"]["card"]["id"], b);
    f.advance(61);
    let live = f.call("readSession", json!({"sessionId":sid}));
    assert_eq!(live["payload"]["progress"]["total"], 3);
    let reveal_b = f.call("revealAnswer", json!({"sessionId":sid}));
    let next = f
        .app
        .execute_json(request(&reveal_b["payload"], "good", "queue-b"))
        .unwrap()["payload"]
        .clone();
    assert_eq!(next["payload"]["card"]["id"], a["id"]);
    assert_ne!(next["payload"]["form"]["id"], aform);
    f.call("setReviewFilter", json!({"sessionId":sid,"filter":"new"}));
    let undo = f.call("undoReview", json!({"sessionId":sid}));
    assert_eq!(undo["payload"]["filter"], "all");
    assert_eq!(undo["payload"]["card"]["id"], b);
    assert_eq!(undo["payload"]["answerRevealed"], true);
}
#[test]
fn hidden_answers_stale_revisions_and_backward_clocks_do_not_grade() {
    use std::sync::atomic::Ordering;
    let mut f = Fixture::new();
    let deck = f.deck("guards");
    let cid = f.card(&deck, "q", "a");
    let start = f.call("startSession", json!({"deckId":deck}));
    let sid = start["payload"]["sessionId"].clone();
    assert_eq!(
        f.app
            .execute_json(request(&start["payload"], "good", "hidden"))
            .unwrap_err()
            .code,
        "answerHidden"
    );
    let reveal = f.call("revealAnswer", json!({"sessionId":sid}));
    let mut stale = request(&reveal["payload"], "good", "stale");
    stale["payload"]["expectedCardRevision"] = json!("outdated");
    assert_eq!(f.app.execute_json(stale).unwrap_err().code, "conflict");
    f.app
        .execute_json(request(&reveal["payload"], "again", "first"))
        .unwrap();
    f.advance(61);
    let second = f.call("startSession", json!({"deckId":deck}));
    let sid2 = second["payload"]["sessionId"].clone();
    let revealed = f.call("revealAnswer", json!({"sessionId":sid2}));
    f.clock.utc.fetch_sub(120_000, Ordering::SeqCst);
    assert_eq!(
        f.app
            .execute_json(request(&revealed["payload"], "good", "backward"))
            .unwrap_err()
            .code,
        "clockMovedBackward"
    );
    assert_eq!(f.call("getCard", json!({"cardId":cid}))["reviewCount"], 1);
}
#[test]
fn editing_or_deleting_a_reserved_card_never_gets_reversed_by_old_undo() {
    let mut f = Fixture::new();
    let deck = f.deck("safe undo");
    let a = f.card(&deck, "A", "a");
    f.advance(1);
    let b = f.card(&deck, "B", "b");
    let start = f.call("startSession", json!({"deckId":deck}));
    let sid = start["payload"]["sessionId"].clone();
    let reveal = f.call("revealAnswer", json!({"sessionId":sid}));
    f.app
        .execute_json(request(&reveal["payload"], "again", "before-edit"))
        .unwrap();
    let next = f.call("getCard", json!({"cardId":b}));
    let mut forms = next["forms"].clone();
    forms[0]["front"] = json!("B changed");
    f.call(
        "updateCard",
        json!({"cardId":b,"draft":{"forms":forms},"expectedRevision":next["revision"]}),
    );
    assert_eq!(
        f.app
            .execute_json(json!({"kind":"undoReview","payload":{"sessionId":sid}}))
            .unwrap_err()
            .code,
        "undoUnavailable"
    );
    assert_eq!(
        f.call("getCard", json!({"cardId":b}))["forms"][0]["front"],
        "B changed"
    );
    let current_a = f.call("getCard", json!({"cardId":a}));
    f.call(
        "deleteCard",
        json!({"cardId":a,"expectedRevision":current_a["revision"]}),
    );
    let view = f.call("readSession", json!({"sessionId":sid}));
    assert_eq!(view["payload"]["progress"]["completed"], 0);
    assert_eq!(count(&f, "reviews"), 0);
}
#[test]
fn leaving_summary_explicitly_invalidates_undo() {
    let mut f = Fixture::new();
    let deck = f.deck("close");
    f.card(&deck, "q", "a");
    let start = f.call("startSession", json!({"deckId":deck}));
    let sid = start["payload"]["sessionId"].clone();
    let reveal = f.call("revealAnswer", json!({"sessionId":sid}));
    f.app
        .execute_json(request(&reveal["payload"], "easy", "close-action"))
        .unwrap();
    f.call("closeSession", json!({"sessionId":sid}));
    assert_eq!(
        f.app
            .execute_json(json!({"kind":"undoReview","payload":{"sessionId":sid}}))
            .unwrap_err()
            .code,
        "undoUnavailable"
    );
}
