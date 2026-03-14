#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="$ROOT_DIR/crates/iso4217-catalog/reference/iso4217"

mkdir -p "$TARGET_DIR"

curl -fsSL "https://www.six-group.com/dam/download/financial-information/data-center/iso-currrency/lists/list-one.xml" -o "$TARGET_DIR/list-one.xml"

echo "Updated ISO list-one XML in $TARGET_DIR"
