use crate::counter::Counter;
use std::time::Instant;

pub struct Countdown {
    started: Instant,
    counter: u64,
    running: bool,
}

impl Countdown {
    pub fn new(count: u64) -> Self {
        Self {
            started: Instant::now(),
            counter: count,
            running: true,
        }
    }
}

impl Counter for Countdown {
    fn check_end(&self) -> bool {
        if self.running {
            self.counter <= self.started.elapsed().as_secs()
        } else {
            self.counter == 0
        }
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn pause(&mut self) {
        if self.running {
            let elapsed = self.started.elapsed().as_secs();
            self.counter = if self.counter > elapsed {
                self.counter - elapsed
            } else {
                0
            };
            self.running = false;
        }
    }

    fn resume(&mut self) {
        if !self.running {
            self.running = true;
            self.started = Instant::now();
        }
    }

    fn counter(&self) -> u64 {
        if self.running {
            let elapsed = self.started.elapsed().as_secs();
            if self.counter > elapsed {
                self.counter - elapsed
            } else {
                0
            }
        } else {
            self.counter
        }
    }
}
