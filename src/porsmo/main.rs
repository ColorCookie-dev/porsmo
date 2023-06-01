mod alert;
mod format;
mod input;
mod notification;
mod pomodoro;
mod sound;
mod stopwatch;
mod terminal;
mod timer;

use std::time::Duration;

use crate::format::fmt_time;
use crate::{pomodoro::pomodoro, stopwatch::stopwatch, timer::timer};
use anyhow::{bail, Result};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "Porsmo",
    author = "HellsNoah <hellsnoah@protonmail.com>",
    version = "0.1.3",
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

pub struct PomodoroUI;

impl PomodoroUI {
    fn short() -> Result<Duration> {
        pomodoro(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(10 * 60),
        )
    }

    fn long() -> Result<Duration> {
        pomodoro(
            Duration::from_secs(55 * 60),
            Duration::from_secs(10 * 60),
            Duration::from_secs(20 * 60),
        )
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.mode {
        Some(Modes::Stopwatch { start_time: time }) => stopwatch(time),
        Some(Modes::Timer { start_time: time }) => timer(time),
        Some(Modes::Pomodoro { mode }) => match mode {
            Some(PomoMode::Short) | None => PomodoroUI::short(),
            Some(PomoMode::Long) => PomodoroUI::long(),
            Some(PomoMode::Custom {
                work_time,
                break_time,
                long_break_time,
            }) => pomodoro(
                Duration::from_secs(work_time),
                Duration::from_secs(break_time),
                Duration::from_secs(long_break_time)
            ),
        },
        None => PomodoroUI::short(),
    }
    .map(|time| {
        println!("{}", fmt_time(time));
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_time() -> Result<()> {
        let ok_cases = vec![
            ("", 0),
            (":", 0),
            ("::10", 10),
            ("1020", 1020),
            ("2:000092", 2 * 60 + 92),
            ("2:", 2 * 60),
            ("2:2:2", (2 * 60 + 2) * 60 + 2),
            ("1:::", 1 * 24 * 60 * 60),
        ];

        for (inp, out) in ok_cases.iter() {
            assert_eq!(parse_time(inp)?, *out);
        }

        let err_cases = vec!["1::::", "kjdf:kjfk", ":kjfk", "1:4k:5"];

        for inp in err_cases.iter() {
            assert!(parse_time(inp).is_err());
        }

        Ok(())
    }
}
