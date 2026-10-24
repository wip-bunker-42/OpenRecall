use crate::{AppError, AppResult, domain::card::MemoryState};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
pub enum Rating {
    Again,
    Hard,
    Good,
    Easy,
}
impl Rating {
    pub fn number(self) -> u32 {
        match self {
            Self::Again => 1,
            Self::Hard => 2,
            Self::Good => 3,
            Self::Easy => 4,
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SchedulePolicy {
    pub desired_retention: f64,
    pub min_interval_seconds: u32,
    pub max_interval_days: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    pub memory: MemoryState,
    pub interval_seconds: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct RatingPreviews {
    pub again: Outcome,
    pub hard: Outcome,
    pub good: Outcome,
    pub easy: Outcome,
}
impl RatingPreviews {
    pub fn chosen(&self, rating: Rating) -> &Outcome {
        match rating {
            Rating::Again => &self.again,
            Rating::Hard => &self.hard,
            Rating::Good => &self.good,
            Rating::Easy => &self.easy,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ReviewObservation {
    pub rating: Rating,
    pub elapsed_days: f32,
}
pub struct FsrsScheduler {
    pub(crate) engine: ::fsrs::FSRS,
}
impl FsrsScheduler {
    pub fn new(parameters: &[f32]) -> AppResult<Self> {
        // The pinned upstream model explicitly maps 34 parameters (or defaults)
        // to FSRS-7. Its version() accessor is intentionally crate-private.
        if (!parameters.is_empty() && parameters.len() != 34)
            || parameters.iter().any(|x| !x.is_finite())
            || ::fsrs::DEFAULT_PARAMETERS.len() != 34
        {
            return Err(AppError::new("invalidParameters"));
        }
        Ok(Self {
            engine: ::fsrs::FSRS::new(parameters)
                .map_err(|_| AppError::new("invalidParameters"))?,
        })
    }
    pub fn preview(
        &self,
        memory: Option<MemoryState>,
        elapsed: f32,
        policy: SchedulePolicy,
    ) -> AppResult<RatingPreviews> {
        if !elapsed.is_finite()
            || elapsed < 0.0
            || !policy.desired_retention.is_finite()
            || !(0.70..=0.99).contains(&policy.desired_retention)
            || !(10..=3600).contains(&policy.min_interval_seconds)
            || !(1..=36500).contains(&policy.max_interval_days)
        {
            return Err(AppError::new("invalidSchedule"));
        }
        if memory.as_ref().is_some_and(|m| !valid_memory(m)) {
            return Err(AppError::new("invalidMemory"));
        }
        let state = memory.map(|m| ::fsrs::MemoryState {
            stability: m.stability,
            stability_fast: m.stability_fast,
            difficulty: m.difficulty,
        });
        let next = self
            .engine
            .next_states_with_elapsed_days(state, policy.desired_retention as f32, elapsed)
            .map_err(|_| AppError::new("invalidSchedule"))?;
        let outcome = |item: ::fsrs::ItemState| -> AppResult<Outcome> {
            let memory = from_upstream(item.memory);
            if !valid_memory(&memory) || !item.interval.is_finite() {
                return Err(AppError::new("invalidSchedule"));
            }
            Ok(Outcome {
                memory,
                interval_seconds: (item.interval as f64 * 86400.0).clamp(
                    policy.min_interval_seconds as f64,
                    policy.max_interval_days as f64 * 86400.0,
                ),
            })
        };
        Ok(RatingPreviews {
            again: outcome(next.again)?,
            hard: outcome(next.hard)?,
            good: outcome(next.good)?,
            easy: outcome(next.easy)?,
        })
    }
    pub fn replay(&self, history: &[ReviewObservation]) -> AppResult<Option<MemoryState>> {
        if history.is_empty() {
            return Ok(None);
        }
        if history[0].elapsed_days != 0.0
            || history
                .iter()
                .any(|r| !r.elapsed_days.is_finite() || r.elapsed_days < 0.0)
        {
            return Err(AppError::new("invalidHistory"));
        }
        let item = ::fsrs::FSRSItem {
            reviews: history
                .iter()
                .map(|r| ::fsrs::FSRSReview {
                    rating: r.rating.number(),
                    delta_t: r.elapsed_days,
                })
                .collect(),
        };
        let memory = from_upstream(
            self.engine
                .memory_state(item, None)
                .map_err(|_| AppError::new("invalidHistory"))?,
        );
        if !valid_memory(&memory) {
            return Err(AppError::new("invalidMemory"));
        }
        Ok(Some(memory))
    }
}
pub fn valid_memory(m: &MemoryState) -> bool {
    m.stability.is_finite()
        && m.stability_fast.is_finite()
        && m.difficulty.is_finite()
        && m.stability > 0.0
        && m.stability <= 36500.0
        && m.stability_fast > 0.0
        && m.stability_fast <= 36500.0
        && (1.0..=10.0).contains(&m.difficulty)
}
fn from_upstream(m: ::fsrs::MemoryState) -> MemoryState {
    MemoryState {
        stability: m.stability,
        stability_fast: m.stability_fast,
        difficulty: m.difficulty,
    }
}
