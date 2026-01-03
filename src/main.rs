mod cli;
mod clock;
mod format;
mod prelude;
mod terminal;

use crate::clock::Clock;
use crate::format::{format_duration, format_duration_short};
use crate::prelude::*;
use crate::terminal::running_color;
use clap::Parser;
use cli::{Cli, CounterMode, PomoMode};
use crossterm::cursor::{MoveTo, MoveToNextLine};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::queue;
use crossterm::style::{Color, Print, Stylize};
use crossterm::terminal::{Clear, ClearType};
use notify_rust::Notification;
use rodio::OutputStreamBuilder;
use std::fmt::Display;
use std::{io::BufReader, io::Cursor, io::Write, time::Duration};
use terminal::TerminalHandler;

pub const TIMEOUT: Duration = Duration::from_millis(250);
const FILE: &'static [u8] = include_bytes!("notify_end.wav");

fn main() -> Result<()> {
    let args = Cli::parse();
    // let exitmessagestring = match args.mode {
    match args.mode {
        Some(CounterMode::Stopwatch) => stopwatch_loop()?,
        Some(CounterMode::Countdown { target }) => timer_loop(target)?,
        Some(CounterMode::Pomodoro {
            mode: PomoMode::Short,
            exitmessage: _,
        })
        | None => pomodoro_loop(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(10 * 60),
        )?,
        Some(CounterMode::Pomodoro {
            mode: PomoMode::Long,
            exitmessage: _,
        }) => pomodoro_loop(
            Duration::from_secs(50 * 60),
            Duration::from_secs(10 * 60),
            Duration::from_secs(20 * 60),
        )?,
        Some(CounterMode::Pomodoro {
            mode:
                PomoMode::Custom {
                    work_time,
                    break_time,
                    long_break,
                },
            exitmessage: _,
        }) => pomodoro_loop(work_time, break_time, long_break)?,
    };

    Ok(())
}

pub fn stopwatch_loop() -> Result<()> {
    let mut terminal = TerminalHandler::new()?;
    let output = terminal.stdout();
    let mut clock = Clock::default();

    loop {
        let elapsed = clock.elapsed();
        let color = running_color(clock.is_running());
        show_ui(
            output,
            "Stopwatch",
            format_duration(elapsed).with(color),
            "[Q]: quit, [Space]: Pause/Resume",
        )?;
        if event::poll(TIMEOUT)? {
            let key = match event::read()? {
                Event::Key(key) => key,
                _ => continue,
            };

            match key {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                } => break,

                KeyEvent {
                    code: KeyCode::Char(' '),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                } => clock.toggle(),

                _ => continue,
            }
        }
    }

    drop(terminal);

    println!(
        "Stopwatch ended at: {}.",
        format_duration_short(clock.elapsed())
    );

    Ok(())
}

