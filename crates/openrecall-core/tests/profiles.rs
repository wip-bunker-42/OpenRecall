mod common;
use common::*;
use serde_json::{Value, json};
fn finish(f: &mut Fixture, deck: &str) {
    let start = f.call("startSession", json!({"deckId":deck}));
    let sid = start["payload"]["sessionId"].clone();
    let view = f.call("revealAnswer", json!({"sessionId":sid}));
    let p = &view["payload"];
    f.call("rateCard",json!({"sessionId":sid,"presentationId":p["presentationId"],"rating":"good","actionId":uuid::Uuid::new_v4().to_string(),"expectedCardRevision":p["cardRevision"]}));
    f.call("endSession", json!({"sessionId":sid}));
    f.call("closeSession", json!({"sessionId":sid}));
}
fn profile(f: &mut Fixture, id: &Value) -> Value {
    f.call("getProfile", json!({"profileId":id}))
}
fn assign(f: &mut Fixture, deck: &str, profile_id: &Value) -> Value {
    let d = f.call("getDeck", json!({"deckId":deck}));
    let p = profile(f, profile_id);
    f.call("assignProfile",json!({"deckId":deck,"profileId":profile_id,"expectedDeckRevision":d["revision"],"expectedProfileRevision":p["revision"]}))
}
fn seed_trained_model(f: &mut Fixture, p: &Value) {
    let conn = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    let raw: String = conn
        .query_row(
            "SELECT payload FROM models WHERE id=?1",
            [p["activeModelRevision"].as_str().unwrap()],
            |r| r.get(0),
        )
        .unwrap();
    let mut model: Value = serde_json::from_str(&raw).unwrap();
    model["id"] = json!(uuid::Uuid::new_v4().to_string());
    model["parameters"][2] = json!(model["parameters"][2].as_f64().unwrap() * 2.0);
    model["origin"] = json!("trained");
    conn.execute(
        "INSERT INTO models(id,payload) VALUES(?1,?2)",
        rusqlite::params![model["id"].as_str().unwrap(), model.to_string()],
    )
    .unwrap();
    let mut changed = p.clone();
    changed["parameters"] = model["parameters"].clone();
    changed["activeModelRevision"] = model["id"].clone();
    changed["modelOrigin"] = json!("trained");
    changed["revision"] = json!(uuid::Uuid::new_v4().to_string());
    changed["modelRevision"] = model;
    conn.execute(
        "UPDATE profiles SET payload=?1 WHERE id=?2",
        rusqlite::params![changed.to_string(), p["id"].as_str().unwrap()],
    )
    .unwrap();
}
#[test]
fn shared_models_replay_without_rescheduling_and_detach_is_independent() {
    let mut f = Fixture::new();
    let a = f.deck("A");
    let b = f.deck("B");
    let c = f.deck("C unrelated");
    let ca = f.card(&a, "A", "a");
    let cb = f.card(&b, "B", "b");
    finish(&mut f, &a);
    finish(&mut f, &b);
    let original = f.call("getCard", json!({"cardId":ca}));
    let unrelated_before = f.call("getDeck", json!({"deckId":c}));
    let shared = f.call("createProfile", json!({"name":"مشترك"}));
    assert_eq!(shared["config"]["schedulePenalties"], true);
    seed_trained_model(&mut f, &shared);
    assign(&mut f, &a, &shared["id"]);
    assign(&mut f, &b, &shared["id"]);
    let replayed = f.call("getCard", json!({"cardId":ca}));
    assert_eq!(replayed["dueAt"], original["dueAt"]);
    assert_eq!(replayed["bag"], original["bag"]);
    assert!((replayed["memory"]["stability"].as_f64().unwrap() - 7.8442).abs() < 0.0001);
    let current = profile(&mut f, &shared["id"]);
    let mut config = current["config"].clone();
    config["schedulePenalties"] = json!(false);
    let changed = f.call(
        "saveTrainingConfig",
        json!({"profileId":shared["id"],"config":config,"expectedRevision":current["revision"]}),
    );
    let before_b = f.call("getCard", json!({"cardId":cb}));
    let d = f.call("getDeck", json!({"deckId":b}));
    let detached = f.call(
        "detachProfile",
        json!({"deckId":b,"newName":"مستقل","expectedDeckRevision":d["revision"]}),
    );
    assert_ne!(detached["id"], shared["id"]);
    assert_eq!(detached["config"], changed["config"]);
    let current = profile(&mut f, &shared["id"]);
    let reset = f.call(
        "resetProfile",
        json!({"profileId":shared["id"],"expectedRevision":current["revision"]}),
    );
    assert_eq!(reset["modelOrigin"], "default");
    assert_eq!(
        f.call("getCard", json!({"cardId":ca}))["memory"],
        original["memory"]
    );
    assert_eq!(f.call("getCard", json!({"cardId":cb})), before_b);
    assert_eq!(f.call("getDeck", json!({"deckId":c})), unrelated_before);
    f.reopen();
    assert_eq!(
        profile(&mut f, &detached["id"])["config"]["schedulePenalties"],
        false
    );
    assert!(f.app.execute_json(json!({"kind":"deleteProfile","payload":{"profileId":reset["id"],"expectedRevision":reset["revision"]}})).is_err());
}
#[test]
fn training_config_limits_and_stale_revisions_are_enforced() {
    let mut f = Fixture::new();
    let p = f.call("createProfile", json!({"name":"config"}));
    for (field, bad) in [
        ("numEpochs", json!(0)),
        ("numEpochs", json!(101)),
        ("batchSize", json!(0)),
        ("batchSize", json!(4097)),
        ("learningRate", json!(0.0)),
        ("learningRate", json!(0.2)),
        ("maxSeqLen", json!(31)),
        ("maxSeqLen", json!(4097)),
        ("gamma", json!(0.0)),
        ("gamma", json!(5.1)),
    ] {
        let mut config = p["config"].clone();
        config[field] = bad;
        assert!(f.app.execute_json(json!({"kind":"saveTrainingConfig","payload":{"profileId":p["id"],"config":config,"expectedRevision":p["revision"]}})).is_err());
    }
    let renamed = f.call(
        "renameProfile",
        json!({"profileId":p["id"],"name":"renamed","expectedRevision":p["revision"]}),
    );
    assert_eq!(renamed["name"], "renamed");
    assert!(f.app.execute_json(json!({"kind":"resetProfile","payload":{"profileId":p["id"],"expectedRevision":p["revision"]}})).is_err());
    f.call(
        "deleteProfile",
        json!({"profileId":renamed["id"],"expectedRevision":renamed["revision"]}),
    );
    assert_eq!(f.call("listProfiles", json!({})), json!([]));
}
#[test]
fn active_sessions_block_model_reassignment_and_reset_but_not_renaming() {
    let mut f = Fixture::new();
    let deck = f.deck("active");
    f.card(&deck, "q", "a");
    let d = f.call("getDeck", json!({"deckId":deck}));
    let p = profile(&mut f, &d["profileId"]);
    let next = f.call("createProfile", json!({"name":"next"}));
    f.call("startSession", json!({"deckId":deck}));
    assert_eq!(f.app.execute_json(json!({"kind":"assignProfile","payload":{"deckId":deck,"profileId":next["id"],"expectedDeckRevision":d["revision"],"expectedProfileRevision":next["revision"]}})).unwrap_err().code,"reviewInProgress");
    assert_eq!(f.app.execute_json(json!({"kind":"resetProfile","payload":{"profileId":p["id"],"expectedRevision":p["revision"]}})).unwrap_err().code,"reviewInProgress");
    assert_eq!(
        f.call(
            "renameProfile",
            json!({"profileId":p["id"],"name":"still active","expectedRevision":p["revision"]})
        )["activeModelRevision"],
        p["activeModelRevision"]
    );
}
#[test]
fn shared_model_decks_can_have_different_retention_without_rewriting_due_dates() {
    let mut f = Fixture::new();
    let a = f.deck("retention A");
    let da = f.call("getDeck", json!({"deckId":a}));
    let db = f.call(
        "createDeck",
        json!({"name":"retention B","profileId":da["profileId"]}),
    );
    let b = db["id"].as_str().unwrap().to_owned();
    let ca = f.card(&a, "qA", "a");
    let cb = f.card(&b, "qB", "a");
    finish(&mut f, &a);
    finish(&mut f, &b);
    let due = f.call("getCard", json!({"cardId":ca}))["dueAt"].clone();
    for (deck, retention) in [(&a, 0.80), (&b, 0.95)] {
        let d = f.call("getDeck", json!({"deckId":deck}));
        f.call("saveDeckPolicy",json!({"deckId":deck,"desiredRetention":retention,"minIntervalSeconds":null,"maxIntervalDays":null,"expectedRevision":d["revision"]}));
    }
    assert_eq!(f.call("getCard", json!({"cardId":ca}))["dueAt"], due);
    f.advance(6 * 86400);
    let va = f.call("startSession", json!({"deckId":a}));
    let vb = f.call("startSession", json!({"deckId":b}));
    let pa = f.call(
        "revealAnswer",
        json!({"sessionId":va["payload"]["sessionId"]}),
    );
    let pb = f.call(
        "revealAnswer",
        json!({"sessionId":vb["payload"]["sessionId"]}),
    );
    assert!(
        pa["payload"]["previews"]["good"]["intervalSeconds"]
            .as_f64()
            .unwrap()
            > pb["payload"]["previews"]["good"]["intervalSeconds"]
                .as_f64()
                .unwrap()
    );
    assert_eq!(f.call("getCard", json!({"cardId":cb}))["reviewCount"], 1);
}
