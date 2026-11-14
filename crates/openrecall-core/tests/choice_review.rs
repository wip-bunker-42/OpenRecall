mod common;
use common::{Fixture, spelling};
use serde_json::{Value, json};

fn setup() -> (Fixture, Value, String) {
    let mut f = Fixture::new();
    let deck = f.deck("Choices");
    f.call("createCard",json!({"deckId":deck,"draft":{"forms":[{
        "questionType":"multipleChoice","front":"Pick the colors","notes":"Check each color.",
        "multipleChoice":{"selectionMode":"multiple","shuffleChoices":false,
          "choices":[{"key":"a","text":"red"},{"key":"b","text":"table"},{"key":"c","text":"green"}],
          "correctChoiceKeys":["a","c"]}
    }]}}));
    let frame = f.call("startSession", json!({"deckId":deck}))["payload"].clone();
    (f, frame, deck)
}

fn submission(frame: &Value, submission: Value) -> Value {
    json!({"sessionId":frame["sessionId"],"presentationId":frame["presentationId"],
        "expectedCardRevision":frame["cardRevision"],"actionId":uuid::Uuid::new_v4().to_string(),
        "submission":submission})
}

fn selected(ids: Value) -> Value {
    json!({"kind":"selected","choiceIds":ids})
}

#[test]
fn correct_confirmation_keeps_four_grades_and_undo_restores_result() {
    let (mut f, frame, _) = setup();
    let order = &frame["choices"]["order"];
    let input = submission(&frame, selected(json!([order[2], order[0]])));
    let checked = f.call("confirmChoices", input.clone())["payload"].clone();
    assert_eq!(checked["canRate"], true);
    assert_eq!(checked["choices"]["confirmedCorrect"], true);
    assert_eq!(checked["card"]["reviewCount"], 0);
    assert_eq!(checked["previews"].as_object().unwrap().len(), 4);
    assert_eq!(
        f.call("confirmChoices", input)["payload"]["choices"],
        checked["choices"]
    );
    assert_eq!(spelling::snapshot(&mut f)["reviews"], json!([]));
    f.call("rateCard",json!({"sessionId":frame["sessionId"],"presentationId":frame["presentationId"],
        "expectedCardRevision":frame["cardRevision"],"actionId":uuid::Uuid::new_v4().to_string(),"rating":"easy"}));
    let undone = f.call("undoReview", json!({"sessionId":frame["sessionId"]}));
    assert_eq!(undone["payload"]["choices"], checked["choices"]);
    assert_eq!(undone["payload"]["canRate"], true);
}

