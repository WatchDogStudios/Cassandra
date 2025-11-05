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

#[cfg(feature = "db")]
impl From<sqlx::Error> for PlatformError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => PlatformError::NotFound("record"),
            sqlx::Error::Database(db_err) => {
                if db_err.code().as_deref() == Some("23505") {
                    PlatformError::Conflict("record")
                } else {
                    PlatformError::Internal("database error")
                }
            }
            _ => PlatformError::Internal("database error"),
        }
    }
}
