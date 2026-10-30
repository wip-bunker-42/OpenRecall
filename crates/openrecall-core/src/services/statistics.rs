use crate::{
    App, AppResult,
    contracts::statistics::*,
    domain::{card::CardDetail, review::ReviewLog, time::study_date},
    statistics::completion::daily_completions,
};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use std::collections::{BTreeMap, HashSet};
fn period(logs: &[&ReviewLog], completed_for_day: u32) -> PeriodStatistics {
    let mut p = PeriodStatistics {
        completed_for_day,
        ..Default::default()
    };
    let mut cards = HashSet::new();
    for l in logs {
        p.attempts += 1;
        cards.insert(&l.card_id);
        p.active_ms = p.active_ms.saturating_add(l.response_ms);
        p.rating_counts[l.rating.number() as usize - 1] += 1;
    }
    p.unique_cards = cards.len() as u32;
    if p.attempts > 0 {
        p.mean_response_ms = Some(p.active_ms as f64 / p.attempts as f64);
        p.observed_recall_rate = Some((p.attempts - p.rating_counts[0]) as f64 / p.attempts as f64);
    }
    p
}
impl App {
    pub(crate) fn deck_statistics(&self, deck_id: &str) -> AppResult<DeckStatistics> {
        let deck = self.deck_detail(deck_id)?;
        let settings = self.store.settings()?;
        let zone = settings
            .timezone
            .parse()
            .map_err(|_| crate::AppError::new("invalidTimezone"))?;
        let now = self.clock.now();
        let today = study_date(now, zone, settings.day_start_minute);
        let cards: Vec<CardDetail> = self
            .store
            .conn
            .prepare("SELECT payload FROM cards WHERE deck_id=?1")?
            .query_map([deck_id], |r| r.get::<_, String>(0))?
            .map(|r| Ok(serde_json::from_str(&r?)?))
            .collect::<AppResult<_>>()?;
        let logs: Vec<ReviewLog> = self
            .store
            .conn
            .prepare("SELECT payload FROM reviews WHERE deck_id=?1 ORDER BY at_ms,rowid")?
            .query_map([deck_id], |r| r.get::<_, String>(0))?
            .map(|r| Ok(serde_json::from_str(&r?)?))
            .collect::<AppResult<_>>()?;
        let mut phases = PhaseCounts::default();
        let mut due_now = 0;
        let mut due_later_today = 0;
        let mut next_due_at: Option<DateTime<Utc>> = None;
        let mut forecast = BTreeMap::<NaiveDate, u32>::new();
        for c in &cards {
            match c.phase.as_str() {
                "new" => phases.new += 1,
                "learning" => phases.learning += 1,
                "relearning" => phases.relearning += 1,
                "review" => phases.review += 1,
                _ => {}
            }
            let due: DateTime<Utc> = c
                .due_at
                .parse()
                .map_err(|_| crate::AppError::new("invalidData"))?;
            let day = study_date(due, zone, settings.day_start_minute).max(today);
            *forecast.entry(day).or_default() += 1;
            if due <= now {
                due_now += 1;
            } else {
                if day == today {
                    due_later_today += 1;
                }
                next_due_at = Some(next_due_at.map_or(due, |n| n.min(due)));
            }
        }
        let completions = daily_completions(&logs, zone, settings.day_start_minute)?;
        let mut history = BTreeMap::<NaiveDate, Vec<&ReviewLog>>::new();
        for l in &logs {
            let at = l
                .reviewed_at
                .parse()
                .map_err(|_| crate::AppError::new("invalidData"))?;
            history
                .entry(study_date(at, zone, settings.day_start_minute))
                .or_default()
                .push(l);
        }
        let window = |days: i64| {
            let v = history
                .range((today - Duration::days(days - 1))..=today)
                .flat_map(|(_, v)| v.iter().copied())
                .collect::<Vec<_>>();
            let completed = completions
                .range((today - Duration::days(days - 1))..=today)
                .map(|(_, n)| n)
                .sum();
            period(&v, completed)
        };
        let forecasts = |days: i64| {
            (0..days)
                .map(|i| {
                    let date = today + Duration::days(i);
                    ForecastDay {
                        date: date.to_string(),
                        cards: *forecast.get(&date).unwrap_or(&0),
                    }
                })
                .collect()
        };
        let profile = self.get_profile(&deck.profile_id)?;
        let eligibility = self.profile_dataset(&deck.profile_id)?.eligibility;
        Ok(DeckStatistics {
            cards: cards.len() as u32,
            forms: cards.iter().map(|c| c.forms.len() as u32).sum(),
            phases,
            due_now,
            due_later_today,
            next_due_at: next_due_at.map(|d| d.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
            forecast_7_days: forecasts(7),
            forecast_30_days: forecasts(30),
            today: window(1),
            last_7_days: window(7),
            last_30_days: window(30),
            all_time: period(&logs.iter().collect::<Vec<_>>(), completions.values().sum()),
            daily_history: history
                .iter()
                .map(|(d, l)| HistoryDay {
                    date: d.to_string(),
                    statistics: period(l, *completions.get(d).unwrap_or(&0)),
                })
                .collect(),
            profile_status: ProfileStatus {
                id: profile.id,
                name: profile.name,
                model_origin: profile.model_origin,
                model_revision: profile.active_model_revision,
                eligibility,
            },
        })
    }
    pub fn library_statistics(&self) -> AppResult<LibraryStatistics> {
        let settings = self.store.settings()?;
        let zone = settings
            .timezone
            .parse()
            .map_err(|_| crate::AppError::new("invalidTimezone"))?;
        let today = study_date(self.clock.now(), zone, settings.day_start_minute);
        let logs: Vec<ReviewLog> = self
            .store
            .conn
            .prepare("SELECT payload FROM reviews ORDER BY at_ms,rowid")?
            .query_map([], |r| r.get::<_, String>(0))?
            .map(|r| Ok(serde_json::from_str(&r?)?))
            .collect::<AppResult<_>>()?;
        let spelling_decks: HashSet<String> = self
            .store
            .conn
            .prepare("SELECT id FROM decks WHERE kind='spelling'")?
            .query_map([], |r| r.get(0))?
            .collect::<Result<_, _>>()?;
        let spelling_logs = logs
            .iter()
            .filter(|log| spelling_decks.contains(&log.deck_id))
            .cloned()
            .collect::<Vec<_>>();
        let completions = daily_completions(&logs, zone, settings.day_start_minute)?;
        let spelling = daily_completions(&spelling_logs, zone, settings.day_start_minute)?;
        let mut history = BTreeMap::<NaiveDate, Vec<&ReviewLog>>::new();
        for log in &logs {
            let at = log
                .reviewed_at
                .parse()
                .map_err(|_| crate::AppError::new("invalidData"))?;
            history
                .entry(study_date(at, zone, settings.day_start_minute))
                .or_default()
                .push(log);
        }
        Ok(LibraryStatistics {
            today: period(
                history.get(&today).map(Vec::as_slice).unwrap_or(&[]),
                *completions.get(&today).unwrap_or(&0),
            ),
            daily_history: history
                .iter()
                .map(|(date, logs)| HistoryDay {
                    date: date.to_string(),
                    statistics: period(logs, *completions.get(date).unwrap_or(&0)),
                })
                .collect(),
            spelling_completed_today: *spelling.get(&today).unwrap_or(&0),
        })
    }
}
