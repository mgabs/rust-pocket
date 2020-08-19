extern crate pocket;

use std::error::Error;
use pocket::{Pocket, PocketGetRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let pocket = Pocket::new(
        &std::env::var("POCKET_CONSUMER_KEY")?,
        &std::env::var("POCKET_ACCESS_TOKEN")?,
    );

    let items = {
        let mut request = PocketGetRequest::new();
        request.count(10);
        request.complete();
        pocket.get(&request).await?
    };
    println!("items: {:?}", items);
    Ok(())
}
