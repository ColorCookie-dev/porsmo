use crate::counter::Counter;
use std::time::Instant;

pub struct Timer {
    started: Instant,
    counter: u64,
    status: Status,
}

enum Status {
    Running,
    Paused,
}

impl Timer {
    pub fn new(count: u64) -> Self {
        Self {
            started: Instant::now(),
            counter: count,
            status: Status::Running,
        }
    }

    fn counter_now(&self) -> u64 {
        let elapsed = self.started.elapsed().as_secs();
        if self.counter > elapsed {
            self.counter - elapsed
        } else {
            0
        }
    }
}

impl Counter for Timer {
    fn has_ended(&self) -> bool {
        self.counter() == 0
    }

    fn is_running(&self) -> bool {
        matches!(self.status, Status::Running)
    }

    fn is_paused(&self) -> bool {
        matches!(self.status, Status::Paused)
    }

    fn counter(&self) -> u64 {
        if self.is_running() {
            self.counter_now()
        } else {
            self.counter
        }
    }

    fn pause(&mut self) {
        if self.is_running() {
            self.counter = self.counter_now();
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
    }
}
