mod alert;
mod cli;
mod error;
mod format;
mod input;
mod pomodoro;
mod prelude;
mod stopwatch;
mod terminal;
mod timer;

use clap::Parser;
use cli::{Cli, CounterMode, PomoMode};
use pomodoro::pomodoro;
use crate::pomodoro::PomoConfig;
use prelude::*;
use stopwatch::stopwatch;
use terminal::TerminalHandler;
use timer::timer;

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut terminal = TerminalHandler::new()?;
    let stdout = terminal.stdout();
    match args.mode {
        Some(CounterMode::Stopwatch { start_time }) => stopwatch(stdout, start_time)?,
        Some(CounterMode::Timer { target }) => timer(stdout, target)?,
        Some(CounterMode::Pomodoro {
            mode: PomoMode::Short,
        }) => pomodoro(stdout, &PomoConfig::short())?,
        Some(CounterMode::Pomodoro {
            mode: PomoMode::Long,
        }) => pomodoro(stdout, &PomoConfig::long())?,
        Some(CounterMode::Pomodoro {
            mode:
                PomoMode::Custom {
                    work_time,
                    break_time,
                    long_break,
                },
        }) => pomodoro(stdout, &PomoConfig::new(work_time, break_time, long_break))?,
        None => pomodoro(stdout, &PomoConfig::short())?,
    }
    Ok(())
}
