use pocket::PocketAuthentication;
use std::error::Error;
use std::io;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let consumer_key = std::env::var("POCKET_CONSUMER_KEY")?;
    println!("consumer key: {}", consumer_key);
    let auth = PocketAuthentication::new(&consumer_key, "rustapi:finishauth");
    let state = Some(format!("{:?}", Instant::now()));
    let code = auth.request(state.as_deref()).await?;
    let url = auth.authorize_url(&code);
    println!(
        "Follow auth URL to provide access and press enter when finished: {}",
        url
    );
    let _ = io::stdin().read_line(&mut String::new());
    let user = auth.authorize(&code, state.as_deref()).await?;
    println!("username: {}", user.username);
    println!("access token: {:?}", user.access_token);
    Ok(())
}
