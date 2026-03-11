//! Management and administration APIs for the Searchcraft client.
//!
//! This module provides endpoint methods for managing indices, documents,
//! federations, authentication keys, health checks, measure events,
//! stopwords, synonyms, and transactions.
//!
//! Methods are implemented as extensions on [`SearchcraftClient`](crate::SearchcraftClient).

pub mod types;

mod auth;
mod documents;
mod federations;
mod health;
mod indices;
mod measure;
mod stopwords;
mod synonyms;
mod transactions;

#[cfg(test)]
mod tests;
