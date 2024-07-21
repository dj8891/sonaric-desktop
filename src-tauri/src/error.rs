use tauri::updater;

// create the error type that represents all errors possible in our program
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error(transparent)]
    Tauri(#[from] tauri::Error),

    #[error(transparent)]
    RequestError(#[from] reqwest::Error),

    #[error(transparent)]
    SemverError(#[from] semver::Error),

    #[error(transparent)]
    FromUtf16Error(#[from] std::string::FromUtf16Error),

    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    UpdaterError(#[from] updater::Error),

    #[error("{0}")]
    RetryError(&'static str),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}

// we must manually implement serde::Serialize
impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        // skip sending specific error types to the sentry
        match self {
            Error::RetryError(_) => {}
            _ => {
                tracing::error!("{:?}", self);
            }
        }

        serializer.serialize_str(self.to_string().as_ref())
    }
}
