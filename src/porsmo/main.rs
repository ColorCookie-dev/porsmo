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

use input::{Command, get_event, TIMEOUT};
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
    let terminal = &mut terminal;
    match args.mode {
        Some(CounterMode::Stopwatch { start_time }) => {
            let start_time = Duration::from_secs(start_time);
            run_counter_ui_state(StopwatchState::new(start_time), terminal)?;
        },
        Some(CounterMode::Timer { start_time: target }) => {
            let target = Duration::from_secs(target);
            let start_time = Duration::ZERO;
            let state = TimerState::new(start_time, target);
            run_counter_ui_state(state, terminal)?;
        },
        Some(CounterMode::Pomodoro {mode: Some(PomoMode::Short) | None}) => {
            let state = PomoState::from(PomoConfig::short());
            run_counter_ui_state(state, terminal)?;
        },
        Some(CounterMode::Pomodoro {mode: Some(PomoMode::Long)}) => {
            let state = PomoState::from(PomoConfig::long());
            run_counter_ui_state(state, terminal)?;
        },
        Some(CounterMode::Pomodoro {
            mode: Some(
                      PomoMode::Custom {
                          work_time,
                          break_time,
                          long_break_time: long_break
                      }
                  )
        }) => {
            let work_time = Duration::from_secs(work_time);
            let break_time = Duration::from_secs(break_time);
            let long_break = Duration::from_secs(long_break);

            let config = PomoConfig::new(work_time, break_time, long_break);
            let state = PomoState::from(config);
            run_counter_ui_state(state, terminal)?;
        },
        None => {
            let state = PomoState::from(PomoConfig::short());
            run_counter_ui_state(state, terminal)?;
        }
    }
    Ok(())
}

pub trait CounterUIState: Sized {
    fn show(&self, terminal: &mut TerminalHandler) -> Result<()>;
    fn handle_command(self, cmd: Command) -> Option<Self>;
}

pub fn run_counter_ui_state(
    mut state: impl CounterUIState,
    terminal: &mut TerminalHandler,
) -> Result<()> {
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
