mod common;
use common::{Fixture, spelling};
use openrecall_core::contracts::review::CopyQuestionInput;
use serde_json::json;

#[test]
fn essay_copy_is_exact_read_only_and_bound_to_current_display() {
    let mut f = Fixture::new();
    let deck = f.deck("Copy");
    f.call(
        "createCard",
        json!({"deckId":deck,"draft":{"forms":[
            {"front":"<b>Explain</b>\nSecond line","questionType":"essay","notes":"private notes"}
        ]}}),
    );
    let frame = f.call("startSession", json!({"deckId":deck}))["payload"].clone();
    let mut input: CopyQuestionInput = serde_json::from_value(json!({
        "sessionId":frame["sessionId"],"displayId":frame["presentationId"],
        "expectedCardRevision":frame["cardRevision"]
    }))
    .unwrap();
    let before = spelling::snapshot(&mut f);
    assert_eq!(
        f.app.question_copy_text(&input).unwrap(),
        "<b>Explain</b>\nSecond line"
    );
    input.display_id = uuid::Uuid::new_v4().to_string();
    assert!(f.app.question_copy_text(&input).is_err());
    assert_eq!(spelling::snapshot(&mut f), before);
}

#[test]
fn mcq_copy_contains_choices_in_display_order_without_solution_markers() {
    let mut f = Fixture::new();
    let deck = f.deck("Choice copy");
    f.call(
        "createCard",
        json!({"deckId":deck,"draft":{"forms":[{
            "front":"Which city?","questionType":"multipleChoice","notes":"Hidden explanation",
            "multipleChoice":{"choices":[{"key":"a","text":"Paris"},{"key":"b","text":"Rome"}],
                "correctChoiceKeys":["a"],"shuffleChoices":false}
        }]}}),
    );
    let frame = f.call("startSession", json!({"deckId":deck}))["payload"].clone();
    let input: CopyQuestionInput = serde_json::from_value(json!({"sessionId":frame["sessionId"],
        "displayId":frame["presentationId"],"expectedCardRevision":frame["cardRevision"]}))
    .unwrap();
    assert_eq!(
        f.app.question_copy_text(&input).unwrap(),
        "Which city?\n\n1. Paris\n2. Rome"
    );
    let feedback=f.call("confirmChoices",json!({"sessionId":frame["sessionId"],"presentationId":frame["presentationId"],
        "expectedCardRevision":frame["cardRevision"],"actionId":uuid::Uuid::new_v4().to_string(),"submission":{"kind":"unknown"}}));
    let data = &feedback["payload"];
    let input: CopyQuestionInput = serde_json::from_value(json!({"sessionId":frame["sessionId"],
        "displayId":data["feedbackId"],"expectedCardRevision":data["cardRevision"]}))
    .unwrap();
    assert_eq!(
        f.app.question_copy_text(&input).unwrap(),
        "Which city?\n\n1. Paris\n2. Rome"
    );
}
