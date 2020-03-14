extern crate pocket;

use pocket::{Pocket, PocketGetRequest};

fn main() {
    let pocket = Pocket::new(
        &std::env::var("POCKET_CONSUMER_KEY").unwrap(),
        Some(&std::env::var("POCKET_ACCESS_TOKEN").unwrap()),
    );

    let items = {
        let mut request = PocketGetRequest::new();
        request.count(10);
        request.complete();
        pocket.get(&request)
    };
    println!("items: {:?}", items);
}
