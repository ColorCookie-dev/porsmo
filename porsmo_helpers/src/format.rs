use std::{
    error::Error,
    fmt,
    num::{ParseFloatError, ParseIntError},
    time::Duration,
};

#[derive(Debug)]
pub enum ParseError {
    ParseIntError(ParseIntError),
    ParseSubSecsError(ParseFloatError),
    BadFormat(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::ParseIntError(err) => writeln!(f, "Error in Parsing Integer: {}", err),
            Self::ParseSubSecsError(err) => writeln!(f, "Error in Parsing sub seconds: {}", err),
            Self::BadFormat(err) => writeln!(f, "Bad Format: {}", err),
        }
    }
}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> Self {
        ParseError::ParseIntError(err)
    }
}

impl From<ParseFloatError> for ParseError {
    fn from(err: ParseFloatError) -> Self {
        ParseError::ParseSubSecsError(err)
    }
}

impl Error for ParseError {}

pub fn parse_time(time_str: &str) -> Result<Duration, ParseError> {
    let mut secs = Duration::new(0, 0);

    for (i, e) in time_str.split(':').rev().enumerate() {
        if e.is_empty() {
            continue;
        }

        if i == 0 {
            let en = e.parse::<f64>()?;
            secs += Duration::from_secs_f64(en);
        } else if i == 1 {
            let en = e.parse::<u64>()?;
            secs += Duration::from_secs(en * 60);
        } else if i == 2 {
            let en = e.parse::<u64>()?;
            secs += Duration::from_secs(en * 60 * 60);
        } else if i == 3 {
            let en = e.parse::<u64>()?;
            secs += Duration::from_secs(en * 3600 * 24);
        } else {
            return Err(ParseError::BadFormat("Bad number of ':'".into()));
        }
    }

    Ok(secs)
}

pub fn fmt_time(time: Duration) -> String {
    let mut time = time.as_millis();
    let mut fmt_str = String::new();

    let millis = time % 1000;
    time /= 1000;

    if millis != 0 {
        fmt_str.push_str(&format!(".{:0>3}", millis));
    }

    let secs = time % 60;
    time /= 60;
    fmt_str.insert_str(0, &format!("{:0>2}", secs));

    if time == 0 {
        return fmt_str;
    }

    let mins = time % 60;
    time /= 60;
    fmt_str.insert_str(0, &format!("{:0>2}:", mins));

    if time == 0 {
        return fmt_str;
    }

    let hours = time % 24;
    time /= 24;
    fmt_str.insert_str(0, &format!("{:0>2}:", hours));

    if time == 0 {
        return fmt_str;
    }

    let days = time;
    format!("{}days {}", days, fmt_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_time() -> Result<(), Box<dyn std::error::Error>> {
        let ok_cases = vec![
            ("", Duration::from_secs(0)),
            (":", Duration::from_secs(0)),
            ("::10", Duration::from_secs(10)),
            ("1020", Duration::from_secs(1020)),
            ("2:000092", Duration::from_secs(2 * 60 + 92)),
            ("2:", Duration::from_secs(2 * 60)),
            ("2:2:2", Duration::from_secs((2 * 60 + 2) * 60 + 2)),
            ("1:::", Duration::from_secs(1 * 24 * 60 * 60)),
            ("0.1", Duration::from_millis(100)),
            ("0.12", Duration::from_millis(120)),
            ("0.012", Duration::from_millis(12)),
        ];

        for (inp, out) in ok_cases.iter() {
            assert_eq!(parse_time(inp)?, *out);
        }

        let err_cases = vec![
            "1::::",
            "kjdf:kjfk",
            ":kjfk",
            "1:4k:5",
            "1:1:1.f",
            "1:1:1.1.1",
            "1:1.0:1.1",
        ];

        for inp in err_cases.iter() {
            assert!(parse_time(inp).is_err());
        }

        Ok(())
    }
}
