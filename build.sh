#!/bin/sh
# Builds the web UI, then the Rust binary (the UI is embedded in it).
set -e
cd "$(dirname "$0")"
(cd web && npm install && npm run build)
cargo build --release
echo "OK : ./target/release/encodetool"
