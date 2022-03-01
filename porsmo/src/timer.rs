use crate::counter::TimeCount;
use std::time::Duration;

pub enum CountType {
    Count(Duration),
    Exceed(Duration),
}

pub struct Timer {
    counter: TimeCount,
    target: Duration,
}

impl Timer {
    pub fn new(target: Duration) -> Self {
        Self {
            counter: TimeCount::default(),
            target,
        }
    }

    pub fn is_running(&self) -> bool {
        self.counter.is_running()
    }

    pub fn is_paused(&self) -> bool {
        self.counter.is_paused()
    }

    pub fn has_ended(&self) -> bool {
        self.counter.counter_at() > self.target
    }

    pub fn pause(&mut self) {
        self.counter.pause();
    }

    pub fn resume(&mut self) {
        self.counter.resume();
    }

    pub fn counter_at(&self) -> CountType {
        match self.target.checked_sub(self.counter.counter_at()) {
            Some(c) => CountType::Count(c),
            None => CountType::Exceed(self.counter.counter_at().saturating_sub(self.target)),
        }
    }

    pub fn toggle(&mut self) {
        self.counter.toggle();
    }
}
