use crate::alert::alert;
use crate::input::{get_event, TIMEOUT};
use crate::terminal::running_color;
use crate::{format::format_duration, input::Command};
use crate::{prelude::*, Alertable, CounterUIState};
use crossterm::terminal::{Clear, ClearType};
use porsmo::counter::Counter;
use std::io::Write;
use std::time::Duration;
use crate::stopwatch::Stopwatch;
use crossterm::{
    queue,
    style::{Stylize, Print},
    cursor::{
        MoveToNextLine,
        MoveTo,
    },
};

pub fn timer(out: &mut impl Write, target: Duration) -> Result<()> {
    let mut stopwatch = Stopwatch::default();
    loop {
        queue!(
            out,
            MoveTo(0, 0),
            Clear(ClearType::All),
        )?;
        let elapsed = stopwatch.elapsed();
        if elapsed < target {
            let time_left = target.saturating_sub(elapsed);
            queue!(
                out,
                Print("Timer"), MoveToNextLine(1),
                Print(format_duration(&time_left)
                          .with(running_color(stopwatch.started()))),
                MoveToNextLine(1),
                Print("[Q]: quit, [Space]: pause/resume"),
                MoveToNextLine(1)
            )?;
        } else {
            let excess_time = elapsed.saturating_sub(target);
            queue!(
                out,
                Print("Timer has ended"), MoveToNextLine(1),
                Print(
                    format!("+{}", format_duration(&excess_time))
                        .with(running_color(stopwatch.started()))
                ), MoveToNextLine(1),
                Print("[Q]: quit, [Space]: pause/resume"), MoveToNextLine(1)
            )?;
        }
        out.flush()?;
        if let Some(cmd) = get_event(TIMEOUT)?.map(Command::from) {
            match cmd {
                Command::Quit => break,
                Command::Pause => stopwatch.stop(),
                Command::Resume => stopwatch.start(),
                Command::Toggle | Command::Enter => stopwatch.toggle(),
                _ => (),
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct TimerState {
    pub counter: Counter,
    pub target: Duration,
    pub alert: bool,
}

impl TimerState {
    pub fn new(target: Duration) -> Self {
        Self {
            counter: Counter::default(),
            target,
            alert: false,
        }
    }
}

impl CounterUIState for TimerState {
    fn show(&self, out: &mut impl Write) -> Result<()> {
        let elapsed = self.counter.elapsed();
        if elapsed < self.target {
            let time_left = self.target.saturating_sub(elapsed);

            queue!(
                out,
                MoveTo(0, 0),
                Clear(ClearType::All),
                Print("Timer"), MoveToNextLine(1),
                Print(format_duration(&time_left)
                      .with(running_color(self.counter.started()))),
                MoveToNextLine(1),
                Print("[Q]: quit, [Space]: pause/resume"),
                MoveToNextLine(1)
            )?;
        } else {
            let excess_time = elapsed.saturating_sub(self.target);
            queue!(
                out,
                MoveTo(0, 0),
                Clear(ClearType::All),
                Print("Timer has ended"), MoveToNextLine(1),
                Print(
                    format_args!(
                        "+{}",
                        format_duration(&excess_time)
                            .with(running_color(self.counter.started()))
                    )
                ), MoveToNextLine(1),
                Print("[Q]: quit, [Space]: pause/resume"), MoveToNextLine(1)
            )?;
        }
        out.flush()?;
        Ok(())
    }

    fn handle_command(self, command: Command) -> Option<Self> {
        match command {
            Command::Quit => None,
            Command::Pause => Some(Self {
                counter: self.counter.stop(),
                ..self
            }),
            Command::Resume => Some(Self {
                counter: self.counter.start(),
                ..self
            }),
            Command::Toggle | Command::Enter => Some(Self {
                counter: self.counter.toggle(),
                ..self
            }),
            _ => Some(self),
        }
    }
}

impl Alertable for TimerState {
    fn alerted(&self) -> bool {
        self.alert
    }

    fn set_alert(&mut self, alert: bool) {
        self.alert = alert;
    }

    fn should_alert(&self) -> bool {
        self.counter.elapsed() > self.target
    }

    fn alert(&mut self) {
        let title = "The timer has ended!";
        let message = format!(
            "Your Timer of {initial} has ended",
            initial = format_duration(&self.target)
        );

        alert(title, message);
        self.alert = true;
    }
}
