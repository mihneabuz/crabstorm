use std::time::{Duration, Instant};

use rand::random;

#[derive(Clone, Debug)]
pub struct Timer {
    last: Instant,
    timeout: Duration,
}

impl Timer {
    pub fn new(base: u64, jitter: u64) -> Self {
        Self {
            timeout: Duration::from_millis(base + random::<u64>() % jitter),
            last: Instant::now(),
        }
    }

    pub fn expired(&self) -> bool {
        self.last.elapsed() > self.timeout
    }

    pub fn reset(&mut self) {
        self.last = Instant::now();
    }
}
