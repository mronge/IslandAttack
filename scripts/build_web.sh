#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/web/dist"
WASM_TARGET_DIR="$ROOT/target/wasm32-unknown-unknown/release"
WASM_PATH="$WASM_TARGET_DIR/tiger.wasm"
MQ_BUNDLE_PATH="$(find "$HOME/.cargo/registry/src" -path '*macroquad-*/js/mq_js_bundle.js' | head -n 1)"

if [[ -z "${MQ_BUNDLE_PATH}" ]]; then
    echo "Could not find macroquad's mq_js_bundle.js in the cargo registry." >&2
    exit 1
fi

rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown

mkdir -p "$DIST"
cp "$ROOT/web/index.html" "$DIST/index.html"
cp "$WASM_PATH" "$DIST/tiger.wasm"
cp "$MQ_BUNDLE_PATH" "$DIST/mq_js_bundle.js"
rm -rf "$DIST/sprites" "$DIST/music"
cp -R "$ROOT/sprites" "$DIST/sprites"
cp -R "$ROOT/music" "$DIST/music"

cat <<EOF
Web build ready in:
  $DIST

Serve it with any static file server, for example:
  cd "$DIST" && python3 -m http.server 8000
EOF
