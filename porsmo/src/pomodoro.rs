pub use crate::{counter::*, timer::*};
use std::time::Duration;

pub type CountType = crate::timer::CountType<Duration>;

#[derive(Copy, Clone, Debug)]
pub enum Mode {
    Work,
    Rest,
    LongRest,
}

#[derive(Debug)]
pub struct PomoSettings {
    work: Duration,
    rest: Duration,
    long_rest: Duration,
}

impl Default for PomoSettings {
    fn default() -> Self {
        Self {
            work: Duration::from_secs(25 * 60),
            rest: Duration::from_secs(5 * 60),
            long_rest: Duration::from_secs(10 * 60),
        }
    }
}

impl PomoSettings {
    pub fn new(work: Duration, rest: Duration, long_rest: Duration) -> Self {
        Self {
            work,
            rest,
            long_rest,
        }
    }
}

pub struct Pomodoro {
    counter: AlertTimer,
    session: u64,
    mode: Mode,
    options: PomoSettings,
    alert: fn(Mode),
}

impl Pomodoro {
    pub fn new(work: Duration, rest: Duration, long_rest: Duration, alert: fn(Mode)) -> Self {
        let options = PomoSettings::new(work, rest, long_rest);

        Self {
            counter: Timer::new_alert_timer(options.work, move || {
                alert(Mode::Rest);
            }),
            session: 0,
            mode: Mode::Work,
            options,
            alert,
        }
    }
}

impl CheckedCount<Duration> for Pomodoro {
    fn checked_counter_at(&self) -> CountType {
        self.counter.checked_counter_at()
    }
}

impl Pausable for Pomodoro {
    fn is_running(&self) -> bool {
        self.counter.is_running()
    }

    fn is_paused(&self) -> bool {
        self.counter.is_paused()
    }

    fn pause(&mut self) {
        self.counter.pause();
    }

    fn resume(&mut self) {
        self.counter.resume();
    }

    fn toggle(&mut self) {
        self.counter.toggle();
    }
}

impl Reset for Pomodoro {
    fn reset(&mut self) {
        self.counter.reset();
    }
}

impl Counter<Duration> for Pomodoro {
    fn counter_at(&self) -> Duration {
        self.counter.counter_at()
    }
}

impl HasEnd for Pomodoro {
    fn has_ended(&self) -> bool {
        self.counter.has_ended()
    }
}

pub trait PomoMode {
    fn mode(&self) -> &Mode;
    fn session(&self) -> u64;
    fn check_next_mode(&self) -> Mode;
    fn next_mode(&mut self);
}

impl PomoMode for Pomodoro {
    fn mode(&self) -> &Mode {
        &self.mode
    }

    fn session(&self) -> u64 {
        self.session
    }

    fn check_next_mode(&self) -> Mode {
        match self.mode {
            Mode::Work if leap_session(self.session()) => Mode::LongRest,
            Mode::Work => Mode::Rest,
            Mode::Rest | Mode::LongRest => Mode::Work,
        }
    }

    fn next_mode(&mut self) {
        let alert = self.alert.clone();

        let target_time = match self.mode {
            Mode::Work if leap_session(self.session()) => {
                self.mode = Mode::LongRest;
                self.options.long_rest
            }
            Mode::Work => {
                self.mode = Mode::Rest;
                self.options.rest
            }
            Mode::Rest | Mode::LongRest => {
                self.session += 1;
                self.mode = Mode::Work;
                self.options.work
            }
        };

        let next = self.check_next_mode();
        self.counter = Timer::new_alert_timer(target_time, move || {
            alert(next);
        })
    }
}

fn leap_session(session: u64) -> bool {
    session % 4 == 0 && session != 0
}
