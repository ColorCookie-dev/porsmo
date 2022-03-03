use std::time::{Duration, Instant};

pub enum Status {
    Running,
    Paused,
}

impl Status {
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self, Self::Paused)
    }
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

    pub fn is_running(&self) -> bool {
        self.status.is_running()
    }

    pub fn is_paused(&self) -> bool {
        self.status.is_paused()
    }

    pub fn pause(&mut self) {
        if self.is_running() {
            self.status = Status::Paused;
            self.elapsed += self.started.elapsed();
        }
    }

    pub fn resume(&mut self) {
        if self.is_paused() {
            self.status = Status::Running;
            self.started = Instant::now();
        }
    }

    pub fn toggle(&mut self) {
        match self.status {
            Status::Running => {
                self.status = Status::Paused;
                self.elapsed += self.started.elapsed();
            }
            Status::Paused => {
                self.status = Status::Running;
                self.started = Instant::now();
            }
        }
    }

    pub fn counter_at(&self) -> Duration {
        match self.status {
            Status::Running => self.elapsed + self.started.elapsed(),
            Status::Paused => self.elapsed,
        }
    }

    pub fn reset(&mut self) {
        self.elapsed = Duration::from_secs(0);
        self.started = Instant::now();
    }
}
