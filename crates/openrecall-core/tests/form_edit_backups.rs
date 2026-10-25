mod common;
use common::*;
use serde_json::{Value, json};

fn two_forms(f: &mut Fixture, deck: &str) -> Value {
    f.call(
        "createCard",
        json!({"deckId":deck,"draft":{"forms":[
            {"front":"question one","back":"answer one","notes":""},
            {"front":"question two","back":"answer two","notes":""}
        ]}}),
    )
}
fn remove_form(f: &mut Fixture, card_id: &Value, removed: &Value) {
    let card = f.call("getCard", json!({"cardId":card_id}));
    let forms: Vec<_> = card["forms"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|form| form["id"] != *removed)
        .cloned()
        .collect();
    assert_eq!(forms.len(), 1);
    f.call(
        "updateCard",
        json!({"cardId":card_id,"draft":{"forms":forms},"expectedRevision":card["revision"]}),
    );
}
fn rate(f: &mut Fixture, session: &Value, rating: &str) -> Value {
    let view = f.call("revealAnswer", json!({"sessionId":session}));
    let frame = &view["payload"];
    f.call("rateCard", json!({"sessionId":session,"presentationId":frame["presentationId"],"expectedCardRevision":frame["cardRevision"],"rating":rating,"actionId":uuid::Uuid::new_v4()}))
}

#[test]
fn removed_forms_do_not_block_export_or_restore_safety_exports_after_leaving_review() {
    for parked in [false, true] {
        let mut f = Fixture::new();
        let deck = f.deck("editable presentation");
        let card = two_forms(&mut f, &deck);
        let view = f.call("startSession", json!({"deckId":deck}));
        let session = &view["payload"]["sessionId"];
        let removed = &view["payload"]["form"]["id"];
        if parked {
            f.call(
                "setReviewFilter",
                json!({"sessionId":session,"filter":"previouslyReviewed"}),
            );
        }
        f.call("closeSession", json!({"sessionId":session}));
        let original = f.dir.path().join("original.json");
        f.app.export_snapshot(&original).unwrap();
        remove_form(&mut f, &card["id"], removed);
        let edited = f.dir.path().join("edited.json");
        assert!(
            f.app.export_snapshot(&edited).is_ok(),
            "a normal form edit must keep the library exportable (parked={parked})"
        );
        f.app.preview_restore(&edited).unwrap();
        let preview = f.app.preview_restore(&original).unwrap();
        let safety = f.dir.path().join("safety.json");
        f.app
            .restore_snapshot(preview.id, preview.expected_library_revision, &safety)
            .unwrap();
        assert!(safety.is_file());
        assert_eq!(
            f.call("getCard", json!({"cardId":card["id"]}))["forms"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }
}

#[test]
fn editing_a_card_parked_in_an_undo_before_image_invalidates_that_undo_without_losing_history() {
    let mut f = Fixture::new();
    let deck = f.deck("parked undo");
    let reviewed = f.card(&deck, "reviewed", "answer");
    let initial = f.call("startSession", json!({"deckId":deck}));
    rate(&mut f, &initial["payload"]["sessionId"], "again");
    f.advance(61);
    let card = two_forms(&mut f, &deck);
    let start = f.call("startSession", json!({"deckId":deck}));
    let session = &start["payload"]["sessionId"];
    assert_eq!(start["payload"]["card"]["id"], reviewed);
    let new = f.call(
        "setReviewFilter",
        json!({"sessionId":session,"filter":"new"}),
    );
    assert_eq!(new["payload"]["card"]["id"], card["id"]);
    let removed = new["payload"]["form"]["id"].clone();
    f.call(
        "setReviewFilter",
        json!({"sessionId":session,"filter":"previouslyReviewed"}),
    );
    let finished = rate(&mut f, session, "easy");
    assert_eq!(finished["state"], "summary");
    assert_eq!(finished["payload"]["canUndo"], true);
    let before = f.dir.path().join("before-edit.json");
    f.app.export_snapshot(&before).unwrap();
    let learning = f.call("getCard", json!({"cardId":reviewed}));
    remove_form(&mut f, &card["id"], &removed);
    let undo = f
        .app
        .execute_json(json!({"kind":"undoReview","payload":{"sessionId":session}}));
    assert!(
        undo.is_err(),
        "an independently edited parked card must invalidate the captured session before-image"
    );
    assert_eq!(undo.unwrap_err().code, "undoUnavailable");
    assert_eq!(f.call("getCard", json!({"cardId":reviewed})), learning);
    assert_eq!(
        f.call("readSession", json!({"sessionId":session}))["payload"]["attempts"],
        1
    );
    let edited = f.dir.path().join("after-edit.json");
    f.app.export_snapshot(&edited).unwrap();
    f.app.preview_restore(&edited).unwrap();
}
