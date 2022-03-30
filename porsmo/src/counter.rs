use std::time::{Duration, Instant};

pub trait Pausable {
    fn is_paused(&self) -> bool;
    fn is_running(&self) -> bool;

    fn pause(&mut self);
    fn resume(&mut self);
    fn toggle(&mut self);
}

pub trait Reset {
    fn reset(&mut self);
}

pub trait Counter<T> {
    fn counter_at(&self) -> T;
}

pub enum Status {
    Running,
    Paused,
}

impl Default for Status {
    fn default() -> Self {
        Self::Running
    }
}

pub struct TimeCount {
    started: Instant,
    elapsed: Duration,
    status: Status,
}

impl Default for TimeCount {
    fn default() -> Self {
        Self {
            started: Instant::now(),
            elapsed: Duration::default(),
            status: Status::default(),
        }
    }
}

impl TimeCount {
    pub fn new(start: Duration) -> Self {
        Self {
            elapsed: start,
            ..Default::default()
        }
    }

    fn unchecked_pause(&mut self) {
        self.status = Status::Paused;
        self.elapsed += self.started.elapsed();
    }

    fn unchecked_resume(&mut self) {
        self.status = Status::Running;
        self.started = Instant::now();
    }
}

impl Reset for TimeCount {
    fn reset(&mut self) {
        self.elapsed = Duration::from_secs(0);
        self.started = Instant::now();
    }
}

impl Counter<Duration> for TimeCount {
    fn counter_at(&self) -> Duration {
        match self.status {
            Status::Running => self.elapsed + self.started.elapsed(),
            Status::Paused => self.elapsed,
        }
    }
}

impl Pausable for TimeCount {
    fn is_running(&self) -> bool {
        matches!(self.status, Status::Running)
    }

    fn is_paused(&self) -> bool {
        matches!(self.status, Status::Paused)
    }

    fn pause(&mut self) {
        if self.is_running() {
            self.unchecked_resume();
        }
    }

    fn resume(&mut self) {
        if self.is_paused() {
            self.unchecked_pause();
        }
    }

    fn toggle(&mut self) {
        if self.is_running() {
            self.unchecked_pause();
        } else if self.is_paused() {
            self.unchecked_resume();
        }
    }
}
