use std::time::Duration;
use std::borrow::Borrow;

use crate::prelude::*;

pub fn format_duration(dur: impl Borrow<Duration>) -> String {
    let dur = dur.borrow();
    let total_secs = dur.as_secs();
    let secs = total_secs % 60;
    let mins = ((total_secs - secs) % (60 * 60)) / 60;
    let hours = (total_secs - mins * 60 - secs) / (60 * 60);
    format!("{hours}h {mins}m {secs}s")
}

pub fn parse_duration(text: &str) -> Result<Duration> {
    let (hours, text) = match text.split_once('h') {
        Some((hours, rest)) => {
            let hours = hours.parse::<u64>()?;
            (Duration::from_secs(hours * 3600), rest)
        },
        None => (Duration::ZERO, text),
    };

    let (mins, text) = match text.split_once('m') {
        Some((mins, text)) => {
            let mins = mins.parse::<u64>()?;
            (Duration::from_secs(mins * 60), text)
        },
        None => (Duration::ZERO, text),
    };

    let (secs, _) = match text.split_once('s') {
        Some((secs, "")) => {
            let secs = secs.parse::<u64>()?;
            (Duration::from_secs(secs), text)
        },
        None if text == "" => (Duration::ZERO, ""),
        _ => return Err(PorsmoError::WrongFormatError),
    };

    Ok(hours + mins + secs)
}
