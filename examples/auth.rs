extern crate hyper;
extern crate pocket;

use pocket::PocketAuthentication;
use std::io;
use std::time::Instant;

fn main() {
    let auth = PocketAuthentication::new(
        &std::env::var("POCKET_CONSUMER_KEY").unwrap(),
        "rustapi:finishauth",
    );
    let state = Some(format!("{:?}", Instant::now()));
    let code = auth.request(state.as_deref()).unwrap();
    let url = auth.authorize_url(&code);
    println!(
        "Follow auth URL to provide access and press enter when finished: {}",
        url
    );
    let _ = io::stdin().read_line(&mut String::new());
    let user = auth.authorize(&code, state.as_deref()).unwrap();
    println!("username: {}", user.username);
    println!("access token: {:?}", user.access_token);
}
