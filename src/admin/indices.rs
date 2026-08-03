//! Index management endpoints.
//!
//! Reads use the read key; create, update, and delete use the ingest key.

use reqwest::Method;
use serde::Serialize;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

use super::types::{
    AllIndexStatsResponse, IndexCapabilities, IndexConfig, IndexListResponse, IndexStats,
};

/// An index configuration with its name folded in, as the engine expects.
#[derive(Serialize)]
struct NamedIndexConfig<'a> {
    name: &'a str,
    #[serde(flatten)]
    config: &'a IndexConfig,
}

/// Body for index create and replace:
/// `{ "index": { "name": ..., ...config }, "override_if_exists": bool }`.
#[derive(Serialize)]
struct IndexCreationBody<'a> {
    index: NamedIndexConfig<'a>,
    override_if_exists: bool,
}

impl SearchcraftClient {
    /// Lists all index names.
    ///
    /// `GET /index`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or [`Error::Authentication`](crate::Error::Authentication)
    /// if the key is rejected.
    pub async fn list_indices(&self) -> error::Result<IndexListResponse> {
        self.transport
            .request_data(Method::GET, "index", Operation::Read, None::<&()>)
            .await
    }

    /// Gets the configuration for a specific index.
    ///
    /// `GET /index/{index_name}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or [`Error::NotFound`](crate::Error::NotFound) if the
    /// index does not exist.
    pub async fn get_index(&self, index_name: &str) -> error::Result<IndexConfig> {
        let path = format!("index/{index_name}");
        self.transport
            .request_data(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }

    /// Creates a new index with the given configuration.
    ///
    /// Sends `POST /index`. To overwrite an index that already exists, use
    /// [`create_index_overwriting`](Self::create_index_overwriting) or
    /// [`replace_index`](Self::replace_index).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> searchcraft::error::Result<()> {
    /// # let client = searchcraft::SearchcraftClient::new("https://x.io", Some("k"), Some("w"))?;
    /// use std::collections::HashMap;
    /// use searchcraft::admin::types::{FieldConfig, FieldType, IndexConfig};
    ///
    /// let config = IndexConfig {
    ///     language: Some("en".into()),
    ///     search_fields: Some(vec!["title".into()]),
    ///     fields: Some(HashMap::from([(
    ///         "title".to_string(),
    ///         FieldConfig {
    ///             stored: Some(true),
    ///             required: Some(true),
    ///             ..FieldConfig::new(FieldType::Text)
    ///         },
    ///     )])),
    ///     ..Default::default()
    /// };
    ///
    /// client.create_index("products", &config).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::Validation`](crate::Error::Validation) if the configuration is
    /// rejected — including when an index of that name already exists.
    pub async fn create_index(
        &self,
        index_name: &str,
        config: &IndexConfig,
    ) -> error::Result<String> {
        self.create_index_inner(index_name, config, false).await
    }

    /// Creates an index, replacing any existing index of the same name.
    ///
    /// Sends `POST /index` with `override_if_exists` set. The existing index
    /// and its documents are discarded.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::Validation`](crate::Error::Validation) if the configuration is
    /// rejected.
    pub async fn create_index_overwriting(
        &self,
        index_name: &str,
        config: &IndexConfig,
    ) -> error::Result<String> {
        self.create_index_inner(index_name, config, true).await
    }

    async fn create_index_inner(
        &self,
        index_name: &str,
        config: &IndexConfig,
        override_if_exists: bool,
    ) -> error::Result<String> {
        let body = IndexCreationBody {
            index: NamedIndexConfig {
                name: index_name,
                config,
            },
            override_if_exists,
        };
        self.transport
            .request_data(Method::POST, "index", Operation::Write, Some(&body))
            .await
    }

    /// Replaces the full schema of an index that already exists.
    ///
    /// Sends `PUT /index/{index_name}`. Unlike
    /// [`update_index`](Self::update_index) this is a whole-schema replacement,
    /// so anything absent from `config` reverts to its default. `ai_enabled` is
    /// preserved unless `config` sets it explicitly.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, [`Error::NotFound`](crate::Error::NotFound) if
    /// the index does not exist — use [`create_index`](Self::create_index) for
    /// a new one — or [`Error::Validation`](crate::Error::Validation) if the
    /// configuration is rejected.
    pub async fn replace_index(
        &self,
        index_name: &str,
        config: &IndexConfig,
    ) -> error::Result<String> {
        let path = format!("index/{index_name}");
        let body = IndexCreationBody {
            index: NamedIndexConfig {
                name: index_name,
                config,
            },
            override_if_exists: true,
        };
        self.transport
            .request_data(Method::PUT, &path, Operation::Write, Some(&body))
            .await
    }

    /// Applies partial configuration changes to an existing index.
    ///
    /// Fields left as `None` on `config` are omitted from the request and stay
    /// unchanged. Two limits come from the engine's patch semantics:
    ///
    /// - [`fields`](IndexConfig::fields) is **not patchable**. The engine
    ///   carries the existing schema fields over verbatim, so sending them here
    ///   is accepted and silently ignored. Use
    ///   [`replace_index`](Self::replace_index) to change the schema.
    /// - [`search_fields`](IndexConfig::search_fields),
    ///   [`weight_multipliers`](IndexConfig::weight_multipliers) and
    ///   [`language`](IndexConfig::language) cannot be *cleared*. The engine
    ///   reads an empty value as "leave unchanged", so an empty list, map or
    ///   string is a no-op rather than a reset.
    ///
    /// [`time_decay_field`](IndexConfig::time_decay_field) can be cleared by
    /// sending an empty string.
    ///
    /// `PATCH /index/{index_name}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, [`Error::NotFound`](crate::Error::NotFound) if
    /// the index does not exist, or
    /// [`Error::Validation`](crate::Error::Validation) if the change is
    /// rejected.
    pub async fn update_index(
        &self,
        index_name: &str,
        config: &IndexConfig,
    ) -> error::Result<String> {
        let path = format!("index/{index_name}");
        // The patch endpoint takes the config at the top level, unlike create
        // and replace which nest it under `index`.
        self.transport
            .request_data(Method::PATCH, &path, Operation::Write, Some(config))
            .await
    }

    /// Deletes an index and every document in it.
    ///
    /// `DELETE /index/{index_name}`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the index does not exist.
    pub async fn delete_index(&self, index_name: &str) -> error::Result<String> {
        let path = format!("index/{index_name}");
        self.transport
            .request_data(Method::DELETE, &path, Operation::Write, None::<&()>)
            .await
    }

    /// Gets document counts for every index on the cluster.
    ///
    /// `GET /index/stats`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or [`Error::Authentication`](crate::Error::Authentication)
    /// if the key is rejected.
    pub async fn get_all_index_stats(&self) -> error::Result<AllIndexStatsResponse> {
        self.transport
            .request_data(Method::GET, "index/stats", Operation::Read, None::<&()>)
            .await
    }

    /// Gets the document count for a specific index.
    ///
    /// `GET /index/{index_name}/stats`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or [`Error::NotFound`](crate::Error::NotFound) if the
    /// index does not exist.
    pub async fn get_index_stats(&self, index_name: &str) -> error::Result<IndexStats> {
        let path = format!("index/{index_name}/stats");
        self.transport
            .request_data(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }

    /// Reports which AI capabilities are configured for an index.
    ///
    /// Added in engine 0.10.0. Call this before
    /// [`search_summary`](Self::search_summary) to check whether summary
    /// generation is available; on older engines the endpoint is absent and
    /// this returns [`Error::NotFound`](crate::Error::NotFound).
    ///
    /// `GET /index/{index_name}/capabilities`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or [`Error::NotFound`](crate::Error::NotFound) if the
    /// index does not exist or the engine predates 0.10.0.
    pub async fn get_index_capabilities(
        &self,
        index_name: &str,
    ) -> error::Result<IndexCapabilities> {
        let path = format!("index/{index_name}/capabilities");
        self.transport
            .request_data(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }
}
