use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum BackgroundError {
    #[error("Navigation failed: {0}")]
    NavigationFailed(String),
    #[error("JavaScript execution failed: {0}")]
    JavaScriptFailed(String),
    #[error("Task timed out")]
    Timeout,
    #[error("Worker creation failed: {0}")]
    WorkerCreationFailed(String),
    #[error("Browser creation failed: {0}")]
    BrowserCreationFailed(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}
