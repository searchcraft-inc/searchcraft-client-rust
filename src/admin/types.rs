//! Request and response types for the Searchcraft management APIs.
//!
//! Request types that have optional fields implement [`Default`], so they are
//! best built with struct-update syntax:
//!
//! ```
//! use searchcraft::admin::types::{FieldConfig, FieldType, IndexConfig};
//!
//! let config = IndexConfig {
//!     language: Some("en".into()),
//!     search_fields: Some(vec!["title".into()]),
//!     ..Default::default()
//! };
//!
//! let title = FieldConfig {
//!     stored: Some(true),
//!     ..FieldConfig::new(FieldType::Text)
//! };
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// The `{ status, data }` envelope used by many Searchcraft endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// HTTP status code repeated in the response body.
    pub status: u16,
    /// The endpoint-specific payload.
    pub data: T,
}

/// Supported field types in Searchcraft indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum FieldType {
    /// Tokenized full-text field.
    Text,
    /// Hierarchical facet field, for faceted navigation.
    Facet,
    /// Boolean field.
    Bool,
    /// 64-bit floating-point field.
    F64,
    /// Unsigned 64-bit integer field.
    U64,
    /// Signed 64-bit integer field.
    I64,
    /// Timestamp field.
    Datetime,
    /// Nested JSON field. See [`FieldConfig::tokenizer`] and
    /// [`FieldConfig::expand_dots`] for the JSON-specific options.
    Json,
}

/// Configuration for a single field in an index.
///
/// Build with [`FieldConfig::new`] and override what you need; every option
/// left as `None` is omitted from the request so the engine applies its own
/// default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldConfig {
    /// The field's data type.
    #[serde(rename = "type")]
    pub field_type: FieldType,
    /// Whether the field is indexed and therefore searchable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed: Option<bool>,
    /// Whether the field's value is stored and returned with search hits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stored: Option<bool>,
    /// Whether the field uses a fast (columnar) representation, required for
    /// sorting and aggregation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fast: Option<bool>,
    /// Whether the field holds multiple values per document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi: Option<bool>,
    /// Whether documents must supply this field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// For json fields: how string values within the JSON are indexed —
    /// `"raw"` (exact, case-sensitive), `"lowercase"` (case-insensitive
    /// exact), or `"default"` (tokenized full-text). Defaults to `"raw"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenizer: Option<String>,
    /// For json fields: treat dots in JSON keys as nested-object separators.
    /// Defaults to `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand_dots: Option<bool>,
}

impl FieldConfig {
    /// Creates a field of the given type, leaving every option unset.
    #[must_use]
    pub fn new(field_type: FieldType) -> Self {
        Self {
            field_type,
            indexed: None,
            stored: None,
            fast: None,
            multi: None,
            required: None,
            tokenizer: None,
            expand_dots: None,
        }
    }
}

/// Supported LLM providers for AI-powered features.
///
/// Values this client does not know deserialize into [`LlmProvider::Other`]
/// rather than failing, so a newer engine release cannot break an older client.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    /// Anthropic's API.
    Anthropic,
    /// Amazon Bedrock. Pair with [`AiConfig::llm_region`].
    Bedrock,
    /// Google's API.
    Google,
    /// A llama.cpp server. Pair with [`AiConfig::llm_base_url`].
    Llamacpp,
    /// Mistral's API.
    Mistral,
    /// A local Ollama instance. Pair with [`AiConfig::llm_base_url`].
    Ollama,
    /// OpenAI's API, or an OpenAI-compatible endpoint.
    Openai,
    /// xAI's API.
    Xai,
    /// A provider this client version does not know about.
    #[serde(untagged)]
    Other(String),
}

/// A single custom instruction appended to the summary prompt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptInstruction {
    /// The instruction text added to the prompt.
    pub custom_instruction: String,
    /// Ordering weight; lower numbers appear earlier in the prompt.
    pub order: i64,
}

/// A keyword-based rule that rewrites the summary prompt when it matches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeywordRule {
    /// Identifier for this rule.
    pub rule_name: String,
    /// Keywords that activate the rule when present in the query.
    pub detect_keywords: Vec<String>,
    /// Term used in the prompt in place of the original query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement_term: Option<String>,
    /// Instruction added to the prompt when this rule matches.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_instruction: Option<String>,
    /// Message emitted when a summary matched by this rule returns no results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub empty_state_message: Option<String>,
    /// Ordering weight; lower numbers appear earlier in the prompt.
    pub order: i64,
}

