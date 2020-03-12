extern crate pocket;

use pocket::Pocket;
use std::io;

fn main() {
    let pocket = Pocket::auth(&std::env::var("POCKET_CONSUMER_KEY").unwrap());
    let pocket = pocket.request("rustapi:finishauth").unwrap();
    println!("Follow auth URL to provide access and press enter when finished: {}", pocket.url());
    let _ = io::stdin().read_line(&mut String::new());
    let user = pocket.authorize().unwrap();
    println!("username: {:?}", user);
    println!("access token: {:?}", user.access_token);

    let pocket = user.pocket();
    let item = pocket.push("https://example.com").unwrap();
    println!("item: {:?}", item);

    let items = {
        let mut f = pocket.filter();
        f.complete();
        f.archived();
        f.videos();
        f.offset(10);
        f.count(10);
        f.sort_by_title(); // sorted by title
        pocket.get(&f)
    };
    println!("items: {:?}", items);
}
