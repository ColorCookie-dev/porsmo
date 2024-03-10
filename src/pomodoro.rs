use crate::alert::Alerter;
use crate::stopwatch::Stopwatch;
use crate::terminal::running_color;
use crate::{format::format_duration, input::Command};
use crate::{prelude::*, CounterUI};
use crossterm::cursor::{MoveTo, MoveToNextLine};
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use crossterm::{queue, style::Color, style::Stylize};

use std::io::Write;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default)]
pub enum Mode {
    #[default]
    Work,
    Break,
    LongBreak,
}

#[derive(Copy, Clone, Debug)]
pub struct PomodoroConfig {
    pub work_time: Duration,
    pub break_time: Duration,
    pub long_break: Duration,
}

impl Default for PomodoroConfig {
    fn default() -> Self {
        Self::short()
    }
}

impl PomodoroConfig {
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

    pub fn current_target(&self, mode: Mode) -> Duration {
        match mode {
            Mode::Work => self.work_time,
            Mode::Break => self.break_time,
            Mode::LongBreak => self.long_break,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Session {
    pub mode: Mode,
    pub round: u32,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            mode: Mode::default(),
            round: 1,
        }
    }
}

impl Session {
    pub fn next(self) -> Self {
        match self.mode {
            Mode::Work if self.round % 4 == 0 => Self {
                mode: Mode::LongBreak,
                ..self
            },
            Mode::Work => Self {
                mode: Mode::Break,
                ..self
            },
            Mode::Break | Mode::LongBreak => Self {
                mode: Mode::Work,
                round: self.round + 1,
            },
        }
    }
}

const CONTROLS: &str = "[Q]: quit, [Shift S]: Skip, [Space]: pause/resume";
const ENDING_CONTROLS: &str = "[Q]: quit, [Shift S]: Skip, [Space]: pause/resume, [Enter]: Next";
const SKIP_CONTROLS: &str = "[Enter]: Yes, [Q/N]: No";

fn default_title(mode: Mode) -> &'static str {
    match mode {
        Mode::Work => "Pomodoro (Work)",
        Mode::Break => "Pomodoro (Break)",
        Mode::LongBreak => "Pomodoro (Long Break)",
    }
}

fn end_title(next_mode: Mode) -> &'static str {
    match next_mode {
        Mode::Work => "Break has ended! Start work?",
        Mode::Break => "Work has ended! Start break?",
        Mode::LongBreak => "Work has ended! Start a long break",
    }
}

fn alert_message(next_mode: Mode) -> (&'static str, &'static str) {
    match next_mode {
        Mode::Work => ("Your break ended!", "Time for some work"),
        Mode::Break => ("Pomodoro ended!", "Time for a short break"),
        Mode::LongBreak => ("Pomodoro 4 sessions complete!", "Time for a long break"),
    }
}

#[derive(Debug, Clone, Copy)]
enum UIMode {
    Skip(Duration),
    Running(Stopwatch),
}

impl Default for UIMode {
    fn default() -> Self {
        Self::Running(Stopwatch::default())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PomodoroUI {
    config: PomodoroConfig,
    session: Session,
    ui_mode: UIMode,
    alerter: Alerter,
}

impl PomodoroUI {
    pub fn new(config: PomodoroConfig) -> Self {
        Self { config, ..Default::default() }
    }
}

impl CounterUI for PomodoroUI {
    fn show(&mut self, out: &mut impl Write) -> Result<()> {
        pomodoro_show(out, &self.config, &self.ui_mode,
                      &self.session, &mut self.alerter)
    }

    fn update(&mut self, command: Command) {
        pomodoro_update(command, &self.config,
                        &mut self.alerter, &mut self.ui_mode, &mut self.session);
    }
}

fn pomodoro_update(
    command: Command,
    config: &PomodoroConfig,
    alerter: &mut Alerter,
    ui_mode: &mut UIMode,
    session: &mut Session,
) {
    match ui_mode {
        UIMode::Skip(elapsed) => match command {
            Command::Quit | Command::No => {
                *ui_mode = UIMode::Running(Stopwatch::new(Some(Instant::now()), *elapsed))
            }
            Command::Enter | Command::Yes => {
                alerter.reset();
                *ui_mode = UIMode::Running(Stopwatch::default());
                *session = session.next();
            }
            _ => (),
        },
        UIMode::Running(ref mut stopwatch) => {
            let elapsed = stopwatch.elapsed();
            let target = config.current_target(session.mode);

            match command {
                Command::Enter if elapsed >= target => {
                    alerter.reset();
                    *ui_mode = UIMode::Running(Stopwatch::default());
                    *session = session.next();
                }
                Command::Pause => stopwatch.stop(),
                Command::Resume => stopwatch.start(),
                Command::Toggle => stopwatch.toggle(),
                Command::Skip => *ui_mode = UIMode::Skip(elapsed),
                _ => (),
            }
        }
    }
}

fn pomodoro_show(
    out: &mut impl Write,
    config: &PomodoroConfig,
    ui_mode: &UIMode,
    session: &Session,
    alerter: &mut Alerter,
) -> Result<()> {
    let target = config.current_target(session.mode);
    let round_number = format!("Session: {}", session.round);
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
                Clear(ClearType::FromCursorDown),
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
                Print(default_title(session.mode)),
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
            let (title, message) = alert_message(session.next().mode);
            alerter.alert_once(title, message);

            queue!(
                out,
                MoveTo(0, 0),
                Print(end_title(session.next().mode)),
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
