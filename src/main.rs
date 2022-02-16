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
use anyhow::Result;
use clap::{AppSettings, Parser, Subcommand};

#[derive(Parser)]
#[clap(
    name = "Porsmo",
    author = "HellsNoah <hellsnoah@protonmail.com",
    version = "0.1.0",
    about = "Timer and Countdown and Pomodoro",
    long_about = None,
)]
#[clap(global_setting(AppSettings::PropagateVersion))]
#[clap(global_setting(AppSettings::UseLongFormatForHelpSubcommand))]
struct Cli {
    #[clap(subcommand, name = "mode")]
    mode: Modes,
}

#[derive(Subcommand)]
enum Modes {
    /// timer, counts up until you tell it to stop
    #[clap(name = "timer")]
    Timer {
        #[clap(default_value_t = 0)]
        /// Lets you start timer from a particular time
        time: u64,
    },
    /// countdown, counts down until you tell it to stop, or it ends
    #[clap(name = "cd")]
    Countdown {
        #[clap(default_value_t = 25*60)]
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
