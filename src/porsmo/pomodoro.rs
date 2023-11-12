use crate::alert::alert;
use crate::terminal::running_color;
use crate::{format::format_duration, input::Command};
use crate::{prelude::*, Alertable, CounterUIState};
use crossterm::cursor::{MoveTo, MoveToNextLine};
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use crossterm::{
    style::Color,
    style::Stylize,
    queue,
};
use porsmo::counter::Counter;
use porsmo::pomodoro::{PomoConfig, PomodoroMode as Mode, PomodoroSession as Session};

use std::io::Write;
use std::time::Duration;

#[derive(Debug, Default)]
pub struct PomoState {
    pub mode: PomoStateMode,
    pub session: Session,
    pub config: PomoConfig,
    pub alert: bool,
}

#[derive(Debug)]
pub enum PomoStateMode {
    Skip { elapsed: Duration },

    Running { counter: Counter },
}

impl Default for PomoStateMode {
    fn default() -> Self {
        let counter = Counter::default().start();
        PomoStateMode::Running { counter }
    }
}

impl From<PomoConfig> for PomoState {
    fn from(config: PomoConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }
}

impl PomoState {
    fn title(mode: Mode) -> &'static str {
        match mode {
            Mode::Work => "Pomodoro (Work)",
            Mode::Break => "Pomodoro (Break)",
            Mode::LongBreak => "Pomodoro (Long Break)",
        }
    }

    fn break_title(next_mode: Mode) -> &'static str {
        match next_mode {
            Mode::Work => "Break has ended! Start work?",
            Mode::Break => "Work has ended! Start break?",
            Mode::LongBreak => "Work has ended! Start a long break",
        }
    }

    const CONTROLS: &str = "[Q]: quit, [Shift S]: Skip, [Space]: pause/resume";
    const ENDING_CONTROLS: &str =
        "[Q]: quit, [Shift S]: Skip, [Space]: pause/resume, [Enter]: Next";
    const SKIP_CONTROLS: &str = "[Enter]: Yes, [Q/N]: No";

    pub fn pomodoro_alert_message(next_mode: Mode) -> (&'static str, &'static str) {
        match next_mode {
            Mode::Work => ("Your break ended!", "Time for some work"),
            Mode::Break => ("Pomodoro ended!", "Time for a short break"),
            Mode::LongBreak => ("Pomodoro 4 sessions complete!", "Time for a long break"),
        }
    }
}

impl CounterUIState for PomoState {
    fn handle_command(self, command: Command) -> Option<Self> {
        match command {
            Command::Quit => match self.mode {
                PomoStateMode::Skip { elapsed } => {
                    let counter = Counter::from(elapsed).start();
                    let mode = PomoStateMode::Running { counter };
                    Some(Self { mode, ..self })
                }
                _ => None,
            },

            Command::No => match self.mode {
                PomoStateMode::Skip { elapsed } => {
                    let counter = Counter::from(elapsed).start();
                    let mode = PomoStateMode::Running { counter };
                    Some(Self { mode, ..self })
                }
                _ => Some(self),
            },

            Command::Enter => match self.mode {
                PomoStateMode::Running { counter }
                    if counter.elapsed() >= self.session.mode.get_time(&self.config) =>
                {
                    let counter = Counter::default().start();
                    let mode = PomoStateMode::Running { counter };
                    let session = self.session.next();
                    Some(Self {
                        mode,
                        session,
                        alert: false,
                        ..self
                    })
                }
                PomoStateMode::Skip { .. } => {
                    let counter = Counter::default().start();
                    let mode = PomoStateMode::Running { counter };
                    let session = self.session.next();
                    Some(Self {
                        mode,
                        session,
                        alert: false,
                        ..self
                    })
                }
                _ => Some(self),
            },

            Command::Yes => match self.mode {
                PomoStateMode::Skip { .. } => {
                    let counter = Counter::default().start();
                    let mode = PomoStateMode::Running { counter };
                    let session = self.session.next();
                    Some(Self {
                        mode,
                        session,
                        alert: false,
                        ..self
                    })
                }
                _ => Some(self),
            },

            Command::Pause => match self.mode {
                PomoStateMode::Running { counter } => {
                    let counter = counter.stop();
                    let mode = PomoStateMode::Running { counter };
                    Some(Self { mode, ..self })
                }
                _ => Some(self),
            },

            Command::Resume => match self.mode {
                PomoStateMode::Running { counter } => {
                    let counter = counter.start();
                    let mode = PomoStateMode::Running { counter };
                    Some(Self { mode, ..self })
                }
                _ => Some(self),
            },

            Command::Toggle => match self.mode {
                PomoStateMode::Running { counter } => {
                    let counter = counter.toggle();
                    let mode = PomoStateMode::Running { counter };
                    Some(Self { mode, ..self })
                }
                _ => Some(self),
            },

            Command::Skip => match self.mode {
                PomoStateMode::Running { counter } => {
                    let elapsed = counter.elapsed();
                    let mode = PomoStateMode::Skip { elapsed };
                    Some(PomoState { mode, ..self })
                }
                _ => Some(self),
            },

            _ => Some(self),
        }
    }

