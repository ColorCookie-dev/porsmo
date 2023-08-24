use std::ops::ControlFlow;
use std::time::Duration;

use crate::input::{CommandIter, get_event, TIMEOUT};
use crate::prelude::*;
use crate::terminal::running_color;
use crate::{
    format::fmt_time,
    input::Command,
    terminal::TerminalHandler,
};
use porsmo::counter::Counter as Stopwatch;

#[derive(Debug)]
pub struct StopwatchState {
    pub counter: Stopwatch,
}

impl StopwatchState {
    pub fn show(
        &self,
        terminal: &mut TerminalHandler,
    ) -> Result<()> {
        terminal
            .clear()?
            .info("Stopwatch")?
            .set_foreground_color(running_color(self.counter.started()))?
            .print(fmt_time(self.counter.elapsed()))?
            .info("[Q]: quit, [Space]: pause/resume")?
            .flush()
    }

    pub fn handle_command(self, command: Command) -> Option<Self> {
        let Self { counter } = self;
        match command {
            Command::Quit => None,
            Command::Pause => Some(Self { counter: counter.stop(), }),
            Command::Resume => Some(Self { counter: counter.start(), }),
            Command::Toggle | Command::Enter =>
                Some(Self { counter: counter.toggle(), }),
            _ => Some(Self { counter }),
        }
    }

    pub fn run(
        terminal: &mut TerminalHandler,
        start_time: Duration
    ) -> Result<()> {
        let counter = Stopwatch::from(start_time).start();
        let mut state = StopwatchState { counter };
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
}
