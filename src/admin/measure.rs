//! Measure / analytics endpoints.

use reqwest::Method;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

use super::types::MeasureEvent;

impl SearchcraftClient {
    /// Returns measurement dashboard summary data.
    ///
    /// `GET /measure/dashboard/summary`
    pub async fn get_measure_dashboard_summary(&self) -> error::Result<serde_json::Value> {
        self.transport
            .request(
                Method::GET,
                "measure/dashboard/summary",
                Operation::Read,
                None::<&()>,
            )
            .await
    }

    /// Tracks a single measure event.
    ///
    /// `POST /measure/event`
    pub async fn track_measure_event(&self, event: &MeasureEvent) -> error::Result<String> {
        self.transport
            .request_text(Method::POST, "measure/event", Operation::Write, Some(event))
            .await
    }

    /// Tracks a batch of measure events.
    ///
    /// `POST /measure/batch`
    pub async fn track_measure_batch(&self, events: &[MeasureEvent]) -> error::Result<String> {
        self.transport
            .request_text(
                Method::POST,
                "measure/batch",
                Operation::Write,
                Some(&events),
            )
            .await
    }
}
