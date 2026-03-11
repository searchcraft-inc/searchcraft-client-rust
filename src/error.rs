//! Error types for the Searchcraft client.
//!
//! All fallible operations return [`Error`], which covers configuration
//! mistakes, authentication failures, HTTP/network problems, and
//! unexpected API responses.

/// The error type returned by all Searchcraft client operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The client configuration is invalid (e.g. missing URL or key).
    #[error("configuration error: {0}")]
    Configuration(String),

    /// The server rejected the request with 401 or 403.
    #[error("authentication error: {message}")]
    Authentication {
        /// Human-readable description.
        message: String,
        /// HTTP status code (401 or 403).
        status: u16,
    },

    /// The requested resource was not found (404).
    #[error("not found: {0}")]
    NotFound(String),

    /// The request payload failed server-side validation (400).
    #[error("validation error: {message}")]
    Validation {
        /// Human-readable description.
        message: String,
        /// Optional field that caused the error.
        field: Option<String>,
    },

    /// The server returned a non-2xx status not covered above.
    #[error("API error (HTTP {status}): {message}")]
    Api {
        /// Human-readable description.
        message: String,
        /// HTTP status code.
        status: u16,
        /// Raw response body, if available.
        body: Option<String>,
    },

    /// A network-level failure (timeout, DNS, connection refused, …).
    #[error("network error: {0}")]
    Network(String),

    /// An unexpected error from the underlying HTTP library.
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

impl Error {
    /// Returns the HTTP status code, if one is associated with this error.
    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Authentication { status, .. } => Some(*status),
            Self::NotFound(_) => Some(404),
            Self::Validation { .. } => Some(400),
            Self::Api { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// Returns `true` if the error indicates the request could be retried.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) | Self::Http(_) => true,
            Self::Api { status, .. } => matches!(status, 429 | 500 | 502 | 503 | 504),
            _ => false,
        }
    }
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Attempt to extract a human-readable message from a JSON error body.
pub(crate) fn extract_error_message(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(String::from))
        .unwrap_or_else(|| body.to_string())
}

/// Map an HTTP status + body into the appropriate [`Error`] variant.
pub(crate) fn map_status_error(status: u16, body: String) -> Error {
    let message = extract_error_message(&body);
    match status {
        401 | 403 => Error::Authentication { message, status },
        404 => Error::NotFound(message),
        400 => Error::Validation {
            message,
            field: None,
        },
        _ => Error::Api {
            message,
            status,
            body: Some(body),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_codes_are_correct() {
        let auth = Error::Authentication {
            message: "bad".into(),
            status: 401,
        };
        assert_eq!(auth.status(), Some(401));

        let not_found = Error::NotFound("gone".into());
        assert_eq!(not_found.status(), Some(404));

        let validation = Error::Validation {
            message: "bad field".into(),
            field: Some("name".into()),
        };
        assert_eq!(validation.status(), Some(400));

        let api = Error::Api {
            message: "fail".into(),
            status: 500,
            body: None,
        };
        assert_eq!(api.status(), Some(500));

        let config = Error::Configuration("missing url".into());
        assert_eq!(config.status(), None);

        let network = Error::Network("timeout".into());
        assert_eq!(network.status(), None);
    }

    #[test]
    fn retryable_classification() {
        assert!(Error::Network("timeout".into()).is_retryable());
        assert!(Error::Api {
            message: "".into(),
            status: 503,
            body: None
        }
        .is_retryable());
        assert!(!Error::Authentication {
            message: "".into(),
            status: 401
        }
        .is_retryable());
        assert!(!Error::NotFound("".into()).is_retryable());
        assert!(!Error::Configuration("".into()).is_retryable());
    }

    #[test]
    fn extract_message_from_json() {
        let body = r#"{"message": "index not found"}"#;
        assert_eq!(extract_error_message(body), "index not found");
    }

    #[test]
    fn extract_message_fallback() {
        assert_eq!(extract_error_message("plain text"), "plain text");
    }

    #[test]
    fn map_status_error_variants() {
        let e = map_status_error(401, r#"{"message":"unauthorized"}"#.into());
        assert!(matches!(e, Error::Authentication { status: 401, .. }));

        let e = map_status_error(404, r#"{"message":"not found"}"#.into());
        assert!(matches!(e, Error::NotFound(_)));

        let e = map_status_error(400, r#"{"message":"bad request"}"#.into());
        assert!(matches!(e, Error::Validation { .. }));

        let e = map_status_error(500, "server error".into());
        assert!(matches!(e, Error::Api { status: 500, .. }));
    }
}
