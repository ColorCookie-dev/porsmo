use anyhow::{Context, Result};
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
    cursor,
    execute,
    terminal,
    style::{SetForegroundColor, Color, Print},
    event::{DisableMouseCapture, EnableMouseCapture},
};

pub struct TerminalHandler(pub Stdout);

impl TerminalHandler {
    pub fn new() -> anyhow::Result<Self> {
        enable_raw_mode().with_context(|| "Unable to enter raw mode")?;
        let mut stdout = std::io::stdout();
        execute!(
            &mut stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            terminal::Clear(terminal::ClearType::All),
        ).with_context(|| "Unable to write to terminal")?;

        Ok(Self(stdout))
    }

    pub fn show_time(
        &mut self,
        time: impl Display,
        running: bool,
        pos: Pos,
    ) -> Result<()> {
        if running {
            self.show_text(time, Color::Green, pos)?;
        } else {
            self.show_text(time, Color::Red, pos)?;
        }
        Ok(())
    }

    pub fn show_counter(
        &mut self,
        title: impl Display,
        time: impl Display,
        running: bool,
        controls: impl Display,
        message: impl Display,
    ) -> Result<()> {
        self.clear()?;
        self.show_text(title, Color::Magenta, (1, 1).into())?;
        self.show_time(time, running, (1, 2).into())?;
        self.show_text(controls, Color::Magenta, (1, 3).into())?;
        self.show_text(message, Color::Yellow, (1, 4).into())?;

        Ok(())
    }

    pub fn show_text(
        &mut self,
        text: impl Display,
        color: Color,
        pos: Pos,
    ) -> Result<()> {
        let stdout = &mut self.0;
        execute!(
            stdout,
            cursor::MoveTo(pos.right, pos.down),
            SetForegroundColor(color),
            Print(text),
        )
        .with_context(|| "failed to display timer")?;
        stdout.flush().with_context(|| "failed to flush stdout")
    }

    pub fn clear(&mut self) -> Result<()> {
        let stdout = &mut self.0;
        execute!(
            stdout,
            cursor::MoveTo(1, 1),
            terminal::Clear(terminal::ClearType::All),
        )
        .with_context(|| "failed to clear the terminal")?;

        stdout.flush().with_context(|| "failed to flush stdout")
    }

    pub fn show_prompt(
        &mut self,
        prompt: impl Display,
        prompt_color: Color,
        message: impl Display,
    ) -> Result<()> {
        self.clear()?;
        self.show_text(prompt, prompt_color, (1, 1).into())?;
        self.show_text(
            "[Enter]: Yes, [Q/N]: No",
            Color::Magenta,
            (1, 2).into(),
        )?;
        self.show_text(message, Color::Yellow, (1, 3).into())?;

        Ok(())
    }

}

impl Drop for TerminalHandler {
    fn drop(&mut self) {
        disable_raw_mode().expect("Failed to disable raw mode");
        execute!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            DisableMouseCapture,
            LeaveAlternateScreen,
        ).expect("Failed to reset screen");
    }
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

