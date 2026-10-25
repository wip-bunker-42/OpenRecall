use openrecall_core::{AppError, contracts::updates::UpdateStatus};
use openrecall_desktop_lib::updates::{Candidate, StatusSink, UpdateManager};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
fn sink() -> StatusSink {
    Arc::new(|_| {})
}
#[tokio::test]
async fn network_is_explicit_failures_are_retryable_and_only_verified_bytes_become_ready() {
    let manager = Arc::new(UpdateManager::<()>::new("0.1.0"));
    let calls = AtomicUsize::new(0);
    assert_eq!(manager.status().unwrap().phase, "idle");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let error = manager
        .check(
            || async {
                calls.fetch_add(1, Ordering::SeqCst);
                Err(AppError::new("updateCheckFailed"))
            },
            sink(),
        )
        .await
        .unwrap_err();
    assert_eq!(error.code, "updateCheckFailed");
    assert_eq!(manager.status().unwrap().phase, "failed");
    assert_eq!(
        manager
            .check(|| async { Ok(None) }, sink())
            .await
            .unwrap()
            .phase,
        "upToDate"
    );
    manager
        .check(
            || async {
                Ok(Some(Candidate {
                    version: "0.2.0".into(),
                    notes: Some("release".into()),
                    value: (),
                }))
            },
            sink(),
        )
        .await
        .unwrap();
    let events = Arc::new(Mutex::new(Vec::<UpdateStatus>::new()));
    let captured = events.clone();
    let emit: StatusSink = Arc::new(move |s| captured.lock().unwrap().push(s));
    let bad = manager
        .download(
            |_, progress| async move {
                progress(2, Some(4));
                progress(2, Some(4));
                Ok(vec![1, 2, 3, 4])
            },
            |_, _| Err(AppError::new("updateVerificationFailed")),
            emit,
        )
        .await
        .unwrap_err();
    assert_eq!(bad.code, "updateVerificationFailed");
    assert_eq!(manager.status().unwrap().phase, "failed");
    assert!(
        manager
            .install(|| Ok(()), |_, _| panic!("unverified bytes"))
            .is_err()
    );
    assert!(
        events
            .lock()
            .unwrap()
            .iter()
            .any(|s| s.downloaded_bytes == 4 && s.total_bytes == Some(4))
    );
    assert_eq!(
        manager
            .download(
                |_, progress| async move {
                    progress(4, Some(4));
                    Ok(vec![1, 2, 3, 4])
                },
                |_, _| Ok(()),
                sink()
            )
            .await
            .unwrap()
            .phase,
        "ready"
    );
    let installed = AtomicUsize::new(0);
    assert_eq!(
        manager
            .install(
                || Err(AppError::new("unsavedChanges")),
                |_, _| {
                    installed.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            )
            .unwrap_err()
            .code,
        "unsavedChanges"
    );
    assert_eq!(installed.load(Ordering::SeqCst), 0);
    assert_eq!(manager.status().unwrap().phase, "ready");
    let durable = AtomicUsize::new(0);
    manager
        .install(
            || {
                durable.store(1, Ordering::SeqCst);
                Ok(())
            },
            |_, bytes| {
                assert_eq!(durable.load(Ordering::SeqCst), 1);
                assert_eq!(bytes, vec![1, 2, 3, 4]);
                installed.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(installed.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn concurrent_update_operations_and_install_without_a_download_are_rejected() {
    let manager = UpdateManager::<()>::new("0.1.0");
    assert_eq!(
        manager
            .install(|| panic!(), |_, _| panic!())
            .unwrap_err()
            .code,
        "updateNotReady"
    );
    manager
        .check(
            || async {
                Ok(Some(Candidate {
                    version: "0.2.0".into(),
                    notes: None,
                    value: (),
                }))
            },
            sink(),
        )
        .await
        .unwrap();
    let result = manager
        .download(
            |_, _| async {
                assert_eq!(
                    manager
                        .check(|| async { Ok(None) }, sink())
                        .await
                        .unwrap_err()
                        .code,
                    "updateBusy"
                );
                assert_eq!(
                    manager
                        .install(|| panic!(), |_, _| panic!())
                        .unwrap_err()
                        .code,
                    "updateBusy"
                );
                Err(AppError::new("updateDownloadFailed"))
            },
            |_, _| Ok(()),
            sink(),
        )
        .await;
    assert_eq!(result.unwrap_err().code, "updateDownloadFailed");
    manager
        .download(|_, _| async { Ok(vec![1]) }, |_, _| Ok(()), sink())
        .await
        .unwrap();
}
#[test]
fn native_install_lock_blocks_replacement_until_failure_releases_it() {
    let lifecycle = openrecall_desktop_lib::lifecycle::Lifecycle::default();
    lifecycle.ensure_available().unwrap();
    lifecycle.begin_install().unwrap();
    assert_eq!(
        lifecycle.ensure_available().unwrap_err().code,
        "updateInProgress"
    );
    assert!(lifecycle.begin_install().is_err());
    lifecycle.finish_install();
    lifecycle.ensure_available().unwrap();
}
