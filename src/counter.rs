use std::{time::{Duration, Instant}, ops::Mul};

pub struct Counter {
    start: Option<Instant>,
    elapsed: Duration,
}

impl Counter {
    pub fn new(start: Option<Instant>, elapsed: Duration) -> Self {
        Self { start, elapsed }
    }
    
    pub fn start(self) -> Self {
        Self { start: Some(Instant::now()), elapsed: self.elapsed() }
    }

    pub fn elapsed(&self) -> Duration {
        match self.start {
            Some(start) => self.elapsed.saturating_add(start.elapsed()),
            None => self.elapsed,
        }
    }

    pub fn toggle(self) -> Self {
        match self.start {
            Some(start) => self.stop(),
            None => self.start(),
        }
    }

    pub fn stop(self) -> Self {
        Self::new(None, self.elapsed())
    }

    pub fn started(&self) -> bool {
        matches!(self.start, Some(_))
    }

    pub fn stopped(&self) -> bool {
        matches!(self.start, None)
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self { start: None, elapsed: Duration::ZERO }
    }
}

pub trait Countable {
    fn is_running(&self) -> bool;
    fn has_ended(&self) -> bool;
    fn elapsed(&self) -> Duration;
    fn pause(&mut self);
    fn resume(&mut self);
    fn end_count(&mut self);
    fn toggle(&mut self);
}

#[test]
fn test_opt_add() -> Result<(), Box<dyn std::error::Error>> {
    let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(v.iter().cloned().reduce(std::ops::Add::add), Some(55));
    Ok(())
}
