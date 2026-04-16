use std::collections::HashMap;

use lighter_sdk::client::SignerClient;
use lighter_sdk::config::Config;
use lighter_sdk::nonce::NonceManagerType;

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("missing env var `{name}`"))
}

#[tokio::test]
#[ignore = "requires an external signer shared library and real API credentials"]
async fn signer_client_smoke() {
    let signer_path = required_env("LIGHTER_SIGNER_LIB_PATH");
    let host = required_env("LIGHTER_SDK_SMOKE_HOST");
    let private_key = required_env("LIGHTER_SDK_SMOKE_PRIVATE_KEY");
    let api_key_index = required_env("LIGHTER_SDK_SMOKE_API_KEY_INDEX")
        .parse::<u8>()
        .expect("invalid api key index");
    let account_index = required_env("LIGHTER_SDK_SMOKE_ACCOUNT_INDEX")
        .parse::<i64>()
        .expect("invalid account index");

    let mut api_private_keys = HashMap::new();
    api_private_keys.insert(api_key_index, private_key);

    let config = Config::new(host).with_signer_lib_path(signer_path);
    let client = SignerClient::new(
        config,
        account_index,
        api_private_keys,
        NonceManagerType::Api,
    )
    .await
    .expect("failed to create signer client");

    client
        .check_client()
        .expect("signer client validation failed");
}
