use openrecall_core::{
    domain::profile::TrainingConfig, scheduling::fsrs::Rating, training::dataset::*,
};
pub fn context() -> TrainingContext {
    TrainingContext {
        profile_id: "profile".into(),
        members: vec![
            TrainingMember {
                deck_id: "A".into(),
                deck_name: "A".into(),
                data_revision: "a-revision".into(),
            },
            TrainingMember {
                deck_id: "B".into(),
                deck_name: "B".into(),
                data_revision: "b-revision".into(),
            },
        ],
        config: TrainingConfig::default(),
        timezone: "Africa/Cairo".into(),
        day_start_minute: 60,
        baseline_parameters: fsrs::DEFAULT_PARAMETERS.to_vec(),
        model_revision: "baseline".into(),
    }
}
pub fn observations() -> Vec<TrainingObservation> {
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/training_histories.json")).unwrap();
    let base: chrono::DateTime<chrono::Utc> = config["base"].as_str().unwrap().parse().unwrap();
    let mut rows = vec![];
    for card in 0..config["cards"].as_u64().unwrap() {
        for review in 0..=config["targetsPerCard"].as_u64().unwrap() {
            let at = base
                + chrono::Duration::days(review as i64)
                + chrono::Duration::minutes(card as i64);
            let deck = if card < 25 { "A" } else { "B" };
            rows.push(TrainingObservation {
                card_id: format!("{deck}-{card}"),
                deck_id: deck.into(),
                reviewed_at: at.to_rfc3339(),
                rating: if review > 0 && (card + review) % 8 == 0 {
                    Rating::Again
                } else {
                    Rating::Good
                },
                undone: false,
                deleted: false,
            });
        }
    }
    rows
}