/// Search summary generation configuration.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SearchSummaryConfig {
    /// The LLM model used for summary generation.
    pub model: String,
    /// Role or persona the model should adopt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Maximum length of the generated summary, in characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character_limit: Option<u64>,
    /// Maximum number of search results to feed into the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u64>,
    /// Maximum length each document is trimmed to, in characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_trim_length: Option<u64>,
    /// Sampling temperature, from 0.0 to 1.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Message used when a summary search returns no results. Supports the
    /// `${searchTopic}` placeholder.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub empty_state_message: Option<String>,
    /// Extra instructions appended to the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_prompt_instructions: Option<Vec<PromptInstruction>>,
    /// Keyword rules that rewrite the prompt on a match.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword_rules: Option<Vec<KeywordRule>>,
}

/// AI configuration attached to an index (engine 0.10.0+).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiConfig {
    /// Configuration for search summary generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_summary: Option<SearchSummaryConfig>,
    /// The LLM provider backing this index's AI features.
    pub llm_provider: LlmProvider,
    /// AWS region, for [`LlmProvider::Bedrock`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_region: Option<String>,
    /// Base URL for Ollama, llama.cpp server mode, or OpenAI-compatible APIs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_base_url: Option<String>,
    /// API key for providers that require one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_api_key: Option<String>,
}

impl AiConfig {
    /// Creates a configuration for the given provider, leaving every option
    /// unset.
    #[must_use]
    pub fn new(llm_provider: LlmProvider) -> Self {
        Self {
            search_summary: None,
            llm_provider,
            llm_region: None,
            llm_base_url: None,
            llm_api_key: None,
        }
    }
}

/// Which AI features are configured for an index.
///
/// The server camelCases these fields, unlike the rest of the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiCapabilities {
    /// Whether AI features are enabled for this index.
    pub enabled: bool,
    /// Whether search summary generation is configured.
    pub search_summary_configured: bool,
    /// Whether an LLM provider is configured.
    pub llm_provider_configured: bool,
    /// Whether an LLM model is configured.
    pub llm_model_configured: bool,
}

/// Response payload from `GET /index/{index_name}/capabilities`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexCapabilities {
    /// AI capability status for the index.
    pub ai: AiCapabilities,
}

/// Document ingestion processing options for an index.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestionConfig {
    /// Enrichment pipeline applied to documents before they are indexed.
    /// `None` performs no enrichment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrichment_pipeline: Option<String>,
}

/// Full index configuration.
///
/// Every field is optional: on create, omitted fields take the engine's
/// defaults; on update, omitted fields are left unchanged.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Delay in milliseconds before buffered writes are committed
    /// automatically.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_commit_delay: Option<u64>,
    /// Whether to apply language-specific stemming.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_language_stemming: Option<bool>,
    /// Whether to strip stopwords at index time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_stop_words: Option<bool>,
    /// Field definitions, keyed by field name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<HashMap<String, FieldConfig>>,
    /// Language code used for stemming and stopwords, e.g. `"en"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Fields searched when a query does not name one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_fields: Option<Vec<String>>,
    /// Datetime field used to decay relevance for older documents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_decay_field: Option<String>,
    /// Per-field relevance boosts, keyed by field name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_multipliers: Option<HashMap<String, f64>>,
    /// Document ingestion processing options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ingestion: Option<IngestionConfig>,
    /// Whether AI-powered features are enabled for this index (engine
    /// 0.10.0+). Changing this requires an admin-level key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_enabled: Option<bool>,
    /// AI configuration for LLM-powered features such as search summarization
    /// (engine 0.10.0+).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai: Option<AiConfig>,
}

/// Response from `GET /index`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexListResponse {
    /// Names of every index on the cluster.
    pub index_names: Vec<String>,
}

/// Statistics for a single index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexStats {
    /// Number of documents currently committed to the index.
    pub document_count: u64,
}

