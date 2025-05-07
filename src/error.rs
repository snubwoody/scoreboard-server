use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The method sent is not supported")]
    UnsupportedMethod,

    #[error("transparent")]
    ClientError(#[from] ClientError),

    #[error(transparent)]
    RedisError(#[from] redis::RedisError),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error(transparent)]
    AxumError(#[from] axum::Error),
    #[error(transparent)]
    AxumHttpError(#[from] axum::http::Error),
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    TungsteniteError(#[from] tokio_tungstenite::tungstenite::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientError {
    kind: ClientErrorKind,
    message: String,
}

impl ClientError {
    pub fn new(message: &str, kind: ClientErrorKind) -> Self {
        Self {
            kind,
            message: message.to_owned(),
        }
    }

    pub fn not_found(message: &str) -> Self {
        Self {
            kind: ClientErrorKind::NotFound,
            message: message.to_owned(),
        }
    }

    pub fn kind(&self) -> ClientErrorKind {
        self.kind
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ClientErrorKind {
    NotFound,
    UnsupportedMethod,
}

impl std::error::Error for ClientError {}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let body = Json(json!({"error":"An unknown error occured"}));

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
