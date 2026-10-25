mod common;
#[test]
fn restart_preparation_persists_completed_background_results_before_reopen() {
    let mut f = common::Fixture::new();
    let (profile, _, _) = common::training::seed_profile(&mut f);
    let job = f.call("startTraining", serde_json::json!({"profileId":profile}));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        match f.app.prepare_for_restart() {
            Ok(()) => break,
            Err(e) => {
                assert_eq!(e.code, "trainingInProgress");
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
    }
    let path = f.dir.path().join("test.sqlite3");
    drop(f.app);
    let mut reopened = openrecall_core::App::open(
        &path,
        f.clock.clone(),
        Box::new(openrecall_core::SystemRandom::default()),
    )
    .unwrap();
    let persisted = reopened
        .execute_json(serde_json::json!({"kind":"getTrainingJob","payload":{"jobId":job["id"]}}))
        .unwrap();
    assert_eq!(persisted["payload"]["status"], "ready");
    assert!(persisted["payload"]["candidateParameters"].is_array());
}
