use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("project not found: {0}")]
    ProjectNotFound(String),
    #[error("project not allowed: {0}")]
    ProjectNotAllowed(String),
    #[error("adapter not found: {0}")]
    AdapterNotFound(String),
    #[error("session already running")]
    SessionAlreadyRunning,
    #[error("approval not found: {0}")]
    ApprovalNotFound(String),
    #[error("process error: {0}")]
    Process(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type AppResult<T> = Result<T, AppError>;
