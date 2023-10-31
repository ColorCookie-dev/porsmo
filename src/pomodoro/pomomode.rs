use super::PomoConfig;
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub enum PomodoroMode {
    Work,
    Break,
    LongBreak,
}

impl Default for PomodoroMode {
    fn default() -> Self {
        Self::Work
    }
}

impl PomodoroMode {
    pub fn get_time(&self, config: &PomoConfig) -> Duration {
        match self {
            Self::Work => config.work_time,
            Self::Break => config.break_time,
            Self::LongBreak => config.long_break,
        }
    }
}
