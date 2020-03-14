extern crate pocket;
extern crate hyper;

use pocket::Pocket;
use std::io;

fn main() {
    let mut pocket = Pocket::new(&std::env::var("POCKET_CONSUMER_KEY").unwrap(), None);
    let url = pocket.get_auth_url().unwrap();
    println!("Follow auth URL to provide access and press enter when finished: {}", url);
    let _ = io::stdin().read_line(&mut String::new());
    let username = pocket.authorize().unwrap();
    println!("username: {}", username);
    println!("access token: {:?}", pocket.access_token());
}
