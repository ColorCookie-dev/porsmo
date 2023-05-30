use crate::{notification::notify_default, sound::play_bell};
use std::thread;

pub fn alert(title: String, message: String) {
    thread::spawn(move || {
        notify_default(&title, &message).unwrap();
        play_bell().unwrap();
    });
}