/// Response from `GET /index/stats`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllIndexStatsResponse {
    /// Number of indices on the cluster.
    pub index_count: u64,
    /// Per-index stats, each entry keyed by index name.
    pub indices: Vec<HashMap<String, IndexStats>>,
    /// Total documents across every index.
    pub total_document_count: u64,
}

/// Configuration for a single index within a federation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FederationIndexConfig {
    /// Name of the federated index.
    pub name: String,
    /// Relevance multiplier applied to hits from this index.
    pub weight_multiplier: f32,
}

/// Full federation configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Federation {
    /// Machine name of the federation, used in URLs.
    pub name: String,
    /// Human-readable display name.
    pub friendly_name: String,
    /// Creation timestamp.
    pub created_at: String,
    /// Identifier of the creating principal, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// Last modification timestamp.
    pub last_modified: String,
    /// Identifier of the principal that last modified the federation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified_by: Option<String>,
    /// Owning organization, when scoped to one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    /// The indices that make up this federation.
    pub index_configurations: Vec<FederationIndexConfig>,
}

/// Request payload to create or update a federation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FederationRequest {
    /// Machine name of the federation, used in URLs.
    pub name: String,
    /// Human-readable display name.
    pub friendly_name: String,
    /// Identifier of the creating principal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// Identifier of the principal making this change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_by: Option<String>,
    /// Organization to scope the federation to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    /// The indices that make up this federation, with their relevance weights.
    pub index_configurations: Vec<FederationIndexConfig>,
}

impl FederationRequest {
    /// Creates a request for a federation over the given indices, each with a
    /// relevance weight of `1.0`.
    ///
    /// ```
    /// use searchcraft::admin::types::FederationRequest;
    ///
    /// let request = FederationRequest::new("global", "Global", ["products", "articles"]);
    /// assert_eq!(request.index_configurations.len(), 2);
    /// ```
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        friendly_name: impl Into<String>,
        indices: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            friendly_name: friendly_name.into(),
            index_configurations: indices
                .into_iter()
                .map(|name| FederationIndexConfig {
                    name: name.into(),
                    weight_multiplier: 1.0,
                })
                .collect(),
            ..Self::default()
        }
    }
}

/// Statistics for a federation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FederationStats {
    /// Name of the federation these stats describe.
    pub federation_name: String,
    /// Total documents across every federated index.
    pub num_docs: u64,
    /// Per-index breakdown.
    pub indices: Vec<FederationIndexStats>,
}

/// Per-index statistics within a federation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FederationIndexStats {
    /// Name of the index.
    pub index_name: String,
    /// Documents held by this index.
    pub num_docs: u64,
    /// Disk usage in bytes.
    pub space_usage: u64,
}

/// Response from document delete operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentDeleteResponse {
    /// Human-readable description of what was deleted.
    pub detail: String,
    /// Number of documents removed.
    pub num_removed: u64,
}

/// Individual permission bits an authentication key can carry.
///
/// The engine stores key permissions as a bitmask; these are the flags that
/// make it up. Combine them with [`AuthKeyPermission`].
pub mod permissions {
    /// Search the key's permitted indices.
    pub const SEARCH: u64 = 1 << 0;
    /// Add and remove synonyms.
    pub const MODIFY_SYNONYMS: u64 = 1 << 1;
    /// Add and remove stopwords.
    pub const MODIFY_STOP_WORDS: u64 = 1 << 2;
    /// Add and remove documents.
    pub const MODIFY_DOCUMENTS: u64 = 1 << 3;
    /// Create and delete indices.
    pub const MODIFY_INDEX: u64 = 1 << 4;
    /// Create, revoke, and modify auth keys.
    pub const MODIFY_AUTH: u64 = 1 << 5;
    /// Generate AI search summaries.
    pub const LLM_RAG_SUMMARIES: u64 = 1 << 6;
    /// Perform LLM-expanded keyword searches.
    pub const LLM_EXPANDED_KEYWORD_SEARCH: u64 = 1 << 7;
    /// Read analytics from the `/measure/dashboard/*` endpoints.
    pub const READ_ANALYTICS: u64 = 1 << 8;
}

