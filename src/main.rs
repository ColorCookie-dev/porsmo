mod countdown;
mod counter;
mod format;
mod input;
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
struct App {
    #[clap(subcommand)]
    mode: Modes,
}

#[derive(Subcommand)]
enum Modes {
    Timer,
    Countdown { time: u64 },
}

fn main() -> Result<()> {
    let args = App::parse();
    match args.mode {
        Modes::Timer => Timer::new()
            .count()
            .map(|counter| println!("{}", fmt_time(counter))),
        Modes::Countdown { time } => Countdown::new(time)
            .count()
            .map(|counter| println!("{}", fmt_time(counter))),
    }
}
