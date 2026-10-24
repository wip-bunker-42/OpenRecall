use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
pub enum ReviewFilter {
    #[default]
    All,
    New,
    PreviouslyReviewed,
}
#[derive(Clone, Debug)]
pub struct QueueEntry {
    pub card_id: String,
    pub due_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub review_count: u32,
}
impl ReviewFilter {
    pub fn matches(self, count: u32) -> bool {
        match self {
            Self::All => true,
            Self::New => count == 0,
            Self::PreviouslyReviewed => count > 0,
        }
    }
    pub(crate) fn sql(self) -> &'static str {
        match self {
            Self::All => "1=1",
            Self::New => "review_count=0",
            Self::PreviouslyReviewed => "review_count>0",
        }
    }
}
pub fn select_due(
    entries: &[QueueEntry],
    filter: ReviewFilter,
    now: DateTime<Utc>,
) -> Vec<QueueEntry> {
    let mut due = entries
        .iter()
        .filter(|e| e.due_at <= now && filter.matches(e.review_count))
        .cloned()
        .collect::<Vec<_>>();
    due.sort_by(|a, b| {
        (&a.due_at, &a.created_at, &a.card_id).cmp(&(&b.due_at, &b.created_at, &b.card_id))
    });
    due
}
