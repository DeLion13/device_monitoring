use thiserror::Error;
use tokio::task::JoinError;

/// Alias for a standard [`Result`], but with [`DeviceSimulatorError`]
pub type DeviceSimulatorResult<T> = Result<T, DeviceSimulatorError>;

/// Highlevel error for device simulator, that helps to cover all results
/// gracefully and avoid unwrapping and panicks everywhere
#[derive(Error, Debug)]
pub enum DeviceSimulatorError {
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    FuturesJoin(#[from] JoinError),

    #[error("POST request error")]
    PostRequest {
        #[source]
        source: reqwest::Error,
    },
}
