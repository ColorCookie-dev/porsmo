use std::{time::Duration, num::ParseIntError};

pub fn fmt_time(time: Duration) -> String {
    let secs = time.as_secs();
    if secs < 60 {
        format!("00:{:0>2}", secs)
    } else if secs < 3600 {
        format!("{:0>2}:{:0>2}", secs / 60, secs % 60)
    } else if secs < 86_400 {
        format!(
            "{:0>2}:{:0>2}:{:0>2}",
            secs / 3600,
            secs / 60 % 60,
            secs % 60
        )
    } else {
        format!(
            "{}days {:0>2}:{:0>2}:{:0>2}",
            secs / 86_400,
            secs / 3600 % 24,
            secs / 60 % 60,
            secs % 60
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TimeParseError {
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),

    #[error("Bad number of `:`")]
    BadNumberOfSeparators,
}

pub fn parse_time(time_str: &str) -> Result<u64, TimeParseError> {
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