#[test]
fn wrong_and_unknown_grade_once_keep_feedback_and_restore_selection_on_undo() {
    for index in 0..4 {
        let (mut f, frame, deck) = setup();
        let order = &frame["choices"]["order"];
        let ids = match index {
            0 => json!([order[0]]),
            1 => order.clone(),
            2 => json!([order[1]]),
            _ => json!([]),
        };
        let input = submission(
            &frame,
            if index == 3 {
                json!({"kind":"unknown"})
            } else {
                selected(ids.clone())
            },
        );
        let response = f.call("confirmChoices", input.clone());
        assert_eq!(response["state"], "choiceFeedback");
        let feedback = &response["payload"];
        assert_eq!(feedback["lastReview"]["rating"], "again");
        assert!(feedback["lastReview"]["dueAt"].is_string());
        assert_eq!(
            spelling::snapshot(&mut f)["reviews"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        let mut changed = input.clone();
        changed["submission"] = if index == 3 {
            selected(json!([order[0]]))
        } else {
            json!({"kind":"unknown"})
        };
        assert_eq!(
            f.app
                .execute_json(json!({"kind":"confirmChoices","payload":changed}))
                .unwrap_err()
                .code,
            "actionConflict"
        );
        f.call("closeSession", json!({"sessionId":frame["sessionId"]}));
        f.reopen();
        let resumed = f.call("resumeSession", json!({"sessionId":frame["sessionId"]}));
        assert_eq!(resumed["payload"]["feedbackId"], feedback["feedbackId"]);
        assert_eq!(resumed["payload"]["canUndo"], true);
        let retry = f.call("confirmChoices", input.clone());
        assert_eq!(retry["payload"]["feedbackId"], feedback["feedbackId"]);
        f.call(
            "setReviewFilter",
            json!({"sessionId":frame["sessionId"],"filter":"new"}),
        );
        let arriving = f.card(&deck, "New arrival", "Answer");
        let before = f.call("getCard", json!({"cardId":arriving}));
        let continued = f.call(
            "continueChoiceFeedback",
            json!({"sessionId":frame["sessionId"],"feedbackId":feedback["feedbackId"]}),
        );
        assert_eq!(continued["payload"]["card"]["id"], arriving);
        let undone = f.call("undoReview", json!({"sessionId":frame["sessionId"]}));
        assert_eq!(undone["payload"]["choices"]["order"], *order);
        let as_set = |value: &Value| {
            value
                .as_array()
                .unwrap()
                .iter()
                .map(|id| id.as_str().unwrap().to_owned())
                .collect::<std::collections::BTreeSet<_>>()
        };
        assert_eq!(
            as_set(&undone["payload"]["choices"]["selectedIds"]),
            as_set(&ids)
        );
        assert_eq!(undone["payload"]["choices"]["confirmedCorrect"], false);
        assert_eq!(
            f.call("getCard", json!({"cardId":arriving}))["bag"],
            before["bag"]
        );
        assert_eq!(spelling::snapshot(&mut f)["reviews"], json!([]));
        assert_eq!(
            f.app
                .execute_json(json!({"kind":"confirmChoices","payload":input}))
                .unwrap_err()
                .code,
            "actionUndone"
        );
    }
}

#[test]
fn last_incorrect_question_stays_until_continue_and_continue_is_idempotent() {
    let (mut f, frame, _) = setup();
    let session = json!({"sessionId":frame["sessionId"]});
    let response = f.call(
        "confirmChoices",
        submission(&frame, json!({"kind":"unknown"})),
    );
    assert_eq!(response["state"], "choiceFeedback");
    assert_eq!(
        f.call("readSession", session.clone())["state"],
        "choiceFeedback"
    );
    let acknowledgement = json!({"sessionId":frame["sessionId"],
        "feedbackId":response["payload"]["feedbackId"]});
    let summary = f.call("continueChoiceFeedback", acknowledgement.clone());
    assert_eq!(summary["state"], "summary");
    assert_eq!(summary["payload"]["attempts"], 1);
    assert_eq!(f.call("continueChoiceFeedback", acknowledgement), summary);
    assert_eq!(f.call("undoReview", session)["state"], "question");
}

#[test]
fn invalid_selections_and_rollback_leave_no_rating() {
    let (mut f, frame, _) = setup();
    let order = &frame["choices"]["order"];
    let before = spelling::snapshot(&mut f);
    for ids in [json!([]), json!([order[0], order[0]]), json!(["not-owned"])] {
        assert!(
            f.app
                .execute_json(
                    json!({"kind":"confirmChoices","payload":submission(&frame,selected(ids))})
                )
                .is_err()
        );
        assert_eq!(spelling::snapshot(&mut f), before);
    }
    let db = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    db.execute_batch("CREATE TRIGGER injected_choice_failure BEFORE INSERT ON undo_records BEGIN SELECT RAISE(ABORT,'test rollback'); END;").unwrap();
    let input = submission(&frame, selected(json!([order[1]])));
    assert!(
        f.app
            .execute_json(json!({"kind":"confirmChoices","payload":input.clone()}))
            .is_err()
    );
    assert_eq!(spelling::snapshot(&mut f), before);
    db.execute_batch("DROP TRIGGER injected_choice_failure")
        .unwrap();
    assert_eq!(f.call("confirmChoices", input)["state"], "choiceFeedback");
}

#[test]
fn a_stale_confirmation_after_type_edit_never_grades() {
    let (mut f, frame, _) = setup();
    f.call("updateCard",json!({"cardId":frame["card"]["id"],"expectedRevision":frame["cardRevision"],"draft":{"forms":[{"id":frame["form"]["id"],"questionType":"essay","front":"Explain"}]}}));
    let error=f.app.execute_json(json!({"kind":"confirmChoices","payload":submission(&frame,selected(json!([frame["choices"]["order"][1]])))})).unwrap_err();
    assert_eq!(error.code, "conflict");
    assert_eq!(spelling::snapshot(&mut f)["reviews"], json!([]));
}
