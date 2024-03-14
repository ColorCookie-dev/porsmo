use crate::alert::Alerter;
use crate::stopwatch::Stopwatch;
use crate::terminal::running_color;
use crate::{format::format_duration, input::Command};
use crate::{prelude::*, CounterUI};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{
    cursor::{MoveTo, MoveToNextLine},
    queue,
    style::{Print, Stylize},
};
use std::io::Write;
use std::time::Duration;

fn timer_show(
    out: &mut impl Write,
    elapsed: Duration,
    target: Duration,
    is_running: bool,
    alerter: &mut Alerter,
) -> Result<()> {
    let (title, timer, controls) = if elapsed < target {
        let time_left = target.saturating_sub(elapsed);
        (
            "Timer",
            format_duration(time_left).with(running_color(is_running)),
            "[Q]: quit, [Space]: pause/resume",
        )
    } else {
        alerter.alert_once(
            "The timer has ended!",
            format!(
                "Your Timer of {initial} has ended",
                initial = format_duration(target)
            ),
        );
        let excess_time = format_duration(elapsed.saturating_sub(target));
        (
            "Timer has ended",
            format!("+{excess_time}").with(running_color(is_running)),
            "[Q]: quit, [Space]: pause/resume",
        )
    };
    queue!(
        out,
        MoveTo(0, 0),
        Print(title),
        Clear(ClearType::UntilNewLine),
        MoveToNextLine(1),
        Print(timer),
        Clear(ClearType::UntilNewLine),
        MoveToNextLine(1),
        Print(controls),
        Clear(ClearType::UntilNewLine),
        MoveToNextLine(1),
        Clear(ClearType::FromCursorDown),
    )?;
    out.flush()?;
    Ok(())
}

fn timer_update(command: Command, stopwatch: &mut Stopwatch) {
    match command {
        Command::Pause => stopwatch.stop(),
        Command::Resume => stopwatch.start(),
        Command::Toggle | Command::Enter => stopwatch.toggle(),
        _ => (),
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TimerUI {
    stopwatch: Stopwatch,
    target: Duration,
    alerter: Alerter,
}

impl TimerUI {
    pub fn new(target: Duration) -> Self {
        Self { target, ..Default::default() }
    }
}

impl CounterUI for TimerUI {
    fn show(&mut self, out: &mut impl Write) -> Result<()> {
        let elapsed = self.stopwatch.elapsed();
        let is_running = self.stopwatch.started();
        timer_show(out, elapsed, self.target, is_running, &mut self.alerter)
    }

    fn update(&mut self, command: Command) {
        timer_update(command, &mut self.stopwatch)
    }
}

