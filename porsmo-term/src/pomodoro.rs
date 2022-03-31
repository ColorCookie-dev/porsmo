use crate::{
    input::{listen_command, Command},
    program_tick_duration, writeraw,
};
use anyhow::Result;
use porsmo::pomodoro::*;
use porsmo_helpers::{alert_pomo, fmt_time};
use std::{
    io::{stdout, Write},
    sync::mpsc::Receiver,
    thread,
    time::Duration,
};
use termion::{color, raw::IntoRawMode};

pub fn pomodoro(work: Duration, rest: Duration, long_rest: Duration) -> Result<()> {
    let mut counter = Pomodoro::new(work, rest, long_rest, alert_pomo);

    {
        let mut stdout = stdout().into_raw_mode()?;
        let rx = listen_command();

        loop {
            let (title, count, control) = match counter.checked_counter_at() {
                CountType::Count(count) => (
                    get_pomo_title(*counter.mode()),
                    fmt_time(count),
                    "[Q]: Quit, [Space]: pause/resume",
                ),
                CountType::Exceed(count) => (
                    get_rest_title(*counter.mode()),
                    format!("+{}", fmt_time(count)),
                    "[Q]: Quit, [Space]: pause/resume, [Enter]: next session",
                ),
            };

            writeraw! {
                stdout, clear,
                %text title, color color::Magenta, (1, 1)%,
                %text count, runcolor counter.is_running(),(1, 2)%,
                %text control, color color::LightYellow, (1, 3)%,
                %text format_args!("Round: {}", counter.session()), (1, 4)%,
            }

            stdout.flush()?;

            match rx.try_recv() {
                Ok(Command::Quit) => break,
                Ok(Command::Skip) => {
                    counter.pause();
                    if skip_prompt(&mut stdout, &rx, &mut counter)? {
                        break;
                    }
                    counter.resume();
                }
                Ok(Command::Enter) if counter.has_ended() => counter.next_mode(),
                Ok(Command::Enter) | Ok(Command::Space) => counter.toggle(),
                _ => (),
            }

            thread::sleep(program_tick_duration!());
        }
    }

    println!();
    Ok(())
}

fn skip_prompt(
    stdout: &mut impl Write,
    rx: &Receiver<Command>,
    counter: &mut Pomodoro,
) -> Result<bool> {
    loop {
        view_skip_prompt(stdout, counter.check_next_mode(), counter.session())?;
        match rx.try_recv() {
            Ok(Command::Quit) => return Ok(true),
            Ok(Command::Yes) | Ok(Command::Enter) | Ok(Command::Space) => {
                counter.next_mode();
                return Ok(false);
            }
            _ => (),
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn view_skip_prompt(stdout: &mut impl Write, next_mode: Mode, session: u64) -> Result<()> {
    let prompt = get_skip_prompt(next_mode);

    writeraw! {
        stdout, clear,
        %text prompt, (1, 1)%,
        %text "[Q]: Quit, [Y]: Yes, [N]: No", color color::LightYellow, (1, 2)%,
        %text format_args!("Round: {}", session), (1, 3)%,
    }

    stdout.flush()?;

    Ok(())
}

fn get_pomo_title(mode: Mode) -> &'static str {
    match mode {
        Mode::Work => "pomodoro (work)",
        Mode::Rest => "pomodoro (break)",
        Mode::LongRest => "pomodoro (long break)",
    }
}

fn get_rest_title(mode: Mode) -> &'static str {
    match mode {
        Mode::Work => "Time for some break!",
        Mode::Rest => "Time to get back to work",
        Mode::LongRest => "Your break has ended!",
    }
}

fn get_skip_prompt(next_mode: Mode) -> String {
    match next_mode {
        Mode::Work => format!("{}Skip to work?", color::Fg(color::Red)),
        Mode::Rest => format!("{}Skip to break?", color::Fg(color::Green)),
        Mode::LongRest => format!("{}Skip to a long break?", color::Fg(color::Green)),
    }
}
