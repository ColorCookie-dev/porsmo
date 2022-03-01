use crate::{
    input::{listen_command, Command},
    terminal::RawTerm,
};
use anyhow::Result;
use porsmo::pomodoro::{CountType, Mode, Pomodoro};
use porsmo_helpers::{alert, fmt_time};
use std::{io::Write, sync::mpsc::Receiver, thread, time::Duration};
use termion::color;

pub fn pomodoro(work: u64, rest: u64, long_rest: u64) -> Result<()> {
    let mut stdout = RawTerm::default();
    let rx = listen_command();
    let mut alerted = false;

    let mut counter = Pomodoro::new(
        Duration::from_secs(work),
        Duration::from_secs(rest),
        Duration::from_secs(long_rest),
    );

    loop {
        stdout.clear()?;
        match counter.counter_at() {
            CountType::Count(c) => {
                stdout.set_color(color::Magenta)?;

                match counter.mode() {
                    Mode::Work => stdout.write_line("Pomodoro (Work)")?,
                    Mode::Rest => stdout.write_line("Pomodoro (Break)")?,
                    Mode::LongRest => stdout.write_line("Pomodoro (Long Break)")?,
                }

                if counter.is_running() {
                    stdout.set_color(color::Green)?;
                } else {
                    stdout.set_color(color::Red)?;
                }

                stdout.write_line(fmt_time(c.as_secs()))?;

                stdout.set_color(color::LightYellow)?;
                stdout.write_line("[Q]: Quit, [Space]: pause/resume")?;
            }
            CountType::Exceed(c) => {
                if !alerted {
                    alerted = true;
                    alert_pomo(counter.check_next_mode());
                }

                match counter.mode() {
                    Mode::Work => {
                        stdout.set_color(color::Magenta)?;
                        stdout.write_line("Time for some break!")?;
                    }
                    Mode::Rest => {
                        stdout.set_color(color::Magenta)?;
                        stdout.write_line("Time to get back to work")?;
                    }
                    Mode::LongRest => {
                        stdout.set_color(color::Magenta)?;
                        stdout.write_line("Your break has ended!")?;
                    }
                }

                if counter.is_running() {
                    stdout.set_color(color::Green)?;
                } else {
                    stdout.set_color(color::Red)?;
                }

                stdout.write_raw_line(format!("+{}", fmt_time(c.as_secs())), 2)?;

                stdout.set_color(color::LightYellow)?;
                stdout.write_line("[Q]: Quit, [Space]: pause/resume, [Enter]: next session")?;
            }
        }

        stdout.set_color(color::LightCyan)?;
        stdout.write_line(format!("Round: {}", counter.session()))?;

        stdout.flush()?;

        match rx.try_recv() {
            Ok(Command::Quit) => {
                break;
            }

            Ok(Command::Space) => {
                counter.toggle();
            }

            Ok(Command::Skip) => {
                counter.pause();

                match skip_prompt(
                    &mut stdout,
                    &rx,
                    counter.check_next_mode(),
                    counter.session(),
                )? {
                    SkipAns::Skip => {
                        counter.next_mode();
                        alerted = false;
                    }
                    SkipAns::Quit => break,
                    SkipAns::No => (),
                };

                counter.resume();
            }

            Ok(Command::Enter) => {
                if counter.has_ended() {
                    counter.next_mode();
                    alerted = false;
                }
            }

            _ => (),
        }

        thread::sleep(Duration::from_millis(100));
    }

    stdout.destroy();
    match counter.counter_at() {
        CountType::Count(c) => println!("{}", fmt_time(c.as_secs())),
        CountType::Exceed(c) => println!("+{}", fmt_time(c.as_secs())),
    };

    Ok(())
}

enum SkipAns {
    Skip,
    No,
    Quit,
}

fn skip_prompt<T: Write>(
    stdout: &mut RawTerm<T>,
    rx: &Receiver<Command>,
    next_mode: Mode,
    session: u64,
) -> Result<SkipAns> {
    loop {
        match rx.try_recv() {
            Ok(Command::Quit) => {
                return Ok(SkipAns::Quit);
            }

            Ok(Command::No) => {
                return Ok(SkipAns::No);
            }

            Ok(Command::Yes) | Ok(Command::Enter) | Ok(Command::Space) => {
                return Ok(SkipAns::Skip);
            }

            _ => (),
        }

        stdout.clear()?;
        match next_mode {
            Mode::Work => {
                stdout.set_color(color::Red)?;
                stdout.write_line("Skip to work?")?;
            }
            Mode::Rest => {
                stdout.set_color(color::Green)?;
                stdout.write_line("Skip to break?")?;
            }
            Mode::LongRest => {
                stdout.set_color(color::Green)?;
                stdout.write_line("Skip to a long break?")?;
            }
        }

        stdout.set_color(color::LightYellow)?;
        stdout.write_line("[Q]: Quit, [Y]: Yes, [N]: No")?;
        stdout.write_line(format!("Round: {}", session))?;
        stdout.flush()?;

        thread::sleep(Duration::from_millis(100));
    }
}

pub fn alert_pomo(next_mode: Mode) {
    let (heading, message) = match next_mode {
        Mode::Work => ("Your break ended!", "Time for some work"),
        Mode::Rest => ("Pomodoro ended!", "Time for a short break"),
        Mode::LongRest => ("Pomodoro 4 sessions complete!", "Time for a long break"),
    };

    alert(heading.into(), message.into());
}
