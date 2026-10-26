mod common;
use common::training::*;
use openrecall_core::{
    contracts::training::TrainingEligibility,
    training::{dataset::*, eligibility::assess},
};
#[test]
fn eligibility_enforces_every_exact_boundary() {
    let good = TrainingEligibility {
        eligible: false,
        usable_observations: 400,
        distinct_cards: 50,
        study_days: 7,
        again_count: 5,
        recalled_count: 395,
        train_count: 320,
        holdout_count: 80,
        holdout_cards: 10,
        unmet_conditions: vec![],
        contributions: vec![],
    };
    assert!(assess(good.clone()).eligible);
    for (field, low, high) in [
        ("observations", 399, 400),
        ("cards", 49, 50),
        ("days", 6, 7),
        ("again", 4, 5),
        ("recalled", 4, 5),
        ("train", 63, 64),
        ("holdout", 79, 80),
        ("holdoutCards", 9, 10),
    ] {
        for (value, eligible) in [(low, false), (high, true)] {
            let mut c = good.clone();
            match field {
                "observations" => c.usable_observations = value,
                "cards" => c.distinct_cards = value,
                "days" => c.study_days = value,
                "again" => c.again_count = value,
                "recalled" => c.recalled_count = value,
                "train" => c.train_count = value,
                "holdout" => c.holdout_count = value,
                _ => c.holdout_cards = value,
            };
            assert_eq!(assess(c).eligible, eligible, "{field}={value}");
        }
    }
}
#[test]
fn dataset_splits_in_time_without_target_or_profile_leakage() {
    let rows = observations();
    let data = build_dataset(&rows, context()).unwrap();
    assert_eq!(data.eligibility.usable_observations, 400);
    assert_eq!(data.eligibility.distinct_cards, 50);
    assert_eq!(data.train_items.len(), 320);
    assert_eq!(data.holdout_targets.len(), 80);
    assert!(data.eligibility.eligible);
    let cutoff: chrono::DateTime<chrono::Utc> = data.cutoff_at.as_ref().unwrap().parse().unwrap();
    assert!(
        data.train_target_times
            .iter()
            .all(|t| t.parse::<chrono::DateTime<chrono::Utc>>().unwrap() < cutoff)
    );
    assert!(
        data.holdout_targets
            .iter()
            .all(|t| t.history.len() + 1 >= 2 && t.history[0].elapsed_days == 0.0)
    );
    assert_eq!(data.train_card_ids.len(), data.train_items.len());
    let mut one = context();
    one.members.retain(|m| m.deck_id == "A");
    let scoped = build_dataset(&rows, one).unwrap();
    assert_eq!(scoped.eligibility.usable_observations, 200);
    assert!(!scoped.eligibility.eligible);
    assert!(scoped.holdout_targets.iter().all(|t| t.deck_id == "A"));
    let mut filtered = rows.clone();
    filtered.last_mut().unwrap().undone = true;
    assert_eq!(
        build_dataset(&filtered, context())
            .unwrap()
            .eligibility
            .usable_observations,
        399
    );
    let mut shortened = context();
    shortened.config.max_seq_len = 32;
    let long_rows = (0..40)
        .map(|n| TrainingObservation {
            card_id: "single".into(),
            deck_id: "A".into(),
            reviewed_at: ("2026-01-01T00:00:00Z"
                .parse::<chrono::DateTime<chrono::Utc>>()
                .unwrap()
                + chrono::Duration::hours(n * 12))
            .to_rfc3339(),
            rating: openrecall_core::scheduling::fsrs::Rating::Good,
            undone: false,
            deleted: false,
        })
        .collect::<Vec<_>>();
    let fractional = build_dataset(&long_rows, shortened).unwrap();
    assert_eq!(fractional.eligibility.usable_observations, 31);
    assert_eq!(fractional.train_items[0].reviews[1].delta_t, 0.5);
}
#[test]
fn context_token_ignores_labels_but_tracks_membership_data_and_settings() {
    let rows = observations();
    let base = build_dataset(&rows, context()).unwrap().revision_token;
    let mut renamed = context();
    renamed.members[0].deck_name = "new label".into();
    assert_eq!(build_dataset(&rows, renamed).unwrap().revision_token, base);
    for field in ["data", "membership", "timezone", "penalties", "model"] {
        let mut c = context();
        match field {
            "data" => c.members[0].data_revision = "changed".into(),
            "membership" => {
                c.members.pop();
            }
            "timezone" => c.timezone = "Asia/Tokyo".into(),
            "penalties" => c.config.schedule_penalties = false,
            _ => c.model_revision = "new-model".into(),
        };
        assert_ne!(build_dataset(&rows, c).unwrap().revision_token, base);
    }
}
#[test]
fn api_counts_only_surviving_nonfirst_reviews_for_the_selected_profile() {
    use serde_json::json;
    let mut f = common::Fixture::new();
    let deck = f.deck("eligible");
    f.card(&deck, "q", "a");
    let d = f.call("getDeck", json!({"deckId":deck}));
    let empty = f.call(
        "getTrainingEligibility",
        json!({"profileId":d["profileId"]}),
    );
    assert_eq!(empty["usableObservations"], 0);
    assert_eq!(empty["eligible"], false);
    let mut last_sid = serde_json::Value::Null;
    for rating in ["again", "good"] {
        let start = f.call("startSession", json!({"deckId":deck}));
        let sid = start["payload"]["sessionId"].clone();
        let revealed = f.call("revealAnswer", json!({"sessionId":sid}));
        let frame = &revealed["payload"];
        f.call("rateCard",json!({"sessionId":sid,"presentationId":frame["presentationId"],"rating":rating,"actionId":uuid::Uuid::new_v4().to_string(),"expectedCardRevision":frame["cardRevision"]}));
        last_sid = sid;
        f.advance(61);
    }
    let one = f.call(
        "getTrainingEligibility",
        json!({"profileId":d["profileId"]}),
    );
    assert_eq!(one["usableObservations"], 1);
    assert_eq!(one["distinctCards"], 1);
    f.call("undoReview", json!({"sessionId":last_sid}));
    assert_eq!(
        f.call(
            "getTrainingEligibility",
            json!({"profileId":d["profileId"]})
        )["usableObservations"],
        0
    );
}
