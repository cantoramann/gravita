#!/usr/bin/env bash
# scripts/tests/run-unit-tests.sh
# Run all unit tests in the workspace
#
# Usage: ./scripts/tests/run-unit-tests.sh [--release]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$ROOT_DIR"

RELEASE_FLAG=""
if [[ "${1:-}" == "--release" ]]; then
    RELEASE_FLAG="--release"
    echo "🧪 Running unit tests (release mode)..."
else
    echo "🧪 Running unit tests..."
fi

cargo test --workspace --lib $RELEASE_FLAG

echo "✅ All unit tests passed"

