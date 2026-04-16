# lighter-rust

`lighter-rust` is an independently maintained Rust client for the Lighter
exchange. The Cargo package published from this repository is `lighter-sdk`.

This repository is open-source work done independently and is not affiliated
with, maintained by, or endorsed by Lighter.

`lighter-sdk` provides:

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

## Examples

The public usage examples live in [`examples/`](examples) and are intended to
be the primary source of truth instead of long README snippets:

- [`examples/public_rest.rs`](examples/public_rest.rs): fetch public exchange
  stats over REST without signer setup
- [`examples/quickstart.rs`](examples/quickstart.rs): initialize a
  `SignerClient` from environment variables and validate the configured key
- [`examples/skip_nonce_order.rs`](examples/skip_nonce_order.rs): sign a create
  order transaction with `SkipNonce = 1` without sending it
- [`examples/README.md`](examples/README.md): example-specific setup notes and
  required environment variables

Run them with:

```bash
cargo run --example public_rest
cargo run --example quickstart
cargo run --example skip_nonce_order
```

The signer-backed examples expect:

- `LIGHTER_HOST`
- `LIGHTER_SIGNER_LIB_PATH`
- `LIGHTER_ACCOUNT_INDEX`
- `LIGHTER_API_KEY_INDEX`
- `LIGHTER_API_PRIVATE_KEY`

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
- Skipping nonces is optional and is done by setting `SkipNonce = 1` through
  `L2TxAttributes`, for example with `L2TxAttributes::skip_nonce_enabled()`.
- Skipping a nonce does not remove nonce ordering requirements. The next nonce
  must still be greater than the previous nonce and less than `2^47 - 1`.
- Auth tokens are bound to API keys and should be regenerated if the backing
  key changes.

## Development

Local validation:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --examples
cargo doc --no-deps
./scripts/check-package.sh
cargo publish --dry-run
```

There is also an ignored smoke test for validating an external signer:

```bash
LIGHTER_SIGNER_LIB_PATH=/path/to/signer \
LIGHTER_SDK_SMOKE_HOST=your-lighter-host \
LIGHTER_SDK_SMOKE_PRIVATE_KEY=... \
LIGHTER_SDK_SMOKE_API_KEY_INDEX=0 \
LIGHTER_SDK_SMOKE_ACCOUNT_INDEX=0 \
cargo test signer_client_smoke -- --ignored
```
