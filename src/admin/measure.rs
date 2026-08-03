//! Measure (analytics) endpoints.
//!
//! Measure collects search and click telemetry. Analytics must be configured
//! server-side for any of this to be recorded — check
//! [`get_measure_status`](SearchcraftClient::get_measure_status) first, since
//! the write endpoints silently succeed as no-ops when analytics are disabled.

use reqwest::Method;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

use super::types::{MeasureDashboardParams, MeasureEvent, MeasureStatus};

impl SearchcraftClient {
    /// Reports whether analytics are configured on the server.
    ///
    /// When `enabled` is `false`, every other `/measure/*` endpoint is a no-op.
    ///
    /// `GET /measure/status`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured. The endpoint itself is unauthenticated, so the key is
    /// sent but not checked.
    pub async fn get_measure_status(&self) -> error::Result<MeasureStatus> {
        self.transport
            .request_data(Method::GET, "measure/status", Operation::Read, None::<&()>)
            .await
    }

    /// Returns the measure dashboard summary.
    ///
    /// The shape of this payload is not stable across engine versions, so it is
    /// returned as raw JSON.
    ///
    /// `GET /measure/dashboard/summary`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or [`Error::NotFound`](crate::Error::NotFound) if
    /// analytics are not configured on the server.
    pub async fn get_measure_dashboard_summary(
        &self,
        params: &MeasureDashboardParams,
    ) -> error::Result<serde_json::Value> {
        let path = format!("measure/dashboard/summary{}", params.to_query_string());
        self.transport
            .request_data(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }

    /// Records a single measure event.
    ///
    /// `POST /measure/event`
    ///
    /// ```no_run
    /// # async fn example() -> searchcraft::error::Result<()> {
    /// # let client = searchcraft::SearchcraftClient::new("https://x.io", Some("k"), Some("w"))?;
    /// use searchcraft::admin::types::{
    ///     event_names, MeasureEvent, MeasureRequestProperties, MeasureRequestUser,
    /// };
    ///
    /// let event = MeasureEvent::new(
    ///     event_names::DOCUMENT_CLICKED,
    ///     MeasureRequestProperties {
    ///         external_document_id: Some("doc-1".into()),
    ///         document_position: Some(3),
    ///         ..MeasureRequestProperties::new(["products"])
    ///     },
    ///     MeasureRequestUser::new("user-42"),
    /// );
    ///
    /// client.track_measure_event(&event).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::Validation`](crate::Error::Validation) if the event is
    /// rejected.
    pub async fn track_measure_event(&self, event: &MeasureEvent) -> error::Result<String> {
        self.transport
            .request_data(Method::POST, "measure/event", Operation::Write, Some(event))
            .await
    }

    /// Records a batch of measure events in one request.
    ///
    /// `POST /measure/batch`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no
    /// ingest key is configured, or
    /// [`Error::Validation`](crate::Error::Validation) if any event is
    /// rejected.
    pub async fn track_measure_batch(&self, events: &[MeasureEvent]) -> error::Result<String> {
        // The batch endpoint expects `{ "items": [...] }`, not a bare array.
        let body = BatchMeasureRequest { items: events };
        self.transport
            .request_data(Method::POST, "measure/batch", Operation::Write, Some(&body))
            .await
    }

    /// Returns the measure dashboard conversion report.
    ///
    /// The shape of this payload is not stable across engine versions, so it is
    /// returned as raw JSON.
    ///
    /// `GET /measure/dashboard/conversion`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or
    /// [`Error::Authentication`](crate::Error::Authentication) if the key lacks
    /// the analytics permission.
    pub async fn get_measure_dashboard_conversion(
        &self,
        params: &MeasureDashboardParams,
    ) -> error::Result<serde_json::Value> {
        let path = format!("measure/dashboard/conversion{}", params.to_query_string());
        self.transport
            .request_data(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }

    /// Returns the measure dashboard usage report.
    ///
    /// The shape of this payload is not stable across engine versions, so it is
    /// returned as raw JSON.
    ///
    /// `GET /measure/dashboard/usage`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read
    /// key is configured, or
    /// [`Error::Authentication`](crate::Error::Authentication) if the key lacks
    /// the analytics permission.
    pub async fn get_measure_dashboard_usage(
        &self,
        params: &MeasureDashboardParams,
    ) -> error::Result<serde_json::Value> {
        let path = format!("measure/dashboard/usage{}", params.to_query_string());
        self.transport
            .request_data(Method::GET, &path, Operation::Read, None::<&()>)
            .await
    }
}

/// Body wrapper for `POST /measure/batch`.
#[derive(serde::Serialize)]
struct BatchMeasureRequest<'a> {
    items: &'a [MeasureEvent],
}
