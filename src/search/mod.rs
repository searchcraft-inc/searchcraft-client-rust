//! Search API and query builder for the Searchcraft client.
//!
//! This module provides:
//! - [`types`] — request/response models for search operations.
//! - [`query`] — an idiomatic builder for constructing search queries.
//! - Extension methods on [`SearchcraftClient`] for
//!   executing index and federation searches.

pub mod query;
pub mod types;

mod sse;

use std::collections::VecDeque;

/// Re-exported so callers can drive [`SearchcraftClient::search_summary`]
/// without depending on a matching `futures-util` version themselves.
pub use futures_util::stream::{Stream, StreamExt};

use reqwest::Method;
use serde::de::DeserializeOwned;

use crate::client::SearchcraftClient;
use crate::config::Operation;
use crate::error;

use self::sse::SseDecoder;
use self::types::{SearchRequest, SearchResponse, SummaryError, SummaryStreamEvent};

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
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read key is configured,
    /// [`Error::NotFound`](crate::Error::NotFound) if the index does not exist, or [`Error::Http`](crate::Error::Http) if a
    /// hit does not deserialize into `T`.
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
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read key is configured,
    /// [`Error::NotFound`](crate::Error::NotFound) if the federation does not exist, or
    /// [`Error::Http`](crate::Error::Http) if a hit does not deserialize into `T`.
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

    /// Streams an AI-generated summary of an index's search results.
    ///
    /// Sends a `POST /index/{index_name}/search/summary` request and decodes
    /// the Server-Sent Events response. Added in engine 0.10.0; requires AI
    /// features to be enabled for the index (see
    /// [`get_index_capabilities`](Self::get_index_capabilities)) and a key with
    /// LLM summary permissions.
    ///
    /// The returned stream yields one
    /// [`Metadata`](types::SummaryStreamEvent::Metadata) event, zero or more
    /// [`Delta`](types::SummaryStreamEvent::Delta) events, and a final
    /// [`Done`](types::SummaryStreamEvent::Done) or
    /// [`Error`](types::SummaryStreamEvent::Error). Malformed frames and
    /// mid-stream transport failures surface as `Error` events rather than
    /// ending the stream abruptly; errors establishing the connection are
    /// returned from this method instead.
    ///
    /// Unlike other requests, the configured timeout bounds only connection
    /// setup, so a slow generation is not cut short.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> searchcraft::error::Result<()> {
    /// use searchcraft::search::StreamExt;
    /// use searchcraft::search::query::QueryBuilder;
    /// use searchcraft::search::types::SummaryStreamEvent;
    /// use searchcraft::SearchcraftClient;
    ///
    /// let client = SearchcraftClient::new(
    ///     "https://my-instance.searchcraft.io",
    ///     Some("sc-read-key"),
    ///     None::<String>,
    /// )?;
    ///
    /// let request = QueryBuilder::fuzzy().term("laptop").build_request();
    /// let mut stream = client.search_summary("products", &request).await?;
    ///
    /// while let Some(event) = stream.next().await {
    ///     match event {
    ///         SummaryStreamEvent::Delta(d) => print!("{}", d.content),
    ///         SummaryStreamEvent::Done(d) => println!("\n({} results)", d.results_count),
    ///         SummaryStreamEvent::Error(e) => eprintln!("summary failed: {}", e.message),
    ///         SummaryStreamEvent::Metadata(_) => {}
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Configuration`](crate::Error::Configuration) if no read key is configured,
    /// [`Error::Authentication`](crate::Error::Authentication) if the key lacks summary permissions, or
    /// [`Error::NotFound`](crate::Error::NotFound) if the index does not exist or the engine predates
    /// 0.10.0.
    ///
    /// Failures *after* the stream opens are delivered as
    /// [`SummaryStreamEvent::Error`] items rather than through this `Result`.
    pub async fn search_summary(
        &self,
        index_name: &str,
        request: &SearchRequest,
    ) -> error::Result<impl Stream<Item = SummaryStreamEvent>> {
        let path = format!("index/{index_name}/search/summary");
        let response = self
            .transport
            .request_stream(Method::POST, &path, Operation::Read, Some(request))
            .await?;

        Ok(summary_stream(Box::pin(response.bytes_stream())))
    }
}

/// Decoder state threaded through the summary stream.
struct SummaryState<S> {
    bytes: S,
    decoder: SseDecoder,
    pending: VecDeque<SummaryStreamEvent>,
    finished: bool,
}

/// Turns a stream of SSE body chunks into a stream of summary events.
///
/// The result is boxed and pinned so callers can drive it with
/// [`StreamExt::next`] directly, without pinning it themselves.
fn summary_stream<S, B, E>(chunks: S) -> impl Stream<Item = SummaryStreamEvent>
where
    S: Stream<Item = Result<B, E>> + Unpin,
    B: AsRef<[u8]>,
    E: std::fmt::Display,
{
    let state = SummaryState {
        bytes: chunks,
        decoder: SseDecoder::new(),
        pending: VecDeque::new(),
        finished: false,
    };

    Box::pin(futures_util::stream::unfold(
        state,
        |mut state| async move {
            loop {
                if let Some(event) = state.pending.pop_front() {
                    return Some((event, state));
                }
                if state.finished {
                    return None;
                }

                match state.bytes.next().await {
                    Some(Ok(chunk)) => {
                        for frame in state.decoder.push(chunk.as_ref()) {
                            if let Some(event) = decode_summary_frame(&frame) {
                                state.pending.push_back(event);
                            }
                        }
                    }
                    Some(Err(e)) => {
                        state.finished = true;
                        state
                            .pending
                            .push_back(SummaryStreamEvent::Error(SummaryError {
                                message: format!("stream interrupted: {e}"),
                            }));
                    }
                    None => {
                        state.finished = true;
                        if let Some(frame) = state.decoder.finish() {
                            if let Some(event) = decode_summary_frame(&frame) {
                                state.pending.push_back(event);
                            }
                        }
                    }
                }
            }
        },
    ))
}

/// Converts one decoded SSE frame into a summary event.
///
/// Frames carrying an unrecognised event name are dropped; frames with a known
/// name but an unusable payload become synthetic `Error` events so callers see
/// the problem without the stream dying.
fn decode_summary_frame(frame: &sse::SseEvent) -> Option<SummaryStreamEvent> {
    let name = frame.event.as_deref()?;
    if !matches!(name, "metadata" | "delta" | "done" | "error") {
        return None;
    }

    let data: serde_json::Value = if frame.data.is_empty() {
        serde_json::Value::Object(serde_json::Map::new())
    } else {
        match serde_json::from_str(&frame.data) {
            Ok(value) => value,
            Err(_) => {
                return Some(SummaryStreamEvent::Error(SummaryError {
                    message: format!("invalid JSON in {name} event: {}", frame.data),
                }))
            }
        }
    };

    let tagged = serde_json::json!({ "type": name, "data": data });
    match serde_json::from_value(tagged) {
        Ok(event) => Some(event),
        Err(_) => Some(SummaryStreamEvent::Error(SummaryError {
            message: format!("malformed {name} payload: {data}"),
        })),
    }
}

#[cfg(test)]
mod tests {
    use futures_util::StreamExt;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::summary_stream;
    use crate::search::query::QueryBuilder;
    use crate::search::types::SummaryStreamEvent;

    fn test_client(base_url: &str) -> crate::SearchcraftClient {
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

        let client = test_client(&server.uri());
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

        let client = test_client(&server.uri());
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

        let client = test_client(&server.uri());
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

        let client = test_client(&server.uri());
        let request = QueryBuilder::fuzzy().term("test").build_request();

        let err = client
            .search_index::<serde_json::Value>("nonexistent", &request)
            .await
            .unwrap_err();

        assert!(matches!(err, crate::error::Error::NotFound(_)));
    }

    // ── Limit handling ───────────────────────────────────────────────

    #[tokio::test]
    async fn oversized_limit_is_forwarded_for_the_server_to_clamp() {
        let server = MockServer::start().await;

        // The engine clamps `limit` to its configured maximum rather than
        // rejecting the request, and that maximum is operator-configurable, so
        // the client must not second-guess it.
        Mock::given(method("POST"))
            .and(path("/index/products/search"))
            .and(body_json(serde_json::json!({
                "query": { "fuzzy": { "ctx": "laptop" } },
                "limit": 500
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": 200,
                "data": { "hits": [], "count": 0, "time_taken": 1.0 }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server.uri());
        let request = QueryBuilder::fuzzy()
            .term("laptop")
            .limit(500)
            .build_request();

        let response = client
            .search_index::<serde_json::Value>("products", &request)
            .await
            .unwrap();
        assert_eq!(response.data.count, 0);
    }

    // ── Summary streaming ────────────────────────────────────────────

    /// Feeds fixed chunks through the summary decoder.
    async fn collect_summary(chunks: Vec<&'static str>) -> Vec<SummaryStreamEvent> {
        let stream = futures_util::stream::iter(
            chunks
                .into_iter()
                .map(|c| Ok::<_, std::io::Error>(c.as_bytes())),
        );
        summary_stream(Box::pin(stream)).collect().await
    }

    #[tokio::test]
    async fn summary_stream_decodes_a_full_generation() {
        let events = collect_summary(vec![
            "event: metadata\ndata: {\"results_count\":3,\"cached\":false}\n\n",
            "event: delta\ndata: {\"content\":\"Gaming \"}\n\n",
            "event: delta\ndata: {\"content\":\"laptops\"}\n\n",
            "event: done\ndata: {\"results_count\":3}\n\n",
        ])
        .await;

        assert_eq!(events.len(), 4);
        match &events[0] {
            SummaryStreamEvent::Metadata(m) => {
                assert_eq!(m.results_count, 3);
                assert!(!m.cached);
            }
            other => panic!("expected metadata, got {other:?}"),
        }

        let text: String = events
            .iter()
            .filter_map(|e| match e {
                SummaryStreamEvent::Delta(d) => Some(d.content.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(text, "Gaming laptops");

        assert!(matches!(events[3], SummaryStreamEvent::Done(_)));
    }

    #[tokio::test]
    async fn summary_stream_reassembles_chunk_split_frames() {
        let events = collect_summary(vec![
            "event: delta\ndata: {\"cont",
            "ent\":\"split\"}\n\nevent: done\ndata: {\"results_count\":1}\n\n",
        ])
        .await;

        assert_eq!(events.len(), 2);
        match &events[0] {
            SummaryStreamEvent::Delta(d) => assert_eq!(d.content, "split"),
            other => panic!("expected delta, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn summary_stream_reports_malformed_frames_without_ending() {
        let events = collect_summary(vec![
            "event: delta\ndata: not json\n\n",
            "event: delta\ndata: {\"content\":\"recovered\"}\n\n",
            ": keep-alive\n\n",
            "event: unknown\ndata: {}\n\n",
            "event: done\ndata: {\"results_count\":1}\n\n",
        ])
        .await;

        // Bad JSON and unknown event names must not stop later events arriving.
        assert_eq!(events.len(), 3);
        match &events[0] {
            SummaryStreamEvent::Error(e) => assert!(e.message.contains("invalid JSON")),
            other => panic!("expected error, got {other:?}"),
        }
        match &events[1] {
            SummaryStreamEvent::Delta(d) => assert_eq!(d.content, "recovered"),
            other => panic!("expected delta, got {other:?}"),
        }
        assert!(matches!(events[2], SummaryStreamEvent::Done(_)));
    }

    #[tokio::test]
    async fn summary_stream_reports_wrong_shaped_payloads() {
        let events = collect_summary(vec!["event: metadata\ndata: {\"cached\":false}\n\n"]).await;

        assert_eq!(events.len(), 1);
        match &events[0] {
            SummaryStreamEvent::Error(e) => assert!(e.message.contains("malformed metadata")),
            other => panic!("expected error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn summary_stream_surfaces_transport_failures_as_events() {
        let stream = futures_util::stream::iter(vec![
            Ok::<&[u8], std::io::Error>(b"event: delta\ndata: {\"content\":\"hi\"}\n\n"),
            Err(std::io::Error::other("connection reset")),
        ]);
        let events: Vec<_> = summary_stream(Box::pin(stream)).collect().await;

        assert_eq!(events.len(), 2);
        match &events[1] {
            SummaryStreamEvent::Error(e) => assert!(e.message.contains("connection reset")),
            other => panic!("expected error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn search_summary_sends_correct_request() {
        let server = MockServer::start().await;

        let body = "event: metadata\ndata: {\"results_count\":1,\"cached\":true}\n\n\
                    event: delta\ndata: {\"content\":\"A laptop.\"}\n\n\
                    event: done\ndata: {\"results_count\":1}\n\n";

        Mock::given(method("POST"))
            .and(path("/index/products/search/summary"))
            .and(header("Authorization", "test-read-key"))
            .and(header("Accept", "text/event-stream"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("Content-Type", "text/event-stream")
                    .set_body_string(body),
            )
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server.uri());
        let request = QueryBuilder::fuzzy().term("laptop").build_request();

        let events: Vec<_> = client
            .search_summary("products", &request)
            .await
            .unwrap()
            .collect()
            .await;

        assert_eq!(events.len(), 3);
        match &events[1] {
            SummaryStreamEvent::Delta(d) => assert_eq!(d.content, "A laptop."),
            other => panic!("expected delta, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn search_summary_surfaces_setup_errors() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/index/products/search/summary"))
            .respond_with(
                ResponseTemplate::new(403)
                    .set_body_json(serde_json::json!({"message": "summary not permitted"})),
            )
            .mount(&server)
            .await;

        let client = test_client(&server.uri());
        let request = QueryBuilder::fuzzy().term("laptop").build_request();

        let err = client
            .search_summary("products", &request)
            .await
            .err()
            .expect("expected an error");

        assert!(matches!(err, crate::error::Error::Authentication { .. }));
    }
}
