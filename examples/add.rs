extern crate pocket;
extern crate hyper;

use pocket::Pocket;
use hyper::client::IntoUrl;

fn main() {
    let pocket = Pocket::new(
        &std::env::var("POCKET_CONSUMER_KEY").unwrap(),
        Some(&std::env::var("POCKET_ACCESS_TOKEN").unwrap()),
    );

    let item = pocket.add(
        "http://example.com".into_url().unwrap(),
        Some("Example"),
        Some("one,two"),
        None,
    ).unwrap();
    println!("item: {:?}", item);
}
