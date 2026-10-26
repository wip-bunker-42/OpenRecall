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
pub fn seed_profile(f: &mut super::Fixture) -> (String, Vec<String>, Vec<String>) {
    use openrecall_core::{
        domain::{card::CardDetail, review::ReviewLog, session::Session},
        scheduling::fsrs::{FsrsScheduler, SchedulePolicy},
    };
    use serde_json::json;
    use std::sync::atomic::Ordering;
    let base: chrono::DateTime<chrono::Utc> = "2026-01-01T12:00:00Z".parse().unwrap();
    f.clock.utc.store(base.timestamp_millis(), Ordering::SeqCst);
    let a = f.deck("Training A");
    let da = f.call("getDeck", json!({"deckId":a}));
    let b = f.call(
        "createDeck",
        json!({"name":"Training B","profileId":da["profileId"]}),
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let profile = da["profileId"].as_str().unwrap().to_owned();
    let model = f.call("getProfile", json!({"profileId":profile}))["activeModelRevision"]
        .as_str()
        .unwrap()
        .to_owned();
    let decks = vec![a, b];
    let mut sessions = vec![];
    for deck in &decks {
        sessions.push(
            f.call("startSession", json!({"deckId":deck}))["payload"]["sessionId"]
                .as_str()
                .unwrap()
                .to_owned(),
        );
    }
    let rows = observations();
    let scheduler = FsrsScheduler::new(&fsrs::DEFAULT_PARAMETERS).unwrap();
    let policy = SchedulePolicy {
        desired_retention: 0.9,
        min_interval_seconds: 60,
        max_interval_days: 36500,
    };
    let mut card_ids = vec![];
    let conn = rusqlite::Connection::open(f.dir.path().join("test.sqlite3")).unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=NORMAL")
        .unwrap();
    for n in 0..50 {
        let index = usize::from(n >= 25);
        let cid = f.card(&decks[index], &format!("Synthetic {n}"), "answer");
        let mut card: CardDetail =
            serde_json::from_value(f.call("getCard", json!({"cardId":cid}))).unwrap();
        let form_id = card.forms[0].id.clone();
        let source_id = format!("{}-{n}", if index == 0 { "A" } else { "B" });
        conn.execute_batch("BEGIN").unwrap();
        for (i, row) in rows.iter().filter(|r| r.card_id == source_id).enumerate() {
            let at: chrono::DateTime<chrono::Utc> = row.reviewed_at.parse().unwrap();
            let previous = card.clone();
            let elapsed = card.reviewed_at.as_ref().map_or(0.0, |p| {
                openrecall_core::domain::time::elapsed_days(p.parse().unwrap(), at)
            });
            let outcome = scheduler
                .preview(card.memory.clone(), elapsed, policy)
                .unwrap()
                .chosen(row.rating)
                .clone();
            card.memory = Some(outcome.memory.clone());
            card.reviewed_at = Some(at.to_rfc3339());
            card.due_at = (at
                + chrono::Duration::milliseconds(
                    (outcome.interval_seconds * 1000.0).round() as i64
                ))
            .to_rfc3339();
            if i == 0 {
                card.created_at = at.to_rfc3339();
            }
            card.review_count += 1;
            if row.rating == Rating::Again {
                card.phase = if card.has_graduated {
                    "relearning"
                } else {
                    "learning"
                }
                .into();
                if i > 0 {
                    card.lapse_count += 1;
                }
            } else if outcome.interval_seconds >= 86400.0 {
                card.has_graduated = true;
                card.phase = "review".into();
            } else if !card.has_graduated {
                card.phase = "learning".into();
            }
            card.bag.remaining.clear();
            card.bag.consumed = vec![form_id.clone()];
            card.bag.last_consumed = Some(form_id.clone());
            card.bag.reserved = None;
            card.bag.cycle = i as u32 + 1;
            let log = ReviewLog {
                id: uuid::Uuid::new_v4().to_string(),
                card_id: cid.clone(),
                deck_id: decks[index].clone(),
                session_id: sessions[index].clone(),
                form_id: form_id.clone(),
                rating: row.rating,
                reviewed_at: at.to_rfc3339(),
                elapsed_days: elapsed,
                response_ms: 1000,
                previous_memory: previous.memory,
                memory: outcome.memory,
                previous_due_at: previous.due_at,
                next_due_at: card.due_at.clone(),
                previous_phase: previous.phase,
                phase: card.phase.clone(),
                policy,
                model_revision: model.clone(),
                was_new: i == 0,
            };
            conn.execute("INSERT INTO reviews(id,card_id,deck_id,session_id,at_ms,model_revision,payload) VALUES(?1,?2,?3,?4,?5,?6,?7)",rusqlite::params![log.id,log.card_id,log.deck_id,log.session_id,at.timestamp_millis(),model,serde_json::to_string(&log).unwrap()]).unwrap();
        }
        let due: chrono::DateTime<chrono::Utc> = card.due_at.parse().unwrap();
        let created: chrono::DateTime<chrono::Utc> = card.created_at.parse().unwrap();
        conn.execute(
            "UPDATE cards SET payload=?1,due_ms=?2,created_ms=?3,review_count=?4 WHERE id=?5",
            rusqlite::params![
                serde_json::to_string(&card).unwrap(),
                due.timestamp_millis(),
                created.timestamp_millis(),
                card.review_count,
                cid
            ],
        )
        .unwrap();
        conn.execute_batch("COMMIT").unwrap();
        card_ids.push(card.id);
    }
    for sid in sessions {
        let raw: String = conn
            .query_row("SELECT payload FROM sessions WHERE id=?1", [&sid], |r| {
                r.get(0)
            })
            .unwrap();
        let mut session: Session = serde_json::from_str(&raw).unwrap();
        session.status = "ended".into();
        session.ended_at = Some((base + chrono::Duration::days(9)).to_rfc3339());
        session.current = None;
        let logs: Vec<ReviewLog> = conn
            .prepare("SELECT payload FROM reviews WHERE session_id=?1")
            .unwrap()
            .query_map([&sid], |r| r.get::<_, String>(0))
            .unwrap()
            .map(|r| serde_json::from_str(&r.unwrap()).unwrap())
            .collect();
        session.attempts = logs.len() as u32;
        session.response_ms = logs.iter().map(|l| l.response_ms).sum();
        session.active_ms = session.response_ms;
        for l in &logs {
            session.ratings[l.rating.number() as usize - 1] += 1;
            if !session.unique_card_ids.contains(&l.card_id) {
                session.unique_card_ids.push(l.card_id.clone());
            }
            if l.was_new && !session.new_card_ids.contains(&l.card_id) {
                session.new_card_ids.push(l.card_id.clone());
            }
        }
        conn.execute(
            "UPDATE sessions SET status='ended',payload=?1 WHERE id=?2",
            rusqlite::params![serde_json::to_string(&session).unwrap(), sid],
        )
        .unwrap();
    }
    f.clock.utc.store(
        (base + chrono::Duration::days(10)).timestamp_millis(),
        Ordering::SeqCst,
    );
    (profile, decks, card_ids)
}
