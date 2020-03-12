extern crate pocket;
extern crate hyper;

use pocket::Pocket;
use std::io;

fn main() {
    let pocket = Pocket::auth(&std::env::var("POCKET_CONSUMER_KEY").unwrap());
    let pocket = pocket.request("rustapi:finishauth").unwrap();
    println!("Follow auth URL to provide access and press enter when finished: {}", pocket.url());
    let _ = io::stdin().read_line(&mut String::new());
    let user = pocket.authorize().unwrap();
    println!("username: {}", user.username);
    println!("access token: {:?}", user.access_token);
}
