use crate::{notification::notify_default, sound::play_bell};
use porsmo::pomodoro::Mode;
use std::thread;

pub fn alert(title: String, message: String) {
    thread::spawn(move || {
        notify_default(&title, &message).unwrap();
        play_bell().unwrap();
    });
}

fn get_alert_messages(next_mode: Mode) -> (&'static str, &'static str) {
    match next_mode {
        Mode::Work => ("Your break ended!", "Time for some work"),
        Mode::Rest => ("Pomodoro ended!", "Time for a short break"),
        Mode::LongRest => ("Pomodoro 4 sessions complete!", "Time for a long break"),
    }
}

pub fn alert_pomo(next_mode: Mode) {
    let (heading, message) = get_alert_messages(next_mode);
    alert(heading.into(), message.into());
}

#[cfg(test)]
mod test {
    use super::*;
    use porsmo::pomodoro::Mode;

    #[test]
    fn alert_msgs() {
        assert_eq!(
            get_alert_messages(Mode::Work),
            ("Your break ended!", "Time for some work")
        );

        assert_eq!(
            get_alert_messages(Mode::Rest),
            ("Pomodoro ended!", "Time for a short break")
        );

        assert_eq!(
            get_alert_messages(Mode::LongRest),
            ("Pomodoro 4 sessions complete!", "Time for a long break")
        );
    }
}
