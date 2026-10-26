use crate::{
    AppError, AppResult,
    contracts::training::TrainingEligibility,
    domain::profile::TrainingConfig,
    scheduling::fsrs::{Rating, ReviewObservation},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingObservation {
    pub card_id: String,
    pub deck_id: String,
    pub reviewed_at: String,
    pub rating: Rating,
    pub undone: bool,
    pub deleted: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingMember {
    pub deck_id: String,
    pub deck_name: String,
    pub data_revision: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingContext {
    pub profile_id: String,
    pub members: Vec<TrainingMember>,
    pub config: TrainingConfig,
    pub timezone: String,
    pub day_start_minute: u16,
    pub baseline_parameters: Vec<f32>,
    pub model_revision: String,
}
#[derive(Clone, Debug)]
pub struct HeldOutTarget {
    pub card_id: String,
    pub deck_id: String,
    pub at: String,
    pub history: Vec<ReviewObservation>,
    pub rating: Rating,
    pub elapsed_days: f32,
}
#[derive(Clone, Debug)]
pub struct TrainingDataset {
    pub train_items: Vec<fsrs::FSRSItem>,
    pub train_card_ids: Vec<i64>,
    pub train_target_times: Vec<String>,
    pub holdout_targets: Vec<HeldOutTarget>,
    pub cutoff_at: Option<String>,
    pub eligibility: TrainingEligibility,
    pub context: TrainingContext,
    pub revision_token: String,
}
pub fn build_dataset(
    history: &[TrainingObservation],
    context: TrainingContext,
) -> AppResult<TrainingDataset> {
    use crate::contracts::training::{DeckContribution, TrainingEligibility};
    use chrono::{DateTime, Utc};
    use sha2::{Digest, Sha256};
    use std::collections::{BTreeMap, HashSet};
    context.config.validate()?;
    crate::scheduling::fsrs::FsrsScheduler::new(&context.baseline_parameters)?;
    let timezone: chrono_tz::Tz = context
        .timezone
        .parse()
        .map_err(|_| AppError::new("invalidTimezone"))?;
    if context.day_start_minute > 1439 {
        return Err(AppError::new("invalidStudyDay"));
    }
    let members = context
        .members
        .iter()
        .map(|m| m.deck_id.clone())
        .collect::<HashSet<_>>();
    if members.len() != context.members.len() {
        return Err(AppError::new("invalidMembership"));
    }
    type CardKey = (String, String);
    type TimedRating = (DateTime<Utc>, Rating);
    let mut groups: BTreeMap<CardKey, Vec<TimedRating>> = BTreeMap::new();
    for row in history
        .iter()
        .filter(|r| !r.undone && !r.deleted && members.contains(&r.deck_id))
    {
        let at = row
            .reviewed_at
            .parse()
            .map_err(|_| AppError::new("invalidHistory"))?;
        groups
            .entry((row.deck_id.clone(), row.card_id.clone()))
            .or_default()
            .push((at, row.rating));
    }
    struct Point {
        at: DateTime<Utc>,
        target: HeldOutTarget,
        prefix: Vec<fsrs::FSRSReview>,
        numeric_id: i64,
    }
    let mut points = vec![];
    for (numeric_id, ((deck, card), mut events)) in groups.into_iter().enumerate() {
        events.sort_by_key(|(at, _)| *at);
        let mut sequence = vec![];
        for (index, (at, rating)) in events.iter().enumerate() {
            let delta = if index == 0 {
                0.0
            } else {
                crate::domain::time::elapsed_days(events[index - 1].0, *at)
            };
            if index > 0 && index < context.config.max_seq_len as usize {
                let target = HeldOutTarget {
                    card_id: card.clone(),
                    deck_id: deck.clone(),
                    at: at.to_rfc3339(),
                    history: sequence.clone(),
                    rating: *rating,
                    elapsed_days: delta,
                };
                let mut prefix = sequence
                    .iter()
                    .map(|r: &ReviewObservation| fsrs::FSRSReview {
                        rating: r.rating.number(),
                        delta_t: r.elapsed_days,
                    })
                    .collect::<Vec<_>>();
                prefix.push(fsrs::FSRSReview {
                    rating: rating.number(),
                    delta_t: delta,
                });
                points.push(Point {
                    at: *at,
                    target,
                    prefix,
                    numeric_id: numeric_id as i64,
                });
            }
            sequence.push(ReviewObservation {
                rating: *rating,
                elapsed_days: delta,
            });
        }
    }
    points.sort_by(|a, b| {
        (&a.at, &a.target.deck_id, &a.target.card_id).cmp(&(
            &b.at,
            &b.target.deck_id,
            &b.target.card_id,
        ))
    });
    let cutoff = points.get(points.len() * 4 / 5).map(|p| p.at);
    let mut train_items = vec![];
    let mut train_card_ids = vec![];
    let mut train_target_times = vec![];
    let mut holdout_targets = vec![];
    let mut cards = HashSet::new();
    let mut days = HashSet::new();
    let mut again = 0;
    let mut contributions: BTreeMap<String, (u32, HashSet<String>)> = BTreeMap::new();
    let total = points.len() as u32;
    for point in points {
        cards.insert((point.target.deck_id.clone(), point.target.card_id.clone()));
        days.insert(crate::domain::time::study_date(
            point.at,
            timezone,
            context.day_start_minute,
        ));
        if point.target.rating == Rating::Again {
            again += 1;
        }
        let entry = contributions
            .entry(point.target.deck_id.clone())
            .or_default();
        entry.0 += 1;
        entry.1.insert(point.target.card_id.clone());
        if cutoff.is_some_and(|cut| point.at < cut) {
            train_items.push(fsrs::FSRSItem {
                reviews: point.prefix,
            });
            train_card_ids.push(point.numeric_id);
            train_target_times.push(point.at.to_rfc3339());
        } else {
            holdout_targets.push(point.target);
        }
    }
    let holdout_cards = holdout_targets
        .iter()
        .map(|t| (&t.deck_id, &t.card_id))
        .collect::<HashSet<_>>()
        .len() as u32;
    let contribution_rows = context
        .members
        .iter()
        .map(|m| {
            let counts = contributions.get(&m.deck_id);
            DeckContribution {
                deck_id: m.deck_id.clone(),
                deck_name: m.deck_name.clone(),
                observations: counts.map_or(0, |c| c.0),
                cards: counts.map_or(0, |c| c.1.len() as u32),
            }
        })
        .collect();
    let eligibility = super::eligibility::assess(TrainingEligibility {
        eligible: false,
        usable_observations: total,
        distinct_cards: cards.len() as u32,
        study_days: days.len() as u32,
        again_count: again,
        recalled_count: total - again,
        train_count: train_items.len() as u32,
        holdout_count: holdout_targets.len() as u32,
        holdout_cards,
        unmet_conditions: vec![],
        contributions: contribution_rows,
    });
    let mut revisions = context
        .members
        .iter()
        .map(|m| (&m.deck_id, &m.data_revision))
        .collect::<Vec<_>>();
    revisions.sort();
    let source = serde_json::to_vec(&serde_json::json!([
        context.profile_id,
        context.model_revision,
        context.baseline_parameters,
        context.config,
        context.timezone,
        context.day_start_minute,
        revisions
    ]))?;
    let revision_token = Sha256::digest(source)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    Ok(TrainingDataset {
        train_items,
        train_card_ids,
        train_target_times,
        holdout_targets,
        cutoff_at: cutoff.map(|t| t.to_rfc3339()),
        eligibility,
        context,
        revision_token,
    })
}
