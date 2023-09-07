use std::{num::ParseIntError, time::Duration};

pub fn fmt_time(time: u64) -> String {
    const MIN: u64 = 60;
    const HOUR: u64 = 60 * MIN;
    const DAY: u64 = 24 * HOUR;

    match time {
        secs if secs < MIN => format!("{secs}s"),
        secs if secs < HOUR => format!(
            "{}m {}",
            secs.div_euclid(MIN),
            fmt_time(secs.rem_euclid(MIN))
        ),
        secs if secs < DAY => format!(
            "{}h {}",
            secs.div_euclid(HOUR),
            fmt_time(secs.rem_euclid(HOUR))
        ),
        secs => format!(
            "{}days {}",
            secs.div_euclid(DAY),
            fmt_time(secs.rem_euclid(DAY))
        ),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TimeParseError {
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),

    #[error("Bad number of `:`")]
    BadNumberOfSeparators,
}

pub fn parse_time(time_str: &str) -> Result<Duration, TimeParseError> {
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
            return Err(TimeParseError::BadNumberOfSeparators);
        }
    }

    Ok(Duration::from_secs(secs))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_time() -> Result<(), TimeParseError> {
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
