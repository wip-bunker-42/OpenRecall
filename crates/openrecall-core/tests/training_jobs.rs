mod common;
use common::training::*;
use openrecall_core::{
    contracts::training::EvaluationMetrics,
    scheduling::fsrs::{Rating, ReviewObservation},
    training::{dataset::*, evaluate::*, runner::train_snapshot},
};
#[test]
fn heldout_metrics_use_only_prior_history_and_recall_outcome() {
    let targets = [Rating::Again, Rating::Good]
        .into_iter()
        .map(|rating| HeldOutTarget {
            card_id: "card".into(),
            deck_id: "A".into(),
            at: "2026-01-10T00:00:00Z".into(),
            history: vec![ReviewObservation {
                rating: Rating::Good,
                elapsed_days: 0.0,
            }],
            rating,
            elapsed_days: 4.7777247,
        })
        .collect::<Vec<_>>();
    let metrics = score_targets(&fsrs::DEFAULT_PARAMETERS, &targets).unwrap();
    assert!((metrics.log_loss - 1.2039728).abs() < 0.0001);
    assert!((metrics.rmse - 0.6403124).abs() < 0.0001);
    assert!(score_targets(&fsrs::DEFAULT_PARAMETERS, &[]).is_err());
}
#[test]
fn candidate_acceptance_obeys_both_metric_tolerances() {
    let base = EvaluationMetrics {
        log_loss: 0.5,
        rmse: 0.3,
    };
    assert!(!candidate_acceptable(
        base,
        EvaluationMetrics {
            log_loss: 0.49995,
            rmse: 0.3
        }
    ));
    assert!(candidate_acceptable(
        base,
        EvaluationMetrics {
            log_loss: 0.4999,
            rmse: 0.3001
        }
    ));
    assert!(!candidate_acceptable(
        base,
        EvaluationMetrics {
            log_loss: 0.49,
            rmse: 0.3002
        }
    ));
    assert!(!candidate_acceptable(
        base,
        EvaluationMetrics {
            log_loss: f64::NAN,
            rmse: 0.2
        }
    ));
}
#[test]
fn real_fsrs7_training_is_finite_repeatable_and_cancellable() {
    let data = build_dataset(&observations(), context()).unwrap();
    let config = data.context.config.clone();
    let progress = fsrs::CombinedProgressState::new_shared();
    let trained = train_snapshot(data.clone(), config.clone(), progress.clone()).unwrap();
    assert_eq!(trained.parameters.len(), 34);
    assert!(trained.parameters.iter().all(|p| p.is_finite()));
    assert!(trained.baseline.log_loss.is_finite());
    assert!(trained.candidate.log_loss.is_finite());
    assert!(!progress.lock().unwrap().splits.is_empty());
    println!(
        "real FSRS-7: baseline log loss {:.6}, candidate {:.6}, RMSE {:.6}, accepted {}",
        trained.baseline.log_loss,
        trained.candidate.log_loss,
        trained.candidate.rmse,
        trained.acceptable
    );
    let repeat = train_snapshot(
        data.clone(),
        config.clone(),
        fsrs::CombinedProgressState::new_shared(),
    )
    .unwrap();
    assert!((trained.candidate.log_loss - repeat.candidate.log_loss).abs() < 0.00001);
    let cancelled = fsrs::CombinedProgressState::new_shared();
    cancelled.lock().unwrap().want_abort = true;
    assert_eq!(
        train_snapshot(data, config, cancelled).unwrap_err().code,
        "cancelled"
    );
    let empty = build_dataset(&[], context()).unwrap();
    let config = empty.context.config.clone();
    assert_eq!(
        train_snapshot(empty, config, fsrs::CombinedProgressState::new_shared())
            .unwrap_err()
            .code,
        "insufficientData"
    );
}
fn wait_job(f: &mut common::Fixture, id: &str) -> serde_json::Value {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let job = f.call("getTrainingJob", serde_json::json!({"jobId":id}));
        if !["queued", "running"].contains(&job["status"].as_str().unwrap()) {
            return job;
        }
        assert!(std::time::Instant::now() < deadline, "training timed out");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
#[test]
fn background_jobs_apply_only_to_linked_decks_and_preserve_due_instants() {
    use serde_json::json;
    let mut f = common::Fixture::new();
    let (profile, decks, cards) = seed_profile(&mut f);
    let other = f.deck("Unrelated");
    let other_deck = f.call("getDeck", json!({"deckId":other}));
    let other_before = f.call("getProfile", json!({"profileId":other_deck["profileId"]}));
    let before = f.call("getCard", json!({"cardId":cards[0]}));
    let started = f.call("startTraining", json!({"profileId":profile}));
    assert_eq!(started["config"]["schedulePenalties"], true);
    f.card(&other, "unrelated new data", "a");
    let job = wait_job(&mut f, started["id"].as_str().unwrap());
    assert_eq!(job["status"], "ready", "{job}");
    let p = f.call("getProfile", json!({"profileId":profile}));
    let applied = f.call(
        "applyTraining",
        json!({"jobId":job["id"],"expectedProfileRevision":p["revision"]}),
    );
    assert_ne!(applied["activeModelRevision"], p["activeModelRevision"]);
    assert_eq!(applied["memberDeckIds"].as_array().unwrap().len(), 2);
    let after = f.call("getCard", json!({"cardId":cards[0]}));
    assert_eq!(after["dueAt"], before["dueAt"]);
    assert_eq!(after["bag"], before["bag"]);
    assert_ne!(after["memory"], before["memory"]);
    assert_eq!(
        f.call("getProfile", json!({"profileId":other_deck["profileId"]})),
        other_before
    );
    assert_eq!(
        f.call("getDeck", json!({"deckId":decks[1]}))["profileId"],
        profile
    );
    assert_eq!(
        f.app
            .execute_json(
                json!({"kind":"startTraining","payload":{"profileId":other_deck["profileId"]}})
            )
            .unwrap_err()
            .code,
        "insufficientData"
    );
}
#[test]
fn candidates_with_changed_data_membership_config_or_baseline_cannot_apply() {
    use serde_json::json;
    for change in ["data", "membership", "config", "baseline"] {
        let mut f = common::Fixture::new();
        let (profile, decks, cards) = seed_profile(&mut f);
        let started = f.call("startTraining", json!({"profileId":profile}));
        let job = wait_job(&mut f, started["id"].as_str().unwrap());
        assert_eq!(job["status"], "ready");
        match change {
            "data" => {
                let card = f.call("getCard", json!({"cardId":cards[0]}));
                let mut forms = card["forms"].clone();
                forms[0]["notes"] = json!("new data");
                f.call("updateCard",json!({"cardId":cards[0],"draft":{"forms":forms},"expectedRevision":card["revision"]}));
            }
            "membership" => {
                let deck = f.call("getDeck", json!({"deckId":decks[1]}));
                f.call("detachProfile",json!({"deckId":decks[1],"newName":"detached","expectedDeckRevision":deck["revision"]}));
            }
            "config" => {
                let p = f.call("getProfile", json!({"profileId":profile}));
                let mut config = p["config"].clone();
                config["schedulePenalties"] = json!(false);
                f.call(
                    "saveTrainingConfig",
                    json!({"profileId":profile,"config":config,"expectedRevision":p["revision"]}),
                );
            }
            _ => {
                let p = f.call("getProfile", json!({"profileId":profile}));
                f.call(
                    "resetProfile",
                    json!({"profileId":profile,"expectedRevision":p["revision"]}),
                );
            }
        }
        let current = f.call("getProfile", json!({"profileId":profile}));
        assert_eq!(f.app.execute_json(json!({"kind":"applyTraining","payload":{"jobId":job["id"],"expectedProfileRevision":current["revision"]}})).unwrap_err().code,"staleCandidate","{change}");
        assert_eq!(f.call("getProfile", json!({"profileId":profile})), current);
    }
}
#[test]
fn cancellation_restart_and_failed_multi_deck_apply_preserve_active_models() {
    use serde_json::json;
    let mut f = common::Fixture::new();
    let (profile, decks, cards) = seed_profile(&mut f);
    let baseline = f.call("getProfile", json!({"profileId":profile}));
    let first = f.call("startTraining", json!({"profileId":profile}));
    f.call("cancelTraining", json!({"jobId":first["id"]}));
    let cancelled = wait_job(&mut f, first["id"].as_str().unwrap());
    assert_eq!(cancelled["status"], "cancelled");
    assert_eq!(f.call("getProfile", json!({"profileId":profile})), baseline);
    let second = f.call("startTraining", json!({"profileId":profile}));
    f.reopen();
    let interrupted = f.call("getTrainingJob", json!({"jobId":second["id"]}));
    assert_eq!(interrupted["status"], "interrupted");
    assert_eq!(f.call("getProfile", json!({"profileId":profile})), baseline);
    let third = f.call("startTraining", json!({"profileId":profile}));
    let ready = wait_job(&mut f, third["id"].as_str().unwrap());
    assert_eq!(ready["status"], "ready");
    let before_a = f.call("getCard", json!({"cardId":cards[0]}));
    let before_b = f.call("getCard", json!({"cardId":cards[25]}));
    let conn = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    conn.execute_batch(&format!("CREATE TRIGGER reject_model BEFORE UPDATE OF payload ON cards WHEN old.deck_id='{}' BEGIN SELECT RAISE(ABORT,'injected'); END",decks[1])).unwrap();
    assert!(f.app.execute_json(json!({"kind":"applyTraining","payload":{"jobId":ready["id"],"expectedProfileRevision":baseline["revision"]}})).is_err());
    assert_eq!(f.call("getProfile", json!({"profileId":profile})), baseline);
    assert_eq!(f.call("getCard", json!({"cardId":cards[0]})), before_a);
    assert_eq!(f.call("getCard", json!({"cardId":cards[25]})), before_b);
    conn.execute_batch("DROP TRIGGER reject_model").unwrap();
    f.call(
        "applyTraining",
        json!({"jobId":ready["id"],"expectedProfileRevision":baseline["revision"]}),
    );
    assert_ne!(
        f.call("getCard", json!({"cardId":cards[25]}))["memory"],
        before_b["memory"]
    );
}
#[test]
fn unchanged_upstream_defaults_are_not_reported_as_optimized() {
    let base: chrono::DateTime<chrono::Utc> = "2026-01-01T12:00:00Z".parse().unwrap();
    let mut rows = vec![];
    for card in 0..400 {
        let first = base + chrono::Duration::days(card / 50) + chrono::Duration::minutes(card % 50);
        for review in 0..2 {
            rows.push(TrainingObservation {
                card_id: format!("card-{card}"),
                deck_id: "A".into(),
                reviewed_at: (first + chrono::Duration::days(review)).to_rfc3339(),
                rating: if review == 1 && card % 20 == 0 {
                    Rating::Again
                } else {
                    Rating::Good
                },
                undone: false,
                deleted: false,
            });
        }
    }
    let data = build_dataset(&rows, context()).unwrap();
    assert!(data.eligibility.eligible);
    let config = data.context.config.clone();
    let result = train_snapshot(data, config, fsrs::CombinedProgressState::new_shared()).unwrap();
    assert_eq!(result.parameters, fsrs::DEFAULT_PARAMETERS);
    assert!(!result.changed);
    assert!(!result.acceptable);
}
