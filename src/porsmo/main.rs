mod prelude;
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
use crate::prelude::*;
use crossterm::event;
use input::Command;
use porsmo::pomodoro::PomoConfig;
use terminal::TerminalHandler;
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
    mode: Option<Modes>,
}

#[derive(Subcommand, Debug)]
enum Modes {
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

fn main() -> Result<()> {
    let args = Cli::parse();
    let pomodoro_short = || PorsmoUI::pomodoro(PomoConfig::short());
    let pomodoro_long  = || PorsmoUI::pomodoro(PomoConfig::long());

    let mut app = match args.mode {
        Some(Modes::Stopwatch { start_time }) =>
            PorsmoUI::stopwatch(Duration::from_secs(start_time)),
        Some(Modes::Timer { start_time }) =>
            PorsmoUI::timer(Duration::from_secs(start_time)),
        Some(Modes::Pomodoro {mode: Some(PomoMode::Short) | None}) =>
            pomodoro_short(),
        Some(Modes::Pomodoro {mode: Some(PomoMode::Long)}) =>
            pomodoro_long(),
        Some(Modes::Pomodoro {
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
        None => pomodoro_short(),
    };

    let mut terminal = TerminalHandler::new()?;
    while !app.ended() {
        app.show(&mut terminal)?;
        match get_event(Duration::from_millis(250))?.map(Command::from) {
            Some(command) => app = app.handle_command(command),
            None => (),
        };
    }

    Ok(())
}

pub fn get_event(timeout: Duration) -> Result<Option<event::Event>> {
    if event::poll(timeout)
        .with_context(|| "Polling failed")? {
        Ok(Some(event::read().with_context(|| "Failed to read event")?))
    } else {
        Ok(None)
    }
}
