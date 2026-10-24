use chrono::{DateTime, Utc};
use rand::{SeedableRng, seq::SliceRandom};
use std::time::Instant;
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
    fn monotonic_ms(&self) -> u64;
}
pub trait RandomSource: Send {
    fn shuffle(&mut self, ids: &mut [String]);
}
pub struct SystemClock(Instant);
impl Default for SystemClock {
    fn default() -> Self {
        Self(Instant::now())
    }
}
impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
    fn monotonic_ms(&self) -> u64 {
        self.0.elapsed().as_millis().min(u64::MAX as u128) as u64
    }
}
pub struct SystemRandom(rand::rngs::StdRng);
impl Default for SystemRandom {
    fn default() -> Self {
        Self(rand::rngs::StdRng::from_os_rng())
    }
}
impl RandomSource for SystemRandom {
    fn shuffle(&mut self, ids: &mut [String]) {
        ids.shuffle(&mut self.0);
    }
}
