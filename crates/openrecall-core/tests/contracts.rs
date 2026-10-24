mod common;
use common::*;
use serde_json::json;
#[test]
fn contracts_round_trip_tagged_messages() {
    let mut f = Fixture::new();
    let response = f
        .app
        .execute_json(json!({"kind":"bootstrap","payload":{}}))
        .unwrap();
    assert_eq!(response["kind"], "bootstrap");
    assert!(response["payload"]["settings"]["revision"].is_string());
    assert!(
        f.app
            .execute_json(json!({"kind":"unexpectedCommand","payload":{}}))
            .is_err()
    );
}
