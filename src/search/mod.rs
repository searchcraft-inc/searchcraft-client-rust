//! Search API and query builder for the Searchcraft client.
//!
//! This module provides:
//! - [`types`] — request/response models for search operations.
//! - [`query`] — an idiomatic builder for constructing search queries.
//! - Extension methods on [`SearchcraftClient`](crate::SearchcraftClient) for
//!   executing index and federation searches.

pub mod query;
pub mod types;

use reqwest::Method;
use serde::de::DeserializeOwned;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

use self::types::{SearchRequest, SearchResponse};

impl SearchcraftClient {
    /// Search a specific index.
    ///
    /// Sends a `POST /index/{index_name}/search` request.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> searchcraft::error::Result<()> {
    /// use searchcraft::SearchcraftClient;
    /// use searchcraft::search::query::QueryBuilder;
    ///
    /// let client = SearchcraftClient::new(
    ///     "https://my-instance.searchcraft.io",
    ///     Some("sc-read-key"),
    ///     None::<String>,
    /// )?;
    ///
    /// let request = QueryBuilder::fuzzy()
    ///     .term("laptop")
    ///     .limit(10)
    ///     .build_request();
    ///
    /// let response = client
    ///     .search_index::<serde_json::Value>("products", &request)
    ///     .await?;
    ///
    /// println!("Found {} results", response.data.count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_index<T: DeserializeOwned>(
        &self,
        index_name: &str,
        request: &SearchRequest,
    ) -> error::Result<SearchResponse<T>> {
        let path = format!("index/{index_name}/search");
        self.transport
            .request::<SearchResponse<T>>(Method::POST, &path, Operation::Read, Some(request))
            .await
    }

    /// Search across a federation of indices.
    ///
    /// Sends a `POST /federation/{federation_name}/search` request.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> searchcraft::error::Result<()> {
    /// use searchcraft::SearchcraftClient;
    /// use searchcraft::search::query::QueryBuilder;
    ///
    /// let client = SearchcraftClient::new(
    ///     "https://my-instance.searchcraft.io",
    ///     Some("sc-read-key"),
    ///     None::<String>,
    /// )?;
    ///
    /// let request = QueryBuilder::fuzzy()
    ///     .term("laptop")
    ///     .limit(10)
    ///     .build_request();
    ///
    /// let response = client
    ///     .search_federation::<serde_json::Value>("global", &request)
    ///     .await?;
    ///
    /// println!("Found {} results", response.data.count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_federation<T: DeserializeOwned>(
        &self,
        federation_name: &str,
        request: &SearchRequest,
    ) -> error::Result<SearchResponse<T>> {
        let path = format!("federation/{federation_name}/search");
        self.transport
            .request::<SearchResponse<T>>(Method::POST, &path, Operation::Read, Some(request))
            .await
    }
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::search::query::QueryBuilder;

    async fn test_client(base_url: &str) -> crate::SearchcraftClient {
        crate::SearchcraftClient::new(base_url, Some("test-read-key"), None::<String>).unwrap()
    }

    #[tokio::test]
    async fn search_index_sends_correct_request() {
        let server = MockServer::start().await;

        let response_body = serde_json::json!({
            "status": 200,
            "data": {
                "hits": [{
                    "doc": {"title": "Laptop"},
                    "document_id": "doc-1",
                    "score": 0.9,
                    "source_index": "products"
                }],
                "count": 1,
                "time_taken": 5.0
            }
        });

        Mock::given(method("POST"))
            .and(path("/index/products/search"))
            .and(header("Authorization", "test-read-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server.uri()).await;
        let request = QueryBuilder::fuzzy()
            .term("laptop")
            .limit(10)
            .build_request();

        let response = client
            .search_index::<serde_json::Value>("products", &request)
            .await
            .unwrap();

        assert_eq!(response.status, 200);
        assert_eq!(response.data.count, 1);
        assert_eq!(response.data.hits.len(), 1);
        assert_eq!(response.data.hits[0].document_id, "doc-1");
    }

    #[tokio::test]
    async fn search_federation_sends_correct_request() {
        let server = MockServer::start().await;

        let response_body = serde_json::json!({
            "status": 200,
            "data": {
                "hits": [],
                "count": 0,
                "time_taken": 2.0
            }
        });

        Mock::given(method("POST"))
            .and(path("/federation/global/search"))
            .and(header("Authorization", "test-read-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server.uri()).await;
        let request = QueryBuilder::exact().term("test").build_request();

        let response = client
            .search_federation::<serde_json::Value>("global", &request)
            .await
            .unwrap();

        assert_eq!(response.data.count, 0);
        assert!(response.data.hits.is_empty());
    }

    #[tokio::test]
    async fn search_index_handles_auth_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/index/products/search"))
            .respond_with(
                ResponseTemplate::new(401)
                    .set_body_json(serde_json::json!({"message": "unauthorized"})),
            )
            .mount(&server)
            .await;

        let client = test_client(&server.uri()).await;
        let request = QueryBuilder::fuzzy().term("test").build_request();

        let err = client
            .search_index::<serde_json::Value>("products", &request)
            .await
            .unwrap_err();

        assert!(matches!(err, crate::error::Error::Authentication { .. }));
    }

    #[tokio::test]
    async fn search_index_handles_not_found() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/index/nonexistent/search"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"message": "index not found"})),
            )
            .mount(&server)
            .await;

        let client = test_client(&server.uri()).await;
        let request = QueryBuilder::fuzzy().term("test").build_request();

        let err = client
            .search_index::<serde_json::Value>("nonexistent", &request)
            .await
            .unwrap_err();

        assert!(matches!(err, crate::error::Error::NotFound(_)));
    }
}
