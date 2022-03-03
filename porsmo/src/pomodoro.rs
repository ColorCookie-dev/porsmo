use crate::timer::Timer;
use std::time::Duration;

pub type CountType = crate::timer::CountType;

pub enum Mode {
    Work,
    Rest,
    LongRest,
}

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
    counter: Timer,
    session: u64,
    mode: Mode,
    options: PomoSettings,
}

impl Pomodoro {
    pub fn new(work: Duration, rest: Duration, long_rest: Duration) -> Self {
        let options = PomoSettings::new(work, rest, long_rest);

        Self {
            counter: Timer::new(options.work),
            session: 0,
            mode: Mode::Work,
            options,
        }
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn session(&self) -> u64 {
        self.session
    }

    pub fn is_running(&self) -> bool {
        self.counter.is_running()
    }

    pub fn is_paused(&self) -> bool {
        self.counter.is_paused()
    }

    pub fn has_ended(&self) -> bool {
        self.counter.has_ended()
    }

    pub fn pause(&mut self) {
        self.counter.pause();
    }

    pub fn resume(&mut self) {
        self.counter.resume();
    }

    pub fn check_next_mode(&self) -> Mode {
        match self.mode {
            Mode::Work => {
                if self.session % 4 == 0 && self.session != 0 {
                    Mode::LongRest
                } else {
                    Mode::Rest
                }
            }
            Mode::Rest | Mode::LongRest => Mode::Work,
        }
    }

    pub fn next_mode(&mut self) {
        match self.mode {
            Mode::Work => {
                if self.session % 4 == 0 && self.session != 0 {
                    self.mode = Mode::LongRest;
                    self.counter = Timer::new(self.options.long_rest);
                } else {
                    self.mode = Mode::Rest;
                    self.counter = Timer::new(self.options.rest);
                }
            }
            Mode::Rest | Mode::LongRest => {
                self.mode = Mode::Work;
                self.counter = Timer::new(self.options.work);
                self.session += 1;
            }
        }
    }

    pub fn reset(&mut self) {
        self.counter.reset();
    }

    pub fn counter_at(&self) -> CountType {
        self.counter.counter_at()
    }

    pub fn toggle(&mut self) {
        self.counter.toggle();
    }
}
