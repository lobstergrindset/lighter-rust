use std::collections::HashMap;

use lighter_sdk::client::SignerClient;
use lighter_sdk::config::Config;
use lighter_sdk::nonce::NonceManagerType;
use lighter_sdk::types::transact_opts::L2TxAttributes;

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("missing env var `{name}`"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = required_env("LIGHTER_HOST");
    let signer_path = required_env("LIGHTER_SIGNER_LIB_PATH");
    let account_index = required_env("LIGHTER_ACCOUNT_INDEX").parse::<i64>()?;
    let api_key_index = required_env("LIGHTER_API_KEY_INDEX").parse::<u8>()?;
    let api_private_key = required_env("LIGHTER_API_PRIVATE_KEY");

    let config = Config::new(host).with_signer_lib_path(signer_path);

    let mut api_private_keys = HashMap::new();
    api_private_keys.insert(api_key_index, api_private_key);

    let client = SignerClient::new(
        config,
        account_index,
        api_private_keys,
        NonceManagerType::Api,
    )
    .await?;

    let signed = client.sign_create_order_with_attributes(
        0,
        123,
        1_000,
        400_000,
        false,
        0,
        1,
        false,
        0,
        -1,
        L2TxAttributes::skip_nonce_enabled(),
        api_key_index,
        2_222,
    )?;

    println!("{}", signed.tx_info);
    Ok(())
}
