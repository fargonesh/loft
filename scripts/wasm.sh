#!/usr/bin/env bash
set -e

# Build WASM package for the playground
wasm-pack build \
    --target web \
    --out-dir www/src/pkg \
    --no-default-features # Disable FFI
