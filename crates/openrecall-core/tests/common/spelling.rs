use super::*;
pub fn create(f: &mut Fixture, source: &str) -> Value {
    let result = f
        .app
        .execute_json(json!({"kind":"createSpellingDeck","payload":{"sourceDeckId":source}}));
    assert!(
        result.is_ok(),
        "native spelling creation must succeed: {result:?}"
    );
    result.unwrap()["payload"].clone()
}
pub fn keys(f: &Fixture, deck: &str) -> Vec<String> {
    let db = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    db.prepare("SELECT canonical FROM spelling_words WHERE deck_id=?1 ORDER BY canonical")
        .unwrap()
        .query_map([deck], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}
pub fn snapshot(f: &mut Fixture) -> Value {
    let report = f
        .app
        .export_snapshot(&f.dir.path().join("snapshot.json"))
        .unwrap();
    serde_json::from_slice::<Value>(&std::fs::read(report.path).unwrap()).unwrap()["data"].clone()
}
pub fn card(f: &mut Fixture, deck: &str, query: &str) -> Value {
    let page = f.call(
        "listCards",
        json!({"deckId":deck,"query":query,"page":1,"pageSize":100}),
    );
    page["items"][0].clone()
}
