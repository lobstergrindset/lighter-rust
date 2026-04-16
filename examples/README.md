# Examples

The examples in this directory are intended to be the primary usage references
for the crate.

Available examples:

- `public_rest.rs`: public REST access without signer setup
- `quickstart.rs`: signer client initialization and `check_client()`
- `skip_nonce_order.rs`: sign a create-order transaction with `SkipNonce = 1`

Run them with:

```bash
cargo run --example public_rest
cargo run --example quickstart
cargo run --example skip_nonce_order
```

Signer-backed examples expect these environment variables:

```bash
export LIGHTER_HOST=your-lighter-host
export LIGHTER_SIGNER_LIB_PATH=/path/to/lighter-signer
export LIGHTER_ACCOUNT_INDEX=123
export LIGHTER_API_KEY_INDEX=0
export LIGHTER_API_PRIVATE_KEY=your-api-private-key
```

`public_rest.rs` defaults to `testnet.zklighter.elliot.ai` if `LIGHTER_HOST` is
not set.
