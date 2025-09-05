#!/bin/bash
set -e

for crate in */; do
  if [ -f "$crate/Cargo.toml" ]; then
    echo "Building $crate"
    cargo build --release --manifest-path "$crate/Cargo.toml"
  fi
done
