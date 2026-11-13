mod common;
use common::{Fixture, spelling};
use serde_json::{Value, json};

fn create(f: &mut Fixture, deck: &str, form: Value) -> openrecall_core::AppResult<Value> {
    f.app
        .execute_json(
            json!({"kind":"createCard","payload":{"deckId":deck,"draft":{"forms":[form]}}}),
        )
        .map(|r| r["payload"].clone())
}

fn multiple() -> Value {
    json!({"questionType":"multipleChoice","front":"Pick a case","multipleChoice":{
        "choices":[{"key":"a","text":"US"},{"key":"b","text":"us"}],
        "correctChoiceKeys":["a"],"selectionMode":"single","shuffleChoices":true
    }})
}

#[test]
fn typed_creation_enforces_answer_rules_and_choice_ownership() {
    let mut f = Fixture::new();
    let deck = f.deck("Typed");
    let essay = create(
        &mut f,
        &deck,
        json!({"questionType":"essay","front":"Explain"}),
    )
    .unwrap();
    assert_eq!(essay["forms"][0]["questionType"], "essay");
    assert_eq!(essay["forms"][0]["back"], "");
    for bad in [
        json!({"front":"Missing answer"}),
        json!({"questionType":"essay","front":"Essay","back":"hidden"}),
        json!({"questionType":"recall","front":"Recall","back":"Answer","multipleChoice":multiple()["multipleChoice"]}),
    ] {
        assert!(create(&mut f, &deck, bad).is_err());
    }
    let mcq = create(&mut f, &deck, multiple()).unwrap();
    let native = &mcq["forms"][0]["multipleChoice"];
    let first = native["choices"][0]["id"].clone();
    let second = native["choices"][1]["id"].clone();
    assert_ne!(first, second);
    assert_eq!(native["correctChoiceIds"], json!([first]));
    let mut edit = multiple();
    edit["id"] = mcq["forms"][0]["id"].clone();
    edit["multipleChoice"]["choices"][0]["id"] = first.clone();
    let update = |f: &mut Fixture, draft: Value| {
        f.app.execute_json(json!({
        "kind":"updateCard","payload":{"cardId":mcq["id"],"expectedRevision":mcq["revision"],"draft":{"forms":[draft]}}
    }))
    };
    let mut foreign = edit.clone();
    foreign["multipleChoice"]["choices"][0]["id"] = json!(uuid::Uuid::new_v4().to_string());
    assert!(update(&mut f, foreign).is_err());
    assert_eq!(
        update(&mut f, edit).unwrap()["payload"]["forms"][0]["multipleChoice"]["choices"][0]["id"],
        first
    );
    for (path, value) in [
        ("/choices/1/text", json!(" US ")),
        ("/choices/1/key", json!("a")),
        ("/choices/1/key", json!("")),
        ("/correctChoiceKeys", json!(["missing"])),
        ("/correctChoiceKeys", json!(["a", "b"])),
    ] {
        let mut bad = multiple();
        *bad["multipleChoice"].pointer_mut(path).unwrap() = value;
        assert!(create(&mut f, &deck, bad).is_err(), "{path}");
    }
}

#[test]
fn bound_spelling_deck_preserves_typed_snapshots_and_ignores_distractors() {
    let mut f = Fixture::new();
    let source = f.deck("Source");
    let result = spelling::create(&mut f, &source);
    let deck = result["deck"]["id"].as_str().unwrap();
    create(
        &mut f,
        &source,
        json!({"questionType":"essay","front":"Explain","notes":"Review"}),
    )
    .unwrap();
    let mut mcq = multiple();
    mcq["multipleChoice"]["choices"][0]["text"] = json!("green");
    mcq["multipleChoice"]["choices"][1]["text"] = json!("wrongword");
    create(&mut f, &source, mcq).unwrap();
    let words = spelling::keys(&f, deck);
    assert!(words.contains(&"green".into()));
    assert!(words.contains(&"explain".into()));
    assert!(!words.contains(&"wrongword".into()));
    let backup = f
        .app
        .export_snapshot(&f.dir.path().join("typed.json"))
        .unwrap();
    let mut target = Fixture::new();
    let preview = target
        .app
        .preview_restore(std::path::Path::new(&backup.path))
        .unwrap();
    target
        .app
        .restore_snapshot(
            preview.id,
            preview.expected_library_revision,
            &target.dir.path().join("safety.json"),
        )
        .unwrap();
    assert_eq!(spelling::keys(&target, deck), words);
}
