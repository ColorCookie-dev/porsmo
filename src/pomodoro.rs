use crate::alert::alert;
use crate::input::{get_event, TIMEOUT};
use crate::stopwatch::Stopwatch;
use crate::terminal::running_color;
use crate::{format::format_duration, input::Command};
use crate::prelude::*;
use crossterm::cursor::{MoveTo, MoveToNextLine};
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use crossterm::{queue, style::Color, style::Stylize};

use std::io::Write;
use std::time::{Duration, Instant};

#[derive(Copy, Clone, Debug)]
pub struct PomoConfig {
    pub work_time: Duration,
    pub break_time: Duration,
    pub long_break: Duration,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Mode {
    #[default]
    Work,
    Break,
    LongBreak,
}

#[derive(Debug, Clone, Copy)]
pub struct Session {
    pub mode: Mode,
    pub number: u32,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            mode: Mode::default(),
            number: 1,
        }
    }
}

impl Session {
    pub fn next(&self) -> Self {
        match self.mode {
            Mode::Work if self.number % 4 == 0 => Self {
                mode: Mode::LongBreak,
                ..*self
            },
            Mode::Work => Self {
                mode: Mode::Break,
                ..*self
            },
            Mode::Break | Mode::LongBreak => Self {
                mode: Mode::Work,
                number: self.number + 1,
            },
        }
    }
}

impl Mode {
    pub fn current_target(&self, config: &PomoConfig) -> Duration {
        match self {
            Self::Work => config.work_time,
            Self::Break => config.break_time,
            Self::LongBreak => config.long_break,
        }
    }
}

impl PomoConfig {
    pub fn new(work_time: Duration, break_time: Duration, long_break: Duration) -> Self {
        Self {
            work_time,
            break_time,
            long_break,
        }
    }

    pub fn short() -> Self {
        Self {
            work_time: Duration::from_secs(25 * 60),
            break_time: Duration::from_secs(5 * 60),
            long_break: Duration::from_secs(10 * 60),
        }
    }

    pub fn long() -> Self {
        Self {
            work_time: Duration::from_secs(55 * 60),
            break_time: Duration::from_secs(10 * 60),
            long_break: Duration::from_secs(20 * 60),
        }
    }
}
const CONTROLS: &str = "[Q]: quit, [Shift S]: Skip, [Space]: pause/resume";
const ENDING_CONTROLS: &str = "[Q]: quit, [Shift S]: Skip, [Space]: pause/resume, [Enter]: Next";
const SKIP_CONTROLS: &str = "[Enter]: Yes, [Q/N]: No";

fn pomodoro_work_title(mode: Mode) -> &'static str {
    match mode {
        Mode::Work => "Pomodoro (Work)",
        Mode::Break => "Pomodoro (Break)",
        Mode::LongBreak => "Pomodoro (Long Break)",
    }
}

fn pomodoro_break_title(next_mode: Mode) -> &'static str {
    match next_mode {
        Mode::Work => "Break has ended! Start work?",
        Mode::Break => "Work has ended! Start break?",
        Mode::LongBreak => "Work has ended! Start a long break",
    }
}

pub fn pomodoro_alert_message(next_mode: Mode) -> (&'static str, &'static str) {
    match next_mode {
        Mode::Work => ("Your break ended!", "Time for some work"),
        Mode::Break => ("Pomodoro ended!", "Time for a short break"),
        Mode::LongBreak => ("Pomodoro 4 sessions complete!", "Time for a long break"),
    }
}

enum UIMode {
    Skip(Duration),
    Running(Stopwatch),
}

pub fn pomodoro(out: &mut impl Write, config: &PomoConfig) -> Result<()> {
    let stopwatch = Stopwatch::default();
    let mut session = Session::default();
    let mut ui_mode = UIMode::Running(stopwatch);
    let mut alerted = false;

    loop {
        pomodoro_show(out, config, &ui_mode, &session, &mut alerted)?;

        if let Some(cmd) = get_event(TIMEOUT)?.map(Command::from) {
            match ui_mode {
                UIMode::Skip(elapsed) => match cmd {
                    Command::Quit | Command::No => {
                        ui_mode = UIMode::Running(Stopwatch::new(Some(Instant::now()), elapsed))
                    }
                    Command::Enter | Command::Yes => {
                        alerted = false;
                        ui_mode = UIMode::Running(Stopwatch::default());
                        session = session.next();
                    }
                    _ => (),
                },
                UIMode::Running(ref mut stopwatch) => {
                    let elapsed = stopwatch.elapsed();
                    let target_time = session.mode.current_target(config);

                    match cmd {
                        Command::Quit => break,
                        Command::Enter if elapsed >= target_time => {
                            alerted = false;
                            ui_mode = UIMode::Running(Stopwatch::default());
                            session = session.next();
                        }
                        Command::Pause => stopwatch.stop(),
                        Command::Resume => stopwatch.start(),
                        Command::Toggle => stopwatch.toggle(),
                        Command::Skip => ui_mode = UIMode::Skip(elapsed),

                        _ => (),
                    }
                }
            }
        }
    }
    Ok(())
}

fn pomodoro_show(
    out: &mut impl Write,
    config: &PomoConfig,
    ui_mode: &UIMode,
    session: &Session,
    alerted: &mut bool,
) -> Result<()> {
    let target = session.mode.current_target(config);
    let round_number = format!("Session: {}", session.number);
    match ui_mode {
        UIMode::Skip(..) => {
            let (color, skip_to) = match session.next().mode {
                Mode::Work => (Color::Red, "skip to work?"),
                Mode::Break => (Color::Green, "skip to break?"),
                Mode::LongBreak => (Color::Green, "skip to long break?"),
            };
            queue!(
                out,
                MoveTo(0, 0),
                Clear(ClearType::All),
                Print(skip_to.with(color)),
                MoveToNextLine(1),
                Print(round_number),
                MoveToNextLine(1),
                Print(SKIP_CONTROLS)
            )?;
        }
        UIMode::Running(stopwatch) if stopwatch.elapsed() < target => {
            let time_left = target.saturating_sub(stopwatch.elapsed());

            queue!(
                out,
                MoveTo(0, 0),
                Clear(ClearType::All),
                Print(pomodoro_work_title(session.mode)),
                MoveToNextLine(1),
                Print(format_duration(&time_left).with(running_color(stopwatch.started())),),
                MoveToNextLine(1),
                Print(CONTROLS),
                MoveToNextLine(1),
                Print(round_number),
            )?;
        }
        UIMode::Running(stopwatch) => {
            let excess_time = stopwatch.elapsed().saturating_sub(target);
            let (title, message) = pomodoro_alert_message(session.next().mode);
            if !*alerted {
                *alerted = true;
                alert(title, message);
            }

            queue!(
                out,
                MoveTo(0, 0),
                Clear(ClearType::All),
                Print(pomodoro_break_title(session.next().mode)),
                MoveToNextLine(1),
                Print(
                    format!("+{}", format_duration(&excess_time),)
                        .with(running_color(stopwatch.started()))
                ),
                MoveToNextLine(1),
                Print(ENDING_CONTROLS),
                MoveToNextLine(1),
                Print(round_number),
                MoveToNextLine(1),
                Print(message),
            )?;
        }
    }
    out.flush()?;
    Ok(())
}
