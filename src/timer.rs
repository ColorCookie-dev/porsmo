use crate::counter::Counter;
use std::time::Instant;

pub struct Timer {
    started: Instant,
    counter: u64,
    running: bool,
}

impl Timer {
    pub fn new(time: u64) -> Self {
        Self {
            started: Instant::now(),
            counter: time,
            running: true,
        }
    }
}

impl Counter for Timer {
    fn is_running(&self) -> bool {
        self.running
    }

    fn check_end(&self) -> bool {
        false
    }

    fn pause(&mut self) {
        if self.running {
            self.counter += self.started.elapsed().as_secs();
            self.running = false;
        }
    }

    fn resume(&mut self) {
        if !self.running {
            self.running = true;
            self.started = Instant::now();
        }
    }

    fn toggle(&mut self) {
        if self.running {
            self.counter += self.started.elapsed().as_secs();
            self.running = false;
        } else {
            self.running = true;
            self.started = Instant::now();
        }
    }

    fn counter(&self) -> u64 {
        if self.running {
            self.counter + self.started.elapsed().as_secs()
        } else {
            self.counter
        }
    }
}
