use std::time::{Instant, Duration};

pub struct Timer {
    time_base: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            time_base: Instant::now()
        }
    }

    pub fn elapsed(&self) -> Duration {
        Instant::now() - self.time_base
    }

    pub fn restart(&mut self) -> Duration {
        let elapsed = self.elapsed();
        self.time_base = Instant::now();

        elapsed
    }
}