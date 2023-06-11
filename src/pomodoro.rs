use std::time::Duration;

use crate::timer::ExcessTimer;

pub struct PomoConfig {
    pub work_time: Duration,
    pub break_time: Duration,
    pub long_break: Duration,
}

impl PomoConfig {
    fn short() -> Self {
        Self {
            work_time: Duration::from_secs(25 * 60),
            break_time: Duration::from_secs(5 * 60),
            long_break: Duration::from_secs(10 * 60),
        }
    }

    fn long() -> Self {
        Self {
            work_time: Duration::from_secs(55 * 60),
            break_time: Duration::from_secs(10 * 60),
            long_break: Duration::from_secs(20 * 60),
        }
    }

    fn custom(work_time: Duration, break_time: Duration, long_break: Duration)
        -> Self {
        Self {
            work_time,
            break_time,
            long_break,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Work,
    Break,
    LongBreak,
}

pub struct Pomodoro {
    timer: ExcessTimer,
    mode: Mode,
    session: u64,
    config: PomoConfig,
}

impl Pomodoro {
    pub fn new(config: PomoConfig) -> Self {
        Self {
            timer: ExcessTimer::new(config.work_time),
            mode: Mode::Work,
            session: 1,
            config,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn check_next_mode(&self) -> Mode {
        match self.mode {
            Mode::Work => {
                if self.session % 4 == 0 {
                    Mode::LongBreak
                } else {
                    Mode::Break
                }
            }
            Mode::Break | Mode::LongBreak => Mode::Work,
        }
    }

    pub fn next_mode(&mut self) {
        match self.check_next_mode() {
            Mode::Work => self.work_mode(),
            Mode::Break => self.break_mode(),
            Mode::LongBreak => self.long_break_mode(),
        }
    }

    pub fn session(&self) -> u64 {
        self.session
    }

    fn work_mode(&mut self) {
        self.session += 1;
        self.mode = Mode::Work;
        self.timer = ExcessTimer::new(self.config.work_time);
    }

    fn break_mode(&mut self) {
        self.mode = Mode::Break;
        self.timer = ExcessTimer::new(self.config.break_time);
    }

    fn long_break_mode(&mut self) {
        self.mode = Mode::LongBreak;
        self.timer = ExcessTimer::new(self.config.long_break);
    }

    pub fn time_left(&self) -> Duration {
        self.timer.time_left()
    }

    pub fn ended(&self) -> bool {
        self.time_left().is_zero()
    }

    pub fn started(&self) -> bool {
        self.timer.started()
    }

    pub fn update(self) -> Self {
        Self { timer: self.timer.update(), ..self }
    }

    pub fn stop(self) -> Self {
        Self { timer: self.timer.stop(), ..self }
    }

    pub fn start(self) -> Self {
        Self { timer: self.timer.start(), ..self }
    }

    pub fn toggle(self) -> Self {
        Self { timer: self.timer.toggle(), ..self }
    }
}
