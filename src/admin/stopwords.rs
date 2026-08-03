//! Stopword management endpoints.
//!
//! Stopwords are terms the engine ignores at index and query time. Reading the
//! list uses the read key; changing it uses the ingest key.

use reqwest::Method;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

impl SearchcraftClient {
    /// Gets the stopword list for an index.
    ///
    /// `GET /index/{index_name}/stopwords`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or [`Error::NotFound`](crate::Error::NotFound) if the
    /// index does not exist.
    pub async fn get_stopwords(&self, index_name: &str) -> error::Result<Vec<String>> {
        let path = format!("index/{index_name}/stopwords");
        self.transport
            .request_data(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }

    /// Gets the engine's built-in stopword list for the index's language.
    ///
    /// These are the defaults the index starts from, distinct from the
    /// customized list returned by [`get_stopwords`](Self::get_stopwords).
    ///
    /// `GET /index/{index_name}/stopwords/default`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or [`Error::NotFound`](crate::Error::NotFound) if the
    /// index does not exist.
    pub async fn get_default_stopwords(&self, index_name: &str) -> error::Result<Vec<String>> {
        let path = format!("index/{index_name}/stopwords/default");
        self.transport
            .request_data(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }

    /// Adds stopwords to an index.
    ///
    /// `POST /index/{index_name}/stopwords`
    ///
    /// ```no_run
    /// # async fn example() -> searchcraft::error::Result<()> {
    /// # let client = searchcraft::SearchcraftClient::new("https://x.io", Some("k"), Some("w"))?;
    /// client.add_stopwords("products", &["the", "and"]).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the index does not exist.
    pub async fn add_stopwords(
        &self,
        index_name: &str,
        stopwords: &[impl AsRef<str>],
    ) -> error::Result<String> {
        let path = format!("index/{index_name}/stopwords");
        let body: Vec<&str> = stopwords.iter().map(AsRef::as_ref).collect();
        self.transport
            .request_data(Method::POST, &path, Operation::Write, Some(&body))
            .await
    }

    /// Removes specific stopwords from an index.
    ///
    /// `DELETE /index/{index_name}/stopwords`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the index does not exist.
    pub async fn delete_stopwords(
        &self,
        index_name: &str,
        stopwords: &[impl AsRef<str>],
    ) -> error::Result<String> {
        let path = format!("index/{index_name}/stopwords");
        let body: Vec<&str> = stopwords.iter().map(AsRef::as_ref).collect();
        self.transport
            .request_data(Method::DELETE, &path, Operation::Write, Some(&body))
            .await
    }

    /// Clears the entire stopword list for an index.
    ///
    /// `DELETE /index/{index_name}/stopwords/all`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the index does not exist.
    pub async fn delete_all_stopwords(&self, index_name: &str) -> error::Result<String> {
        let path = format!("index/{index_name}/stopwords/all");
        self.transport
            .request_data(Method::DELETE, &path, Operation::Write, None::<&()>)
            .await
    }
}
