pub fn fmt_time(secs: u64) -> String {
    if secs < 60 {
        format!("00:{:0>2}", secs)
    } else if secs < 3600 {
        format!("{:0>2}:{:0>2}", secs / 60, secs % 60)
    } else if secs < 86_400 {
        format!(
            "{:0>2}:{:0>2}:{:0>2}",
            secs / 3600,
            secs / 60 % 60,
            secs % 60
        )
    } else {
        format!(
            "{}days {:0>2}:{:0>2}:{:0>2}",
            secs / 86_400,
            secs / 3600 % 24,
            secs / 60 % 60,
            secs % 60
        )
    }
}
