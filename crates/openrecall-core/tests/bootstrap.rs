mod common;
use common::*;
use openrecall_core::App;
use serde_json::json;
#[test]
fn bootstrap_persists_defaults_and_rejects_future_schema() {
    let mut f = Fixture::new();
    let boot = f.call("bootstrap", json!({}));
    assert_eq!(boot["settings"]["timezone"], "Africa/Cairo");
    assert_eq!(boot["settings"]["dayStartMinute"], 60);
    assert_eq!(boot["settings"]["desiredRetention"], 0.9);
    f.reopen();
    assert_eq!(f.call("bootstrap", json!({})), boot);
    let newer = f.dir.path().join("future.sqlite3");
    rusqlite::Connection::open(&newer)
        .unwrap()
        .execute_batch("PRAGMA user_version=999")
        .unwrap();
    let result = App::open(&newer, f.clock.clone(), Box::new(SeededRandom::new(1)));
    assert_eq!(result.err().unwrap().code, "unsupportedSchema");
}
