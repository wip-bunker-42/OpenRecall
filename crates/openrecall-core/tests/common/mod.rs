#![allow(dead_code)]
use chrono::{DateTime, Utc};
use openrecall_core::{App, Clock, RandomSource};
use rand::{SeedableRng, seq::SliceRandom};
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicI64, AtomicU64, Ordering},
};
pub struct FixedClock {
    pub utc: AtomicI64,
    pub mono: AtomicU64,
}
impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_millis(self.utc.load(Ordering::SeqCst)).unwrap()
    }
    fn monotonic_ms(&self) -> u64 {
        self.mono.load(Ordering::SeqCst)
    }
}
pub struct SeededRandom(rand::rngs::StdRng);
impl SeededRandom {
    pub fn new(seed: u64) -> Self {
        Self(rand::rngs::StdRng::seed_from_u64(seed))
    }
}
impl RandomSource for SeededRandom {
    fn shuffle(&mut self, ids: &mut [String]) {
        ids.shuffle(&mut self.0);
    }
}
pub struct Fixture {
    pub dir: tempfile::TempDir,
    pub clock: Arc<FixedClock>,
    pub app: App,
}
impl Fixture {
    pub fn deck(&mut self, name: &str) -> String {
        self.call("createDeck", json!({"name":name,"profileId":null}))["id"]
            .as_str()
            .unwrap()
            .into()
    }
    pub fn card(&mut self, deck: &str, front: &str, back: &str) -> String {
        self.call(
            "createCard",
            json!({"deckId":deck,"draft":{"forms":[{"front":front,"back":back,"notes":""}]}}),
        )["id"]
            .as_str()
            .unwrap()
            .into()
    }
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let clock = Arc::new(FixedClock {
            utc: AtomicI64::new(1792800000000),
            mono: AtomicU64::new(0),
        });
        let app = App::open(
            &dir.path().join("test.sqlite3"),
            clock.clone(),
            Box::new(SeededRandom::new(42)),
        )
        .unwrap();
        Self { dir, clock, app }
    }
    pub fn call(&mut self, kind: &str, payload: Value) -> Value {
        self.app
            .execute_json(json!({"kind":kind,"payload":payload}))
            .unwrap()["payload"]
            .clone()
    }
    pub fn advance(&self, seconds: i64) {
        self.clock.utc.fetch_add(seconds * 1000, Ordering::SeqCst);
        self.clock
            .mono
            .fetch_add(seconds.max(0) as u64 * 1000, Ordering::SeqCst);
    }
    pub fn reopen(&mut self) {
        self.app = App::open(
            &self.dir.path().join("test.sqlite3"),
            self.clock.clone(),
            Box::new(SeededRandom::new(42)),
        )
        .unwrap();
    }
}
