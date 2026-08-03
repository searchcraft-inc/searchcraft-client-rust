//! Transaction management endpoints.
//!
//! Document writes are buffered until committed. An index's
//! [`auto_commit_delay`](super::types::IndexConfig::auto_commit_delay) commits
//! them on a timer; these endpoints let you commit or discard the buffer
//! immediately — useful when you need a write to be searchable right away.

use reqwest::Method;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

impl SearchcraftClient {
    /// Commits an index's buffered writes, making them searchable.
    ///
    /// `POST /index/{index_name}/commit`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the index does not exist.
    pub async fn commit_transaction(&self, index_name: &str) -> error::Result<String> {
        let path = format!("index/{index_name}/commit");
        self.transport
            .request_data(Method::POST, &path, Operation::Write, None::<&()>)
            .await
    }

    /// Discards an index's buffered, uncommitted writes.
    ///
    /// `POST /index/{index_name}/rollback`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the index does not exist.
    pub async fn rollback_transaction(&self, index_name: &str) -> error::Result<String> {
        let path = format!("index/{index_name}/rollback");
        self.transport
            .request_data(Method::POST, &path, Operation::Write, None::<&()>)
            .await
    }
}
