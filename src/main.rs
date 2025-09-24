use reqwest;
use rss::Channel;
use tokio::task;

async fn fetch_rss(url: &str) -> Result<Channel, Box<dyn std::error::Error + Send + Sync>> {
    println!("Fetching RSS feed from: {}", url);

    let resp = reqwest::get(url).await?;
    println!("HTTP Status: {}", resp.status());

    let content = resp.bytes().await?;
    println!(
        "First 200 bytes of response:\n{}",
        String::from_utf8_lossy(&content[..std::cmp::min(200, content.len())])
    );

    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

async fn fetch_article(url: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    println!("Fetching article: {}", url);
    let resp = reqwest::get(&url).await?.text().await?;
    Ok(format!(
        "Article from {} downloaded, size: {} bytes",
        url,
        resp.len()
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rss_url = "https://blog.rust-lang.org/feed.xml";

    println!("Fetching RSS feed...");
    match fetch_rss(rss_url).await {
        Ok(channel) => {
            println!("Found {} items in RSS", channel.items().len());

            let mut tasks = vec![];
            for item in channel.items() {
                if let Some(link) = item.link() {
                    let url = link.to_string();
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
            eprintln!("Error fetching RSS: {}", e);
        }
    }

    println!("All done!");
    Ok(())
}
