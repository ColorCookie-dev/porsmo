use crate::counter::*;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub trait HasEnd {
    fn has_ended(&self) -> bool;
}

pub trait Extended<T> {
    fn extended_at(&self) -> T;
}

pub enum CountType<T> {
    Count(T),
    Exceed(T),
}

pub trait CheckedCount<T> {
    fn checked_counter_at(&self) -> CountType<T>;
}

impl<T, U> CheckedCount<U> for T
where
    T: HasEnd + Extended<U> + Counter<U>,
{
    fn checked_counter_at(&self) -> CountType<U> {
        if self.has_ended() {
            CountType::Exceed(self.extended_at())
        } else {
            CountType::Count(self.counter_at())
        }
    }
}

pub struct Timer {
    counter: TimeCount,
    target: Duration,
}

impl Timer {
    pub fn new_alert_timer(target: Duration, alert: impl Fn() + Send + 'static) -> AlertTimer {
        let counter = Arc::new(Mutex::new(Self {
            counter: TimeCount::default(),
            target,
        }));

        let counter_clone = counter.clone();
        thread::spawn(move || {
            let counter = counter_clone;

            loop {
                thread::sleep(Duration::from_millis(100));

                let counter = counter.lock().unwrap();
                if counter.has_ended() {
                    alert();
                    break;
                }
            }
        });

        counter
    }

    pub fn new(target: Duration) -> Self {
        Self {
            counter: TimeCount::default(),
            target,
        }
    }
}

impl HasEnd for Timer {
    fn has_ended(&self) -> bool {
        self.counter.counter_at() > self.target
    }
}

impl Pausable for Timer {
    fn is_running(&self) -> bool {
        self.counter.is_running()
    }

    fn is_paused(&self) -> bool {
        self.counter.is_paused()
    }

    fn pause(&mut self) {
        self.counter.pause();
    }

    fn resume(&mut self) {
        self.counter.resume();
    }

    fn toggle(&mut self) {
        self.counter.toggle();
    }
}

impl Reset for Timer {
    fn reset(&mut self) {
        self.counter.reset();
    }
}

impl Counter<Duration> for Timer {
    fn counter_at(&self) -> Duration {
        self.target.saturating_sub(self.counter.counter_at())
    }
}

impl Extended<Duration> for Timer {
    fn extended_at(&self) -> Duration {
        self.counter.counter_at().saturating_sub(self.target)
    }
}

// ------------ Not in use
pub type AlertTimer = Arc<Mutex<Timer>>;

impl HasEnd for AlertTimer {
    fn has_ended(&self) -> bool {
        let counter = self.lock().unwrap();
        counter.has_ended()
    }
}

impl Pausable for AlertTimer {
    fn is_running(&self) -> bool {
        let counter = self.lock().unwrap();
        counter.is_running()
    }

    fn is_paused(&self) -> bool {
        let counter = self.lock().unwrap();
        counter.is_paused()
    }

    fn pause(&mut self) {
        let mut counter = self.lock().unwrap();
        counter.pause();
    }

    fn resume(&mut self) {
        let mut counter = self.lock().unwrap();
        counter.resume();
    }

    fn toggle(&mut self) {
        let mut counter = self.lock().unwrap();
        counter.toggle();
    }
}

impl Reset for AlertTimer {
    fn reset(&mut self) {
        let mut counter = self.lock().unwrap();
        counter.reset();
    }
}

impl Counter<Duration> for AlertTimer {
    fn counter_at(&self) -> Duration {
        let counter = self.lock().unwrap();
        counter.counter_at()
    }
}

impl Extended<Duration> for AlertTimer {
    fn extended_at(&self) -> Duration {
        let counter = self.lock().unwrap();
        counter.extended_at()
    }
}
