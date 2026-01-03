use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    start_time: Option<Instant>,
    accumulated: Duration,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            start_time: Some(Instant::now()),
            accumulated: Duration::ZERO,
        }
    }
}

impl Clock {
    pub fn is_running(&self) -> bool {
        self.start_time.is_some()
    }

    pub fn elapsed(&self) -> Duration {
        match self.start_time {
            Some(start_time) => start_time.elapsed() + self.accumulated,
            None => self.accumulated,
        }
    }

    pub fn toggle(&mut self) {
        match self.start_time {
            Some(start_time) => {
                self.accumulated += start_time.elapsed();
                self.start_time = None;
            }
            None => self.start_time = Some(Instant::now()),
        }
    }

    // pub fn resume(&mut self) {
    //     if self.start_time.is_none() {
    //         self.start_time = Some(Instant::now());
    //     }
    // }

    // pub fn pause(&mut self) {
    //     if let Some(start_time) = self.start_time {
    //         self.accumulated += start_time.elapsed();
    //         self.start_time = None;
    //     }
    // }

    pub fn reset(&mut self) {
        self.start_time = Some(Instant::now());
        self.accumulated = Duration::ZERO;
    }
}
