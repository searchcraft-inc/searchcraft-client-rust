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
use searchcraft::search::types::SummaryStreamEvent;
use searchcraft::search::StreamExt;
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
        let score = hit
            .score
            .map_or_else(|| "n/a".to_string(), |s| format!("{s:.3}"));
        println!("  {} (score: {score})", hit.document_id);
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

    // --- Streaming AI summary (engine 0.10.0+) ---
    // Only available when AI features are configured for the index.
    let capabilities = client.get_index_capabilities("products").await?;
    if capabilities.ai.enabled && capabilities.ai.search_summary_configured {
        println!("\n=== AI summary for \"laptop\" ===");
        let request = QueryBuilder::fuzzy()
            .term("laptop")
            .limit(5)
            .build_request();
        let mut stream = client.search_summary("products", &request).await?;

        while let Some(event) = stream.next().await {
            match event {
                SummaryStreamEvent::Metadata(m) => {
                    println!(
                        "(summarizing {} results, cached: {})",
                        m.results_count, m.cached
                    );
                }
                SummaryStreamEvent::Delta(d) => print!("{}", d.content),
                SummaryStreamEvent::Done(_) => println!(),
                SummaryStreamEvent::Error(e) => eprintln!("\nsummary failed: {}", e.message),
            }
        }
    } else {
        println!("\n=== AI summary skipped: not configured for this index ===");
    }

    Ok(())
}
