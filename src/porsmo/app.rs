use std::time::Duration;
use porsmo::pomodoro::PomoConfig;

use crate::{
    input::Command,
    pomodoro::PomodoroUI,
    stopwatch::StopwatchUI,
    terminal::{TerminalHandler, TerminalError},
    timer::TimerUI,
};

pub enum PorsmoUI {
    Stopwatch(StopwatchUI),
    Timer(TimerUI),
    Pomodoro(PomodoroUI),
}

impl PorsmoUI {
    pub fn handle_command(self, command: Command) -> Self {
        match self {
            Self::Timer(timer) => Self::Timer(timer.handle_command(command)),
            Self::Stopwatch(st) => Self::Stopwatch(st.handle_command(command)),
            Self::Pomodoro(pomo) => Self::Pomodoro(pomo.handle_command(command)),
        }
    }

    pub fn show(&self, terminal: &mut TerminalHandler)
    -> Result<(), TerminalError> {
        match self {
            Self::Stopwatch(st) => st.show(terminal),
            Self::Timer(timer) => timer.show(terminal),
            Self::Pomodoro(pomo) => pomo.show(terminal),
        }
    }

    pub fn ended(&self) -> bool {
        match self {
            Self::Stopwatch(st) => st.ended(),
            Self::Timer(timer) => timer.ended(),
            Self::Pomodoro(pomo) => pomo.ended(),
        }
    }

    pub fn stopwatch(initial: Duration) -> Self {
        PorsmoUI::Stopwatch(StopwatchUI::new(initial))
    }

    pub fn timer(initial: Duration) -> Self {
        PorsmoUI::Timer(TimerUI::new(initial))
    }

    pub fn pomodoro(config: PomoConfig) -> Self {
        PorsmoUI::Pomodoro(PomodoroUI::new(config))
    }
}

