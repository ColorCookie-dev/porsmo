use notify_rust::{error::Error, Notification};

#[derive(Debug)]
pub struct NotifyError(Error);

impl std::fmt::Display for NotifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self(error) => {
                write!(f, "Failed to notify: {:?}", error)
            }
        }
    }
}

impl std::error::Error for NotifyError {}

impl From<notify_rust::error::Error> for NotifyError {
    fn from(error: notify_rust::error::Error) -> Self {
        Self(error)
    }
}

pub fn notify_default(title: &str, message: &str) -> Result<(), NotifyError> {
    Notification::new()
        .appname("Porsmo")
        .summary(title)
        .body(message)
        .show()?;
    Ok(())
}
