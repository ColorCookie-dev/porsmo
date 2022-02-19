use crate::counter::Counter;
use std::time::Instant;

pub struct Stopwatch {
    started: Instant,
    counter: u64,
    status: Status,
}

enum Status {
    Running,
    Paused,
    Ended,
}

impl Stopwatch {
    pub fn new(count: u64) -> Self {
        Self {
            started: Instant::now(),
            counter: count,
            status: Status::Running,
        }
    }
}

impl Counter for Stopwatch {
    fn has_ended(&self) -> bool {
        matches!(self.status, Status::Ended)
    }

    fn is_running(&self) -> bool {
        matches!(self.status, Status::Running)
    }

    fn is_paused(&self) -> bool {
        matches!(self.status, Status::Paused)
    }

    fn counter(&self) -> u64 {
        if self.is_running() {
            self.counter + self.started.elapsed().as_secs()
        } else {
            self.counter
        }
    }

    fn pause(&mut self) {
        if self.is_running() {
            self.counter += self.started.elapsed().as_secs();
            self.status = Status::Paused;
        }
    }

    fn resume(&mut self) {
        if self.is_paused() {
            self.status = Status::Running;
            self.started = Instant::now();
        }
    }

    fn end_count(&mut self) {
        self.pause();
        self.status = Status::Ended;
    }
}
