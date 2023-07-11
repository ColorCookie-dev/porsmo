use crate::terminal::running_color;
use crate::{
    format::fmt_time,
    input::Command,
    terminal::{TerminalHandler, TerminalError},
};
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

    pub fn quit(self) -> Self {
        Self { counter: self.counter.stop(), quit: true, ..self }
    }

    pub fn ended(&self) -> bool {
        self.quit
    }

    pub fn handle_command(mut self, command: Command) -> Self {
        self.counter = match command {
            Command::Quit => return self.quit(),
            Command::Pause => self.counter.stop(),
            Command::Resume => self.counter.start(),
            Command::Toggle | Command::Enter => self.counter.toggle(),
            _ => self.counter,
        };
        self
    }

    pub fn show(&self, terminal: &mut TerminalHandler) -> Result<(), TerminalError> {
        terminal
            .clear()?
            .info("Stopwatch")?
            .set_foreground_color(running_color(self.counter.started()))?
            .print(fmt_time(self.counter.elapsed()))?
            .info("[Q]: quit, [Space]: pause/resume")?
            .flush()
    }
}
