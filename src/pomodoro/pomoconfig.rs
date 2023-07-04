use std::time::Duration;

#[derive(Copy, Clone, Debug)]
pub struct PomoConfig {
    pub work_time: Duration,
    pub break_time: Duration,
    pub long_break: Duration,
}

impl Default for PomoConfig {
    fn default() -> Self {
        Self::short()
    }
}

impl PomoConfig {
    pub fn new(
        work_time: Duration,
        break_time: Duration,
        long_break: Duration,
    ) -> Self {
        Self {
            work_time,
            break_time,
            long_break,
        }
    }

    pub fn short() -> Self {
        Self {
            work_time: Duration::from_secs(25 * 60),
            break_time: Duration::from_secs(5 * 60),
            long_break: Duration::from_secs(10 * 60),
        }
    }

    pub fn long() -> Self {
        Self {
            work_time: Duration::from_secs(55 * 60),
            break_time: Duration::from_secs(10 * 60),
            long_break: Duration::from_secs(20 * 60),
        }
    }
}

