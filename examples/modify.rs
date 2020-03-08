extern crate pocket;
extern crate hyper;

use pocket::{Pocket, PocketSendRequest, PocketSendAction};

fn main() {
    let pocket = Pocket::new(
        &std::env::var("POCKET_CONSUMER_KEY").unwrap(),
        Some(&std::env::var("POCKET_ACCESS_TOKEN").unwrap()),
    );
    let item_id = std::env::var("POCKET_ITEM_ID").unwrap().parse::<u64>().unwrap();

    let results = pocket.send(&PocketSendRequest {
        actions: &[
            &PocketSendAction::Archive { item_id, time: None },
            &PocketSendAction::TagsAdd { item_id, tags: "one,two".to_string(), time: None },
        ]
    }).unwrap();
    println!("results: {:?}", results);
}
