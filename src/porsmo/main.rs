mod prelude;
mod error;
mod alert;
mod format;
mod input;
mod pomodoro;
mod stopwatch;
mod terminal;
mod timer;
mod cli;

use pomodoro::PomoState;
use prelude::*;
use stopwatch::StopwatchState;
use timer::TimerState;
use std::time::Duration;
use porsmo::pomodoro::PomoConfig;
use terminal::TerminalHandler;
use cli::{Cli, CounterMode, PomoMode};
use clap::Parser;

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut terminal = TerminalHandler::new()?;
    match args.mode {
        Some(CounterMode::Stopwatch { start_time }) =>
            StopwatchState::run(&mut terminal, Duration::from_secs(start_time))?,
        Some(CounterMode::Timer { start_time }) =>
            TimerState::run(&mut terminal, Duration::from_secs(start_time))?,
        Some(CounterMode::Pomodoro {mode: Some(PomoMode::Short) | None}) =>
            PomoState::run(&mut terminal, PomoConfig::short())?,
        Some(CounterMode::Pomodoro {mode: Some(PomoMode::Long)}) =>
            PomoState::run(&mut terminal, PomoConfig::long())?,
        Some(CounterMode::Pomodoro {
            mode: Some(
                      PomoMode::Custom {
                          work_time,
                          break_time,
                          long_break_time
                      }
                  )
        }) => PomoState::run(
                &mut terminal,
                PomoConfig::new(
                    Duration::from_secs(work_time),
                    Duration::from_secs(break_time),
                    Duration::from_secs(long_break_time),
                )
            )?,
        None => PomoState::run(&mut terminal, PomoConfig::short())?,
    }
    Ok(())
}

