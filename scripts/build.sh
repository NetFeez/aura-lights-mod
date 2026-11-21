#!/usr/bin/env bash
set -euo pipefail

# Build the distributable .hmod at the workspace root, next to the signing folder.
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
root="$(dirname "$repo")"
cd "$repo"

id="$(sed -n 's/^id = "\([^"]*\)"/\1/p' horizon-mod.toml | head -1)"
version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' horizon-mod.toml | head -1)"
artifact="$root/${id}-${version}.hmod"

cargo horizon-mod build
cargo horizon-mod check "target/horizon-mod/${id}.hmod"
cp "target/horizon-mod/${id}.hmod" "$artifact"

echo "Built $artifact"
sha256sum "$artifact"
