use std::{
    io::stdin,
    sync::mpsc::{self, Receiver},
    thread,
};
use termion::{event::Key, input::TermRead};

pub enum Command {
    Quit,
    Pause,
    Resume,
    Toggle,
    Enter,
    Skip,
    No,
}

pub fn listen_for_inputs() -> Receiver<Command> {
    let (tx, rx) = mpsc::sync_channel::<Command>(3);

    thread::spawn(move || {
        let stdin = stdin().keys();
        for c in stdin {
            match c.unwrap() {
                Key::Char('q') => tx.try_send(Command::Quit).ok(),
                Key::Ctrl('c') => tx.try_send(Command::Quit).ok(),
                Key::Ctrl('z') => tx.try_send(Command::Quit).ok(),

                Key::Char('S') => tx.try_send(Command::Skip).ok(),
                Key::Char('n') => tx.try_send(Command::No).ok(),

                Key::Char('\n') => tx.try_send(Command::Enter).ok(),
                Key::Char('t') => tx.try_send(Command::Toggle).ok(),
                Key::Char(' ') => tx.try_send(Command::Toggle).ok(),

                Key::Char('p') => tx.try_send(Command::Pause).ok(),
                Key::Char('c') => tx.try_send(Command::Resume).ok(),

                _ => None,
            };
        }
    });

    rx
}
