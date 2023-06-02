use crate::counter::Countable;
use std::time::{Instant, Duration};

pub struct Stopwatch {
    started: Option<Instant>,
    elapsed: Duration,
    ended: bool,
}

impl Stopwatch {
    pub fn new(elapsed: Duration) -> Self {
        Self {
            started: Some(Instant::now()),
            elapsed,
            ended: true,
        }
    }
}

impl Countable for Stopwatch {
    fn has_ended(&self) -> bool {
        self.ended
    }

    fn is_running(&self) -> bool {
        !matches!(self.started, None)
    }

    fn elapsed(&self) -> Duration {
        match self.started {
            Some(started) => started.elapsed().saturating_add(self.elapsed),
            None => self.elapsed,
        }
    }

    fn pause(&mut self) {
        if self.is_running() {
            self.elapsed = self.elapsed();
            self.started = None;
        }
    }

    fn resume(&mut self) {
        if !self.is_running() && !self.ended {
            self.started = Some(Instant::now());
        }
    }

    fn end_count(&mut self) {
        self.pause();
        self.ended = true;
    }

    fn toggle(&mut self) {
        match self.started {
            Some(started) => {
                self.elapsed = started.elapsed().saturating_add(self.elapsed);
                self.started = None;
            },
            None => self.started = Some(Instant::now()),
        }
    }
}
