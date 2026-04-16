# Release Checklist

This repo uses a manual release flow.

## Before You Publish

1. Confirm the crate version in `Cargo.toml`.
2. Move release notes from `CHANGELOG.md` `Unreleased` into a dated version
   section.
3. Verify the README compatibility table and example docs still match the
   release.

## Validation

Run from the repo root:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --examples
cargo doc --no-deps
./scripts/check-package.sh
cargo publish --dry-run
```

Optional signer smoke test:

```bash
LIGHTER_SIGNER_LIB_PATH=/path/to/signer \
LIGHTER_SDK_SMOKE_HOST=your-lighter-host \
LIGHTER_SDK_SMOKE_PRIVATE_KEY=... \
LIGHTER_SDK_SMOKE_API_KEY_INDEX=0 \
LIGHTER_SDK_SMOKE_ACCOUNT_INDEX=0 \
cargo test signer_client_smoke -- --ignored
```

## Publish

```bash
git tag vX.Y.Z
cargo publish
git push origin main --tags
```

## After Publish

1. Confirm the release on crates.io.
2. Confirm docs.rs built successfully.
3. Create or update the GitHub release notes.
4. If needed, add crate owners with `cargo owner --add ...`.
