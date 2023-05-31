use crate::counter::Counter;
use std::time::{Instant, Duration};

pub struct Timer {
    started: Option<Instant>,
    remaining: Duration,
}

impl Timer {
    pub fn new(remaining: Duration) -> Self {
        Self {
            started: Some(Instant::now()),
            remaining,
        }
    }
}

impl Counter for Timer {
    fn has_ended(&self) -> bool {
        self.elapsed().is_zero()
    }

    fn is_running(&self) -> bool {
        !matches!(self.started, None)
    }

    fn elapsed(&self) -> Duration {
        match self.started {
            Some(started) => self.remaining.saturating_sub(started.elapsed()),
            None => self.remaining,
        }
    }

    fn pause(&mut self) {
        if self.is_running() {
            self.remaining = self.elapsed();
            self.started = None;
        }
    }

    fn resume(&mut self) {
        if !self.is_running() {
            self.started = Some(Instant::now());
        }
    }

    fn end_count(&mut self) {
        self.pause();
    }

    fn toggle(&mut self) {
        match self.started {
            Some(started) => {
                self.remaining = self.remaining.saturating_sub(started.elapsed());
                self.started = None;
            },
            None => self.started = Some(Instant::now()),
        }
    }
}
