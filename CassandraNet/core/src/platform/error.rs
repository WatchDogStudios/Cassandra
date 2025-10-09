use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("not found: {0}")]
    NotFound(&'static str),
    #[error("conflict: {0}")]
    Conflict(&'static str),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
    #[error("internal error: {0}")]
    Internal(&'static str),
}

pub type PlatformResult<T> = Result<T, PlatformError>;
