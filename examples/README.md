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

## Nonce Notes

- Each API key has its own nonce stream.
- `NonceManagerType::Api` is the simplest starting point for signer-backed
  clients.
- `NonceManagerType::Optimistic` is useful when you want lower-latency local
  nonce reservation and are managing a heavier transaction flow.
- `SkipNonce = 1` is optional and can be set through `L2TxAttributes`, for
  example with `L2TxAttributes::skip_nonce_enabled()`.
- Skipping nonces does not remove nonce ordering requirements. You still need
  to provide an explicit nonce, and the next nonce must be greater than the
  previous nonce and less than `2^47 - 1`.

## Auth Notes

- Auth tokens are bound to API keys and should be regenerated if the backing
  key changes.
