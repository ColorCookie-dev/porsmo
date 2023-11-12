use crate::alert::alert;
use crate::input::{get_event, TIMEOUT};
use crate::stopwatch::Stopwatch;
use crate::terminal::running_color;
use crate::{format::format_duration, input::Command};
use crate::prelude::*;
use crossterm::terminal::{Clear, ClearType};
use crossterm::{
    cursor::{MoveTo, MoveToNextLine},
    queue,
    style::{Print, Stylize},
};
use std::io::Write;
use std::time::Duration;

pub fn timer(out: &mut impl Write, target: Duration) -> Result<()> {
    let mut stopwatch = Stopwatch::default();
    let mut alerted = false;

    loop {
        queue!(out, MoveTo(0, 0), Clear(ClearType::All),)?;
        let elapsed = stopwatch.elapsed();
        if elapsed < target {
            let time_left = target.saturating_sub(elapsed);
            queue!(
                out,
                Print("Timer"),
                MoveToNextLine(1),
                Print(format_duration(&time_left).with(running_color(stopwatch.started()))),
                MoveToNextLine(1),
                Print("[Q]: quit, [Space]: pause/resume"),
                MoveToNextLine(1)
            )?;
        } else {
            if !alerted {
                alerted = true;
                alert(
                    "The timer has ended!",
                    format!(
                        "Your Timer of {initial} has ended",
                        initial = format_duration(&target)
                    ),
                );
            }
            let excess_time = elapsed.saturating_sub(target);
            queue!(
                out,
                Print("Timer has ended"),
                MoveToNextLine(1),
                Print(
                    format!("+{}", format_duration(&excess_time))
                        .with(running_color(stopwatch.started()))
                ),
                MoveToNextLine(1),
                Print("[Q]: quit, [Space]: pause/resume"),
                MoveToNextLine(1)
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
