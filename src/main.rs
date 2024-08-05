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

use crate::input::{get_event, Command, TIMEOUT};
use crate::pomodoro::PomodoroConfig;
use clap::Parser;
use cli::{Cli, CounterMode, PomoMode};
use pomodoro::PomodoroUI;
use prelude::*;
use std::io::Write;
use stopwatch::StopwatchUI;
use terminal::TerminalHandler;
use timer::TimerUI;

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut terminal = TerminalHandler::new()?;
    let stdout = terminal.stdout();
    let exitmessagestring = match args.mode {
        Some(CounterMode::Stopwatch) => StopwatchUI::default().run_ui(stdout)?,
        Some(CounterMode::Timer { target }) => TimerUI::new(target).run_ui(stdout)?,
        Some(CounterMode::Pomodoro {
            mode: PomoMode::Short,
            exitmessage: _,
        }) => PomodoroUI::new(PomodoroConfig::short()).run_ui(stdout)?,
        Some(CounterMode::Pomodoro {
            mode: PomoMode::Long,
            exitmessage: _,
        }) => PomodoroUI::new(PomodoroConfig::long()).run_ui(stdout)?,
        Some(CounterMode::Pomodoro {
            mode:
                PomoMode::Custom {
                    work_time,
                    break_time,
                    long_break,
                },
            exitmessage: _,
        }) => PomodoroUI::new(PomodoroConfig::new(work_time, break_time, long_break))
            .run_ui(stdout)?,
        None => PomodoroUI::new(PomodoroConfig::short()).run_ui(stdout)?,
    };
    drop(terminal);
    if matches!(
        args.mode,
        Some(CounterMode::Pomodoro {
            mode: _,
            exitmessage: true
        })
    ) {
        println!("{}", exitmessagestring);
    }
    Ok(())
}

pub trait CounterUI: Sized {
    fn show(&mut self, out: &mut impl Write) -> Result<()>;
    fn update(&mut self, command: Command);
    fn run_ui(mut self, out: &mut impl Write) -> Result<String> {
        loop {
            self.show(out)?;
            if let Some(cmd) = get_event(TIMEOUT)?.map(Command::from) {
                match cmd {
                    Command::Quit => break,
                    cmd => self.update(cmd),
                }
            }
        }
        Ok(String::new())
    }
}
