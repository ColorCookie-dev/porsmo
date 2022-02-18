use crate::format::fmt_time;
use anyhow::{Context, Result};
use std::io::Write;
use std::io::{stdout, Stdout};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor};

pub struct TermRawMode {
    pub stdout: RawTerminal<Stdout>,
}

impl TermRawMode {
    pub fn new() -> TermRawMode {
        TermRawMode {
            stdout: stdout().into_raw_mode().unwrap(),
        }
    }
}

impl Drop for TermRawMode {
    fn drop(&mut self) {
        reset_terminal(&mut self.stdout).unwrap();
    }
}

pub fn reset_terminal(stdout: &mut impl Write) -> Result<()> {
    write!(
        stdout,
        "{top}{clear}{show}{color}",
        top = cursor::Goto(1, 1),
        clear = clear::All,
        color = color::Fg(color::Reset),
        show = termion::cursor::Show
    )
    .with_context(|| "failed to release raw mode")?;

    stdout.flush().with_context(|| "failed to flush stdout")
}

pub fn clear(stdout: &mut impl Write) -> Result<()> {
    write!(
        stdout,
        "{top}{clear}",
        top = cursor::Goto(1, 1),
        clear = clear::All,
    )
    .with_context(|| "failed to clear the terminal")?;

    stdout.flush().with_context(|| "failed to flush stdout")
}

pub fn show_view(stdout: &mut impl Write, time: u64, running: bool) -> Result<()> {
    show_time(stdout, time, running)?;
    show_message(stdout, "[Q]: quit, [Space]: pause/resume", 1)
}

pub fn show_time(stdout: &mut impl Write, time: u64, running: bool) -> Result<()> {
    if running {
        show_time_running(stdout, time)
    } else {
        show_time_paused(stdout, time)
    }
}

pub fn show_time_paused(stdout: &mut impl Write, time: u64) -> Result<()> {
    write!(
        stdout,
        "{clear}{cursor}{color}{time}",
        clear = clear::All,
        color = color::Fg(color::Red),
        cursor = cursor::Goto(1, 1),
        time = fmt_time(time)
    )
    .with_context(|| "failed to display timer")?;

    stdout.flush().with_context(|| "failed to flush stdout")
}

pub fn show_time_running(stdout: &mut impl Write, time: u64) -> Result<()> {
    write!(
        stdout,
        "{clear}{cursor}{color}{time}",
        clear = clear::All,
        color = color::Fg(color::Green),
        cursor = cursor::Goto(1, 1),
        time = fmt_time(time)
    )
    .with_context(|| "failed to display timer")?;

    stdout.flush().with_context(|| "failed to flush stdout")
}

pub fn show_message(stdout: &mut impl Write, msg: &str, down: u16) -> Result<()> {
    write!(
        stdout,
        "{cursor}{color}{time}",
        cursor = cursor::Goto(1, 1 + down),
        color = color::Fg(color::Magenta),
        time = msg
    )
    .with_context(|| "failed to display message")?;
    stdout.flush().with_context(|| "failed to flush stdout")
}

pub fn show_message_red(stdout: &mut impl Write, msg: &str, down: u16) -> Result<()> {
    write!(
        stdout,
        "{cursor}{color}{time}",
        cursor = cursor::Goto(1, 1 + down),
        color = color::Fg(color::Red),
        time = msg
    )
    .with_context(|| "failed to display message")?;
    stdout.flush().with_context(|| "failed to flush stdout")
}

pub fn show_message_yellow(stdout: &mut impl Write, msg: &str, down: u16) -> Result<()> {
    write!(
        stdout,
        "{cursor}{color}{time}",
        cursor = cursor::Goto(1, 1 + down),
        color = color::Fg(color::LightYellow),
        time = msg
    )
    .with_context(|| "failed to display message")?;
    stdout.flush().with_context(|| "failed to flush stdout")
}

pub fn show_message_green(stdout: &mut impl Write, msg: &str, down: u16) -> Result<()> {
    write!(
        stdout,
        "{cursor}{color}{time}",
        cursor = cursor::Goto(1, 1 + down),
        color = color::Fg(color::Green),
        time = msg
    )
    .with_context(|| "failed to display message")?;
    stdout.flush().with_context(|| "failed to flush stdout")
}

#[cfg(test)]
mod tests {
    use crate::terminal::*;

    #[test]
    fn test_show_message() {
        let mut buf = Vec::<u8>::new();
        let msg = "Hello World";
        let res = show_message(&mut buf, msg, 0);

        let buf = String::from_utf8(buf.to_vec()).unwrap();
        assert!(res.is_ok());
        assert!(buf.contains(msg));
    }

    #[test]
    fn test_show_time() {
        let mut buf = Vec::<u8>::new();
        let res = show_time_running(&mut buf, 300);

        let new_buf = String::from_utf8(buf.to_vec()).unwrap();
        assert!(res.is_ok());
        assert!(new_buf.contains("05:00"));

        buf.clear();

        let res = show_time_paused(&mut buf, 300);

        let new_buf = String::from_utf8(buf.to_vec()).unwrap();
        assert!(res.is_ok());
        assert!(new_buf.contains("05:00"));
    }
}
