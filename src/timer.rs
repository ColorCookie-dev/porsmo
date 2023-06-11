use crate::counter::Counter;
use std::{time::Duration, ops::{Sub, Add}};

pub struct Timer {
    counter: Counter,
    initial: Duration,
}

impl Timer {
    pub fn new(initial: Duration) -> Self {
        Self { counter: Counter::ZERO, initial }
    }

    pub fn time_left(&self) -> Duration {
        self.initial.saturating_sub(self.counter.elapsed())
    }

    pub fn ended(&self) -> bool {
        self.time_left().is_zero()
    }

    pub fn started(&self) -> bool {
        self.counter.started()
    }

    pub fn update(self) -> Self {
        Self { counter: self.counter.update(), ..self }
    }

    pub fn stop(self) -> Self {
        Self { counter: self.counter.stop(), ..self }
    }

    pub fn start(self) -> Self {
        Self { counter: self.counter.start(), ..self }
    }

    pub fn toggle(self) -> Self {
        Self { counter: self.counter.toggle(), ..self }
    }
}

pub enum DoubleEndedDuration {
    Positive(Duration),
    Negative(Duration),
}

impl From<Duration> for DoubleEndedDuration {
    fn from(value: Duration) -> Self {
        DoubleEndedDuration::Positive(value)
    }
}

impl Add for DoubleEndedDuration {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Positive(lhs), Self::Positive(rhs)) =>
                Self::Positive(lhs.saturating_add(rhs)),
            (Self::Negative(lhs), Self::Positive(rhs)) =>
                Self::Positive(rhs) - Self::Positive(lhs),
            (Self::Positive(lhs), Self::Negative(rhs)) =>
                Self::Positive(lhs) - Self::Positive(rhs),
            (Self::Negative(lhs), Self::Negative(rhs)) =>
                Self::Negative(lhs.saturating_add(rhs))
        }
    }
}

impl DoubleEndedDuration {
    fn negate(self) -> Self {
        match self {
            Self::Positive(dur) => Self::Negative(dur),
            Self::Negative(dur) => Self::Positive(dur),
        }
    }
}

impl Sub for DoubleEndedDuration {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Positive(lhs), Self::Positive(rhs)) if lhs >= rhs =>
                Self::Positive(lhs.saturating_sub(rhs)),
            (Self::Positive(lhs), Self::Positive(rhs)) =>
                Self::Negative(rhs.saturating_sub(lhs)),
            (Self::Positive(lhs), Self::Negative(rhs)) =>
                Self::Positive(lhs.saturating_add(rhs)),
            (Self::Negative(lhs), Self::Positive(rhs)) =>
                Self::Negative(lhs.saturating_add(rhs)),
            (Self::Negative(lhs), Self::Negative(rhs)) =>
                Self::Positive(rhs) - Self::Positive(lhs),
        }
    }
}

pub type ExcessTimer = Timer;

impl ExcessTimer {
    pub fn excess_time_left(self) -> DoubleEndedDuration {
        let updated_time = self.update();
        DoubleEndedDuration::Positive(updated_time.initial) -
            DoubleEndedDuration::Positive(updated_time.counter.get_elapsed())
    }
}
