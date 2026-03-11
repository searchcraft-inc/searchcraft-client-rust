//! Index management endpoints.

use reqwest::Method;
use serde::Serialize;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

use super::types::{
    AllIndexStatsResponse, ApiResponse, IndexConfig, IndexListResponse, IndexStats,
};

/// Body wrapper for index create: `{ "index": { "name": ..., ...config } }`.
#[derive(Serialize)]
struct CreateIndexBody {
    index: serde_json::Value,
}

/// Body wrapper for index update: `{ "index": partialConfig }`.
#[derive(Serialize)]
struct UpdateIndexBody<'a> {
    index: &'a IndexConfig,
}

impl SearchcraftClient {
    /// Lists all index names.
    ///
    /// `GET /index`
    pub async fn list_indices(&self) -> error::Result<IndexListResponse> {
        let resp: ApiResponse<IndexListResponse> = self
            .transport
            .request(Method::GET, "index", Operation::Read, None::<&()>)
            .await?;
        Ok(resp.data)
    }

    /// Gets the configuration for a specific index.
    ///
    /// `GET /index/{index_name}`
    pub async fn get_index(&self, index_name: &str) -> error::Result<IndexConfig> {
        let path = format!("index/{index_name}");
        let resp: ApiResponse<IndexConfig> = self
            .transport
            .request(Method::GET, &path, Operation::Read, None::<&()>)
            .await?;
        Ok(resp.data)
    }

    /// Creates a new index with the given configuration.
    ///
    /// `PUT /index/{index_name}`
    pub async fn create_index(
        &self,
        index_name: &str,
        config: &IndexConfig,
    ) -> error::Result<String> {
        let path = format!("index/{index_name}");
        let mut value = serde_json::to_value(config).map_err(|e| {
            crate::error::Error::Configuration(format!("failed to serialize index config: {e}"))
        })?;
        value
            .as_object_mut()
            .unwrap()
            .insert("name".into(), serde_json::Value::String(index_name.into()));
        let body = CreateIndexBody { index: value };
        let resp: ApiResponse<String> = self
            .transport
            .request(Method::PUT, &path, Operation::Write, Some(&body))
            .await?;
        Ok(resp.data)
    }

    /// Updates an existing index with partial configuration changes.
    ///
    /// `PATCH /index/{index_name}`
    pub async fn update_index(
        &self,
        index_name: &str,
        config: &IndexConfig,
    ) -> error::Result<String> {
        let path = format!("index/{index_name}");
        let body = UpdateIndexBody { index: config };
        let resp: ApiResponse<String> = self
            .transport
            .request(Method::PATCH, &path, Operation::Write, Some(&body))
            .await?;
        Ok(resp.data)
    }

    /// Deletes an index.
    ///
    /// `DELETE /index/{index_name}`
    pub async fn delete_index(&self, index_name: &str) -> error::Result<String> {
        let path = format!("index/{index_name}");
        let resp: ApiResponse<String> = self
            .transport
            .request(Method::DELETE, &path, Operation::Write, None::<&()>)
            .await?;
        Ok(resp.data)
    }

    /// Gets statistics for all indices.
    ///
    /// `GET /index/stats`
    pub async fn get_all_index_stats(&self) -> error::Result<AllIndexStatsResponse> {
        let resp: ApiResponse<AllIndexStatsResponse> = self
            .transport
            .request(Method::GET, "index/stats", Operation::Read, None::<&()>)
            .await?;
        Ok(resp.data)
    }

    /// Gets statistics for a specific index.
    ///
    /// `GET /index/{index_name}/stats`
    pub async fn get_index_stats(&self, index_name: &str) -> error::Result<IndexStats> {
        let path = format!("index/{index_name}/stats");
        let resp: ApiResponse<IndexStats> = self
            .transport
            .request(Method::GET, &path, Operation::Read, None::<&()>)
            .await?;
        Ok(resp.data)
    }
}
