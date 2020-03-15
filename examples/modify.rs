extern crate hyper;
extern crate pocket;

use hyper::client::IntoUrl;
use pocket::{Pocket, PocketSendAction, PocketSendRequest};

fn main() {
    let pocket = Pocket::new(
        &std::env::var("POCKET_CONSUMER_KEY").unwrap(),
        &std::env::var("POCKET_ACCESS_TOKEN").unwrap(),
    );
    let item_id = std::env::var("POCKET_ITEM_ID")
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let results = pocket
        .send(&PocketSendRequest {
            actions: &[
                &PocketSendAction::Add {
                    item_id: None,
                    ref_id: None,
                    tags: Some("example-tag".to_string()),
                    time: None,
                    title: Some("Example title".to_string()),
                    url: Some("https://example.com".into_url().unwrap()),
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
        })
        .unwrap();
    println!("results: {:?}", results);
}
