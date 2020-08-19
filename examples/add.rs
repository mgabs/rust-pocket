use pocket::{Pocket, PocketAddRequest};
use std::error::Error;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pocket = Pocket::new(
        &std::env::var("POCKET_CONSUMER_KEY")?,
        &std::env::var("POCKET_ACCESS_TOKEN")?,
    );

    let url = Url::parse("https://example.com")?;
    let item = pocket
        .add(
            &PocketAddRequest::new(&url)
                .title("Example title")
                .tags(&["example-tag"])
                .tweet_id("example_tweet_id"),
        )
        .await?;
    println!("item: {:?}", item);
    Ok(())
}
