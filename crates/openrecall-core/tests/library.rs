mod common;
use common::*;
use serde_json::json;
#[test]
fn decks_start_independent_and_share_only_when_selected() {
    let mut f = Fixture::new();
    let first = f.call("createDeck", json!({"name":"الطب","profileId":null}));
    let second = f.call(
        "createDeck",
        json!({"name":"Languages 🌍","profileId":null}),
    );
    assert_ne!(first["profileId"], second["profileId"]);
    let shared = f.call(
        "createDeck",
        json!({"name":"الجراحة","profileId":first["profileId"]}),
    );
    let boot = f.call("bootstrap", json!({}));
    assert_eq!(boot["decks"].as_array().unwrap().len(), 3);
    let profile = boot["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == first["profileId"])
        .unwrap();
    assert_eq!(profile["config"]["schedulePenalties"], true);
    assert_eq!(profile["memberDeckIds"].as_array().unwrap().len(), 2);
    assert_eq!(first["logicalCards"], 0);
    let renamed = f.call(
        "renameDeck",
        json!({"deckId":first["id"],"name":"الطب البشري","expectedRevision":first["revision"]}),
    );
    assert_eq!(renamed["name"], "الطب البشري");
    assert!(f.app.execute_json(json!({"kind":"renameDeck","payload":{"deckId":first["id"],"name":"x","expectedRevision":first["revision"]}})).is_err());
    f.call(
        "deleteDeck",
        json!({"deckId":shared["id"],"expectedRevision":shared["revision"]}),
    );
    f.reopen();
    let boot = f.call("bootstrap", json!({}));
    assert_eq!(boot["decks"].as_array().unwrap().len(), 2);
    let profile = boot["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == first["profileId"])
        .unwrap();
    assert_eq!(profile["memberDeckIds"], json!([first["id"]]));
}
#[test]
fn invalid_decks_and_failed_insert_do_not_leave_orphan_profiles() {
    let mut f = Fixture::new();
    for payload in [
        json!({"name":"  ","profileId":null}),
        json!({"name":"name","profileId":"missing"}),
    ] {
        assert!(
            f.app
                .execute_json(json!({"kind":"createDeck","payload":payload}))
                .is_err()
        );
    }
    let conn = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    conn.execute_batch("CREATE TRIGGER reject_deck BEFORE INSERT ON decks BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(
        f.app
            .execute_json(
                json!({"kind":"createDeck","payload":{"name":"must rollback","profileId":null}})
            )
            .is_err()
    );
    let boot = f.call("bootstrap", json!({}));
    assert_eq!(boot["decks"], json!([]));
    assert_eq!(boot["profiles"], json!([]));
}
