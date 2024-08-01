use std::time::Duration;

use crate::format::parse_duration;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand, name = "mode")]
    pub mode: Option<CounterMode>,
}

#[derive(Subcommand)]
pub enum CounterMode {
    /// alias: s, stopwatch, counts up until you tell it to stop
    #[command(name = "stopwatch", alias = "s")]
    Stopwatch /* {
        #[arg(
            value_parser = parse_duration,
            default_value = "0s",
            value_name = "time"
        )]
        /// start from a particular time: example values: 30m 20m 40m 2h25m30s
        start_time: Duration,
    }*/,
    /// alias: t, timer, counts down until you tell it to stop, or it ends
    #[command(name = "timer", alias = "t")]
    Timer {
        /// target time: example values 30m 20m 40m 2h25m30s
        #[arg(value_parser = parse_duration, value_name = "time")]
        target: Duration,
    },
    /// alias: p, pomodoro, for all you productivity needs (default)
    #[command(name = "pomodoro", alias = "p")]
    Pomodoro {
        #[clap(subcommand, name = "mode")]
        mode: PomoMode,
        ///Display a message after quitting the pomodoro timer
        #[arg(short, name = "exitmessage")]
        exitmessage: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum PomoMode {
    /// alias: s, short pomodoro, with 25m, 5m, 10m values (default)
    #[command(name = "short", alias = "s")]
    Short,
    /// alias: l, long pomodoro, with 55m, 10m, 20m values
    #[command(name = "long", alias = "l")]
    Long,
    /// alias: c, custom pomodoro, with any specified values
    #[command(name = "custom", alias = "c")]
    Custom {
        /// target work time: example values 30m 20m 40m 2h25m30s
        #[arg(value_parser = parse_duration, value_name = "work-time")]
        work_time: Duration,
        /// target break time: example values 30m 20m 40m 2h25m30s
        #[arg(value_parser = parse_duration, value_name = "break-time")]
        break_time: Duration,
        /// target long break time: example values 30m 20m 40m 2h25m30s
        #[arg(value_parser = parse_duration, value_name = "long-break-time")]
        long_break: Duration,
    },
}
