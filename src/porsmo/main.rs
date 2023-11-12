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
use input::{get_event, Command, TIMEOUT};
use pomodoro::PomoState;
use porsmo::pomodoro::PomoConfig;
use prelude::*;
use std::io::Write;
use stopwatch::stopwatch;
use terminal::TerminalHandler;
use timer::timer;

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut terminal = TerminalHandler::new()?;
    let stdout = terminal.stdout();
    match args.mode {
        Some(CounterMode::Stopwatch { start_time }) => {
            stopwatch(stdout, start_time)?;
        }
        Some(CounterMode::Timer { target }) => {
            timer(stdout, target)?;
        }
        Some(CounterMode::Pomodoro { mode: PomoMode::Short }) => {
            PomoState::from(PomoConfig::short()).run_alerted(stdout)?;
        }
        Some(CounterMode::Pomodoro { mode: PomoMode::Long }) => {
            PomoState::from(PomoConfig::long()).run_alerted(stdout)?;
        }
        Some(CounterMode::Pomodoro {
            mode: PomoMode::Custom { work_time, break_time, long_break, },
        }) => {
            let config = PomoConfig::new(work_time, break_time, long_break);
            PomoState::from(config).run_alerted(stdout)?;
        }
        None => {
            PomoState::from(PomoConfig::short()).run_alerted(stdout)?;
        }
    }
    Ok(())
}

pub trait CounterUIState: Sized {
    fn show(&self, terminal: &mut impl Write) -> Result<()>;
    fn handle_command(self, cmd: Command) -> Option<Self>;
    fn run(mut self, stdout: &mut impl Write) -> Result<()> {
        loop {
            self.show(stdout)?;
            if let Some(cmd) = get_event(TIMEOUT)?.map(Command::from) {
                match self.handle_command(cmd) {
                    Some(new_state) => self = new_state,
                    None => return Ok(()),
                }
            }
        }
    }

    fn run_alerted(mut self, stdout: &mut impl Write) -> Result<()>
    where
        Self: Alertable,
    {
        loop {
            self.show(stdout)?;
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