    fn show(&self, out: &mut impl Write) -> Result<()> {
        let target = self.target();
        let round_number = format!("Session: {}", self.session.number);
        match self.mode {
            PomoStateMode::Skip { .. } => {
                let (color, skip_to) = match self.session.next().mode {
                    Mode::Work => (Color::Red, "skip to work?"),
                    Mode::Break => (Color::Green, "skip to break?"),
                    Mode::LongBreak => (Color::Green, "skip to long break?"),
                };
                queue!(
                    out,
                    MoveTo(0, 0),
                    Clear(ClearType::All),
                    Print(skip_to.with(color)), MoveToNextLine(1),
                    Print(round_number), MoveToNextLine(1),
                    Print(Self::SKIP_CONTROLS)
                )?;
            }
            PomoStateMode::Running { counter } if counter.elapsed() < target => {
                let time_left = target.saturating_sub(counter.elapsed());

                queue!(
                    out,
                    MoveTo(0, 0),
                    Clear(ClearType::All),
                    Print(Self::title(self.session.mode)), MoveToNextLine(1),
                    Print(
                        format_duration(&time_left)
                            .with(running_color(counter.started())),
                    ), MoveToNextLine(1),
                    Print(Self::CONTROLS), MoveToNextLine(1),
                    Print(round_number),
                )?;
            }
            PomoStateMode::Running { counter } => {
                let excess_time = counter.elapsed().saturating_sub(target);
                let (_, message) = Self::pomodoro_alert_message(self.session.next().mode);

                queue!(
                    out,
                    MoveTo(0, 0),
                    Clear(ClearType::All),
                    Print(Self::break_title(self.session.next().mode)), MoveToNextLine(1),
                    Print(
                        format_args!(
                            "+{}",
                            format_duration(&excess_time)
                                .with(running_color(counter.started())),
                        ),
                    ), MoveToNextLine(1),
                    Print(Self::ENDING_CONTROLS), MoveToNextLine(1),
                    Print(message),
                )?;
            }
        }
        out.flush()?;
        Ok(())
    }
}

impl PomoState {
    fn elpased(&self) -> Duration {
        match self.mode {
            PomoStateMode::Running { counter } => counter.elapsed(),
            PomoStateMode::Skip { elapsed } => elapsed,
        }
    }

    fn target(&self) -> Duration {
        self.session.mode.get_time(&self.config)
    }
}

impl Alertable for PomoState {
    fn alert(&mut self) {
        let (title, message) = Self::pomodoro_alert_message(self.session.next().mode);
        alert(title, message);
    }

    fn alerted(&self) -> bool {
        self.alert
    }

    fn set_alert(&mut self, alert: bool) {
        self.alert = alert;
    }

    fn should_alert(&self) -> bool {
        self.elpased() > self.target()
    }
}
