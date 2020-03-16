extern crate hyper;
extern crate pocket;

use hyper::client::IntoUrl;
use pocket::{Pocket, PocketAddRequest};

fn main() {
    let pocket = Pocket::new(
        &std::env::var("POCKET_CONSUMER_KEY").unwrap(),
        &std::env::var("POCKET_ACCESS_TOKEN").unwrap(),
    );

    let url = "https://example.com".into_url().unwrap();
    let item = pocket
        .add(&PocketAddRequest::new(&url)
            .title("Example title")
            .tags(&["example-tag"])
            .tweet_id("example_tweet_id")
        )
        .unwrap();
    println!("item: {:?}", item);
}
