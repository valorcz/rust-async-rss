use reqwest;
use rss::Channel;
use tokio::task;

async fn fetch_rss(url: &str) -> Result<Channel, Box<dyn std::error::Error>> {
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

async fn fetch_article(url: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let resp = reqwest::get(&url).await?.text().await?;
    Ok(format!("Article from {} downloaded, size: {} bytes", url, resp.len()))
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rss_url = "https://blog.rust-lang.org/feed.xml";

    // 1. Download RSS
    println!("Fetching RSS feed...");
    let channel = fetch_rss(rss_url).await?;
    println!("Found {} items", channel.items().len());

    // 2. Spawn workers for each article
    let mut tasks = vec![];
    for item in channel.items() {
        if let Some(link) = item.link() {
            let url = link.to_string();
            println!("{}", url);
            tasks.push(task::spawn(fetch_article(url)));
        }
    }

    // 3. Wait for all workers
    for task in tasks {
        match task.await {
            Ok(Ok(result)) => println!("{}", result),
            Ok(Err(e)) => eprintln!("Error downloading article: {}", e),
            Err(e) => eprintln!("Worker task failed: {}", e),
        }
    }

    println!("All articles processed!");
    Ok(())
}
