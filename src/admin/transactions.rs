//! Transaction management endpoints.

use reqwest::Method;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

impl SearchcraftClient {
    /// Commits a write transaction for the given index.
    ///
    /// `POST /index/{index_name}/commit`
    pub async fn commit_transaction(&self, index_name: &str) -> error::Result<String> {
        let path = format!("index/{index_name}/commit");
        self.transport
            .request_text(Method::POST, &path, Operation::Write, None::<&()>)
            .await
    }

    /// Rolls back a write transaction for the given index.
    ///
    /// `POST /index/{index_name}/rollback`
    pub async fn rollback_transaction(&self, index_name: &str) -> error::Result<String> {
        let path = format!("index/{index_name}/rollback");
        self.transport
            .request_text(Method::POST, &path, Operation::Write, None::<&()>)
            .await
    }
}
