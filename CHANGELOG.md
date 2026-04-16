# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project follows SemVer while it is in `0.x`.

## [Unreleased]

### Added

- Compile-checked Rust examples under `examples/` for public REST access,
  signer setup, and skip-nonce transaction signing.
- Optional nonce skipping for L2 transactions via `L2TxAttributes` and the new
  `*_with_attributes(...)` helpers on `SignerClient`.

### Changed

- README usage guidance now points to the Rust examples directory instead of
  keeping the primary examples inline, and now describes the project as an
  independent open-source crate rather than a publishing-oriented workspace.
- Nonce validation now rejects values greater than or equal to `2^47 - 1`.
- REST responses with HTTP `405` or `429` are now both classified as rate
  limiting in the Rust SDK.

## [0.1.0] - 2026-04-16

### Added

- Standalone `lighter-rust` repository for the publishable `lighter-sdk` crate.
- Publish-ready Cargo metadata, release checks, and external signer docs.

### Changed

- Stopped bundling signer binaries inside the crate package.