/// The permission bitmask attached to an authentication key.
///
/// The engine models permissions as a set of bit flags (see the
/// [`permissions`] module), not a fixed set of levels, and it grows over time.
/// This type therefore wraps the raw mask and offers named presets plus
/// [`contains`](Self::contains) for testing individual bits.
///
/// The presets come in two kinds. Levels ([`READ`](Self::READ),
/// [`INGEST`](Self::INGEST), [`ADMIN`](Self::ADMIN),
/// [`SUPER_USER`](Self::SUPER_USER)) say what a key can change. Add-ons
/// ([`AI`](Self::AI), [`ANALYTICS`](Self::ANALYTICS)) are features a key can
/// use on top of its level. Combine them with `|`.
///
/// ```
/// use searchcraft::admin::types::{permissions, AuthKeyPermission};
///
/// let key = AuthKeyPermission::INGEST;
/// assert!(key.contains(permissions::MODIFY_DOCUMENTS));
/// assert!(!key.contains(permissions::MODIFY_AUTH));
///
/// // A public key that can also request AI summaries.
/// let public_ai = AuthKeyPermission::READ | AuthKeyPermission::AI;
/// assert_eq!(public_ai.bits(), 65);
///
/// // Raw bits mix in too, and `|=` adds to an existing mask.
/// let mut custom = AuthKeyPermission::READ | permissions::READ_ANALYTICS;
/// custom |= AuthKeyPermission::AI;
/// assert_eq!(custom.bits(), 321);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuthKeyPermission(u64);

impl AuthKeyPermission {
    /// Search only — the conventional "read key" (mask `1`).
    pub const READ: Self = Self(permissions::SEARCH);

    /// Search plus document, synonym, and stopword writes — the conventional
    /// "ingest key" (mask `15`).
    pub const INGEST: Self = Self(
        permissions::SEARCH
            | permissions::MODIFY_SYNONYMS
            | permissions::MODIFY_STOP_WORDS
            | permissions::MODIFY_DOCUMENTS,
    );

    /// Everything an ingest key can do plus index and auth management
    /// (mask `63`). Note this does **not** include the AI or analytics bits;
    /// use [`SUPER_USER`](Self::SUPER_USER) for a full-privilege key.
    pub const ADMIN: Self = Self(
        permissions::SEARCH
            | permissions::MODIFY_SYNONYMS
            | permissions::MODIFY_STOP_WORDS
            | permissions::MODIFY_DOCUMENTS
            | permissions::MODIFY_INDEX
            | permissions::MODIFY_AUTH,
    );

    /// Every permission the engine defines, including AI features and
    /// analytics (mask `511`).
    pub const SUPER_USER: Self = Self(
        permissions::SEARCH
            | permissions::MODIFY_SYNONYMS
            | permissions::MODIFY_STOP_WORDS
            | permissions::MODIFY_DOCUMENTS
            | permissions::MODIFY_INDEX
            | permissions::MODIFY_AUTH
            | permissions::LLM_RAG_SUMMARIES
            | permissions::LLM_EXPANDED_KEYWORD_SEARCH
            | permissions::READ_ANALYTICS,
    );

    /// Add-on: AI summaries and semantic search (mask `64`).
    pub const AI: Self = Self(permissions::LLM_RAG_SUMMARIES);

    /// Add-on: read the measure dashboards (mask `256`).
    pub const ANALYTICS: Self = Self(permissions::READ_ANALYTICS);

    /// Builds a permission mask from raw bits.
    #[must_use]
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    /// Returns the raw bitmask as the API represents it.
    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Returns `true` if every bit in `flags` is set.
    #[must_use]
    pub const fn contains(self, flags: u64) -> bool {
        self.0 & flags == flags
    }

    /// Returns `true` if any bit in `flags` is set.
    #[must_use]
    pub const fn intersects(self, flags: u64) -> bool {
        self.0 & flags != 0
    }
}

impl From<u64> for AuthKeyPermission {
    fn from(bits: u64) -> Self {
        Self(bits)
    }
}

impl From<AuthKeyPermission> for u64 {
    fn from(value: AuthKeyPermission) -> Self {
        value.0
    }
}

impl std::ops::BitOr for AuthKeyPermission {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOr<u64> for AuthKeyPermission {
    type Output = Self;

