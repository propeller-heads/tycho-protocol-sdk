#!/bin/bash
set -e

if [ "$#" -lt 1 ]; then
  echo "Usage: $0 test1 [test2 ...]"
  exit 1
fi

for test in "$@"; do
  tycho-protocol-sdk --package "$test"
done