pub fn timer_loop(target: Duration) -> Result<()> {
    let mut terminal = TerminalHandler::new()?;
    let output = terminal.stdout();
    let mut clock = Clock::default();
    let mut alerted = false;
    let mut stream_handle = OutputStreamBuilder::open_default_stream()?;
    stream_handle.log_on_drop(false);
    let mut _sink;

    loop {
        let elapsed = clock.elapsed();
        let color = running_color(clock.is_running());
        let timer_ended = elapsed >= target;
        const CONTROLS: &str = "[Q]: quit, [Space]: pause/resume, [R]: Reset";
        if timer_ended {
            if !alerted {
                alerted = true;
                let file = BufReader::new(Cursor::new(FILE));
                _sink = rodio::play(stream_handle.mixer(), file)?;
                // alert(
                //     &stream_handle,
                //     "Porsmo Timer",
                //     format!(
                //         "Your timer of {} has ended!",
                //         format_duration_short(target),
                //     ),
                // )?;
            }
            let excess_time = format_duration(elapsed.saturating_sub(target));
            show_ui(
                output,
                "Timer has ended".with(Color::Red),
                format!("+{excess_time}").with(color),
                CONTROLS,
            )?;
        } else {
            let time_left = target.saturating_sub(elapsed);
            show_ui(
                output,
                "Timer",
                format_duration(time_left).with(color),
                CONTROLS,
            )?;
        }
        if event::poll(TIMEOUT)? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => break,

                Event::Key(KeyEvent {
                    code: KeyCode::Char(' '),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => clock.toggle(),

                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => clock.toggle(),

                Event::Key(KeyEvent {
                    code: KeyCode::Char('r'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => {
                    clock.reset();
                }

                _ => continue,
            }
        }
    }
    Ok(())
}

pub enum Mode {
    Work,
    Break,
    LongBreak,
}

pub fn pomodoro_loop(
    work_time: Duration,
    break_time: Duration,
    long_break_time: Duration,
) -> Result<()> {
    let mut terminal = TerminalHandler::new()?;
    let mut alerted = false;
    let output = terminal.stdout();
    let mut worked_time = Duration::ZERO;
    let mut rested_time = Duration::ZERO;
    let mut clock = Clock::default();
    let mut session = 1;
    let mut mode = Mode::Work;
    let mut is_skip_pressed = false;
    let mut stream_handle = OutputStreamBuilder::open_default_stream()?;
    stream_handle.log_on_drop(false);
    let mut _sink;

    loop {
        let elapsed = clock.elapsed();
        let color = running_color(clock.is_running());

        const SKIP_PROMPT: &str = "[Q]: Quit, [Enter/Y]: Yes, [N/Esc]: No";
        const POMO_PROMPT: &str = "[Q]: Quit, [S]: Skip, [Space]: Pause/Resume, [R]: Reset";
        const END_PROMPT: &str = "[Q]: Quit, [Enter]: Next, [Space]: Pause/Resume, [R]: Reset";

        match mode {
            _ if is_skip_pressed => show_pomo_ui(
                output,
                String::from("Skip this session?").with(Color::Red),
                format_duration(work_time.saturating_sub(elapsed)).with(color),
                SKIP_PROMPT,
                session,
            )?,

            Mode::Work if elapsed < work_time => show_pomo_ui(
                output,
                "Pomodoro (Work)",
                format_duration(work_time.saturating_sub(elapsed)).with(color),
                POMO_PROMPT,
                session,
            )?,

            Mode::Work => {
                // Work Ended
                if !alerted {
                    alerted = true;
                    let file = BufReader::new(Cursor::new(FILE));
                    _sink = rodio::play(stream_handle.mixer(), file)?;
                    // alert(
                    //     &stream_handle,
                    //     "Pomodoro Ended",
                    //     "Time for a break!")?;
                }
                show_pomo_ui(
                    output,
                    "Time for a break!".with(Color::Red),
                    format!("+{}", format_duration(elapsed.saturating_sub(work_time))).with(color),
                    END_PROMPT,
                    session,
                )?
            }

            Mode::Break if elapsed < break_time => show_pomo_ui(
                output,
                "Enjoy your break!".with(Color::Blue),
                format_duration(break_time.saturating_sub(elapsed)).with(color),
                POMO_PROMPT,
                session,
            )?,

            Mode::Break => {
                // Break time ended
                if !alerted {
                    alerted = true;
                    let file = BufReader::new(Cursor::new(FILE));
                    _sink = rodio::play(stream_handle.mixer(), file)?;
                    // alert(&stream_handle,
                    //     "Break Ended",
                    //     "Time to start working!")?;
                }
                show_pomo_ui(
                    output,
                    "Time to start working!".with(Color::Red),
                    format!("+{}", format_duration(elapsed.saturating_sub(break_time))).with(color),
                    END_PROMPT,
                    session,
                )?
            }

            Mode::LongBreak if elapsed < long_break_time => show_pomo_ui(
                output,
                "Give your mind some rest!".with(Color::Blue),
                format_duration(long_break_time.saturating_sub(elapsed)).with(color),
                POMO_PROMPT,
                session,
            )?,

            Mode::LongBreak => {
                // Long break ended
                if !alerted {
                    alerted = true;
                    let file = BufReader::new(Cursor::new(FILE));
                    _sink = rodio::play(stream_handle.mixer(), file)?;
                    // alert(&stream_handle,
                    //     "Break Ended",
                    //     "Time to start working!")?;
                }
                show_pomo_ui(
                    output,
                    "Time to start working!".with(Color::Red),
                    format!(
                        "+{}",
                        format_duration(elapsed.saturating_sub(long_break_time))
                    )
                    .with(color),
                    END_PROMPT,
                    session,
                )?
            }
        };

        if event::poll(TIMEOUT)? {
            let ev = match event::read()? {
                Event::Key(ev) => ev,
                _ => continue,
            };

            if is_skip_pressed {
                match ev {
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        kind: KeyEventKind::Press,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => break,

                    KeyEvent {
                        code: KeyCode::Char('s'),
                        kind: KeyEventKind::Press,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        is_skip_pressed = true;
                    }

                    KeyEvent {
                        code: KeyCode::Enter,
                        kind: KeyEventKind::Press,
                        modifiers: KeyModifiers::NONE,
                        ..
                    }
                    | KeyEvent {
                        code: KeyCode::Char('y'),
                        kind: KeyEventKind::Press,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        is_skip_pressed = false;
                        match mode {
                            Mode::Work if session % 4 == 0 => {
                                mode = Mode::LongBreak;
                                worked_time += work_time;
                            }
                            Mode::Work => {
                                mode = Mode::Break;
                                worked_time += break_time;
                            }
                            Mode::Break => {
                                mode = Mode::Work;
                                session += 1;
                                rested_time += break_time;
                            }
                            Mode::LongBreak => {
                                mode = Mode::Work;
                                session += 1;
                                rested_time += long_break_time;
                            }
                        }
                        clock.reset();
                        alerted = false;
                    }

                    KeyEvent {
                        code: KeyCode::Char('n'),
                        kind: KeyEventKind::Press,
                        modifiers: KeyModifiers::NONE,
                        ..
                    }
                    | KeyEvent {
                        code: KeyCode::Esc,
                        kind: KeyEventKind::Press,
                        modifiers: KeyModifiers::NONE,
                        ..
                    } => {
                        is_skip_pressed = false;
                    }

                    _ => continue,
                }
            } else {
                match mode {
                    Mode::Work => match ev {
                        KeyEvent {
                            code: KeyCode::Char('q'),
                            kind: KeyEventKind::Press,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => break,

                        KeyEvent {
                            code: KeyCode::Char('s'),
                            kind: KeyEventKind::Press,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } if elapsed < work_time => {
                            is_skip_pressed = true;
                        }

                        KeyEvent {
                            code: KeyCode::Enter,
                            kind: KeyEventKind::Press,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } if elapsed >= work_time => {
                            if session % 4 == 0 {
                                mode = Mode::LongBreak;
                            } else {
                                mode = Mode::Break;
                            }
                            worked_time += work_time;
                            clock.reset();
                            alerted = false;
                        }

                        KeyEvent {
                            code: KeyCode::Char(' '),
                            kind: KeyEventKind::Press,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => clock.toggle(),

                        KeyEvent {
                            code: KeyCode::Char('r'),
                            kind: KeyEventKind::Press,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => clock.reset(),

                        _ => continue,
                    },

                    Mode::Break => match ev {
                        KeyEvent {
                            code: KeyCode::Char('q'),
                            kind: KeyEventKind::Press,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => break,

                        KeyEvent {
                            code: KeyCode::Char('s'),
                            kind: KeyEventKind::Press,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } if elapsed < break_time => {
                            is_skip_pressed = true;
                        }

                        KeyEvent {
                            code: KeyCode::Enter,
                            kind: KeyEventKind::Press,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } if elapsed >= break_time => {
                            mode = Mode::Work;
                            session += 1;
                            rested_time += break_time;
                            clock.reset();
                            alerted = false;
                        }

                        KeyEvent {
                            code: KeyCode::Char(' '),
                            kind: KeyEventKind::Press,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => clock.toggle(),

                        KeyEvent {
                            code: KeyCode::Char('r'),
                            kind: KeyEventKind::Press,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => clock.reset(),

                        _ => continue,
                    },

                    Mode::LongBreak => match ev {
                        KeyEvent {
                            code: KeyCode::Char('q'),
                            kind: KeyEventKind::Press,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => break,

                        KeyEvent {
                            code: KeyCode::Char('s'),
                            kind: KeyEventKind::Press,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } if elapsed < long_break_time => {
                            is_skip_pressed = true;
                        }

                        KeyEvent {
                            code: KeyCode::Enter,
                            kind: KeyEventKind::Press,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } if elapsed >= long_break_time => {
                            mode = Mode::Work;
                            session += 1;
                            rested_time += long_break_time;
                            clock.reset();
                            alerted = false;
                        }

                        KeyEvent {
                            code: KeyCode::Char(' '),
                            kind: KeyEventKind::Press,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => clock.toggle(),

                        KeyEvent {
                            code: KeyCode::Char('r'),
                            kind: KeyEventKind::Press,
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => clock.reset(),

                        _ => continue,
                    },
                }
            }
        }
    }
    drop(terminal);

    println!(
        "You have worked for {}, and rested {}.",
        format_duration_short(worked_time),
        format_duration_short(rested_time),
    );

    Ok(())
}

pub fn show_ui(
    output: &mut impl Write,
    title: impl Display,
    clock: impl Display,
    controls: &str,
) -> Result<()> {
    queue!(
        output,
        MoveTo(0, 0),
        Print(title),
        Clear(ClearType::UntilNewLine),
        MoveToNextLine(1),
        Print(clock),
        Clear(ClearType::UntilNewLine),
        MoveToNextLine(1),
        Print(controls),
        Clear(ClearType::FromCursorDown),
    )?;
    output.flush()?;
    Ok(())
}

pub fn show_pomo_ui(
    output: &mut impl Write,
    title: impl Display,
    clock: impl Display,
    controls: impl Display,
    session: u32,
) -> Result<()> {
    queue!(
        output,
        MoveTo(0, 0),
        Print(title),
        Clear(ClearType::UntilNewLine),
        MoveToNextLine(1),
        Print(clock),
        Clear(ClearType::UntilNewLine),
        MoveToNextLine(1),
        Print(controls),
        Clear(ClearType::UntilNewLine),
        MoveToNextLine(1),
        Print(format!("Session: {}", session)),
        Clear(ClearType::FromCursorDown),
    )?;
    output.flush()?;
    Ok(())
}

// pub fn alert(
//     stream_handle: &OutputStream,
//     title: impl AsRef<str>,
//     message: impl AsRef<str>
// ) -> Result<()> {
//     // notifica::notify("Porsmo", "Your Timer is Up!").unwrap();
//     Ok(())
// }

// pub fn notify_default(title: impl AsRef<str>, message: impl AsRef<str>) -> Result<()> {
//     notify_rust::Notification::new()
//         .appname("Porsmo")
//         .summary("Porsmo Update")
//         .summary("Your Timer is Up!")
//         .timeout(6000)
//         .show()?;
//     Ok(())
// }
