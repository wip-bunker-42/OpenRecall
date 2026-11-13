mod common;
use common::{Fixture, spelling};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[test]
fn committed_feedback_survives_restore_and_rejects_inconsistent_payloads() {
    let mut f = Fixture::new();
    let deck = f.deck("Backup");
    f.call("createCard",json!({"deckId":deck,"draft":{"forms":[{
        "questionType":"multipleChoice","front":"Choose the color",
        "multipleChoice":{"choices":[{"key":"a","text":"Red"},{"key":"b","text":"Book"}],"correctChoiceKeys":["a"]}
    }]}}));
    let frame = f.call("startSession", json!({"deckId":deck}))["payload"].clone();
    let wrong = &frame["form"]["multipleChoice"]["choices"][1]["id"];
    let response = f.call(
        "confirmChoices",
        json!({"sessionId":frame["sessionId"],"presentationId":frame["presentationId"],
        "expectedCardRevision":frame["cardRevision"],"actionId":uuid::Uuid::new_v4().to_string(),
        "submission":{"kind":"selected","choiceIds":[wrong]}}),
    );
    let path = f.dir.path().join("portable.json");
    let report = f.app.export_snapshot(&path).unwrap();
    let original: Value = serde_json::from_slice(&std::fs::read(report.path).unwrap()).unwrap();
    let mut target = Fixture::new();
    let before = spelling::snapshot(&mut target);
    for change in 0..5 {
        let mut raw = original.clone();
        match change {
            0 => {
                raw["data"]["sessions"][0]["choiceFeedback"]["choices"]["order"] =
                    json!([wrong, wrong])
            }
            1 => {
                raw["data"]["sessions"][0]["choiceFeedback"]["choices"]["confirmedCorrect"] =
                    json!(true)
            }
            2 => raw["data"]["reviews"][0]["rating"] = json!("good"),
            3 => {
                raw["data"]["sessions"][0]["choiceFeedback"]["actionId"] =
                    json!(uuid::Uuid::new_v4().to_string())
            }
            _ => raw["data"]["actions"][0]["choiceOutcome"] = json!("correct"),
        }
        let mut data = raw["data"].clone();
        data.sort_all_objects();
        raw["payloadSha256"] = json!(
            Sha256::digest(serde_json::to_vec(&data).unwrap())
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
        let invalid = target.dir.path().join("invalid.json");
        std::fs::write(&invalid, serde_json::to_vec(&raw).unwrap()).unwrap();
        assert_ne!(
            target
                .app
                .preview_restore(&invalid)
                .expect_err(&format!("mutation {change}"))
                .code,
            "backupDigest",
            "{change}"
        );
        assert_eq!(spelling::snapshot(&mut target), before);
    }
    let p = target.app.preview_restore(&path).unwrap();
    target
        .app
        .restore_snapshot(
            p.id,
            p.expected_library_revision,
            &target.dir.path().join("safety.json"),
        )
        .unwrap();
    let restored = target.call("readSession", json!({"sessionId":frame["sessionId"]}));
    assert_eq!(
        restored["payload"]["feedbackId"],
        response["payload"]["feedbackId"]
    );
    let undo = target.call("undoReview", json!({"sessionId":frame["sessionId"]}));
    assert_eq!(undo["payload"]["choices"]["selectedIds"], json!([wrong]));
    assert_eq!(undo["payload"]["card"]["reviewCount"], 0);
}
