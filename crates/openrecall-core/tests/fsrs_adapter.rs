use openrecall_core::scheduling::fsrs::*;
fn policy() -> SchedulePolicy {
    SchedulePolicy {
        desired_retention: 0.9,
        min_interval_seconds: 60,
        max_interval_days: 36500,
    }
}
#[test]
fn intraday_previews_match_pinned_upstream_golden_values() {
    let scheduler = FsrsScheduler::new(&[]).unwrap();
    let previews = scheduler.preview(None, 0.0, policy()).unwrap();
    let golden: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/fsrs7_golden.json")).unwrap();
    for (name, rating) in [
        ("again", Rating::Again),
        ("hard", Rating::Hard),
        ("good", Rating::Good),
        ("easy", Rating::Easy),
    ] {
        let outcome = previews.chosen(rating);
        let g = &golden[name];
        let seconds = (g["days"].as_f64().unwrap() * 86400.0).max(60.0);
        assert!(
            (outcome.interval_seconds - seconds).abs() < 0.1,
            "{name}: {} != {seconds}",
            outcome.interval_seconds
        );
        assert!(
            (outcome.memory.stability_fast as f64 - g["stabilityFast"].as_f64().unwrap()).abs()
                < 0.00001
        );
    }
    let memory = previews.good.memory.clone();
    let fractional = scheduler
        .preview(Some(memory.clone()), 0.5, policy())
        .unwrap();
    let integer = scheduler.preview(Some(memory), 0.0, policy()).unwrap();
    assert_ne!(fractional.good.memory, integer.good.memory);
    let replay = scheduler
        .replay(&[
            ReviewObservation {
                rating: Rating::Good,
                elapsed_days: 0.0,
            },
            ReviewObservation {
                rating: Rating::Good,
                elapsed_days: 0.5,
            },
        ])
        .unwrap()
        .unwrap();
    assert_eq!(replay, fractional.good.memory);
    let capped = scheduler
        .preview(
            None,
            0.0,
            SchedulePolicy {
                max_interval_days: 1,
                ..policy()
            },
        )
        .unwrap();
    assert_eq!(capped.easy.interval_seconds, 86400.0);
}
#[test]
fn wrong_model_invalid_memory_and_invalid_policies_are_rejected() {
    assert!(FsrsScheduler::new(&fsrs::FSRS6_DEFAULT_PARAMETERS).is_err());
    assert!(FsrsScheduler::new(&[f32::NAN; 34]).is_err());
    assert!(FsrsScheduler::new(&[1.0; 5]).is_err());
    let s = FsrsScheduler::new(&[]).unwrap();
    assert!(s.preview(None, f32::NAN, policy()).is_err());
    assert!(
        s.preview(
            None,
            0.0,
            SchedulePolicy {
                desired_retention: 1.0,
                ..policy()
            }
        )
        .is_err()
    );
    assert!(
        s.preview(
            Some(openrecall_core::domain::card::MemoryState {
                stability: 1.0,
                stability_fast: f32::NAN,
                difficulty: 5.0
            }),
            1.0,
            policy()
        )
        .is_err()
    );
    assert!(s.replay(&[]).unwrap().is_none());
}
