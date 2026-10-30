use crate::{
    AppError, AppResult,
    domain::{review::ReviewLog, time::study_date},
};
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::BTreeMap;
/// One result per logical card and study day. Equal instants retain the later
/// input entry, so callers must preserve database insertion order for ties.
pub fn daily_completions(
    logs: &[ReviewLog],
    zone: chrono_tz::Tz,
    start_minute: u16,
) -> AppResult<BTreeMap<NaiveDate, u32>> {
    let mut latest = BTreeMap::<(NaiveDate, &str), (DateTime<Utc>, bool)>::new();
    for log in logs {
        let at: DateTime<Utc> = log
            .reviewed_at
            .parse()
            .map_err(|_| AppError::new("invalidData"))?;
        let due: DateTime<Utc> = log
            .next_due_at
            .parse()
            .map_err(|_| AppError::new("invalidData"))?;
        let day = study_date(at, zone, start_minute);
        let key = (day, log.card_id.as_str());
        if latest.get(&key).is_none_or(|(previous, _)| at >= *previous) {
            latest.insert(key, (at, study_date(due, zone, start_minute) > day));
        }
    }
    let mut counts = BTreeMap::new();
    for ((day, _), (_, completed)) in latest {
        *counts.entry(day).or_default() += u32::from(completed);
    }
    Ok(counts)
}