    fn bitor(self, rhs: u64) -> Self {
        Self(self.0 | rhs)
    }
}

impl std::ops::BitOrAssign for AuthKeyPermission {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitOrAssign<u64> for AuthKeyPermission {
    fn bitor_assign(&mut self, rhs: u64) {
        self.0 |= rhs;
    }
}

/// Status of an authentication key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum AuthKeyStatus {
    /// The key may be used.
    #[default]
    Active,
    /// The key exists but is rejected on use.
    Inactive,
}

/// An authentication key as returned by the API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthKey {
    /// The key value used in the `Authorization` header.
    ///
    /// The engine calls this field `token` on the wire.
    #[serde(rename = "token")]
    pub token: String,
    /// When the key was created. Unchanged by updates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// Human-readable name for the key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// What the key is permitted to do.
    pub permissions: AuthKeyPermission,
    /// Indices the key may access. `None` means every index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_indexes: Option<Vec<String>>,
    /// Whether the key is currently usable.
    #[serde(default)]
    pub status: AuthKeyStatus,
    /// Owning organization, if scoped to one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    /// Name of the owning organization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_name: Option<String>,
    /// Owning application, if scoped to one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_id: Option<String>,
    /// Name of the owning application.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_name: Option<String>,
    /// Federation this key is scoped to. Read keys only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub federation_name: Option<String>,
}

/// Request payload to create a new authentication key.
///
/// Build with [`CreateAuthKeyRequest::new`], which defaults the key to
/// [`AuthKeyStatus::Active`] and leaves every scoping field unset.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateAuthKeyRequest {
    /// Indices the new key may access. `None` grants access to every index.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_indexes: Option<Vec<String>>,
    /// What the new key is permitted to do.
    pub permissions: AuthKeyPermission,
    /// Human-readable name for the key.
    pub name: String,
    /// Whether the key is usable on creation.
    pub status: AuthKeyStatus,
    /// Organization to scope the key to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    /// Name of the organization to scope the key to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_name: Option<String>,
    /// Application to scope the key to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_id: Option<String>,
    /// Name of the application to scope the key to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_name: Option<String>,
    /// Federation to scope the key to. Read keys only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub federation_name: Option<String>,
}

impl CreateAuthKeyRequest {
    /// Creates an active key request with the given name, permissions, and
    /// index scope.
    ///
    /// ```
    /// use searchcraft::admin::types::{AuthKeyPermission, CreateAuthKeyRequest};
    ///
    /// let request = CreateAuthKeyRequest::new(
    ///     "web-frontend",
    ///     AuthKeyPermission::READ,
    ///     ["products"],
    /// );
    /// ```
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        permissions: AuthKeyPermission,
        allowed_indexes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            allowed_indexes: Some(allowed_indexes.into_iter().map(Into::into).collect()),
            permissions,
            name: name.into(),
            status: AuthKeyStatus::Active,
            ..Self::default()
        }
    }
}

/// Request payload to update an existing authentication key.
///
/// Only the fields you set are sent, and only those are changed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateAuthKeyRequest {
    /// Replacement list of accessible indices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_indexes: Option<Vec<String>>,
    /// Replacement permission level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<AuthKeyPermission>,
    /// Replacement name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Replacement status, e.g. to deactivate the key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AuthKeyStatus>,
    /// Replacement organization scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<u64>,
    /// Replacement organization name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_name: Option<String>,
    /// Replacement application scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_id: Option<u64>,
    /// Replacement application name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_name: Option<String>,
    /// Replacement federation scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub federation_name: Option<String>,
}

/// Response from the health check endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// HTTP status code repeated in the response body.
    pub status: u16,
    /// Health description, `"ok"` on a healthy node.
    pub data: String,
}

/// Response payload from `GET /measure/status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasureStatus {
    /// `true` when analytics are configured on the server. When `false`, every
    /// other `/measure/*` endpoint is a no-op.
    pub enabled: bool,
}

/// Measure event names emitted by the engine and the official SDKs.
///
/// [`MeasureEvent::event_name`] is a plain `String` so unknown names stay
/// forward-compatible; these constants cover the names in use today.
pub mod event_names {
    /// An SDK finished initializing.
    pub const SDK_INITIALIZED: &str = "sdk_initialized";
    /// A search was issued by a client.
    pub const SEARCH_REQUESTED: &str = "search_requested";
    /// A client received a search response.
    pub const SEARCH_RESPONSE_RECEIVED: &str = "search_response_received";
    /// A search interaction completed.
    pub const SEARCH_COMPLETED: &str = "search_completed";
    /// A user clicked through to a document.
    pub const DOCUMENT_CLICKED: &str = "document_clicked";
    /// An API request reached the engine.
    pub const API_REQUESTED: &str = "api_requested";
    /// The engine executed a query.
    pub const API_QUERIED: &str = "api_queried";
    /// A search summary was requested (engine 0.10.0+).
    pub const API_SUMMARY_REQUESTED: &str = "api_summary_requested";
}

