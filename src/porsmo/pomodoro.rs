use crate::{
    alert::alert,
    format::fmt_time,
    input::Command,
    terminal::TerminalHandler,
};
use crate::prelude::*;
use crossterm::{style::Color, event};
use porsmo::{
    counter::Counter,
    pomodoro::{Mode, Pomodoro},
    stopwatch::Stopwatch,
};
use std::{fmt::Display, thread, time::Duration};

pub struct PomodoroUI;

impl PomodoroUI {
    pub fn short() -> Result<Duration> {
        pomodoro(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(10 * 60),
        )
    }

    pub fn long() -> Result<Duration> {
        pomodoro(
            Duration::from_secs(55 * 60),
            Duration::from_secs(10 * 60),
            Duration::from_secs(20 * 60),
        )
    }

    pub fn from_secs(work_time: u64, break_time: u64, long_break_time: u64)
    -> Result<Duration> {
        pomodoro(
            Duration::from_secs(work_time),
            Duration::from_secs(break_time),
            Duration::from_secs(long_break_time)
        )
    }
}

pub fn pomodoro(
    work_time: Duration,
    break_time: Duration,
    long_break_time: Duration
    ) -> Result<Duration> {
    let mut pomo = Pomodoro::new(work_time, break_time, long_break_time);
    let mut terminal = TerminalHandler::new()?;

    loop {
        if event::poll(Duration::from_millis(250))
            .with_context(|| "Polling failed")? {
            let event = event::read().with_context(|| "Failed to read event")?;
            let command = Command::from(event);
            match command {
                Command::Quit => break,
                Command::Pause => pomo.pause(),
                Command::Resume => pomo.resume(),
                Command::Toggle | Command::Enter => pomo.toggle(),
                Command::Skip => {
                    pomo.pause();
                    if skip_prompt(&mut terminal, pomo.check_next_mode(), pomo.session())? {
                        pomo.next_mode()
                    } else {
                        pomo.resume();
                    }
                },
                _ => (),
            }
        }

        if pomo.has_ended() {
            alert_pomo(pomo.check_next_mode());
            let (counter, next) =
                start_excess_counting(&mut terminal, pomo.check_next_mode(), pomo.session())?;
            if next {
                pomo.next_mode();
            } else {
                return Ok(counter);
            }
        }

        let title = match pomo.mode() {
            Mode::Work => "Pomodoro (Work)",
            Mode::Break => "Pomodoro (Break)",
            Mode::LongBreak => "Pomodor (Long Break)",
        };

        terminal.show_counter(
            title,
            fmt_time(pomo.elapsed()),
            pomo.is_running(),
            "[Q]: quit, [Space]: pause/resume.",
            format!("Round: {}", pomo.session()),
        )?;

    }

    Ok(pomo.elapsed())
}

fn skip_prompt(
    terminal: &mut TerminalHandler,
    next_mode: Mode,
    session: u64,
) -> Result<bool> {
    loop {
        if event::poll(Duration::from_millis(250))
            .with_context(|| "Polling failed")? {
            let event = event::read().with_context(|| "Failed to read event")?;
            let command = Command::from(event);
            match command {
                Command::Quit | Command::No => return Ok(false),
                Command::Yes => return Ok(true),
                _ => show_prompt_pomo(
                    terminal,
                    next_mode,
                    format!("Round: {}", session)
                )?,
            }
        }

    }
}

fn start_excess_counting(
    terminal: &mut TerminalHandler,
    next_mode: Mode,
    session: u64,
) -> Result<(Duration, bool)> {
    let mut st = Stopwatch::new(Duration::ZERO);

    loop {
        if event::poll(Duration::from_millis(250))
            .with_context(|| "Polling failed")? {
            let event = event::read().with_context(|| "Failed to read event")?;
            let command = Command::from(event);
            match command {
                Command::Quit => {
                    st.end_count();
                    break;
                },
                Command::Pause => st.pause(),
                Command::Resume => st.resume(),
                Command::Toggle => st.toggle(),
                Command::Enter => return Ok((st.elapsed(), true)),
                _ => (),
            }
        }

        let title = match next_mode {
            Mode::Work => "Break has ended! Start work?",
            Mode::Break => "Work has ended! Start break?",
            Mode::LongBreak => "Work has ended! Start a long break",
        };

        terminal.show_counter(
            title,
            format!("+{}", fmt_time(st.elapsed())),
            st.is_running(),
            "[Q]: Quit, [Enter]: Start, [Space]: toggle",
            format!("Round: {}", session),
        )?;

    }

    Ok((st.elapsed(), false))
}

pub fn alert_pomo(next_mode: Mode) {
    let (heading, message) = match next_mode {
        Mode::Work => ("Your break ended!", "Time for some work"),
        Mode::Break => ("Pomodoro ended!", "Time for a short break"),
        Mode::LongBreak => ("Pomodoro 4 sessions complete!", "Time for a long break"),
    };

    alert(heading.into(), message.into());
}

pub fn show_prompt_pomo(
    terminal: &mut TerminalHandler,
    next_mode: Mode,
    message: impl Display,
) -> Result<()> {
    terminal.clear()?;
    match next_mode {
        Mode::Work => terminal.show_prompt("skip to work?", Color::Red, message),
        Mode::Break => terminal.show_prompt("skip to break?", Color::Green, message),
        Mode::LongBreak => terminal.show_prompt("skip to long break?", Color::Green, message),
    }
}
