//! Minimal Server-Sent Events decoder.
//!
//! Supports the `event:`, `data:`, and `id:` fields (with multi-line `data:`
//! coalescing) and dispatches whenever a blank line is seen, matching the SSE
//! spec. Event boundaries are either `\n\n` or `\r\n\r\n`.

/// A single decoded SSE message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SseEvent {
    /// Value of the last `event:` field for this message, if any.
    pub event: Option<String>,
    /// Concatenated `data:` lines, joined by `\n`.
    pub data: String,
    /// Value of the `id:` field, if sent.
    pub id: Option<String>,
}

/// Incremental decoder that turns a byte stream into [`SseEvent`]s.
///
/// Buffers raw bytes rather than decoded text so that multi-byte UTF-8
/// sequences split across chunk boundaries survive intact.
#[derive(Debug, Default)]
pub(crate) struct SseDecoder {
    buffer: Vec<u8>,
}

impl SseDecoder {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Feeds a chunk of bytes, returning every event completed by it.
    pub(crate) fn push(&mut self, chunk: &[u8]) -> Vec<SseEvent> {
        self.buffer.extend_from_slice(chunk);

        let mut events = Vec::new();
        while let Some((index, length)) = find_boundary(&self.buffer) {
            let block = self.buffer[..index].to_vec();
            self.buffer.drain(..index + length);
            if let Some(event) = parse_event_block(&String::from_utf8_lossy(&block)) {
                events.push(event);
            }
        }
        events
    }

    /// Flushes any trailing block left when the stream ends without a final
    /// boundary.
    pub(crate) fn finish(&mut self) -> Option<SseEvent> {
        if self.buffer.is_empty() {
            return None;
        }
        let block = std::mem::take(&mut self.buffer);
        parse_event_block(&String::from_utf8_lossy(&block))
    }
}

/// Finds the first event boundary, returning its offset and length.
fn find_boundary(buffer: &[u8]) -> Option<(usize, usize)> {
    for i in 0..buffer.len() {
        let rest = &buffer[i..];
        if rest.starts_with(b"\r\n\r\n") {
            return Some((i, 4));
        }
        if rest.starts_with(b"\n\n") {
            return Some((i, 2));
        }
    }
    None
}

/// Parses one boundary-delimited block into an event.
///
/// Returns `None` for blocks that carry no fields at all (e.g. comment-only
/// keep-alive frames).
fn parse_event_block(block: &str) -> Option<SseEvent> {
    let mut event_name = None;
    let mut id = None;
    let mut data_parts: Vec<&str> = Vec::new();

    for line in block.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        // Blank lines separate nothing here; lines starting with ':' are comments.
        if line.is_empty() || line.starts_with(':') {
            continue;
        }

        let (field, value) = match line.find(':') {
            Some(idx) => (
                &line[..idx],
                line[idx + 1..]
                    .strip_prefix(' ')
                    .unwrap_or(&line[idx + 1..]),
            ),
            None => (line, ""),
        };

        match field {
            "event" => event_name = Some(value.to_string()),
            "data" => data_parts.push(value),
            "id" => id = Some(value.to_string()),
            _ => {}
        }
    }

    if data_parts.is_empty() && event_name.is_none() && id.is_none() {
        return None;
    }

    Some(SseEvent {
        event: event_name,
        data: data_parts.join("\n"),
        id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_a_single_event() {
        let mut decoder = SseDecoder::new();
        let events = decoder.push(b"event: delta\ndata: {\"content\":\"hi\"}\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.as_deref(), Some("delta"));
        assert_eq!(events[0].data, r#"{"content":"hi"}"#);
    }

    #[test]
    fn decodes_crlf_boundaries() {
        let mut decoder = SseDecoder::new();
        let events = decoder.push(b"event: done\r\ndata: {}\r\n\r\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.as_deref(), Some("done"));
        assert_eq!(events[0].data, "{}");
    }

    #[test]
    fn coalesces_multiline_data() {
        let mut decoder = SseDecoder::new();
        let events = decoder.push(b"data: line one\ndata: line two\n\n");
        assert_eq!(events[0].data, "line one\nline two");
    }

    #[test]
    fn splits_multiple_events_in_one_chunk() {
        let mut decoder = SseDecoder::new();
        let events = decoder.push(b"event: a\ndata: 1\n\nevent: b\ndata: 2\n\n");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event.as_deref(), Some("a"));
        assert_eq!(events[1].event.as_deref(), Some("b"));
    }

    #[test]
    fn buffers_events_split_across_chunks() {
        let mut decoder = SseDecoder::new();
        assert!(decoder.push(b"event: delta\ndata: par").is_empty());
        let events = decoder.push(b"tial\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "partial");
    }

    #[test]
    fn survives_utf8_split_across_chunks() {
        let mut decoder = SseDecoder::new();
        // "é" is 0xC3 0xA9 — split it across two pushes.
        assert!(decoder.push(b"data: caf\xc3").is_empty());
        let events = decoder.push(b"\xa9\n\n");
        assert_eq!(events[0].data, "café");
    }

    #[test]
    fn skips_comment_only_frames() {
        let mut decoder = SseDecoder::new();
        let events = decoder.push(b": keep-alive\n\n");
        assert!(events.is_empty());
    }

    #[test]
    fn finish_flushes_trailing_block() {
        let mut decoder = SseDecoder::new();
        assert!(decoder.push(b"event: done\ndata: {}").is_empty());
        let event = decoder.finish().unwrap();
        assert_eq!(event.event.as_deref(), Some("done"));
        assert_eq!(event.data, "{}");
        assert!(decoder.finish().is_none());
    }

    #[test]
    fn field_without_value_is_empty() {
        let mut decoder = SseDecoder::new();
        let events = decoder.push(b"data\n\n");
        assert_eq!(events[0].data, "");
    }
}
