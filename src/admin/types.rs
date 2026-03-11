//! Request and response types for the Searchcraft management APIs.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// The `{ status, data }` envelope used by many Searchcraft endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: u16,
    pub data: T,
}

/// Supported field types in Searchcraft indices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    Text,
    Facet,
    Bool,
    F64,
    U64,
    I64,
    Datetime,
    Bytes,
}

/// Configuration for a single field in an index.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldConfig {
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stored: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fast: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<String>,
}

/// Full index configuration.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IndexConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_commit_delay: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_language_stemming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_stop_words: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<HashMap<String, FieldConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_fields: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_decay_field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_multipliers: Option<HashMap<String, f64>>,
}

/// Response from `GET /index`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexListResponse {
    pub index_names: Vec<String>,
}

/// Statistics for a single index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub document_count: u64,
}

/// Response from `GET /index/stats`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllIndexStatsResponse {
    pub index_count: u64,
    pub indices: Vec<HashMap<String, IndexStats>>,
    pub total_document_count: u64,
}

/// Configuration for a single index within a federation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationIndexConfig {
    pub name: String,
    pub weight_multiplier: f64,
}

/// Full federation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Federation {
    pub name: String,
    pub friendly_name: String,
    pub created_at: String,
    pub created_by: String,
    pub last_modified: String,
    pub last_modified_by: String,
    pub organization_id: String,
    pub index_configurations: Vec<FederationIndexConfig>,
}

/// Statistics for a federation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationStats {
    pub federation_name: String,
    pub num_docs: u64,
    pub indices: Vec<FederationIndexStats>,
}

/// Per-index stats within a federation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationIndexStats {
    pub index_name: String,
    pub num_docs: u64,
    pub space_usage: u64,
}

/// Response from document delete operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDeleteResponse {
    pub detail: String,
    pub num_removed: u64,
}

/// Key permission levels: 1 = read, 15 = ingest, 63 = admin.
///
/// Serializes as an integer (e.g. `1`, `15`, `63`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthKeyPermission {
    /// Read-only access (permission level 1).
    Read,
    /// Ingest (write) access (permission level 15).
    Ingest,
    /// Full admin access (permission level 63).
    Admin,
}

impl Serialize for AuthKeyPermission {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Read => serializer.serialize_u8(1),
            Self::Ingest => serializer.serialize_u8(15),
            Self::Admin => serializer.serialize_u8(63),
        }
    }
}

impl<'de> Deserialize<'de> for AuthKeyPermission {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u8::deserialize(deserializer)?;
        match value {
            1 => Ok(Self::Read),
            15 => Ok(Self::Ingest),
            63 => Ok(Self::Admin),
            other => Err(serde::de::Error::custom(format!(
                "unknown permission level: {other}"
            ))),
        }
    }
}

/// Status of an authentication key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthKeyStatus {
    Active,
    Inactive,
}

/// An authentication key as returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthKey {
    pub key: String,
    pub name: String,
    pub permissions: AuthKeyPermission,
    pub allowed_indexes: Vec<String>,
    pub status: AuthKeyStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub federation_name: Option<String>,
}

/// Request payload to create a new authentication key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuthKeyRequest {
    pub allowed_indexes: Vec<String>,
    pub permissions: AuthKeyPermission,
    pub name: String,
    pub status: AuthKeyStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub federation_name: Option<String>,
}

/// Request payload to update an existing authentication key.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateAuthKeyRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_indexes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<AuthKeyPermission>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AuthKeyStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub federation_name: Option<String>,
}

/// Response from the health check endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: u16,
    pub data: String,
}

/// Properties attached to a measure event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureRequestProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub searchcraft_organization_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub searchcraft_application_id: Option<String>,
    pub searchcraft_index_names: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_term: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_documents: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_position: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// User properties attached to a measure event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureRequestUser {
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdk_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdk_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
}

/// A measure event request payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureEvent {
    pub event_name: String,
    pub properties: MeasureRequestProperties,
    pub user: MeasureRequestUser,
}

/// Synonym map: synonym key → list of terms.
pub type SynonymsMap = HashMap<String, Vec<String>>;
