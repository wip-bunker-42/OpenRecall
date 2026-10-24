mod common;
use chrono::{DateTime, Utc};
use common::*;
use openrecall_core::domain::time::{elapsed_days, study_date};
use serde_json::json;
fn at(s: &str) -> DateTime<Utc> {
    s.parse().unwrap()
}
#[test]
fn cairo_day_boundary_and_dst_are_calendar_aware() {
    let cairo = chrono_tz::Africa::Cairo;
    for (instant, day) in [
        ("2026-10-23T21:59:00Z", "2026-10-23"),
        ("2026-10-23T22:00:00Z", "2026-10-24"),
        ("2026-04-23T21:59:00Z", "2026-04-23"),
        ("2026-04-23T22:00:00Z", "2026-04-24"),
        ("2026-10-29T22:59:00Z", "2026-10-29"),
        ("2026-10-29T23:00:00Z", "2026-10-30"),
    ] {
        assert_eq!(
            study_date(at(instant), cairo, 60).to_string(),
            day,
            "{instant}"
        );
    }
    assert_eq!(
        elapsed_days(at("2026-10-24T12:00:00Z"), at("2026-10-24T00:00:00Z")),
        0.0
    );
    assert_eq!(
        elapsed_days(at("2026-10-24T00:00:00Z"), at("2026-10-24T12:00:00Z")),
        0.5
    );
}
#[test]
fn settings_validate_all_limits_and_persist_with_revision_conflicts() {
    let mut f = Fixture::new();
    let original = f.call("bootstrap", json!({}))["settings"].clone();
    for (field, bad) in [
        ("timezone", json!("Missing/Zone")),
        ("dayStartMinute", json!(1440)),
        ("desiredRetention", json!(0.69)),
        ("desiredRetention", json!(1.0)),
        ("minIntervalSeconds", json!(9)),
        ("minIntervalSeconds", json!(3601)),
        ("maxIntervalDays", json!(0)),
        ("maxIntervalDays", json!(36501)),
        ("pageSize", json!(200)),
        ("textScale", json!(0.1)),
        ("reducedMotion", json!("unknown")),
    ] {
        let mut changed = original.clone();
        changed[field] = bad;
        assert!(f.app.execute_json(json!({"kind":"saveSettings","payload":{"settings":changed,"expectedRevision":original["revision"]}})).is_err());
        assert_eq!(f.call("bootstrap", json!({}))["settings"], original);
    }
    let mut changed = original.clone();
    changed["timezone"] = json!("Europe/London");
    changed["dayStartMinute"] = json!(180);
    let saved = f.call(
        "saveSettings",
        json!({"settings":changed,"expectedRevision":original["revision"]}),
    );
    assert_eq!(saved["timezone"], "Europe/London");
    assert_ne!(saved["revision"], original["revision"]);
    let stale = f.app.execute_json(json!({"kind":"saveSettings","payload":{"settings":original,"expectedRevision":original["revision"]}})).unwrap_err();
    assert_eq!(stale.code, "conflict");
    f.reopen();
    assert_eq!(f.call("bootstrap", json!({}))["settings"], saved);
}