/// Time bucket size for dashboard reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum MeasureQueryGranularity {
    /// Bucket by minute.
    Minutes,
    /// Bucket by hour.
    Hours,
    /// Bucket by day.
    Days,
    /// Bucket by week.
    Weeks,
    /// Bucket by month.
    Months,
    /// Bucket by year.
    Years,
}

impl MeasureQueryGranularity {
    /// Returns the wire representation.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Minutes => "minutes",
            Self::Hours => "hours",
            Self::Days => "days",
            Self::Weeks => "weeks",
            Self::Months => "months",
            Self::Years => "years",
        }
    }
}

/// Filters accepted by the `/measure/dashboard/*` reports.
///
/// Every field is optional. Scope fields the calling key already carries
/// ([`organization_id`](Self::organization_id),
/// [`application_id`](Self::application_id)) are injected server-side when
/// omitted, and supplying one that conflicts with the key is rejected.
/// Super-user keys may omit `organization_id` to read across tenants.
///
/// ```
/// use searchcraft::admin::types::{MeasureDashboardParams, MeasureQueryGranularity};
///
/// let params = MeasureDashboardParams {
///     organization_id: Some("org-1".into()),
///     index_names: vec!["products".into(), "articles".into()],
///     granularity: Some(MeasureQueryGranularity::Days),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MeasureDashboardParams {
    /// Organization to report on. Must match the calling key unless it is a
    /// super user.
    pub organization_id: Option<String>,
    /// Application to report on. Must be accessible to the calling key.
    pub application_id: Option<String>,
    /// Restrict the report to these indices. Sent pipe-delimited.
    pub index_names: Vec<String>,
    /// Restrict to a single user.
    pub user_id: Option<String>,
    /// Restrict to authenticated or anonymous sessions.
    pub user_type: Option<MeasureUserType>,
    /// Restrict to a single session.
    pub session_id: Option<String>,
    /// Restrict to one event name. See [`event_names`].
    pub event_name: Option<String>,
    /// Start of the reporting window, as a Unix timestamp in seconds.
    pub date_start: Option<u32>,
    /// End of the reporting window, as a Unix timestamp in seconds.
    pub date_end: Option<u32>,
    /// Time bucket size for series data.
    pub granularity: Option<MeasureQueryGranularity>,
    /// Results per page.
    pub rpp: Option<u32>,
    /// Zero-based page number.
    pub page: Option<u32>,
}

impl MeasureDashboardParams {
    /// Serializes the filters into a query string beginning with `?`, or an
    /// empty string when nothing is set.
    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut pairs: Vec<(&str, String)> = Vec::new();

        if let Some(v) = &self.organization_id {
            pairs.push(("organization_id", v.clone()));
        }
        if let Some(v) = &self.application_id {
            pairs.push(("application_id", v.clone()));
        }
        if !self.index_names.is_empty() {
            // The engine parses this as a pipe-delimited list.
            pairs.push(("index_names", self.index_names.join("|")));
        }
        if let Some(v) = &self.user_id {
            pairs.push(("user_id", v.clone()));
        }
        if let Some(v) = self.user_type {
            let encoded = match v {
                MeasureUserType::Anonymous => "anonymous",
                MeasureUserType::Authenticated => "authenticated",
            };
            pairs.push(("user_type", encoded.to_string()));
        }
        if let Some(v) = &self.session_id {
            pairs.push(("session_id", v.clone()));
        }
        if let Some(v) = &self.event_name {
            pairs.push(("event_name", v.clone()));
        }
        if let Some(v) = self.date_start {
            pairs.push(("date_start", v.to_string()));
        }
        if let Some(v) = self.date_end {
            pairs.push(("date_end", v.to_string()));
        }
        if let Some(v) = self.granularity {
            pairs.push(("granularity", v.as_str().to_string()));
        }
        if let Some(v) = self.rpp {
            pairs.push(("rpp", v.to_string()));
        }
        if let Some(v) = self.page {
            pairs.push(("page", v.to_string()));
        }

