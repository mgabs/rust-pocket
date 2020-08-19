extern crate hyper;
extern crate pocket;

use std::error::Error;
use pocket::{Pocket, PocketSendAction, PocketSendRequest};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pocket = Pocket::new(
        &std::env::var("POCKET_CONSUMER_KEY")?,
        &std::env::var("POCKET_ACCESS_TOKEN")?,
    );
    let item_id = std::env::var("POCKET_ITEM_ID")?
        .parse::<u64>()?;

    let results = pocket
        .send(&PocketSendRequest {
            actions: &[
                &PocketSendAction::Add {
                    item_id: None,
                    ref_id: None,
                    tags: Some("example-tag".to_string()),
                    time: None,
                    title: Some("Example title".to_string()),
                    url: Some(Url::parse("https://example.com")?),
                },
                &PocketSendAction::Archive {
                    item_id,
                    time: None,
                },
                &PocketSendAction::TagsAdd {
                    item_id,
                    tags: "one,two".to_string(),
                    time: None,
                },
                &PocketSendAction::TagRename {
                    old_tag: "one".to_string(),
                    new_tag: "1".to_string(),
                    time: None,
                },
            ],
        }).await?;
    println!("results: {:?}", results);
    Ok(())
}
