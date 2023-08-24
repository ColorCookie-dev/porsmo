use crate::input::{TIMEOUT, get_event};
use crate::prelude::*;
use crate::alert::Alert;
use crate::terminal::running_color;
use crate::{
    format::fmt_time,
    input::Command,
    terminal::TerminalHandler,
};
use crossterm::style::Color;
use porsmo::counter::Counter;
use porsmo::pomodoro::{
    PomodoroMode as Mode,
    PomoConfig,
    PomodoroSession as Session,
};

use std::time::Duration;

#[derive(Debug, Default)]
pub struct PomoState {
    pub mode:    PomoStateMode,
    pub session: Session,
    pub config:  PomoConfig,
}

#[derive(Debug)]
pub enum PomoStateMode {
    Skip {
        elapsed: Duration,
    },

    Running {
        counter: Counter,
    }
}

impl Default for PomoStateMode {
    fn default() -> Self {
        let counter = Counter::default().start();
        PomoStateMode::Running { counter }
    }
}

impl PomoState {
    pub fn default_with_config(config: PomoConfig) -> Self {
        Self { config, ..Default::default() }
    }

    pub fn handle_command(self, command: Command,) -> Option<Self> {
        match command {
            Command::Quit => match self.mode {
                PomoStateMode::Skip { elapsed } => {
                    let counter = Counter::from(elapsed).start();
                    let mode = PomoStateMode::Running { counter };
                    Some(Self { mode, ..self })
                },
                _ => None,
            },

            Command::No => match self.mode {
                PomoStateMode::Skip { elapsed } => {
                    let counter = Counter::from(elapsed).start();
                    let mode = PomoStateMode::Running { counter };
                    Some(Self { mode, ..self })
                },
                _ => Some(self),
            },

            Command::Enter => match self.mode {
                PomoStateMode::Running { counter }
                if counter.elapsed() < self.session.mode.get_time(&self.config)
                    => {
                    let counter = Counter::default().start();
                    let mode = PomoStateMode::Running { counter };
                    let session = self.session.next();
                    Some(Self { mode, session, ..self })
                }
                PomoStateMode::Skip { .. } => {
                    let counter = Counter::default().start();
                    let mode = PomoStateMode::Running { counter };
                    let session = self.session.next();
                    Some(PomoState{ mode, session, ..self })
                },
                _ => Some(self),
            },

            Command::Yes => match self.mode {
                PomoStateMode::Skip { .. } => {
                    let counter = Counter::default().start();
                    let mode = PomoStateMode::Running { counter };
                    let session = self.session.next();
                    Some(Self { mode, session, ..self })
                },
                _ => Some(self),
            },

            Command::Pause => match self.mode {
                PomoStateMode::Running { counter } => {
                    let counter = counter.stop();
                    let mode = PomoStateMode::Running { counter };
                    Some(Self { mode, ..self })
                },
                _ => Some(self),
            }

            Command::Resume => match self.mode {
                PomoStateMode::Running { counter } => {
                    let counter = counter.start();
                    let mode = PomoStateMode::Running { counter };
                    Some(Self { mode, ..self })
                },
                _ => Some(self),
            }

            Command::Toggle => match self.mode {
                PomoStateMode::Running { counter } => {
                    let counter = counter.toggle();
                    let mode = PomoStateMode::Running { counter };
                    Some(Self { mode, ..self })
                },
                _ => Some(self),
            }

            Command::Skip => match self.mode {
                PomoStateMode::Running { counter } => {
                    let elapsed = counter.elapsed();
                    let mode = PomoStateMode::Skip { elapsed };
                    Some(PomoState { mode, ..self })
                },
                _ => Some(self),
            }

            _ => Some(self),
        }
    }

    fn title(mode: Mode) -> &'static str {
        match mode {
            Mode::Work => "Pomodoro (Work)",
            Mode::Break => "Pomodoro (Break)",
            Mode::LongBreak => "Pomodor (Long Break)",
        }
    }

    fn break_title(next_mode: Mode) -> &'static str {
        match next_mode {
            Mode::Work => "Break has ended! Start work?",
            Mode::Break => "Work has ended! Start break?",
            Mode::LongBreak => "Work has ended! Start a long break",
        }
    }

    pub fn show(
        &self,
        terminal: &mut TerminalHandler,
    ) -> Result<()> {
        let target = self.session.mode.get_time(&self.config);
        let round_number = format!("Round: {}", self.session.number);
        match self.mode {
            PomoStateMode::Skip { .. } => {
                let (color, skip_to) = match self.session.next().mode {
                    Mode::Work => (Color::Red, "skip to work?"),
                    Mode::Break => (Color::Green, "skip to break?"),
                    Mode::LongBreak => (Color::Green, "skip to long break?"),
                };
                terminal
                    .clear()?
                    .set_foreground_color(color)?
                    .print(skip_to)?
                    .info(round_number)?
                    .info(Self::SKIP_CONTROLS)?
                    .flush()?;
            },
            PomoStateMode::Running { counter } if counter.elapsed() < target => {
                let time_left = target.saturating_sub(counter.elapsed());

                terminal
                    .clear()?
                    .info(Self::title(self.session.mode))?
                    .set_foreground_color(running_color(counter.started()))?
                    .print(fmt_time(time_left))?
                    .info(Self::CONTROLS)?
                    .status(round_number)?
                    .flush()?;
            },
            PomoStateMode::Running { counter } => {
                let excess_time = counter.elapsed().saturating_sub(target);
                let (title, message) = Self::pomodoro_alert_message(
                    self.session.next().mode
                );
                // TODO: Alert

                terminal
                    .clear()?
                    .info(Self::break_title(self.session.next().mode))?
                    .set_foreground_color(running_color(counter.started()))?
                    .print(format_args!("+{}", fmt_time(excess_time)))?
                    .info(Self::ENDING_CONTROLS)?
                    .status(message)?
                    .flush()?;
            },
        }
        Ok(())
    }

    pub fn run(
        terminal: &mut TerminalHandler,
        config: PomoConfig
    ) -> Result<()> {
        let mut state = Self::default_with_config(config);

        loop {
            state.show(terminal)?;
            if let Some(cmd) = get_event(TIMEOUT)?.map(Command::from) {
                match state.handle_command(cmd) {
                    Some(new_state) => state = new_state,
                    None => return Ok(()),
                }
            }
        }
    }

    const CONTROLS: &str = "[Q]: quit, [Shift S]: Skip, [Space]: pause/resume";
    const ENDING_CONTROLS: &str =
        "[Q]: quit, [Shift S]: Skip, [Space]: pause/resume, [Enter]: Next";
    const SKIP_CONTROLS: &str = "[Enter]: Yes, [Q/N]: No";

    pub fn pomodoro_alert_message(
        next_mode: Mode
    ) -> (&'static str, &'static str) {
        match next_mode {
            Mode::Work => ("Your break ended!", "Time for some work"),
            Mode::Break => ("Pomodoro ended!", "Time for a short break"),
            Mode::LongBreak => ("Pomodoro 4 sessions complete!", "Time for a long break"),
        }
    }
}
