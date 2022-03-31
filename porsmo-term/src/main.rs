#[macro_use]
mod terminal;

mod input;
mod pomodoro;
mod stopwatch;
mod timer;

use crate::{pomodoro::pomodoro, stopwatch::stopwatch, timer::timer};
use anyhow::Result;
use clap::{Parser, Subcommand};
use porsmo_helpers::parse_time;
use std::time::Duration;

#[derive(Parser)]
#[clap(
    name = "Porsmo",
    author = "HellsNoah <hellsnoah@protonmail.com",
    version = "0.1.3",
    about = "Timer and Stopwatch and Pomodoro",
    long_about = None,
)]
struct Cli {
    #[clap(subcommand, name = "mode")]
    mode: Option<Modes>,
}

#[derive(Subcommand)]
enum Modes {
    /// stopwatch, counts up until you tell it to stop
    #[clap(name = "stopwatch", alias = "s")]
    Stopwatch {
        #[clap(parse(try_from_str=parse_time), value_name = "time")]
        /// Lets you start timer from a particular time, default: 0 secs
        time: Option<Duration>,
    },
    /// timer, counts down until you tell it to stop, or it ends
    #[clap(name = "timer", alias = "t")]
    Timer {
        #[clap(parse(try_from_str=parse_time), value_name = "time")]
        /// Lets you start timer from a particular time, default: 25 mins
        time: Option<Duration>,
    },
    /// pomodoro, for all you productivity needs (default)
    #[clap(name = "pomodoro", alias = "p")]
    Pomodoro {
        #[clap(subcommand, name = "mode")]
        /// Modes: short (25, 5, 10), long (55, 10, 20), custom
        mode: Option<PomoMode>,
    },
}

#[derive(Subcommand)]
enum PomoMode {
    /// short pomodoro, with 25, 5, 10 min values (default)
    #[clap(name = "short", alias = "-s")]
    Short,
    /// long pomodoro, with 55, 10, 20 min values
    #[clap(name = "long", alias = "-l")]
    Long,
    /// custom pomodoro, with any specified values
    #[clap(name = "custom", alias = "-c")]
    Custom {
        #[clap(parse(try_from_str=parse_time),
               value_name = "work time")]
        work_time: Duration,
        #[clap(parse(try_from_str=parse_time),
               value_name = "break time")]
        break_time: Duration,
        #[clap(parse(try_from_str=parse_time),
               value_name = "long break time")]
        long_break_time: Duration,
    },
}

macro_rules! minutes {
    ($t:expr) => {
        Duration::from_secs($t * 60)
    };
}

#[macro_export]
macro_rules! program_tick_duration {
    () => {
        Duration::from_millis(25)
    };
}

fn main() -> Result<()> {
    log4rs::init_file("logging_config.yaml", Default::default()).unwrap();

    let args = Cli::parse();
    match args.mode {
        Some(Modes::Stopwatch { time: None }) => stopwatch(minutes!(0)),
        Some(Modes::Stopwatch { time: Some(time) }) => stopwatch(time),

        Some(Modes::Timer { time: None }) => timer(minutes!(25)),
        Some(Modes::Timer { time: Some(time) }) => timer(time),

        None
        | Some(Modes::Pomodoro { mode: None })
        | Some(Modes::Pomodoro {
            mode: Some(PomoMode::Short),
        }) => pomodoro(minutes!(25), minutes!(5), minutes!(10)),

        Some(Modes::Pomodoro {
            mode: Some(PomoMode::Long),
        }) => pomodoro(minutes!(55), minutes!(10), minutes!(20)),

        Some(Modes::Pomodoro {
            mode:
                Some(PomoMode::Custom {
                    work_time,
                    break_time,
                    long_break_time,
                }),
        }) => pomodoro(work_time, break_time, long_break_time),
    }
}
