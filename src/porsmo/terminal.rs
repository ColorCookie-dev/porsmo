use crate::{error::PorsmoError, prelude::*};
use crossterm::{
    cursor::MoveTo,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    style::Color,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    terminal::{Clear, ClearType},
};
use std::io::{stdout, Stdout};

pub struct TerminalHandler(Stdout);

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
