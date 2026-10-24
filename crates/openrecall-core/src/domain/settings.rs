use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub timezone: String,
    pub day_start_minute: u16,
    pub desired_retention: f64,
    pub min_interval_seconds: u32,
    pub max_interval_days: u32,
    pub shortcuts_enabled: bool,
    pub text_scale: f64,
    pub reduced_motion: String,
    pub page_size: u16,
    pub revision: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            timezone: "Africa/Cairo".into(),
            day_start_minute: 60,
            desired_retention: 0.9,
            min_interval_seconds: 60,
            max_interval_days: 36500,
            shortcuts_enabled: true,
            text_scale: 1.0,
            reduced_motion: "followSystem".into(),
            page_size: 20,
            revision: uuid::Uuid::new_v4().to_string(),
        }
    }
}
impl Settings {
    pub fn validate(&self) -> crate::AppResult<()> {
        use crate::AppError;
        if self.timezone.parse::<chrono_tz::Tz>().is_err() {
            return Err(AppError::field("timezone", "invalidTimezone"));
        }
        if self.day_start_minute > 1439 {
            return Err(AppError::field("dayStartMinute", "range"));
        }
        if !self.desired_retention.is_finite() || !(0.70..=0.99).contains(&self.desired_retention) {
            return Err(AppError::field("desiredRetention", "range"));
        }
        if !(10..=3600).contains(&self.min_interval_seconds) {
            return Err(AppError::field("minIntervalSeconds", "range"));
        }
        if !(1..=36500).contains(&self.max_interval_days) {
            return Err(AppError::field("maxIntervalDays", "range"));
        }
        if !self.text_scale.is_finite() || !(0.8..=2.0).contains(&self.text_scale) {
            return Err(AppError::field("textScale", "range"));
        }
        if ![10, 20, 50, 100].contains(&self.page_size) {
            return Err(AppError::field("pageSize", "range"));
        }
        if !["followSystem", "reduce"].contains(&self.reduced_motion.as_str()) {
            return Err(AppError::field("reducedMotion", "invalidOption"));
        }
        Ok(())
    }
}
