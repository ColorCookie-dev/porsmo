use crate::{
    format::fmt_time,
    input::{listen_for_inputs, Command},
    notification::notify_default,
    sound::play_bell,
    terminal::{clear, show_message, show_message_green, show_message_red, show_view, TermRawMode},
};
use anyhow::Result;
use porsmo::{counter::Counter, stopwatch::Stopwatch, timer::Timer};
use std::{io::Stdout, sync::mpsc::Receiver, thread, time::Duration};
use termion::raw::RawTerminal;

pub fn timer(time: u64) -> Result<u64> {
    let mut c = Timer::new(time);
    let mut stdout = &mut TermRawMode::new().stdout;
    let rx = listen_for_inputs();
    let counter_ended_at;

    loop {
        match rx.try_recv() {
            Ok(Command::Quit) => {
                c.end_count();
                counter_ended_at = c.counter();
                break;
            }

            Ok(Command::Pause) => {
                c.pause();
            }

            Ok(Command::Resume) => {
                c.resume();
            }

            Ok(Command::Toggle) | Ok(Command::Enter) => {
                c.toggle();
            }

            _ => (),
        }

        if c.has_ended() {
            c.end_count();
            alert_timer_end();
            counter_ended_at = start_excess_counting(&rx, stdout)?;
            break;
        }

        show_view(&mut stdout, c.counter(), c.is_running())?;
        thread::sleep(Duration::from_millis(100));
    }

    Ok(counter_ended_at)
}

fn start_excess_counting(rx: &Receiver<Command>, stdout: &mut RawTerminal<Stdout>) -> Result<u64> {
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

            Ok(Command::Toggle) | Ok(Command::Enter) => {
                st.toggle();
            }

            _ => (),
        }

        show_timer_end(stdout, st.counter(), st.is_running())?;

        thread::sleep(Duration::from_millis(100));
    }

    Ok(st.counter())
}

fn show_timer_end(stdout: &mut RawTerminal<Stdout>, counter: u64, running: bool) -> Result<()> {
    clear(stdout)?;
    show_message_red(stdout, "Timer has ended", 0)?;
    show_message(stdout, "[Q]: Quit, [Space]: Toggle excess counter", 1)?;
    show_extended_time(stdout, counter, running)?;

    Ok(())
}

fn show_extended_time(stdout: &mut RawTerminal<Stdout>, counter: u64, running: bool) -> Result<()> {
    if running {
        show_message_green(stdout, &format!("-{}", fmt_time(counter)), 2)?;
    } else {
        show_message_red(stdout, &format!("-{}", fmt_time(counter)), 2)?;
    };
    Ok(())
}

fn alert_timer_end() {
    thread::spawn(move || {
        notify_default("Timer ended!", "You Porsmo timer has ended").unwrap();
        play_bell().unwrap();
    });
}
