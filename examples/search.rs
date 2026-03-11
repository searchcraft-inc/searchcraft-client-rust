//! Search example — demonstrates querying a Searchcraft index.
//!
//! # Usage
//!
//! ```sh
//! export SEARCHCRAFT_URL="https://my-instance.searchcraft.io"
//! export SEARCHCRAFT_READ_KEY="sc-read-..."
//! cargo run --example search
//! ```

use searchcraft::search::query::QueryBuilder;
use searchcraft::types::SortDirection;
use searchcraft::SearchcraftClient;

#[tokio::main]
async fn main() -> Result<(), searchcraft::Error> {
    let url = std::env::var("SEARCHCRAFT_URL").expect("set SEARCHCRAFT_URL");
    let read_key = std::env::var("SEARCHCRAFT_READ_KEY").expect("set SEARCHCRAFT_READ_KEY");

    let client = SearchcraftClient::new(&url, Some(&read_key), None::<String>)?;

    // --- Simple fuzzy search ---
    println!("=== Fuzzy search for \"laptop\" ===");
    let request = QueryBuilder::fuzzy()
        .term("laptop")
        .limit(5)
        .build_request();

    let response = client
        .search_index::<serde_json::Value>("products", &request)
        .await?;

    println!(
        "Found {} results in {:.1}ms",
        response.data.count, response.data.time_taken
    );
    for hit in &response.data.hits {
        println!("  {} (score: {:.3})", hit.document_id, hit.score);
    }

    // --- Boolean composition ---
    println!("\n=== Boolean: laptop AND gaming, NOT refurbished ===");
    let request = QueryBuilder::fuzzy()
        .term("laptop")
        .and("gaming")
        .not("refurbished")
        .limit(5)
        .build_request();

    let response = client
        .search_index::<serde_json::Value>("products", &request)
        .await?;

    println!("Found {} results", response.data.count);

    // --- Field query with sorting ---
    println!("\n=== Field query: category=electronics, sorted by price ===");
    let request = QueryBuilder::exact()
        .field("category", "electronics")
        .order_by("price", SortDirection::Asc)
        .limit(10)
        .build_request();

    let response = client
        .search_index::<serde_json::Value>("products", &request)
        .await?;

    println!("Found {} results", response.data.count);
    for hit in &response.data.hits {
        println!("  {}: {:?}", hit.document_id, hit.doc);
    }

    Ok(())
}
