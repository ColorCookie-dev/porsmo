use anyhow::{Context, Result};
use notify_rust::Notification;

pub fn notify_default(title: &str, message: &str) -> Result<()> {
    Notification::new()
        .appname("Porsmo")
        .summary(title)
        .body(message)
        .show()
        .with_context(|| "Failed to show notification")?;
    Ok(())
}
