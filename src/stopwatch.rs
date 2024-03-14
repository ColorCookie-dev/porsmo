use std::time::Instant;
use std::{io::Write, time::Duration};

use crate::{prelude::*, CounterUI};
use crate::terminal::running_color;
use crate::{format::format_duration, input::Command};
use crossterm::{
    cursor::{MoveTo, MoveToNextLine},
    queue,
    style::{Print, Stylize},
    terminal::{Clear, ClearType},
};

#[derive(Debug, Clone, Copy)]
pub struct Stopwatch {
    start_time: Option<Instant>,
    elapsed_before: Duration,
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self {
            start_time: Some(Instant::now()),
            elapsed_before: Duration::ZERO,
        }
    }
}

impl Stopwatch {
    pub fn new(start_time: Option<Instant>, elapsed_before: Duration) -> Self {
        Self {
            start_time,
            elapsed_before,
        }
    }

    pub fn elapsed(&self) -> Duration {
        match self.start_time {
            Some(start_time) => self.elapsed_before + start_time.elapsed(),
            None => self.elapsed_before,
        }
    }

    pub fn started(&self) -> bool {
        if matches!(self.start_time, None) {
            false
        } else {
            true
        }
    }

    pub fn start(&mut self) {
        if matches!(self.start_time, None) {
            self.start_time = Some(Instant::now());
        }
    }

    pub fn stop(&mut self) {
        if let Some(start_time) = self.start_time {
            self.elapsed_before += start_time.elapsed();
            self.start_time = None;
        }
    }

    pub fn toggle(&mut self) {
        match self.start_time {
            Some(start_time) => {
                self.elapsed_before += start_time.elapsed();
                self.start_time = None;
            }
            None => {
                self.start_time = Some(Instant::now());
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StopwatchUI {
    stopwatch: Stopwatch,
}

impl CounterUI for StopwatchUI {
    fn show(&mut self, out: &mut impl Write) -> Result<()> {
        let elapsed = self.stopwatch.elapsed();
        let is_running = self.stopwatch.started();
        queue!(
            out,
            MoveTo(0, 0),
            Print("Stopwatch"),
            Clear(ClearType::UntilNewLine),
            MoveToNextLine(1),
            Print(format_duration(elapsed).with(running_color(is_running))),
            Clear(ClearType::UntilNewLine),
            MoveToNextLine(1),
            Print("[Q]: quit, [Space]: pause/resume"),
            Clear(ClearType::FromCursorDown),
        )?;
        out.flush()?;
        Ok(())
    }

    fn update(&mut self, command: Command) {
        match command {
            Command::Pause => self.stopwatch.stop(),
            Command::Resume => self.stopwatch.start(),
            Command::Toggle | Command::Enter => self.stopwatch.toggle(),
            _ => (),
        }
    }
}

