mod countdown;
mod counter;
mod format;
mod input;
mod notification;
mod pomodoro;
mod sound;
mod terminal;
mod timer;

use crate::format::fmt_time;
use crate::notification::notify_default;
use crate::pomodoro::Pomodoro;
use crate::sound::play_bell;
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
    mode: Option<Modes>,
}

#[derive(Subcommand)]
enum Modes {
    /// timer, counts up until you tell it to stop
    #[clap(name = "timer", alias = "t")]
    Timer {
        #[clap(parse(try_from_str=parse_time),
               default_value_t = 0,
               value_name = "time")]
        /// Lets you start timer from a particular time
        time: u64,
    },
    /// countdown, counts down until you tell it to stop, or it ends
    #[clap(name = "countdown", alias = "c")]
    Countdown {
        #[clap(parse(try_from_str=parse_time),
               default_value_t = 25*60,
               value_name = "time")]
        time: u64,
    },
    /// pomodoro, for all you productivity needs (default)
    #[clap(name = "pomodoro", alias = "p")]
    Pomodoro {
        #[clap(subcommand, name = "mode")]
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
        work_time: u64,
        #[clap(parse(try_from_str=parse_time),
               value_name = "break time")]
        break_time: u64,
        #[clap(parse(try_from_str=parse_time),
               value_name = "long break time")]
        long_break_time: u64,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.mode {
        Some(Modes::Timer { time }) => Timer::new(time)
            .count()
            .map(|counter| println!("{}", fmt_time(counter))),
        Some(Modes::Countdown { time }) => Countdown::new(time).count().map(|counter| {
            if counter == 0 {
                after_end().expect("failed to notify or sound the end of countdown!");
            }
            println!("{}", fmt_time(counter))
        }),
        Some(Modes::Pomodoro { mode }) => match mode {
            Some(PomoMode::Short) | None => Pomodoro::new(25 * 60, 5 * 60, 10 * 60)
                .count()
                .map(|counter| println!("{}", fmt_time(counter))),
            Some(PomoMode::Long) => Pomodoro::new(55 * 60, 10 * 60, 20 * 60)
                .count()
                .map(|counter| println!("{}", fmt_time(counter))),
            Some(PomoMode::Custom {
                work_time,
                break_time,
                long_break_time,
            }) => Pomodoro::new(work_time, break_time, long_break_time)
                .count()
                .map(|counter| println!("{}", fmt_time(counter))),
        },
        None => Pomodoro::new(25 * 60, 5 * 60, 10 * 60)
            .count()
            .map(|counter| println!("{}", fmt_time(counter))),
    }
}

fn after_end() -> Result<()> {
    notify_default("Countdown ended!", "Your Porsmo Countdown Ended")?;
    play_bell()
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
            ("1020", 1020),
            ("2:000092", 2 * 60 + 92),
            ("2:", 2 * 60),
            ("::10", 10),
            ("", 0),
            (":", 0),
            ("::", 0),
            (":::", 0),
            ("1:::", 1 * 24 * 60 * 60),
            ("::::", 0),
            (":::::", 0),
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
