mod common;
use common::{Fixture, spelling};
use serde_json::json;

#[test]
fn essay_direct_rating_is_retry_safe_and_can_be_undone_after_restore() {
    let mut f = Fixture::new();
    let deck = f.deck("Essay");
    let essay = f.call("createCard",json!({"deckId":deck,"draft":{"forms":[{"front":"Explain the concept","questionType":"essay"}]}}));
    let frame = f.call("startSession", json!({"deckId":deck}))["payload"].clone();
    assert_eq!(frame["canRate"], true);
    assert_eq!(frame["answerRevealed"], false);
    assert!(frame["previews"].as_object().is_some_and(|p| p.len() == 4));
    assert!(
        f.app
            .execute_json(json!({"kind":"revealAnswer","payload":{"sessionId":frame["sessionId"]}}))
            .is_err()
    );
    let input = json!({"sessionId":frame["sessionId"],"presentationId":frame["presentationId"],
        "rating":"good","actionId":uuid::Uuid::new_v4().to_string(),"expectedCardRevision":frame["cardRevision"]});
    f.call("rateCard", input.clone());
    f.call("rateCard", input);
    assert_eq!(
        spelling::snapshot(&mut f)["reviews"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let export = f
        .app
        .export_snapshot(&f.dir.path().join("essay.json"))
        .unwrap();
    let mut restored = Fixture::new();
    let p = restored
        .app
        .preview_restore(std::path::Path::new(&export.path))
        .unwrap();
    restored
        .app
        .restore_snapshot(
            p.id,
            p.expected_library_revision,
            &restored.dir.path().join("safety.json"),
        )
        .unwrap();
    let undo =
        restored.call("undoReview", json!({"sessionId":frame["sessionId"]}))["payload"].clone();
    assert_eq!(undo["presentationId"], frame["presentationId"]);
    assert_eq!(undo["form"]["questionType"], "essay");
    assert_eq!(undo["card"]["dueAt"], essay["dueAt"]);
    assert_eq!(undo["card"]["reviewCount"], 0);
}

#[test]
fn direct_grades_remain_blocked_on_unrevealed_recall_and_unchecked_choices() {
    let mut f = Fixture::new();
    let recall = f.deck("Recall");
    f.card(&recall, "Question", "Answer");
    let choices = f.deck("Choices");
    f.call("createCard",json!({"deckId":choices,"draft":{"forms":[{
        "questionType":"multipleChoice","front":"Capital?",
        "multipleChoice":{"choices":[{"key":"a","text":"Paris"},{"key":"b","text":"Rome"}],"correctChoiceKeys":["a"]}
    }]}}));
    for deck in [recall, choices] {
        let frame = f.call("startSession", json!({"deckId":deck}))["payload"].clone();
        assert_eq!(frame["canRate"], false);
        let input = json!({"sessionId":frame["sessionId"],"presentationId":frame["presentationId"],
            "rating":"good","actionId":uuid::Uuid::new_v4().to_string(),"expectedCardRevision":frame["cardRevision"]});
        assert_eq!(
            f.app
                .execute_json(json!({"kind":"rateCard","payload":input}))
                .unwrap_err()
                .code,
            "answerHidden"
        );
    }
    assert_eq!(spelling::snapshot(&mut f)["reviews"], json!([]));
}
