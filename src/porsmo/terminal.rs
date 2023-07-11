use std::{
    fmt::Display,
    io::{stdout, Write, Stdout},
};
use crossterm::{
    terminal::{
        enable_raw_mode,
        disable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    cursor::{MoveTo, MoveToNextLine},
    execute,
    terminal::{Clear, ClearType},
    style::{SetForegroundColor, Color, Print},
    event::{DisableMouseCapture, EnableMouseCapture},
};

#[derive(thiserror::Error, Debug)]
pub enum TerminalError {
    #[error("Error entering raw mode in terminal")]
    FailedRawModeEnter(#[source] crossterm::ErrorKind),

    #[error("Error initializing terminal with alternate screen and mouse capture")]
    FailedInitialization(#[source] crossterm::ErrorKind),

    #[error("Error clearing terminal")]
    FailedClear(#[source] crossterm::ErrorKind),

    #[error("Failed to flush to terminal")]
    FailedFlush(#[source] std::io::Error),

    #[error("Failed to set foreground color to {1:?}")]
    ForegroundColorSetFailed(#[source] crossterm::ErrorKind, Color),

    #[error("Failed to print to screen")]
    FailedPrint(#[source] crossterm::ErrorKind),
}

pub type Result<T> = core::result::Result<T, TerminalError>;

pub struct TerminalHandler(pub Stdout);

impl TerminalHandler {
    pub fn new() -> Result<Self> {
        enable_raw_mode()
            .map_err(|e| TerminalError::FailedRawModeEnter(e))?;

        let mut stdout = std::io::stdout();
        execute!(
            &mut stdout,
            EnterAlternateScreen, EnableMouseCapture,
            Clear(ClearType::All), MoveTo(0, 0),
        ).map_err(|e| TerminalError::FailedInitialization(e))?;

        Ok(Self(stdout))
    }

    pub fn stdout(&mut self) -> &mut Stdout {
        &mut self.0
    }

    pub fn clear(&mut self) -> Result<&mut Self> {
        let stdout = &mut self.0;
        execute!(
            stdout,
            MoveTo(0, 0), Clear(ClearType::All),
        )
        .map_err(|e| TerminalError::FailedClear(e))?;

        stdout.flush().map_err(|e| TerminalError::FailedFlush(e))?;
        Ok(self)
    }

    pub fn set_foreground_color(&mut self, color: Color) -> Result<&mut Self> {
        execute!(
            self.stdout(),
            SetForegroundColor(color),
        ).map_err(|e| TerminalError::ForegroundColorSetFailed(e, color))?;
        Ok(self)
    }

    pub fn print(&mut self, text: impl Display) -> Result<&mut Self> {
        execute!(
            self.stdout(),
            Print(text), MoveToNextLine(1),
        ).map_err(|e| TerminalError::FailedPrint(e))?;
        Ok(self)
    }

    pub fn info(&mut self, text: impl Display) -> Result<&mut Self> {
        self
            .set_foreground_color(Color::Magenta)?
            .print(text)
    }

    pub fn status(&mut self, text: impl Display) -> Result<&mut Self> {
        self
            .set_foreground_color(Color::Yellow)?
            .print(text)
    }

    pub fn flush(&mut self) -> Result<()> {
        self.stdout().flush().map_err(|e| TerminalError::FailedFlush(e))
    }
}

impl Drop for TerminalHandler {
    fn drop(&mut self) {
        disable_raw_mode().expect("Failed to disable raw mode");
        execute!(
            stdout(),
            Clear(ClearType::All),
            DisableMouseCapture, LeaveAlternateScreen,
        ).expect("Failed to reset screen");
    }
}

pub fn running_color(running: bool) -> Color {
    match running {
        true => Color::Green,
        false => Color::Red,
    }
}
