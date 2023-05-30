use anyhow::{Context, Result};
use std::{
    fmt::Display,
    io::{stdout, Stdout, Write},
};
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

pub fn show_time(
    stdout: &mut impl Write,
    time: impl Display,
    running: bool,
    pos: Pos,
) -> Result<()> {
    if running {
        show_text(stdout, time, color::Green, pos)?;
    } else {
        show_text(stdout, time, color::Red, pos)?;
    }
    Ok(())
}

pub fn show_counter(
    stdout: &mut impl Write,
    title: impl Display,
    time: impl Display,
    running: bool,
    controls: impl Display,
    message: impl Display,
) -> Result<()> {
    clear(stdout)?;
    show_text(stdout, title, color::Magenta, (1, 1).into())?;
    show_time(stdout, time, running, (1, 2).into())?;
    show_text(stdout, controls, color::Magenta, (1, 3).into())?;
    show_text(stdout, message, color::LightYellow, (1, 4).into())?;

    Ok(())
}

pub struct Pos {
    right: u16,
    down: u16,
}

impl From<(u16, u16)> for Pos {
    fn from(pos: (u16, u16)) -> Self {
        let (right, down) = pos;
        Self { right, down }
    }
}

pub fn show_text(
    stdout: &mut impl Write,
    text: impl Display,
    color: impl color::Color,
    pos: Pos,
) -> Result<()> {
    write!(
        stdout,
        "{cursor}{color}{text}",
        color = color::Fg(color),
        cursor = cursor::Goto(pos.right, pos.down),
        text = text,
    )
    .with_context(|| "failed to display timer")?;

    stdout.flush().with_context(|| "failed to flush stdout")
}

pub fn show_prompt(
    stdout: &mut impl Write,
    prompt: impl Display,
    prompt_color: impl color::Color,
    message: impl Display,
) -> Result<()> {
    clear(stdout)?;
    show_text(stdout, prompt, prompt_color, (1, 1).into())?;
    show_text(
        stdout,
        "[Enter]: Yes, [Q/N]: No",
        color::Magenta,
        (1, 2).into(),
    )?;
    show_text(stdout, message, color::LightYellow, (1, 3).into())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::terminal::*;

    #[test]
    fn test_show_message() {
        let mut buf = Vec::<u8>::new();
        let msg = "Hello World";

        let res = show_message(&mut buf, msg, 0);
        assert!(res.is_ok());
        assert!(String::from_utf8(buf.to_vec()).unwrap().contains(msg));

        buf.clear();
        let res = show_message_red(&mut buf, msg, 0);
        assert!(res.is_ok());
        assert!(String::from_utf8(buf.to_vec()).unwrap().contains(msg));

        buf.clear();
        let res = show_message_green(&mut buf, msg, 0);
        assert!(res.is_ok());
        assert!(String::from_utf8(buf.to_vec()).unwrap().contains(msg));

        buf.clear();
        let res = show_message_yellow(&mut buf, msg, 0);
        assert!(res.is_ok());
        assert!(String::from_utf8(buf.to_vec()).unwrap().contains(msg));
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
