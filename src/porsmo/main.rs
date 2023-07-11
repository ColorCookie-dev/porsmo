mod alert;
mod format;
mod input;
mod pomodoro;
mod stopwatch;
mod terminal;
mod timer;
mod app;

use std::time::Duration;
use app::PorsmoUI;
use crate::format::parse_time;
use crossterm::event;
use input::Command;
use porsmo::pomodoro::PomoConfig;
use terminal::{TerminalHandler, TerminalError};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "Porsmo",
    author = "HellsNoah <hellsnoah@protonmail.com>",
    version = "0.2.2",
    about = "Timer and Stopwatch and Pomodoro",
    long_about = None,
)]
struct Cli {
    #[command(subcommand, name = "mode")]
    mode: Option<CounterMode>,
}

#[derive(Subcommand, Debug)]
enum CounterMode {
    /// alias: s, stopwatch, counts up until you tell it to stop
    #[command(name = "stopwatch", alias = "s")]
    Stopwatch {
        #[arg(value_parser = parse_time,
               default_value_t = 0,
               value_name = "time")]
        /// Lets you start timer from a particular time
        start_time: u64,
    },
    /// alias: t, timer, counts down until you tell it to stop, or it ends
    #[command(name = "timer", alias = "t")]
    Timer {
        #[arg(value_parser = parse_time,
               default_value_t = 25*60,
               value_name = "time")]
        start_time: u64,
    },
    /// alias: p, pomodoro, for all you productivity needs (default)
    #[command(name = "pomodoro", alias = "p")]
    Pomodoro {
        #[clap(subcommand, name = "mode")]
        mode: Option<PomoMode>,
    },
}

#[derive(Subcommand, Debug)]
enum PomoMode {
    /// alias: s, short pomodoro, with 25, 5, 10 min values (default)
    #[command(name = "short", alias = "s")]
    Short,
    /// alias: l, long pomodoro, with 55, 10, 20 min values
    #[command(name = "long", alias = "l")]
    Long,
    /// alias: c, custom pomodoro, with any specified values
    #[command(name = "custom", alias = "c")]
    Custom {
        #[arg(value_parser = parse_time, value_name = "work-time")]
        work_time: u64,
        #[arg(value_parser = parse_time, value_name = "break-time")]
        break_time: u64,
        #[arg(value_parser = parse_time, value_name = "long-break-time")]
        long_break_time: u64,
    },
}

#[derive(Debug, thiserror::Error)]
enum Errors {
    #[error(transparent)]
    TerminalError(#[from] TerminalError),

    #[error(transparent)]
    InputEventError(#[from] InputEventError),
}

fn main() -> Result<(), Errors> {
    let args = Cli::parse();
    let mut app = get_ui_from_counter_mode(args.mode);
    let mut terminal = TerminalHandler::new()?;

    while !app.ended() {
        app.show(&mut terminal)?;

        match get_event(Duration::from_millis(250)) {
            Ok(command) => app = app.handle_command(Command::from(command)),
            Err(InputEventError::NoEventsToPoll) => (),
            Err(e) => return Err(e.into()),
        };
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum InputEventError {
    #[error("Polling events failed")]
    PollFailed(#[source] crossterm::ErrorKind),

    #[error("Reading events failed")]
    EventReadFailed(#[source] crossterm::ErrorKind),

    #[error("Currently there are no new events")]
    NoEventsToPoll,
}

pub fn get_event(timeout: Duration)
-> Result<event::Event, InputEventError> {
    if event::poll(timeout)
        .map_err(|e| InputEventError::PollFailed(e))? {
        Ok (event::read().map_err(|e| InputEventError::EventReadFailed(e))?)
    } else {
        Err(InputEventError::NoEventsToPoll)
    }
}

fn get_ui_from_counter_mode(mode: Option<CounterMode>) -> PorsmoUI {
    match mode {
        Some(CounterMode::Stopwatch { start_time }) =>
            PorsmoUI::stopwatch(Duration::from_secs(start_time)),
        Some(CounterMode::Timer { start_time }) =>
            PorsmoUI::timer(Duration::from_secs(start_time)),
        Some(CounterMode::Pomodoro {mode: Some(PomoMode::Short) | None}) =>
            PorsmoUI::pomodoro(PomoConfig::short()),
        Some(CounterMode::Pomodoro {mode: Some(PomoMode::Long)}) =>
            PorsmoUI::pomodoro(PomoConfig::long()),
        Some(CounterMode::Pomodoro {
            mode: Some(
                      PomoMode::Custom {
                          work_time,
                          break_time,
                          long_break_time
                      }
                  )
        }) => PorsmoUI::pomodoro(
                PomoConfig::new(
                    Duration::from_secs(work_time),
                    Duration::from_secs(break_time),
                    Duration::from_secs(long_break_time),
                )
            ),
        None => PorsmoUI::pomodoro(PomoConfig::short()),
    }
}

