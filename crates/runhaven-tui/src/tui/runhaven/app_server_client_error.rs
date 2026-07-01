use std::error::Error;
use std::fmt;

use super::super::protocol::UnsupportedFamily;
use super::super::service::RunHavenServiceError;

#[derive(Debug)]
pub(crate) enum TypedRequestError {
    Transport {
        method: String,
        message: String,
    },
    Unsupported {
        method: String,
        family: UnsupportedFamily,
    },
    Validation {
        method: String,
        message: String,
    },
    Backend {
        method: String,
        message: String,
    },
    Deserialize {
        method: String,
        source: serde_json::Error,
    },
}

impl fmt::Display for TypedRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport { method, message } => {
                write!(f, "{method} transport error: {message}")
            }
            Self::Unsupported { method, .. } => {
                write!(f, "{method} is not supported by the RunHaven TUI backend")
            }
            Self::Validation { method, message } => {
                write!(f, "{method} validation error: {message}")
            }
            Self::Backend { method, message } => {
                write!(f, "{method} backend error: {message}")
            }
            Self::Deserialize { method, source } => {
                write!(f, "{method} response decode error: {source}")
            }
        }
    }
}

impl Error for TypedRequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Deserialize { source, .. } => Some(source),
            Self::Transport { .. }
            | Self::Unsupported { .. }
            | Self::Validation { .. }
            | Self::Backend { .. } => None,
        }
    }
}

pub(super) fn service_error_to_typed(
    method: String,
    error: RunHavenServiceError,
) -> TypedRequestError {
    match error {
        RunHavenServiceError::Unsupported { family, .. } => {
            TypedRequestError::Unsupported { method, family }
        }
        RunHavenServiceError::Validation { message, .. } => {
            TypedRequestError::Validation { method, message }
        }
        RunHavenServiceError::Backend { message, .. } => {
            TypedRequestError::Backend { method, message }
        }
    }
}
