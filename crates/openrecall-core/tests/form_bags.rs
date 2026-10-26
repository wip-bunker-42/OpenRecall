mod common;
use common::SeededRandom;
use openrecall_core::scheduling::forms::*;
use std::collections::HashSet;
#[test]
fn saved_reserved_form_is_not_lost_after_reopen_and_edit() {
    use serde_json::json;
    let mut f = common::Fixture::new();
    let deck = f.call("createDeck", json!({"name":"forms","profileId":null}));
    let mut card=f.call("createCard",json!({"deckId":deck["id"],"draft":{"forms":[{"front":"one","back":"a"},{"front":"two","back":"a"}]}}));
    let a = card["forms"][0]["id"].as_str().unwrap().to_owned();
    let b = card["forms"][1]["id"].as_str().unwrap().to_owned();
    card["bag"] =
        json!({"remaining":[a,b],"consumed":[],"lastConsumed":null,"reserved":a,"cycle":1});
    {
        let c = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
        c.execute(
            "UPDATE cards SET payload=?1 WHERE id=?2",
            rusqlite::params![card.to_string(), card["id"].as_str().unwrap()],
        )
        .unwrap();
    }
    f.reopen();
    let loaded = f.call("getCard", json!({"cardId":card["id"]}));
    assert_eq!(loaded["bag"], card["bag"]);
    let edited=f.call("updateCard",json!({"cardId":card["id"],"expectedRevision":loaded["revision"],"draft":{"forms":loaded["forms"]}}));
    assert_eq!(edited["bag"]["reserved"], a);
}
#[test]
fn randomized_cycles_never_repeat_and_reservations_survive_serialization() {
    let ids = vec!["a".into(), "b".into(), "c".into(), "d".into()];
    let mut orders = HashSet::new();
    for seed in 0..32 {
        let mut rng = SeededRandom::new(seed);
        let mut bag = FormBag::default();
        let mut last = None;
        for _ in 0..5 {
            let mut cycle = vec![];
            for i in 0..4 {
                let selected = reserve_form(&mut bag, &ids, &mut rng).unwrap();
                if i == 0 {
                    assert_ne!(last.as_ref(), Some(&selected));
                }
                let before = bag.clone();
                assert_eq!(reserve_form(&mut bag, &ids, &mut rng).unwrap(), selected);
                assert_eq!(bag, before);
                bag = serde_json::from_str(&serde_json::to_string(&bag).unwrap()).unwrap();
                assert_eq!(reserve_form(&mut bag, &ids, &mut rng).unwrap(), selected);
                consume_form(&mut bag, &selected).unwrap();
                cycle.push(selected.clone());
                last = Some(selected);
            }
            assert_eq!(cycle.iter().collect::<HashSet<_>>().len(), 4);
            orders.insert(cycle);
        }
    }
    assert!(orders.len() > 1);
}
#[test]
fn edits_keep_reserved_forms_and_new_forms_do_not_reuse_consumed_ids() {
    let mut rng = SeededRandom::new(4);
    let mut bag = FormBag::default();
    let ids = vec!["a".into(), "b".into(), "c".into()];
    let first = reserve_form(&mut bag, &ids, &mut rng).unwrap();
    consume_form(&mut bag, &first).unwrap();
    let reserved = reserve_form(&mut bag, &ids, &mut rng).unwrap();
    let mut edited = ids.clone();
    edited.push("d".into());
    reconcile_forms(&mut bag, &edited, &mut rng).unwrap();
    assert_eq!(reserve_form(&mut bag, &edited, &mut rng).unwrap(), reserved);
    assert!(!bag.remaining.contains(&first));
    assert!(bag.remaining.contains(&"d".into()));
    let before = bag.clone();
    consume_form(&mut bag, &reserved).unwrap();
    bag = before.clone();
    assert_eq!(reserve_form(&mut bag, &edited, &mut rng).unwrap(), reserved);
    assert_eq!(bag, before);
    edited.retain(|x| x != &reserved);
    reconcile_forms(&mut bag, &edited, &mut rng).unwrap();
    assert_ne!(reserve_form(&mut bag, &edited, &mut rng).unwrap(), reserved);
    let mut single = FormBag::default();
    for _ in 0..4 {
        let choice = reserve_form(&mut single, &["only".into()], &mut rng).unwrap();
        assert_eq!(choice, "only");
        consume_form(&mut single, &choice).unwrap();
    }
}
