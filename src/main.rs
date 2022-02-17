mod countdown;
mod counter;
mod format;
mod input;
mod notification;
mod sound;
mod terminal;
mod timer;

use crate::format::fmt_time;
use crate::{countdown::Countdown, counter::Counter, timer::Timer};
use anyhow::{bail, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(
    name = "Porsmo",
    author = "HellsNoah <hellsnoah@protonmail.com",
    version = "0.1.0",
    about = "Timer and Countdown and Pomodoro",
    long_about = None,
)]
struct Cli {
    #[clap(subcommand, name = "mode")]
    mode: Modes,
}

#[derive(Subcommand)]
enum Modes {
    /// timer, counts up until you tell it to stop
    #[clap(name = "timer")]
    Timer {
        #[clap(parse(try_from_str=parse_time),
               default_value_t = 0,
               value_name = "time")]
        /// Lets you start timer from a particular time
        time: u64,
    },
    /// countdown, counts down until you tell it to stop, or it ends
    #[clap(name = "cd")]
    Countdown {
        #[clap(parse(try_from_str=parse_time),
               default_value_t = 25*60,
               value_name = "time")]
        time: u64,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.mode {
        Modes::Timer { time } => Timer::new(time)
            .count()
            .map(|counter| println!("{}", fmt_time(counter))),
        Modes::Countdown { time } => Countdown::new(time)
            .count()
            .map(|counter| println!("{}", fmt_time(counter))),
    }
}

fn parse_time(time_str: &str) -> Result<u64> {
    let mut secs = 0u64;

    for (i, e) in time_str.split(':').rev().enumerate() {
        if e.is_empty() {
            continue;
        }

        let en = e.parse::<u64>()?;

        if i == 0 {
            secs += en;
        } else if i == 1 {
            secs += en * 60;
        } else if i == 2 {
            secs += en * 60 * 60;
        } else if i == 3 {
            secs += en * 3600 * 24;
        } else {
            bail!("Bad number of ':'");
        }
    }

    Ok(secs)
}
