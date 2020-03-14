extern crate pocket;

use pocket::Pocket;
use std::io;

fn main() {
    let mut pocket = Pocket::new(&*option_env!("POCKET_CONSUMER_KEY").unwrap(), None);
    let url = pocket.get_auth_url().unwrap();
    println!("Follow auth URL to provide access and press enter when finished: {}", url);
    let _ = io::stdin().read_line(&mut String::new());
    let username = pocket.authorize().unwrap();
    println!("username: {}", username);
    println!("access token: {:?}", pocket.access_token());

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
