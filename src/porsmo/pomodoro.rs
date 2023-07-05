use crate::alert::Alert;
use crate::terminal::running_color;
use crate::{
    alert::alert,
    format::fmt_time,
    input::Command,
    terminal::TerminalHandler,
};
use crate::prelude::*;
use crossterm::style::Color;
use porsmo::counter::{DoubleEndedDuration, Counter};
use porsmo::pomodoro::{
    PomodoroMode as Mode,
    PomoConfig,
    PomodoroSession as Session,
};

use std::time::Duration;

pub struct PomodoroUI {
    counter: Counter,
    config: PomoConfig,
    session: Session,
    quit: bool,
    skip: bool,
    alert: Alert,
}

impl PomodoroUI {
    pub fn new(config: PomoConfig) -> Self {
        Self {
            counter: Counter::default().start(),
            session: Session::default(),
            alert: Alert::default(),
            config,
            quit: false,
            skip: false,
        }
    }

    pub fn from_secs(work_time: u64, break_time: u64, long_break: u64)
    -> Self {
        Self::new(PomoConfig {
            work_time: Duration::from_secs(work_time),
            break_time: Duration::from_secs(break_time),
            long_break: Duration::from_secs(long_break),
        })
    }

    pub fn ended(&self) -> bool {
        self.quit
    }

    pub fn time_left(&self) -> Duration {
        self.counter
            .saturating_time_left(self.session
                                      .mode()
                                      .initial(self.config))
    }

    pub fn excess_time_left(&self) -> DoubleEndedDuration {
        self.counter.checked_time_left(self.session.mode().initial(self.config))
    }

    pub fn quit(self) -> Self {
        Self { counter: self.counter.stop(), quit: true, ..self }
    }

    pub fn next_mode(self) -> Self {
        Self {
            counter: Counter::default().start(),
            session: self.session.next(),
            ..self
        }
    }

    pub fn check_next_mode(&self) -> Mode {
        self.session.next().mode()
    }

    const CONTROLS: &str = "[Q]: quit, [Shift S]: Skip, [Space]: pause/resume";
    const ENDING_CONTROLS: &str =
        "[Q]: quit, [Shift S]: Skip, [Space]: pause/resume, [Enter]: Next";
    const SKIP_CONTROLS: &str = "[Enter]: Yes, [Q/N]: No";

    pub fn show(&self, terminal: &mut TerminalHandler) -> Result<()> {
        if self.skip {
            terminal.clear()?;
            let message = format!("Round: {}", self.session.session());
            match self.check_next_mode() {
                Mode::Work =>
                    terminal
                        .set_foreground_color(Color::Red)?
                        .print("skip to work?")?
                        .info(message)?
                        .info(Self::SKIP_CONTROLS)?
                        .flush()?,
                Mode::Break =>
                    terminal
                        .set_foreground_color(Color::Green)?
                        .print("skip to break?")?
                        .info(message)?
                        .info(Self::SKIP_CONTROLS)?
                        .flush()?,
                Mode::LongBreak =>
                    terminal
                        .set_foreground_color(Color::Green)?
                        .print("skip to long break?")?
                        .info(message)?
                        .info(Self::SKIP_CONTROLS)?
                        .flush()?,
            };
            return Ok(())
        }

        match self.excess_time_left() {
            DoubleEndedDuration::Positive(elapsed) => {
                self.alert.reset();

                let title = match self.session.mode() {
                    Mode::Work => "Pomodoro (Work)",
                    Mode::Break => "Pomodoro (Break)",
                    Mode::LongBreak => "Pomodor (Long Break)",
                };

                terminal
                    .clear()?
                    .info(title)?
                    .set_foreground_color(running_color(self.counter.started()))?
                    .print(fmt_time(elapsed))?
                    .info(Self::CONTROLS)?
                    .status(format_args!("Round: {}", self.session.session()))?
                    .flush()?;
            }
            DoubleEndedDuration::Negative(elapsed) => {
                let (title, message) = Self::pomodoro_alert_message(
                    self.check_next_mode()
                );
                self.alert.alert(title, message);

                let title = match self.check_next_mode() {
                    Mode::Work => "Break has ended! Start work?",
                    Mode::Break => "Work has ended! Start break?",
                    Mode::LongBreak => "Work has ended! Start a long break",
                };

                terminal
                    .clear()?
                    .info(title)?
                    .set_foreground_color(running_color(self.counter.started()))?
                    .print(format_args!("+{}", fmt_time(elapsed)))?
                    .info(Self::ENDING_CONTROLS)?
                    .status(format!("Round: {}", self.session.session()))?
                    .flush()?;
            }
        }
        Ok(())
    }

    pub fn start_skip(self) -> Self {
        Self { counter: self.counter.stop(), skip: true, ..self }
    }

    pub fn cancel_skip(self) -> Self {
        Self { skip: false, ..self }
    }

    pub fn handle_command(mut self, command: Command) -> Self {
        match command {
            Command::Quit | Command::No if self.skip =>
                return self.cancel_skip(),

            Command::Enter | Command::Yes if self.skip =>
                return self.cancel_skip().next_mode(),

            Command::Enter if self.time_left().is_zero() =>
                return self.next_mode(),

            Command::Quit =>
                return self.quit(),

            Command::Pause =>
                self.counter = self.counter.stop(),

            Command::Resume =>
                self.counter = self.counter.start(),

            Command::Toggle =>
                self.counter = self.counter.toggle(),

            Command::Skip =>
                return self.start_skip(),

            _ => (),
        };
        self
    }

    pub fn pomodoro_alert_message(next_mode: Mode) -> (String, String) {
        let (heading, message) = match next_mode {
            Mode::Work => ("Your break ended!", "Time for some work"),
            Mode::Break => ("Pomodoro ended!", "Time for a short break"),
            Mode::LongBreak => ("Pomodoro 4 sessions complete!", "Time for a long break"),
        };

        (heading.into(), message.into())
    }
}


