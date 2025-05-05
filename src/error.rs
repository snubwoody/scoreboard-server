use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    RedisError(#[from] redis::RedisError),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}
