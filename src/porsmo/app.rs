use std::time::Duration;
use porsmo::pomodoro::PomoConfig;

use crate::prelude::*;
use crate::{
    TIMEOUT,
    input::Command,
    pomodoro::PomodoroUI,
    terminal::TerminalHandler,
    timer::TimerUI, get_event, prelude::PorsmoError,
};

pub enum PorsmoUI {
    Timer(TimerUI),
    Pomodoro(PomodoroUI),
}

impl PorsmoUI {
    pub fn handle_command(self, command: Command) -> Self {
        match self {
            Self::Timer(timer) => Self::Timer(timer.handle_command(command)),
            Self::Pomodoro(pomo) => Self::Pomodoro(pomo.handle_command(command)),
        }
    }

    pub fn run_loop(mut self, terminal: &mut TerminalHandler) -> Result<()> {
        while !self.ended() {
            self.show(terminal)?;

            match get_event(TIMEOUT) {
                Ok(command) => self = self.handle_command(Command::from(command)),
                Err(PorsmoError::NoEventsToPoll) => (),
                Err(e) => return Err(e.into()),
            };
        }

        Ok(())
    }

    pub fn show(&self, terminal: &mut TerminalHandler) -> Result<()> {
        match self {
            Self::Timer(timer) => timer.show(terminal),
            Self::Pomodoro(pomo) => pomo.show(terminal),
        }
    }

    pub fn ended(&self) -> bool {
        match self {
            Self::Timer(timer) => timer.ended(),
            Self::Pomodoro(pomo) => pomo.ended(),
        }
    }

    pub fn timer(initial: Duration) -> Self {
        PorsmoUI::Timer(TimerUI::new(initial))
    }

    pub fn pomodoro(config: PomoConfig) -> Self {
        PorsmoUI::Pomodoro(PomodoroUI::new(config))
    }
}

