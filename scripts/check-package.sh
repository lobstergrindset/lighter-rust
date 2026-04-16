#!/usr/bin/env bash
set -euo pipefail

crate_name="lighter-sdk"
crate_limit_bytes=$((10 * 1024 * 1024))

rm -rf target/package
cargo package --allow-dirty >/dev/null

crate_file="$(find target/package -maxdepth 1 -name "${crate_name}-*.crate" -print -quit)"
if [[ -z "${crate_file}" ]]; then
  echo "could not find packaged crate artifact"
  exit 1
fi

crate_size="$(wc -c < "${crate_file}")"
if (( crate_size >= crate_limit_bytes )); then
  echo "packaged crate is too large: ${crate_size} bytes"
  exit 1
fi

if tar -tf "${crate_file}" | grep -E '(^|/)(signers/|.*\.(so|dylib|dll|h)$)' >/dev/null; then
  echo "packaged crate still contains signer artifacts"
  exit 1
fi

echo "package contents look good: ${crate_file}"
