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
            StopwatchState::new(start_time)
                .run(terminal)?;
        },
        Some(CounterMode::Timer { start_time: target }) => {
            let target = Duration::from_secs(target);
            let start_time = Duration::ZERO;
            TimerState::new(start_time, target)
                .run_alerted(terminal)?;
        },
        Some(CounterMode::Pomodoro {mode: Some(PomoMode::Short) | None}) => {
            PomoState::from(PomoConfig::short())
                .run_alerted(terminal)?;
        },
        Some(CounterMode::Pomodoro {mode: Some(PomoMode::Long)}) => {
            PomoState::from(PomoConfig::long())
                .run_alerted(terminal)?;
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
            PomoState::from(config)
                .run_alerted(terminal)?;
        },
        None => {
            PomoState::from(PomoConfig::short())
                .run_alerted(terminal)?;
        }
    }
    Ok(())
}

pub trait CounterUIState: Sized {
    fn show(&self, terminal: &mut TerminalHandler) -> Result<()>;
    fn handle_command(self, cmd: Command) -> Option<Self>;
    fn run(mut self, terminal: &mut TerminalHandler) -> Result<()> {
        loop {
            self.show(terminal)?;
            if let Some(cmd) = get_event(TIMEOUT)?.map(Command::from) {
                match self.handle_command(cmd) {
                    Some(new_state) => self = new_state,
                    None => return Ok(()),
                }
            }
        }
    }

    fn run_alerted(mut self, terminal: &mut TerminalHandler) -> Result<()>
    where Self: Alertable
    {
        loop {
            self.show(terminal)?;
            if self.should_alert() && !self.alerted() {
                self.set_alert(true);
                self.alert();
            }
            if let Some(cmd) = get_event(TIMEOUT)?.map(Command::from) {
                match self.handle_command(cmd) {
                    Some(new_state) => self = new_state,
                    None => return Ok(()),
                }
            }
        }
    }
}

pub trait Alertable {
    fn alerted(&self) -> bool;
    fn set_alert(&mut self, alert: bool);
    fn should_alert(&self) -> bool;
    fn alert(&mut self);
}

