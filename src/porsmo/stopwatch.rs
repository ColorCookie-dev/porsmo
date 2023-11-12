use std::{time::Duration, io::Write};

use crate::terminal::running_color;
use crate::{format::format_duration, input::Command};
use crate::{prelude::*, CounterUIState};
use porsmo::counter::Counter as Stopwatch;
use crossterm::{
    queue,
    cursor::{
        MoveTo,
        MoveToNextLine,
    },
    terminal::{
        ClearType,
        Clear,
    },
    style::{
        Print,
        Stylize,
    },
};

#[derive(Debug)]
pub struct StopwatchState {
    pub counter: Stopwatch,
}

impl StopwatchState {
    pub fn new(start_time: Duration) -> Self {
        let counter = Stopwatch::from(start_time).start();
        Self { counter }
    }
}

impl CounterUIState for StopwatchState {
    fn show(&self, out: &mut impl Write) -> Result<()> {
        const CONTROL: &str = "[Q]: quit, [Space]: pause/resume";
        queue!(
            out,
            MoveTo(0, 0),
            Clear(ClearType::All),
            Print("Stopwatch"), MoveToNextLine(1),
            Print(
                format_duration(&self.counter.elapsed())
                    .with(running_color(self.counter.started()))
            ), MoveToNextLine(1),
            Print(CONTROL),
        )?;
        out.flush()?;
        Ok(())
    }

    fn handle_command(self, cmd: Command) -> Option<Self> {
        let Self { counter } = self;
        match cmd {
            Command::Quit => None,
            Command::Pause => Some(counter.stop()),
            Command::Resume => Some(counter.start()),
            Command::Toggle | Command::Enter => Some(counter.toggle()),
            _ => Some(counter),
        }
        .map(|counter| Self { counter })
    }
}
