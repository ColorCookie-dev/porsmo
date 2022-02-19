pub trait Counter {
    fn is_running(&self) -> bool;

    fn is_paused(&self) -> bool;

    fn has_ended(&self) -> bool;

    fn counter(&self) -> u64;

    fn pause(&mut self);

    fn resume(&mut self);

    fn end_count(&mut self);

    fn toggle(&mut self) {
        if self.is_running() {
            self.pause();
        } else if self.is_paused() {
            self.resume();
        }
    }
}
