use std::time::Duration;

use crate::format::parse_duration;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "Porsmo",
    author = "HellsNoah <hellsnoah@protonmail.com>",
    version,
    about
)]
pub struct Cli {
    #[command(subcommand, name = "mode")]
    pub mode: Option<CounterMode>,
}

#[derive(Subcommand)]
pub enum CounterMode {
    /// alias: s, stopwatch, counts up until you tell it to stop
    #[command(name = "stopwatch", alias = "s")]
    Stopwatch {
        #[arg(value_parser = parse_duration, value_name = "time")]
        /// Lets you start timer from a particular time
        start_time: Option<Duration>,
    },
    /// alias: t, timer, counts down until you tell it to stop, or it ends
    #[command(name = "timer", alias = "t")]
    Timer {
        #[arg(value_parser = parse_duration, value_name = "time")]
        target: Option<Duration>,
    },
    /// alias: p, pomodoro, for all you productivity needs (default)
    #[command(name = "pomodoro", alias = "p")]
    Pomodoro {
        #[clap(subcommand, name = "mode")]
        mode: Option<PomoMode>,
    },
}

#[derive(Subcommand, Debug)]
pub enum PomoMode {
    /// alias: s, short pomodoro, with 25, 5, 10 min values (default)
    #[command(name = "short", alias = "s")]
    Short,
    /// alias: l, long pomodoro, with 55, 10, 20 min values
    #[command(name = "long", alias = "l")]
    Long,
    /// alias: c, custom pomodoro, with any specified values
    #[command(name = "custom", alias = "c")]
    Custom {
        #[arg(value_parser = parse_duration, value_name = "work-time")]
        work_time: Duration,
        #[arg(value_parser = parse_duration, value_name = "break-time")]
        break_time: Duration,
        #[arg(value_parser = parse_duration, value_name = "long-break-time")]
        long_break: Duration,
    },
}
