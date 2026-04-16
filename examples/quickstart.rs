use std::collections::HashMap;

use lighter_sdk::client::SignerClient;
use lighter_sdk::config::Config;
use lighter_sdk::nonce::NonceManagerType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new("testnet.lighter.xyz").with_signer_lib_path("/path/to/signer");

    let mut api_private_keys = HashMap::new();
    api_private_keys.insert(0u8, "your-private-key".to_string());

    let client = SignerClient::new(config, 0, api_private_keys, NonceManagerType::Api).await?;
    client.check_client()?;
    Ok(())
}
