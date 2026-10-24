use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
pub fn study_date(at: DateTime<Utc>, zone: Tz, start_minute: u16) -> NaiveDate {
    (at.with_timezone(&zone).naive_local() - chrono::Duration::minutes(start_minute as i64)).date()
}
pub fn elapsed_days(previous: DateTime<Utc>, now: DateTime<Utc>) -> f32 {
    ((now - previous).num_milliseconds().max(0) as f64 / 86_400_000.0) as f32
}
