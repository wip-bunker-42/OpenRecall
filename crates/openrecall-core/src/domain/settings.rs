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
