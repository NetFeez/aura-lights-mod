#!/usr/bin/env bash
set -euo pipefail

# Sign the .hmod built by scripts/build.sh and verify it the way the registry does.
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
root="$(dirname "$repo")"
cd "$repo"

id="$(sed -n 's/^id = "\([^"]*\)"/\1/p' horizon-mod.toml | head -1)"
version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' horizon-mod.toml | head -1)"
artifact="$root/${id}-${version}.hmod"
sig="${artifact}.minisig"
key="$root/signing/horizon-mods.key"
pub="$root/signing/horizon-mods.pub"

for file in "$artifact" "$key" "$pub"; do
    if [ ! -f "$file" ]; then
        echo "Missing $file" >&2
        exit 1
    fi
done

public_key="$(tail -1 "$pub")"

rsign sign "$artifact" \
    -s "$key" -p "$pub" -W \
    -x "$sig" \
    -c "NetFeez Aurora Lights ${version}" \
    -t "release ${version}"
rsign verify "$artifact" -P "$public_key" -x "$sig"
cargo horizon-mod check "$artifact" --key "$public_key" --signature "$sig"

entry="$root/registry/mods/${id}.toml"
if [ -f "$entry" ]; then
    python3 - "$sig" "$entry" <<'PY'
import sys
from pathlib import Path

sig_path, entry_path = map(Path, sys.argv[1:3])
text = entry_path.read_text()
marker = 'signature = """'
if marker not in text:
    sys.exit(f"{entry_path} has no signature field")
start = text.index(marker)
end = text.index('"""', start + len(marker)) + 3
entry_path.write_text(text[:start] + marker + "\n" + sig_path.read_text() + '"""' + text[end:])
print(f"Updated {entry_path}")
PY
fi

echo "Signed $artifact"
