//! Stopwords management endpoints.

use reqwest::Method;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

impl SearchcraftClient {
    /// Gets all stopwords for an index.
    ///
    /// `GET /index/{index_name}/stopwords`
    pub async fn get_stopwords(&self, index_name: &str) -> error::Result<Vec<String>> {
        let path = format!("index/{index_name}/stopwords");
        self.transport
            .request(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }

    /// Adds stopwords to an index.
    ///
    /// `POST /index/{index_name}/stopwords`
    pub async fn add_stopwords(
        &self,
        index_name: &str,
        stopwords: &[String],
    ) -> error::Result<String> {
        let path = format!("index/{index_name}/stopwords");
        self.transport
            .request_text(Method::POST, &path, Operation::Write, Some(&stopwords))
            .await
    }

    /// Deletes specific stopwords from an index.
    ///
    /// `DELETE /index/{index_name}/stopwords`
    pub async fn delete_stopwords(
        &self,
        index_name: &str,
        stopwords: &[String],
    ) -> error::Result<String> {
        let path = format!("index/{index_name}/stopwords");
        self.transport
            .request_text(Method::DELETE, &path, Operation::Write, Some(&stopwords))
            .await
    }

    /// Deletes all stopwords from an index.
    ///
    /// `DELETE /index/{index_name}/stopwords/all`
    pub async fn delete_all_stopwords(&self, index_name: &str) -> error::Result<String> {
        let path = format!("index/{index_name}/stopwords/all");
        self.transport
            .request_text(Method::DELETE, &path, Operation::Write, None::<&()>)
            .await
    }
}
