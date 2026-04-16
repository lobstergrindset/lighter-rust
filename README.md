# lighter-rust

`lighter-rust` is the standalone home of the publishable `lighter-sdk` crate.

`lighter-sdk` is a Rust SDK for interacting with the Lighter exchange. It
provides:

- a signer-backed transaction client
- REST API access
- WebSocket streaming
- nonce management helpers

> [!WARNING]
> This repository was written with AI assistance and does not have complete
> feature or test coverage. Treat it as a useful starting point rather than a
> fully hardened SDK. If you plan to depend on it heavily, developing from your
> own fork may be the safest way to use it.
>
> PRs or feature requests are welcome.

## Install

```toml
[dependencies]
lighter-sdk = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Quickstart

```rust,no_run
use std::collections::HashMap;

use lighter_sdk::client::SignerClient;
use lighter_sdk::config::Config;
use lighter_sdk::nonce::NonceManagerType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new("testnet.lighter.xyz")
        .with_signer_lib_path("/path/to/signer/or/directory");

    let mut api_private_keys = HashMap::new();
    api_private_keys.insert(0u8, "your-private-key".to_string());

    let client = SignerClient::new(
        config,
        0,
        api_private_keys,
        NonceManagerType::Api,
    )
    .await?;

    client.check_client()?;
    Ok(())
}
```

A compile-checked example also lives at [`examples/quickstart.rs`](examples/quickstart.rs).

## Signer Installation

`lighter-sdk` does not ship the signer shared library inside the crate package.
Install the signer separately from the
[`lighter-go` releases](https://github.com/elliottech/lighter-go/releases) or
build it from source with the
[`lighter-go` justfile](https://github.com/elliottech/lighter-go/blob/main/justfile).

Supported runtime lookup order:

1. `Config::with_signer_lib_path(...)`
2. `LIGHTER_SIGNER_LIB_PATH`
3. next to the current executable, or in `signers/` next to it
4. the current working directory, or `./signers`

Supported library filenames:

- macOS arm64: `lighter-signer-darwin-arm64.dylib`
- Linux amd64: `lighter-signer-linux-amd64.so`
- Linux arm64: `lighter-signer-linux-arm64.so`
- Windows amd64: `lighter-signer-windows-amd64.dll`

## Compatibility

| `lighter-sdk` | Tested signer source | Notes |
| --- | --- | --- |
| `0.1.0` | [`lighter-go`](https://github.com/elliottech/lighter-go) | Shared library release/build source |

Reference ports used while building this crate:

| SDK | Repository |
| --- | --- |
| Go reference signer | [`lighter-go`](https://github.com/elliottech/lighter-go) |
| Python reference SDK | [`lighter-python`](https://github.com/elliottech/lighter-python) |

## Nonce and Auth Notes

- Each API key has its own nonce stream.
- `NonceManagerType::Api` is the simplest starting point.
- `NonceManagerType::Optimistic` reduces latency when you manage order flow
  heavily and want local nonce reservation.
- Auth tokens are bound to API keys and should be regenerated if the backing
  key changes.

## Development

Release validation is intentionally manual for the first public releases:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps
./scripts/check-package.sh
cargo publish --dry-run
```

There is also an ignored smoke test for validating an external signer:

```bash
LIGHTER_SIGNER_LIB_PATH=/path/to/signer \
LIGHTER_SDK_SMOKE_HOST=testnet.lighter.xyz \
LIGHTER_SDK_SMOKE_PRIVATE_KEY=... \
LIGHTER_SDK_SMOKE_API_KEY_INDEX=0 \
LIGHTER_SDK_SMOKE_ACCOUNT_INDEX=0 \
cargo test signer_client_smoke -- --ignored
```