        if pairs.is_empty() {
            return String::new();
        }

        let query: Vec<String> = pairs
            .into_iter()
            .map(|(key, value)| format!("{key}={}", urlencode(&value)))
            .collect();
        format!("?{}", query.join("&"))
    }
}

/// Percent-encodes a query-string value.
fn urlencode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// Session segmentation for measure events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MeasureUserType {
    /// The session is not tied to a signed-in user.
    Anonymous,
    /// The session belongs to a signed-in user.
    Authenticated,
}

/// Properties attached to a measure event.
///
/// Only [`searchcraft_index_names`](Self::searchcraft_index_names) is required;
/// build the rest with struct-update syntax over [`Default`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MeasureRequestProperties {
    /// Organization the event belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub searchcraft_organization_id: Option<String>,
    /// Application the event belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub searchcraft_application_id: Option<String>,
    /// Indices the event relates to.
    pub searchcraft_index_names: Vec<String>,
    /// Federation the event relates to, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub searchcraft_federation_name: Option<String>,
    /// The query string the user searched for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_term: Option<String>,
    /// Query kind, e.g. `"fuzzy"`, `"exact"`, or `"dynamic"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_kind: Option<String>,
    /// LLM provider that served the event. Populated for
    /// [`event_names::API_SUMMARY_REQUESTED`] (engine 0.10.0+).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_provider: Option<String>,
    /// Number of documents returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_documents: Option<u64>,
    /// Your own identifier for a document the event concerns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_document_id: Option<String>,
    /// Rank of the document within the result list, for click events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_position: Option<u64>,
    /// Identifier tying related events into one session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl MeasureRequestProperties {
    /// Creates properties for events against the given indices, leaving every
    /// optional field unset.
    #[must_use]
    pub fn new(index_names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            searchcraft_index_names: index_names.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }
}

/// User properties attached to a measure event.
///
/// Only [`user_id`](Self::user_id) is required; build the rest with
/// struct-update syntax over [`Default`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MeasureRequestUser {
    /// Your identifier for the user or anonymous session.
    pub user_id: String,
    /// Whether the session is authenticated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_type: Option<MeasureUserType>,
    /// User's country.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// User's city.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Device identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    /// Client IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_ip: Option<String>,
    /// Locale, e.g. `"en-US"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// Operating system name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    /// Platform the client runs on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    /// Region within the country.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// Name of the SDK that emitted the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdk_name: Option<String>,
    /// Version of the SDK that emitted the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdk_version: Option<String>,
    /// Client user agent string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// Latitude of the user, in degrees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    /// Longitude of the user, in degrees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
}

impl MeasureRequestUser {
    /// Creates a user payload with the given identifier, leaving every
    /// optional field unset.
    #[must_use]
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            ..Self::default()
        }
    }
}

/// A measure event payload.
///
/// ```
/// use searchcraft::admin::types::{
///     event_names, MeasureEvent, MeasureRequestProperties, MeasureRequestUser,
/// };
///
/// let event = MeasureEvent::new(
///     event_names::DOCUMENT_CLICKED,
///     MeasureRequestProperties {
///         external_document_id: Some("doc-1".into()),
///         ..MeasureRequestProperties::new(["products"])
///     },
///     MeasureRequestUser::new("user-42"),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasureEvent {
    /// The event name. See [`event_names`] for the known values.
    pub event_name: String,
    /// What the event describes.
    pub properties: MeasureRequestProperties,
    /// Who the event is attributed to.
    pub user: MeasureRequestUser,
}

impl MeasureEvent {
    /// Creates an event from a name, its properties, and its user.
    #[must_use]
    pub fn new(
        event_name: impl Into<String>,
        properties: MeasureRequestProperties,
        user: MeasureRequestUser,
    ) -> Self {
        Self {
            event_name: event_name.into(),
            properties,
            user,
        }
    }
}

/// Synonym map: synonym key to the list of terms it expands to.
pub type SynonymsMap = HashMap<String, Vec<String>>;
