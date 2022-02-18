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
    let (tx, rx) = mpsc::channel::<Command>();

    thread::spawn(move || {
        let stdin = stdin().keys();
        for c in stdin {
            match c.unwrap() {
                Key::Char('q') => tx.send(Command::Quit).unwrap(),
                Key::Ctrl('c') => tx.send(Command::Quit).unwrap(),
                Key::Ctrl('z') => tx.send(Command::Quit).unwrap(),

                Key::Char('S') => tx.send(Command::Skip).unwrap(),
                Key::Char('n') => tx.send(Command::No).unwrap(),

                Key::Char('\n') => tx.send(Command::Enter).unwrap(),
                Key::Char('t') => tx.send(Command::Toggle).unwrap(),
                Key::Char(' ') => tx.send(Command::Toggle).unwrap(),

                Key::Char('p') => tx.send(Command::Pause).unwrap(),
                Key::Char('c') => tx.send(Command::Resume).unwrap(),

                _ => (),
            }
        }
    });

    rx
}
