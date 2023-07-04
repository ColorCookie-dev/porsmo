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
    cursor::{MoveTo, MoveToNextLine},
    execute,
    terminal::{Clear, ClearType},
    style::{SetForegroundColor, Color, Print},
    event::{DisableMouseCapture, EnableMouseCapture},
};

pub struct TerminalHandler(pub Stdout);

impl TerminalHandler {
    pub fn new() -> Result<Self> {
        enable_raw_mode().with_context(|| "Unable to enter raw mode")?;
        let mut stdout = std::io::stdout();
        execute!(
            &mut stdout,
            EnterAlternateScreen, EnableMouseCapture,
            Clear(ClearType::All), MoveTo(0, 0),
        ).with_context(|| "Unable to write to terminal")?;

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
        .with_context(|| "failed to clear the terminal")?;

        stdout.flush().with_context(|| "failed to flush stdout")?;
        Ok(self)
    }

    pub fn set_foreground_color(&mut self, color: Color) -> Result<&mut Self> {
        execute!(
            self.stdout(),
            SetForegroundColor(color),
        ).with_context(|| "failed to set foreground color to: {color:?}")?;
        Ok(self)
    }

    pub fn print(&mut self, text: impl Display) -> Result<&mut Self> {
        execute!(
            self.stdout(),
            Print(text), MoveToNextLine(1),
        ).with_context(|| format!("showing info failed"))?;
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
        self.stdout().flush().with_context(|| "failed to flush stdout")
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

pub fn show_counter(
    stdout: &mut Stdout,
    title: impl Display,
    time: impl Display,
    running: bool,
    controls: impl Display,
    message: impl Display,
) -> Result<()> {
    execute!(
        stdout,
        Clear(ClearType::All), MoveTo(0, 0),

        SetForegroundColor(Color::Magenta),
        Print(title), MoveToNextLine(1),

        SetForegroundColor(running_color(running)),
        Print(time), MoveToNextLine(1),

        SetForegroundColor(Color::Magenta),
        Print(controls), MoveToNextLine(1),

        SetForegroundColor(Color::Yellow),
        Print(message), MoveToNextLine(1),
    ).with_context(|| "failed to display timer")?;
    stdout.flush().with_context(|| "failed to flush stdout")?;

    Ok(())
}

pub fn show_prompt(
    stdout: &mut Stdout,
    prompt: impl Display,
    prompt_color: Color,
    message: impl Display,
) -> Result<()> {
    execute!(
        stdout,
        Clear(ClearType::All), MoveTo(0, 0),

        SetForegroundColor(prompt_color),
        Print(prompt), MoveToNextLine(1),

        SetForegroundColor(Color::Magenta),
        Print("[Enter]: Yes, [Q/N]: No"), MoveToNextLine(1),

        SetForegroundColor(Color::Yellow),
        Print(message), MoveToNextLine(1),
    ).with_context(|| "failed to show prompt")?;
    stdout.flush().with_context(|| "failed to flush stdout")?;

    Ok(())
}

