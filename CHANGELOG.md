# Changelog

## Unreleased

### Added

- `AuthKeyPermission::AI` (`LLM_RAG_SUMMARIES`, mask `64`) and
  `AuthKeyPermission::ANALYTICS` (`READ_ANALYTICS`, mask `256`). These are
  add-ons, combined with a level using `|`, e.g.
  `AuthKeyPermission::READ | AuthKeyPermission::AI` for a public key that can
  request AI summaries.
- `AuthKeyPermission` can be combined with raw `permissions` bits
  (`AuthKeyPermission::READ | permissions::READ_ANALYTICS`), and supports `|=`
  with either form.

### Removed

- **Breaking:** the `fieldnorms` and `precision` fields are removed from
  `FieldConfig`. Both are engine internals that are not part of the documented
  Searchcraft API, so the client no longer exposes them. Omitting them leaves
  the engine's defaults in place, and existing indexes are unaffected. Configs
  that still contain these keys continue to deserialize.

## 0.1.0

Initial release. An async Rust client for the Searchcraft API, targeting
Searchcraft engine **0.11.0**.

The client covers the full API surface — search and federated search, index and
document management, federations, authentication keys, synonyms and stopwords,
transactions, and analytics — with typed requests and responses throughout.

- **Search.** A composable, immutable query builder covering every query kind
  the engine supports, with boolean composition, field-scoped matching, range
  and comparison filters, facets, sorting, and pagination.

- **Streaming AI summaries.** LLM-generated summaries of a result set arrive as
  they are produced, decoded from Server-Sent Events into a typed stream.
  Malformed frames and dropped connections surface as error events rather than
  ending the stream, and the request timeout bounds only connection setup so a
  slow generation is never cut short.

- **Management.** Index, document, federation, auth key, synonym, stopword, and
  transaction operations, each sending the appropriate one of the read, ingest,
  and admin keys you configure.

- **Analytics.** Event and batch tracking, plus the dashboard reports with the
  full set of filters the engine accepts.

- **Errors you can act on.** A typed error enum separating configuration
  mistakes from authentication, not-found, validation, API, and network
  failures, with helpers for the HTTP status and whether a retry is worth
  attempting.

- **Built to age well.** Values the engine adds later — new LLM providers,
  new permission bits — round-trip instead of failing to deserialize, so a
  newer engine release does not break an older client.

- **Configurable transport.** Built on [`reqwest`](https://docs.rs/reqwest)
  with `rustls` by default and an optional `native-tls` backend, plus
  configurable timeouts and custom headers. API keys are held in
  [`secrecy`](https://docs.rs/secrecy) wrappers so they are not logged by
  accident.
