use thiserror::Error;

#[derive(Debug, Error)]
pub enum GbError {
    #[error("not a git repository (or any of the parent directories)")]
    NotARepository,

    #[error("git command failed with exit code {code}: {stderr}")]
    GitFailed { code: i32, stderr: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
