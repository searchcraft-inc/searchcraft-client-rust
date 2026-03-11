//! Synonyms management endpoints.

use reqwest::Method;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

use super::types::SynonymsMap;

impl SearchcraftClient {
    /// Gets all synonyms for an index.
    ///
    /// `GET /index/{index_name}/synonyms`
    pub async fn get_synonyms(&self, index_name: &str) -> error::Result<SynonymsMap> {
        let path = format!("index/{index_name}/synonyms");
        self.transport
            .request(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }

    /// Adds synonyms to an index.
    ///
    /// Synonyms use the format `"synonym:original-term"`.
    ///
    /// `POST /index/{index_name}/synonyms`
    pub async fn add_synonyms(
        &self,
        index_name: &str,
        synonyms: &[String],
    ) -> error::Result<String> {
        let path = format!("index/{index_name}/synonyms");
        self.transport
            .request_text(Method::POST, &path, Operation::Write, Some(&synonyms))
            .await
    }

    /// Deletes specific synonyms from an index.
    ///
    /// `DELETE /index/{index_name}/synonyms`
    pub async fn delete_synonyms(
        &self,
        index_name: &str,
        synonyms: &[String],
    ) -> error::Result<String> {
        let path = format!("index/{index_name}/synonyms");
        self.transport
            .request_text(Method::DELETE, &path, Operation::Write, Some(&synonyms))
            .await
    }

    /// Deletes all synonyms from an index.
    ///
    /// `DELETE /index/{index_name}/synonyms/all`
    pub async fn delete_all_synonyms(&self, index_name: &str) -> error::Result<String> {
        let path = format!("index/{index_name}/synonyms/all");
        self.transport
            .request_text(Method::DELETE, &path, Operation::Write, None::<&()>)
            .await
    }
}
