use std::time::Instant;
use std::{time::Duration, io::Write};

use crate::input::{get_event, TIMEOUT};
use crate::terminal::running_color;
use crate::{format::format_duration, input::Command};
use crate::{prelude::*, CounterUIState};
use crossterm::{
    queue,
    cursor::{ MoveTo, MoveToNextLine, },
    terminal::{ ClearType, Clear, },
    style::{ Print, Stylize, },
};
use porsmo::counter::Counter;

#[derive(Debug)]
pub struct StopwatchState {
    pub counter: Counter,
}

impl StopwatchState {
    pub fn new(start_time: Duration) -> Self {
        let counter = Counter::from(start_time).start();
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

struct Stopwatch {
    start_time: Option<Instant>,
    elapsed_before: Duration,
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self {
            start_time: Some(Instant::now()),
            elapsed_before: Duration::ZERO
        }
    }
}

impl Stopwatch {
    fn new(start_time: Option<Instant>, elapsed_before: Duration) -> Self {
        Self { start_time, elapsed_before }
    }

    fn elapsed(&self) -> Duration {
        match self.start_time {
            Some(start_time) => self.elapsed_before + start_time.elapsed(),
            None => self.elapsed_before,
        }
    }

    fn started(&self) -> bool {
        if matches!(self.start_time, None) {
            false
        } else {
            true
        }
    }

    fn start(&mut self) {
        if matches!(self.start_time, None) {
            self.start_time = Some(Instant::now());
        }
    }

    fn stop(&mut self) {
        if let Some(start_time) = self.start_time {
            self.elapsed_before += start_time.elapsed();
            self.start_time = None;
        }
    }

    fn toggle(&mut self) {
        match self.start_time {
            Some(start_time) => {
                self.elapsed_before += start_time.elapsed();
                self.start_time = None;
            },
            None => {
                self.start_time = Some(Instant::now());
            }
        }
    }
}

pub fn stopwatch_ui(out: &mut impl Write, start_time: Duration) -> Result<()> {
    let mut counter = Stopwatch::new(Some(Instant::now()), start_time);
    loop {
        queue!(
            out,
            MoveTo(0, 0),
            Clear(ClearType::All),
            Print("Stopwatch"), MoveToNextLine(1),
            Print(
                format_duration(&counter.elapsed())
                    .with(running_color(counter.started()))
            ), MoveToNextLine(1),
            Print("[Q]: quit, [Space]: pause/resume"),
        )?;
        out.flush()?;
        if let Some(cmd) = get_event(TIMEOUT)?.map(Command::from) {
            match cmd {
                Command::Quit => break,
                Command::Pause => counter.stop(),
                Command::Resume => counter.start(),
                Command::Toggle | Command::Enter => counter.toggle(),
                _ => (),
            }
        }
    }

    Ok(())
}
