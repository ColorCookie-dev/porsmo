use crate::terminal::running_color;
use crate::{
    format::fmt_time,
    input::Command,
    terminal::TerminalHandler,
};
use crate::prelude::*;
use porsmo::counter::Counter as Stopwatch;
use std::time::Duration;

pub struct StopwatchUI {
    counter: Stopwatch,
    quit: bool,
}

impl StopwatchUI {
    pub fn new(time: Duration) -> Self {
        Self { counter: Stopwatch::new(None, time).start(), quit: false }
    }
}

impl CounterApp for StopwatchUI {
    type Output = Self;

    fn quit(self) -> Self::Output {
        Self { counter: self.counter.stop(), quit: true, ..self }
    }

    fn ended(&self) -> bool {
        self.quit
    }

    fn handle_command(mut self, command: Command) -> Self::Output {
        self.counter = match command {
            Command::Quit => {
                self.quit = true;
                self.counter.stop()
            },
            Command::Pause => self.counter.stop(),
            Command::Resume => self.counter.start(),
            Command::Toggle | Command::Enter => self.counter.toggle(),
            _ => self.counter,
        };
        self
    }

    fn show(&self, terminal: &mut TerminalHandler) -> Result<()> {
        terminal
            .clear()?
            .info("Stopwatch")?
            .set_foreground_color(running_color(self.counter.started()))?
            .print(fmt_time(self.counter.elapsed()))?
            .info("[Q]: quit, [Space]: pause/resume")?
            .flush()
    }
}

pub trait CounterApp {
    type Output;
    fn show(&self, terminal: &mut TerminalHandler) -> Result<()>;
    fn ended(&self) -> bool;
    fn quit(self) -> Self::Output;
    fn handle_command(self, command: Command) -> Self::Output;
}
