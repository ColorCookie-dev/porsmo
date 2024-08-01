use crate::{error::PorsmoError, prelude::*};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    style::{Color, Print},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::io::{stdout, Stdout};

pub struct TerminalHandler(Stdout, String);

impl TerminalHandler {
    pub fn new() -> Result<Self> {
        enable_raw_mode().map_err(PorsmoError::FailedRawModeEnter)?;

        let mut stdout = std::io::stdout();
        execute!(
            &mut stdout,
            EnterAlternateScreen,
            Hide,
            Clear(ClearType::All),
            MoveTo(0, 0),
        )
        .map_err(PorsmoError::FailedInitialization)?;

        Ok(Self(stdout, String::new()))
    }

    pub fn stdout(&mut self) -> &mut Stdout {
        &mut self.0
    }

    pub fn set_exit_message(&mut self, s: String) {
        self.1 = s;
    }
}

impl Drop for TerminalHandler {
    fn drop(&mut self) {
        disable_raw_mode().expect("Failed to disable raw mode");
        execute!(
            stdout(),
            Clear(ClearType::All),
            Show,
            LeaveAlternateScreen,
            Print(self.1.clone())
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
