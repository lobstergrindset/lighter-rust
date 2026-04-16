use lighter_sdk::config::Config;
use lighter_sdk::rest::client::LighterRestClient;

fn env_or_default(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = env_or_default("LIGHTER_HOST", "testnet.zklighter.elliot.ai");
    let client = LighterRestClient::new(&Config::new(host))?;

    let stats = client.get_exchange_stats().await?;

    println!(
        "Fetched exchange stats for {} markets. Daily volume: {:?}",
        stats.order_book_stats.len(),
        stats.daily_usd_volume
    );

    Ok(())
}
