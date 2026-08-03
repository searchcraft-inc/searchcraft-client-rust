//! Synonym management endpoints.
//!
//! Reading the synonym map uses the read key; changing it uses the ingest key.

use reqwest::Method;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

use super::types::SynonymsMap;

impl SearchcraftClient {
    /// Gets the synonym map for an index.
    ///
    /// `GET /index/{index_name}/synonyms`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or [`Error::NotFound`](crate::Error::NotFound) if the
    /// index does not exist.
    pub async fn get_synonyms(&self, index_name: &str) -> error::Result<SynonymsMap> {
        let path = format!("index/{index_name}/synonyms");
        self.transport
            .request_data(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }

    /// Adds synonyms to an index.
    ///
    /// Each entry uses the format `"synonym:original-term"`.
    ///
    /// `POST /index/{index_name}/synonyms`
    ///
    /// ```no_run
    /// # async fn example() -> searchcraft::error::Result<()> {
    /// # let client = searchcraft::SearchcraftClient::new("https://x.io", Some("k"), Some("w"))?;
    /// client.add_synonyms("products", &["notebook:laptop"]).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, [`Error::NotFound`](crate::Error::NotFound) if
    /// the index does not exist, or
    /// [`Error::Validation`](crate::Error::Validation) if an entry is malformed.
    pub async fn add_synonyms(
        &self,
        index_name: &str,
        synonyms: &[impl AsRef<str>],
    ) -> error::Result<String> {
        let path = format!("index/{index_name}/synonyms");
        let body: Vec<&str> = synonyms.iter().map(AsRef::as_ref).collect();
        self.transport
            .request_data(Method::POST, &path, Operation::Write, Some(&body))
            .await
    }

    /// Removes specific synonyms from an index.
    ///
    /// `DELETE /index/{index_name}/synonyms`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the index does not exist.
    pub async fn delete_synonyms(
        &self,
        index_name: &str,
        synonyms: &[impl AsRef<str>],
    ) -> error::Result<String> {
        let path = format!("index/{index_name}/synonyms");
        let body: Vec<&str> = synonyms.iter().map(AsRef::as_ref).collect();
        self.transport
            .request_data(Method::DELETE, &path, Operation::Write, Some(&body))
            .await
    }

    /// Clears the entire synonym map for an index.
    ///
    /// `DELETE /index/{index_name}/synonyms/all`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the index does not exist.
    pub async fn delete_all_synonyms(&self, index_name: &str) -> error::Result<String> {
        let path = format!("index/{index_name}/synonyms/all");
        self.transport
            .request_data(Method::DELETE, &path, Operation::Write, None::<&()>)
            .await
    }
}
