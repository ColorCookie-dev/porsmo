use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct Counter {
    start: Option<Instant>,
    elapsed: Duration,
}

impl Counter {
    pub const ZERO: Self = Self { start: None, elapsed: Duration::ZERO };

    pub fn new(start: Option<Instant>, elapsed: Duration) -> Self {
        Self { start, elapsed }
    }

    pub fn started(&self) -> bool {
        matches!(self.start, Some(_))
    }

    pub fn stopped(&self) -> bool {
        matches!(self.start, None)
    }

    pub fn start(self) -> Self {
        Self { start: Some(Instant::now()), ..self }
    }

    pub fn stop(self) -> Self {
        Self { start: None, ..self }
    }

    pub fn toggle(self) -> Self {
        match self.start {
            Some(_start) => self.stop(),
            None         => self.start(),
        }
    }

    pub fn get_elapsed(&self) -> Duration {
        self.elapsed
    }

    pub fn with_elapsed(self, duration: Duration) -> Self {
        Self { elapsed: duration, ..self }
    }

    pub fn elapsed(self) -> Duration {
        self.update().elapsed
    }

    pub fn update(self) -> Self {
        self.update_with(Self::count_up)
    }

    pub fn update_with(self, updater: impl Fn(Duration, Duration) -> Duration)
        -> Self {
        self.with_elapsed(
            self.start
                .map(|start| updater(self.elapsed, start.elapsed()))
                .unwrap_or(self.elapsed)
        )
    }

    pub fn count_up(before: Duration, elapsed_now: Duration) -> Duration {
        before.saturating_add(elapsed_now)
    }

    pub fn count_down(before: Duration, elapsed_now: Duration) -> Duration {
        before.saturating_sub(elapsed_now)
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::ZERO
    }
}

#[test]
fn test_opt_add() -> Result<(), Box<dyn std::error::Error>> {
    let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(v.iter().cloned().reduce(std::ops::Add::add), Some(55));
    Ok(())
}
