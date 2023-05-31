use std::time::Duration;

pub trait Counter {
    fn is_running(&self) -> bool;

    fn has_ended(&self) -> bool;

    fn elapsed(&self) -> Duration;

    fn pause(&mut self);

    fn resume(&mut self);

    fn end_count(&mut self);

    fn toggle(&mut self);
}
