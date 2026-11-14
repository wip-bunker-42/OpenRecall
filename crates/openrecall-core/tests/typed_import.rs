mod common;
use common::Fixture;
use serde_json::{Value, json};

fn preview(f: &mut Fixture, deck: &str, values: Value) -> Value {
    f.call(
        "previewImport",
        json!({"deckId":deck,"text":values.to_string()}),
    )
}

fn mcq(first: &str, second: &str, answer: &str) -> Value {
    json!({"questionType":"multipleChoice","front":"Pick a case",
        "choices":[{"key":"first","text":first},{"key":"second","text":second}],
        "correctChoices":[answer]})
}

fn commit(f: &mut Fixture, p: &Value, selected: Value, excluded: Value) {
    f.call(
        "commitImport",
        json!({"previewId":p["previewId"],"libraryRevision":p["libraryRevision"],
        "selectedCardKeys":selected,"excludedFormKeys":excluded}),
    );
}

#[test]
fn typed_preview_commit_resolves_keys_and_searches_choices() {
    let mut f = Fixture::new();
    let deck = f.deck("Typed import");
    let p = preview(
        &mut f,
        &deck,
        json!([
            {"questionType":"essay","front":"Explain spacing"},
            {"questionType":"multipleChoice","front":"Capital of France?",
             "choices":[{"key":"p","text":"Paris"},{"key":"l","text":"London"}],
             "correctChoices":["p"],
             "variants":[
               {"front":"Capital of Germany?","choices":[{"key":"b","text":"Berlin"},{"key":"p","text":"Paris"}],"correctChoices":["b"]},
               {"questionType":"essay","front":"Explain capital cities"},
               {"front":"Missing its own options"}
             ]},
            {"front":"Legacy missing answer"}
        ]),
    );
    let card = &p["cards"][1];
    assert_eq!(p["cards"][0]["primary"]["valid"], true);
    assert_eq!(card["primary"]["questionType"], "multipleChoice");
    assert_eq!(card["variants"][0]["valid"], true);
    assert_eq!(card["variants"][1]["questionType"], "essay");
    assert_eq!(card["variants"][2]["valid"], false);
    assert_eq!(p["cards"][2]["valid"], false);
    commit(
        &mut f,
        &p,
        json!([p["cards"][0]["key"], card["key"]]),
        json!([card["variants"][2]["key"]]),
    );
    let list = f.call(
        "listCards",
        json!({"deckId":deck,"page":1,"pageSize":20,"query":"London"}),
    );
    assert_eq!(list["total"], 1);
    let forms = list["items"][0]["forms"].as_array().unwrap();
    assert_eq!(forms.len(), 3);
    let choices = &forms[0]["multipleChoice"];
    let paris = choices["choices"][0]["id"].as_str().unwrap();
    assert!(uuid::Uuid::parse_str(paris).is_ok());
    assert_eq!(choices["correctChoiceIds"], json!([paris]));
}

#[test]
fn malformed_choice_metadata_keeps_authored_options_and_marks_the_actual_field() {
    let mut f = Fixture::new();
    let deck = f.deck("Import errors");
    for (field, value, code) in [
        ("shuffleChoices", Some(json!("false")), "booleanExpected"),
        (
            "selectionMode",
            Some(json!("bogus")),
            "invalidSelectionMode",
        ),
        ("correctChoices", None, "required"),
    ] {
        let mut row = mcq("Red", "Book", "first");
        if let Some(value) = value {
            row[field] = value;
        } else {
            row.as_object_mut().unwrap().remove(field);
        }
        let p = preview(&mut f, &deck, json!([row]));
        let form = &p["cards"][0]["primary"];
        assert_eq!(form["valid"], false);
        assert_eq!(
            form["issues"],
            json!([{"field":format!("$[0].{field}"),"code":code}])
        );
        assert_eq!(form["multipleChoice"]["choices"][0]["text"], "Red");
    }
    let mut row = mcq("Red", "Book", "first");
    row["choices"][0]["id"] = json!("supplied-id");
    let p = preview(&mut f, &deck, json!([row]));
    assert_eq!(
        p["cards"][0]["primary"]["issues"],
        json!([{"field":"$[0].choices.0.id","code":"invalidOwner"}])
    );
}

#[test]
fn reordered_keys_are_duplicate_but_case_and_correctness_are_meaningful() {
    let mut f = Fixture::new();
    let deck = f.deck("Duplicates");
    let first = preview(&mut f, &deck, json!([mcq("US", "us", "first")]));
    commit(&mut f, &first, json!([first["cards"][0]["key"]]), json!([]));
    let mut duplicate = mcq("us", "US", "second");
    duplicate["choices"][1]["key"] = json!("renamed");
    duplicate["correctChoices"] = json!(["renamed"]);
    duplicate["notes"] = json!("different notes");
    duplicate["shuffleChoices"] = json!(false);
    let p = preview(
        &mut f,
        &deck,
        json!([
            duplicate,
            mcq("US", "us", "second"),
            mcq("A B", " A  B ", "first")
        ]),
    );
    assert_eq!(p["cards"][0]["primary"]["duplicate"], true);
    assert_eq!(p["cards"][1]["primary"]["duplicate"], false);
    assert_eq!(p["cards"][1]["valid"], true);
    assert_eq!(p["cards"][2]["valid"], false);
}
