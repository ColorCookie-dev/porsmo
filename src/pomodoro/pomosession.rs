use super::PomodoroMode as Mode;

#[derive(Debug, Clone, Copy)]
pub struct PomodoroSession {
    pub mode: Mode,
    pub number: u32,
}

impl Default for PomodoroSession {
    fn default() -> Self {
        Self {
            mode: Mode::default(),
            number: 1,
        }
    }
}

impl PomodoroSession {
    pub fn next(&self) -> Self {
        match self.mode {
            Mode::Work if self.number % 4 == 0 => Self {
                mode: Mode::LongBreak,
                ..*self
            },
            Mode::Work => Self {
                mode: Mode::Break,
                ..*self
            },
            Mode::Break | Mode::LongBreak => Self {
                mode: Mode::Work,
                number: self.number + 1,
            },
        }
    }
}
