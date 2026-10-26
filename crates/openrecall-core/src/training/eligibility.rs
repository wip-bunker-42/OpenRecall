use crate::contracts::training::TrainingEligibility;
pub fn assess(mut counts: TrainingEligibility) -> TrainingEligibility {
    counts.unmet_conditions.clear();
    for (name, valid) in [
        ("observations", counts.usable_observations >= 400),
        ("cards", counts.distinct_cards >= 50),
        ("days", counts.study_days >= 7),
        ("again", counts.again_count >= 5),
        ("recalled", counts.recalled_count >= 5),
        ("train", counts.train_count >= 64),
        ("holdout", counts.holdout_count >= 80),
        ("holdoutCards", counts.holdout_cards >= 10),
    ] {
        if !valid {
            counts.unmet_conditions.push(name.into());
        }
    }
    counts.eligible = counts.unmet_conditions.is_empty();
    counts
}
