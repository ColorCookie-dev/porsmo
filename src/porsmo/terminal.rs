use crate::{error::PorsmoError, prelude::*};
use crossterm::{
    cursor::{MoveTo, MoveToNextLine},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    terminal::{Clear, ClearType},
};
use std::{
    fmt::Display,
    io::{stdout, Stdout, Write},
};

pub struct TerminalHandler(pub Stdout);

impl TerminalHandler {
    pub fn new() -> Result<Self> {
        enable_raw_mode().map_err(PorsmoError::FailedRawModeEnter)?;

        let mut stdout = std::io::stdout();
        execute!(
            &mut stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            Clear(ClearType::All),
            MoveTo(0, 0),
        )
        .map_err(PorsmoError::FailedInitialization)?;

        Ok(Self(stdout))
    }

    pub fn stdout(&mut self) -> &mut Stdout {
        &mut self.0
    }

    pub fn clear(&mut self) -> Result<&mut Self> {
        let stdout = &mut self.0;
        execute!(stdout, MoveTo(0, 0), Clear(ClearType::All),).map_err(PorsmoError::FailedClear)?;

        stdout.flush().map_err(PorsmoError::FlushError)?;
        Ok(self)
    }

    pub fn set_foreground_color(&mut self, color: Color) -> Result<&mut Self> {
        execute!(self.stdout(), SetForegroundColor(color),)
            .map_err(PorsmoError::ForegroundColorSetFailed)?;
        Ok(self)
    }

    pub fn print(&mut self, text: impl Display) -> Result<&mut Self> {
        execute!(self.stdout(), Print(text), MoveToNextLine(1),)
            .map_err(PorsmoError::FailedPrint)?;
        Ok(self)
    }

    pub fn info(&mut self, text: impl Display) -> Result<&mut Self> {
        self.set_foreground_color(Color::Magenta)?.print(text)
    }

    pub fn status(&mut self, text: impl Display) -> Result<&mut Self> {
        self.set_foreground_color(Color::Yellow)?.print(text)
    }

    pub fn flush(&mut self) -> Result<()> {
        self.stdout().flush().map_err(PorsmoError::FlushError)
    }
}

impl Drop for TerminalHandler {
    fn drop(&mut self) {
        disable_raw_mode().expect("Failed to disable raw mode");
        execute!(
            stdout(),
            Clear(ClearType::All),
            DisableMouseCapture,
            LeaveAlternateScreen,
        )
        .expect("Failed to reset screen");
    }
}

pub fn running_color(running: bool) -> Color {
    match running {
        true => Color::Green,
        false => Color::Red,
    }
}
