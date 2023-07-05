use super::PomodoroMode as Mode;

pub struct PomodoroSession {
    mode: Mode,
    session: u32,
}

impl Default for PomodoroSession {
    fn default() -> Self {
        Self {
            mode: Mode::default(),
            session: 1,
        }
    }
}

impl PomodoroSession {
    pub fn new() -> Self {
        Self {
            mode: Mode::default(),
            session: 1,
        }
    }

    pub fn session(&self) -> u32 {
        self.session
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn next(&self) -> Self {
        match self.mode {
            Mode::Work if self.session % 4 == 0 =>
                    Self { mode: Mode::LongBreak, ..*self },
            Mode::Work => Self { mode: Mode::Break, ..*self },
            Mode::Break | Mode::LongBreak =>
                Self { mode: Mode::Work, session: self.session + 1 }
        }
    }
}
