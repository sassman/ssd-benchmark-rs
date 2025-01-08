use std::time::{Duration, Instant};

pub struct Timer {
    pub start: Instant,
}

impl Timer {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn stop(&self) -> Duration {
        self.start.elapsed()
    }
}
