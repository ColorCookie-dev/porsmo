use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct Counter {
    start: Option<Instant>,
    elapsed: Duration,
}

impl Default for Counter {
    fn default() -> Self {
        Self {
            start: Some(Instant::now()),
            elapsed: Duration::ZERO,
        }
    }
}

impl From<Duration> for Counter {
    fn from(elapsed: Duration) -> Self {
        Self {
            start: None,
            elapsed,
        }
    }
}

impl Into<Duration> for Counter {
    fn into(self) -> Duration {
        self.elapsed()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DoubleEndedDuration {
    Positive(Duration),
    Negative(Duration),
}

impl Counter {
    pub fn new(start: Option<Instant>, elapsed: Duration) -> Self {
        Self { start, elapsed }
    }

    pub fn started(&self) -> bool {
        matches!(self.start, Some(_))
    }

    pub fn stopped(&self) -> bool {
        matches!(self.start, None)
    }

    pub fn saturating_time_left(&self, initial: Duration) -> Duration {
        initial.saturating_sub(self.elapsed())
    }

    pub fn checked_time_left(&self, initial: Duration) -> DoubleEndedDuration {
        match initial.checked_sub(self.elapsed()) {
            Some(x) => DoubleEndedDuration::Positive(x),
            None => DoubleEndedDuration::Negative(self.elapsed().saturating_sub(initial)),
        }
    }

    pub fn start(mut self) -> Self {
        if let None = self.start {
            self.start = Some(Instant::now());
        }
        self
    }

    pub fn stop(self) -> Self {
        Self {
            start: None,
            elapsed: self.elapsed(),
        }
    }

    pub fn elapsed(self) -> Duration {
        self.start
            .map(|start| start.elapsed())
            .map_or(self.elapsed, |dur| self.elapsed.saturating_add(dur))
    }

    pub fn toggle(self) -> Self {
        match self.start {
            Some(_) => self.stop(),
            None => self.start(),
        }
    }
}

#[test]
fn test_opt_add() -> Result<(), Box<dyn std::error::Error>> {
    let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(v.iter().cloned().reduce(std::ops::Add::add), Some(55));
    Ok(())
}
