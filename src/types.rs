//! Shared types used across the Searchcraft client.
//!
//! Endpoint-specific request/response types will be added by later tasks.
//! This module provides the common building blocks.

use serde::{Deserialize, Serialize};

/// Sort direction for search results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

/// A generic operation response returned by many management endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResponse {
    /// Whether the operation succeeded.
    pub success: bool,
    /// Optional human-readable message.
    #[serde(default)]
    pub message: Option<String>,
}

/// A generic paginated list wrapper (if the API uses one).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse<T> {
    /// The items in this page.
    pub items: Vec<T>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_direction_serde_roundtrip() {
        let json = serde_json::to_string(&SortDirection::Asc).unwrap();
        assert_eq!(json, r#""asc""#);
        let parsed: SortDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, SortDirection::Asc);
    }

    #[test]
    fn operation_response_deserialize() {
        let json = r#"{"success": true, "message": "created"}"#;
        let resp: OperationResponse = serde_json::from_str(json).unwrap();
        assert!(resp.success);
        assert_eq!(resp.message.as_deref(), Some("created"));
    }

    #[test]
    fn operation_response_without_message() {
        let json = r#"{"success": true}"#;
        let resp: OperationResponse = serde_json::from_str(json).unwrap();
        assert!(resp.success);
        assert!(resp.message.is_none());
    }
}
