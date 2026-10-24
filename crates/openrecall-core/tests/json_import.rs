mod common;
use common::*;
use serde_json::{Value, json};
fn preview(f: &mut Fixture, deck: &str, rows: Value) -> Value {
    f.call(
        "previewImport",
        json!({"deckId":deck,"text":rows.to_string()}),
    )
}
fn commit(
    f: &mut Fixture,
    p: &Value,
    selected: Vec<Value>,
    excluded: Vec<Value>,
) -> Result<Value, openrecall_core::AppError> {
    f.app.execute_json(json!({"kind":"commitImport","payload":{"previewId":p["previewId"],"libraryRevision":p["libraryRevision"],"selectedCardKeys":selected,"excludedFormKeys":excluded}})).map(|r|r["payload"].clone())
}
#[test]
fn preview_is_read_only_and_commit_is_atomic() {
    let mut f = Fixture::new();
    let deck = f.deck("استيراد");
    let before = f.call("bootstrap", json!({}));
    let observer = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    let version = || {
        observer
            .pragma_query_value(None, "data_version", |r| r.get::<_, i64>(0))
            .unwrap()
    };
    let original_version = version();
    let p = preview(
        &mut f,
        &deck,
        json!([{"front":"سؤال\nEnglish","back":"جواب","extra":true,"variants":[{"front":"بديل","back":"Answer"},{"front":7,"back":"bad"}]}]),
    );
    assert_eq!(f.call("bootstrap", json!({})), before);
    assert_eq!(version(), original_version);
    assert!(p["counts"]["warnings"].as_u64().unwrap() > 0);
    let key = p["cards"][0]["key"].clone();
    let invalid = p["cards"][0]["variants"][1]["key"].clone();
    assert!(commit(&mut f, &p, vec![key.clone()], vec![]).is_err());
    let report = commit(&mut f, &p, vec![key], vec![invalid]).unwrap();
    assert_eq!(report["insertedCards"], 1);
    assert_eq!(report["insertedForms"], 2);
    let cards = f.call(
        "listCards",
        json!({"deckId":deck,"query":"","page":1,"pageSize":20}),
    );
    assert_eq!(cards["items"][0]["forms"][0]["front"], "سؤال\nEnglish");
    assert_eq!(cards["items"][0]["forms"][1]["notes"], "");
}
#[test]
fn duplicates_cover_primary_variants_other_decks_and_file_rows() {
    let mut f = Fixture::new();
    let existing = f.deck("existing");
    f.call("createCard",json!({"deckId":existing,"draft":{"forms":[{"front":"café","back":"answer","notes":"old"},{"front":"Alt","back":"answer","notes":""}]}}));
    let target = f.deck("target");
    let p = preview(
        &mut f,
        &target,
        json!([
            {"front":" cafe\u{301} \n","back":" answer ","notes":"different"},
            {"front":"unique","back":"a","variants":[{"front":"Alt","back":"answer"},{"front":"own","back":"b"},{"front":"own","back":"b"}]},
            {"front":"own","back":"b"},{"front":"Unique","back":"a"},{"front":"unique","back":"different"}
        ]),
    );
    assert_eq!(p["cards"][0]["primary"]["duplicate"], true);
    assert_eq!(p["cards"][1]["variants"][0]["duplicate"], true);
    assert_eq!(p["cards"][1]["variants"][2]["duplicate"], true);
    assert_eq!(p["cards"][2]["primary"]["duplicate"], true);
    assert_eq!(p["counts"]["validCards"], 3);
    let selected = vec![
        p["cards"][1]["key"].clone(),
        p["cards"][3]["key"].clone(),
        p["cards"][4]["key"].clone(),
    ];
    let excluded = vec![
        p["cards"][1]["variants"][0]["key"].clone(),
        p["cards"][1]["variants"][2]["key"].clone(),
    ];
    let report = commit(&mut f, &p, selected, excluded).unwrap();
    assert_eq!(report["insertedCards"], 3);
    assert_eq!(report["insertedForms"], 4);
}
#[test]
fn invalid_alternatives_can_be_excluded_and_wrong_types_have_paths() {
    let mut f = Fixture::new();
    let deck = f.deck("validation");
    for bad in ["{", "{}", "null"] {
        assert!(
            f.app
                .execute_json(json!({"kind":"previewImport","payload":{"deckId":deck,"text":bad}}))
                .is_err()
        );
    }
    let p = preview(
        &mut f,
        &deck,
        json!([{ "front":"valid","back":"a","variants":[{"front":"nested","back":"a","variants":[]}]},{"front":"null notes","back":"a","notes":null},{"front":"x","back":false},{"front":"bad array","back":"a","variants":null},42]),
    );
    assert_eq!(p["cards"][0]["valid"], true);
    assert_eq!(p["cards"][0]["variants"][0]["valid"], false);
    assert_eq!(p["cards"][1]["valid"], false);
    assert_eq!(p["cards"][2]["primary"]["issues"][0]["field"], "$[2].back");
    assert_eq!(p["cards"][3]["valid"], false);
    assert_eq!(p["cards"][4]["valid"], false);
    commit(
        &mut f,
        &p,
        vec![p["cards"][0]["key"].clone()],
        vec![p["cards"][0]["variants"][0]["key"].clone()],
    )
    .unwrap();
    let bom = f.call(
        "previewImport",
        json!({"deckId":deck,"text":"\u{feff}[{\"front\":\"BOM\",\"back\":\"works\"}]"}),
    );
    assert_eq!(bom["counts"]["validCards"], 1);
    let huge = " ".repeat(50 * 1024 * 1024 + 1);
    assert_eq!(
        f.app
            .execute_json(json!({"kind":"previewImport","payload":{"deckId":deck,"text":huge}}))
            .unwrap_err()
            .code,
        "importTooLarge"
    );
}
#[test]
fn changed_library_and_second_insert_failure_never_partially_import() {
    let mut f = Fixture::new();
    let deck = f.deck("atomic");
    let p = preview(
        &mut f,
        &deck,
        json!([{"front":"first","back":"a"},{"front":"second","back":"a"}]),
    );
    let selected = p["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["key"].clone())
        .collect::<Vec<_>>();
    let observer = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    observer.execute_batch("CREATE TRIGGER fail_second BEFORE INSERT ON cards WHEN json_extract(new.payload,'$.forms[0].front')='second' BEGIN SELECT RAISE(ABORT,'injected'); END").unwrap();
    assert!(commit(&mut f, &p, selected.clone(), vec![]).is_err());
    assert_eq!(f.call("getDeck", json!({"deckId":deck}))["logicalCards"], 0);
    observer.execute_batch("DROP TRIGGER fail_second").unwrap();
    let report = commit(&mut f, &p, selected, vec![]).unwrap();
    assert_eq!(report["insertedCards"], 2);
    let cards = f.call(
        "listCards",
        json!({"deckId":deck,"query":"","page":1,"pageSize":20}),
    );
    assert_eq!(cards["items"][0]["dueAt"], cards["items"][1]["dueAt"]);
    let stale = preview(&mut f, &deck, json!([{"front":"third","back":"a"}]));
    f.card(&deck, "other", "a");
    assert_eq!(
        commit(
            &mut f,
            &stale,
            vec![stale["cards"][0]["key"].clone()],
            vec![]
        )
        .unwrap_err()
        .code,
        "conflict"
    );
}
