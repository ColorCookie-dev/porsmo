use crate::{
    alert::alert,
    format::fmt_time,
    input::{listen_for_inputs, Command},
    terminal::{clear, show_prompt},
    terminal::{show_counter, TermRawMode},
};
use anyhow::Result;
use porsmo::{
    counter::Counter,
    pomodoro::{Mode, Pomodoro},
    stopwatch::Stopwatch,
};
use std::{fmt::Display, io::Write, sync::mpsc::Receiver, thread, time::Duration};
use termion::color;

pub fn pomodoro(work_time: u64, break_time: u64, long_break_time: u64) -> Result<u64> {
    let mut pomo = Pomodoro::new(work_time, break_time, long_break_time);
    let stdout = &mut TermRawMode::new().stdout;
    let rx = listen_for_inputs();

    loop {
        match rx.try_recv() {
            Ok(Command::Quit) => {
                break;
            }

            Ok(Command::Pause) => {
                pomo.pause();
            }

            Ok(Command::Resume) => {
                pomo.resume();
            }

            Ok(Command::Toggle) | Ok(Command::Enter) => {
                pomo.toggle();
            }

            Ok(Command::Skip) => {
                pomo.pause();
                if skip_prompt(stdout, &rx, pomo.check_next_mode(), pomo.session())? {
                    pomo.next_mode()
                } else {
                    pomo.resume();
                }
            }

            _ => (),
        }

        if pomo.has_ended() {
            alert_pomo(pomo.check_next_mode());
            let (counter, next) =
                start_excess_counting(stdout, &rx, pomo.check_next_mode(), pomo.session())?;
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

        show_counter(
            stdout,
            title,
            fmt_time(pomo.counter()),
            pomo.is_running(),
            "[Q]: quit, [Space]: pause/resume.",
            format!("Round: {}", pomo.session()),
        )?;

        thread::sleep(Duration::from_millis(100));
    }

    Ok(pomo.counter())
}

fn skip_prompt(
    stdout: &mut impl Write,
    rx: &Receiver<Command>,
    next_mode: Mode,
    session: u64,
) -> Result<bool> {
    loop {
        match rx.try_recv() {
            Ok(Command::Quit) | Ok(Command::No) => {
                return Ok(false);
            }

            Ok(Command::Yes) | Ok(Command::Enter) => {
                return Ok(true);
            }

            _ => (),
        }

        show_prompt_pomo(stdout, next_mode, format!("Round: {}", session))?;

        thread::sleep(Duration::from_millis(100));
    }
}

fn start_excess_counting(
    stdout: &mut impl Write,
    rx: &Receiver<Command>,
    next_mode: Mode,
    session: u64,
) -> Result<(u64, bool)> {
    let mut st = Stopwatch::new(0);

    loop {
        match rx.try_recv() {
            Ok(Command::Quit) => {
                st.end_count();
                break;
            }

            Ok(Command::Pause) => {
                st.pause();
            }

            Ok(Command::Resume) => {
                st.resume();
            }

            Ok(Command::Toggle) => {
                st.toggle();
            }

            Ok(Command::Enter) => return Ok((st.counter(), true)),

            _ => (),
        }

        let title = match next_mode {
            Mode::Work => "Break has ended! Start work?",
            Mode::Break => "Work has ended! Start break?",
            Mode::LongBreak => "Work has ended! Start a long break",
        };

        show_counter(
            stdout,
            title,
            format!("+{}", fmt_time(st.counter())),
            st.is_running(),
            "[Q]: Quit, [Enter]: Start, [Space]: toggle",
            format!("Round: {}", session),
        )?;

        thread::sleep(Duration::from_millis(100));
    }

    Ok((st.counter(), false))
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
    stdout: &mut impl Write,
    next_mode: Mode,
    message: impl Display,
) -> Result<()> {
    clear(stdout)?;
    match next_mode {
        Mode::Work => show_prompt(stdout, "skip to work?", color::Red, message),
        Mode::Break => show_prompt(stdout, "skip to break?", color::Green, message),
        Mode::LongBreak => show_prompt(stdout, "skip to long break?", color::Green, message),
    }
}
