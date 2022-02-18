use crate::counter::Counter;
use crate::input::{listen_for_inputs, Command};
use crate::notification::notify_default;
use crate::sound::play_bell;
use crate::terminal::{
    clear, show_message, show_message_green, show_message_red, show_message_yellow,
    show_time_paused, show_time_running, TermRawMode,
};
use anyhow::Result;
use std::{io::Stdout, sync::mpsc::Receiver, thread, time::Instant};
use termion::raw::RawTerminal;

pub struct Pomodoro {
    started: Instant,
    counter: u64,
    status: Status,
    mode: Mode,
    session: u64,

    work_time: u64,
    break_time: u64,
    long_break_time: u64,

    stdout_raw: TermRawMode,
    input_receiver: Receiver<Command>,
}

enum Mode {
    Work,
    Break,
    LongBreak,
}

enum Status {
    Running,
    Paused,
    ModeEnded,
    Prompt,
    Ended,
}

impl Pomodoro {
    pub fn new(work_time: u64, break_time: u64, long_break_time: u64) -> Self {
        let stdout_raw = TermRawMode::new();
        let input_receiver = listen_for_inputs();

        Self {
            started: Instant::now(),
            counter: work_time,
            status: Status::Running,
            mode: Mode::Work,
            session: 0,

            work_time,
            break_time,
            long_break_time,

            input_receiver,
            stdout_raw,
        }
    }

    pub fn session(&self) -> u64 {
        self.session + 1
    }

    fn check_next_mode(&self) -> Mode {
        match self.mode {
            Mode::Work => {
                if self.session() % 4 == 0 {
                    Mode::LongBreak
                } else {
                    Mode::Break
                }
            }
            Mode::Break | Mode::LongBreak => Mode::Work,
        }
    }

    fn next_mode(&mut self) {
        match self.check_next_mode() {
            Mode::Work => {
                self.mode = Mode::Work;
                self.counter = self.work_time;
                self.session += 1;
                self.started = Instant::now();
            }
            Mode::Break => {
                self.started = Instant::now();
                self.mode = Mode::Break;
                self.counter = self.break_time;
            }
            Mode::LongBreak => {
                self.started = Instant::now();
                self.mode = Mode::LongBreak;
                self.counter = self.long_break_time;
            }
        }
    }

    fn get_mut_stdout(&mut self) -> &mut RawTerminal<Stdout> {
        &mut self.stdout_raw.stdout
    }

    fn show_prompt(&mut self) -> Result<()> {
        clear(self.get_mut_stdout())?;
        match self.mode {
            Mode::Work => {
                show_message(self.get_mut_stdout(), "skip this work session?", 0)?;
            }
            Mode::Break => {
                show_message(self.get_mut_stdout(), "skip this break?", 0)?;
            }
            Mode::LongBreak => {
                show_message(self.get_mut_stdout(), "skip this long break?", 0)?;
            }
        }

        self.show_session()?;

        show_message(self.get_mut_stdout(), "[Q]: Quit, [Enter]: Yes, [N]: No", 2)?;

        Ok(())
    }

    fn show_session(&mut self) -> Result<()> {
        let session = self.session();
        show_message_yellow(self.get_mut_stdout(), &format!("(Round: {})", session), 1)
    }

    fn show_mode_change(&mut self) -> Result<()> {
        clear(self.get_mut_stdout())?;
        match self.check_next_mode() {
            Mode::Work => show_message_red(self.get_mut_stdout(), "start work?", 0)?,
            Mode::Break => show_message_green(self.get_mut_stdout(), "start break?", 0)?,
            Mode::LongBreak => show_message_green(self.get_mut_stdout(), "start long break?", 0)?,
        }
        self.show_session()?;

        show_message(self.get_mut_stdout(), "[Q]: Quit, [Enter]: Start", 2)?;

        Ok(())
    }

    fn show_counter(&mut self) -> Result<()> {
        let counter = self.counter();

        match self.status {
            Status::Running => {
                show_time_running(self.get_mut_stdout(), counter)?;
                self.show_session()?;
                show_message(self.get_mut_stdout(), "[Q]: quit, [Space]: pause/resume", 2)?;
            }

            Status::Paused => {
                show_time_paused(self.get_mut_stdout(), counter)?;
                self.show_session()?;
                show_message(self.get_mut_stdout(), "[Q]: quit, [Space]: pause/resume", 2)?;
            }
            _ => (),
        }

        Ok(())
    }

    fn alert(&self) {
        let heading;
        let message;

        match self.check_next_mode() {
            Mode::Work => {
                heading = "You break ended!";
                message = "Time for some work"
            }
            Mode::Break => {
                heading = "Pomodoro ended!";
                message = "Time for a short break"
            }
            Mode::LongBreak => {
                heading = "Pomodoro ended!";
                message = "Time for a long break"
            }
        }

        thread::spawn(move || {
            notify_default(heading, message).unwrap();
            play_bell().unwrap();
        });
    }
}

impl Counter for Pomodoro {
    fn has_ended(&self) -> bool {
        matches!(self.status, Status::Ended)
    }

    fn is_running(&self) -> bool {
        matches!(self.status, Status::Running)
    }

    fn is_paused(&self) -> bool {
        matches!(self.status, Status::Paused)
    }

    fn counter(&self) -> u64 {
        if self.is_running() {
            let elapsed = self.started.elapsed().as_secs();
            if self.counter > elapsed {
                self.counter - elapsed
            } else {
                0
            }
        } else {
            self.counter
        }
    }

    fn pause(&mut self) {
        if self.is_running() {
            self.counter = self.counter();
            self.status = Status::Paused;
        }
    }

    fn resume(&mut self) {
        if self.is_paused() {
            self.status = Status::Running;
            self.started = Instant::now();
        }
    }

    fn update(&mut self) -> Result<()> {
        match self.input_receiver.try_recv() {
            Ok(Command::Quit) => {
                self.status = Status::Ended;
                return Ok(());
            }

            Ok(Command::Pause) => {
                self.pause();
            }

            Ok(Command::Resume) => {
                self.resume();
            }

            Ok(Command::Toggle) => {
                self.toggle();
            }

            Ok(Command::Enter) => match self.status {
                Status::ModeEnded | Status::Prompt => {
                    self.next_mode();
                    self.status = Status::Running;
                    return Ok(());
                }
                _ => (),
            },

            Ok(Command::Skip) => match self.status {
                Status::Running | Status::Paused => {
                    self.status = Status::Prompt;
                    return Ok(());
                }
                _ => (),
            },

            Ok(Command::No) => {
                if let Status::Prompt = self.status {
                    self.status = Status::Running;
                    return Ok(());
                }
            }

            _ => (),
        }

        if self.counter() == 0 {
            self.status = Status::ModeEnded;
            self.alert();
        }

        match self.status {
            Status::ModeEnded => {
                self.show_mode_change()?;
            }

            Status::Running | Status::Paused => {
                self.show_counter()?;
            }

            Status::Prompt => {
                self.show_prompt()?;
            }

            Status::Ended => (),
        }

        Ok(())
    }
}
