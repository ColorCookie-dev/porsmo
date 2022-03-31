use std::{
    io::stdin,
    sync::mpsc::{self, Receiver},
    thread,
};
use termion::{event::Key, input::TermRead};

#[derive(Debug)]
pub enum Command {
    Quit,
    Space,
    Enter,
    Reset,
    Skip,
    Yes,
    No,
}

pub fn listen_command() -> Receiver<Command> {
    let (tx, rx) = mpsc::sync_channel::<Command>(3);

    thread::spawn(move || {
        let stdin = stdin().keys();
        for c in stdin {
            match c.unwrap() {
                Key::Char('q') => tx.try_send(Command::Quit).ok(),
                Key::Ctrl('c') => tx.try_send(Command::Quit).ok(),
                Key::Ctrl('z') => tx.try_send(Command::Quit).ok(),

                Key::Char('S') => tx.try_send(Command::Skip).ok(),
                Key::Char('y') => tx.try_send(Command::Yes).ok(),
                Key::Char('n') => tx.try_send(Command::No).ok(),

                Key::Ctrl('r') => tx.try_send(Command::Reset).ok(),

                Key::Char('\n') => tx.try_send(Command::Enter).ok(),
                Key::Char(' ') => tx.try_send(Command::Space).ok(),

                _ => None,
            };
        }
    });

    rx
}
