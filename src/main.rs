use feed_rs::parser;
use reqwest;
use tokio::task;
use std::io::Cursor;

async fn fetch_feed(url: &str) -> Result<feed_rs::model::Feed, Box<dyn std::error::Error + Send + Sync>> {
    println!("Fetching feed from: {}", url);

    let resp = reqwest::get(url).await?;
    println!("HTTP Status: {}", resp.status());

    let bytes = resp.bytes().await?;
    println!(
        "First 200 bytes of response:\n{}",
        String::from_utf8_lossy(&bytes[..std::cmp::min(200, bytes.len())])
    );

    let feed = parser::parse(Cursor::new(bytes))?;
    Ok(feed)
}

async fn fetch_article(url: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    println!("Fetching article: {}", url);
    let resp = reqwest::get(&url).await?.text().await?;
    Ok(format!("Article from {} downloaded, size: {} bytes", url, resp.len()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let feed_url = "https://blog.rust-lang.org/feed.xml";

    println!("Fetching feed...");
    match fetch_feed(feed_url).await {
        Ok(feed) => {
            println!("Found {} entries in feed", feed.entries.len());

            let mut tasks = vec![];
            for entry in feed.entries {
                if let Some(link) = entry.links.first() {
                    let url = link.href.clone();
                    tasks.push(task::spawn(fetch_article(url)));
                }
            }

            for task in tasks {
                match task.await {
                    Ok(Ok(result)) => println!("{}", result),
                    Ok(Err(e)) => eprintln!("Error downloading article: {}", e),
                    Err(e) => eprintln!("Worker task failed: {}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching feed: {}", e);
        }
    }

    println!("All done!");
    Ok(())
}
