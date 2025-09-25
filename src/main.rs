use feed_rs::parser;
use reqwest;
use std::io::Cursor;
use tokio::task;
use tracing::{debug, error, info};
use tracing_subscriber;
use tracing_subscriber::filter::EnvFilter;

async fn fetch_feed(
    url: &str,
) -> Result<feed_rs::model::Feed, Box<dyn std::error::Error + Send + Sync>> {
    info!("Fetching feed from: {}", url);

    let resp = reqwest::get(url).await?;
    info!("HTTP Status: {}", resp.status());

    let bytes = resp.bytes().await?;
    debug!(
        "First 200 bytes of response:\n{}",
        String::from_utf8_lossy(&bytes[..std::cmp::min(200, bytes.len())])
    );

    let feed = parser::parse(Cursor::new(bytes))?;
    Ok(feed)
}

async fn fetch_article(url: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!("Fetching article: {}", url);
    let resp = reqwest::get(&url).await?.text().await?;
    Ok(format!(
        "Article from {} downloaded, size: {} bytes",
        url,
        resp.len()
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Respect RUST_LOG; default to "info" if RUST_LOG isn't set
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_thread_ids(true)
        .init();

    let feed_url = "https://blog.rust-lang.org/feed.xml";

    info!("Starting feed fetch...");
    match fetch_feed(feed_url).await {
        Ok(feed) => {
            info!("Found {} entries in feed", feed.entries.len());

            let mut tasks = vec![];
            for entry in feed.entries {
                if let Some(link) = entry.links.first() {
                    let url = link.href.clone();
                    tasks.push(task::spawn(fetch_article(url)));
                }
            }

            for task in tasks {
                match task.await {
                    Ok(Ok(result)) => info!("{}", result),
                    Ok(Err(e)) => error!("Error downloading article: {}", e),
                    Err(e) => error!("Worker task failed: {}", e),
                }
            }
        }
        Err(e) => {
            error!("Error fetching feed: {}", e);
        }
    }

    info!("All done!");
    Ok(())
}
