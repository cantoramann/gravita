#!/usr/bin/env bash
# scripts/tests/check-format.sh
# Check that all code is properly formatted
#
# Usage: ./scripts/tests/check-format.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$ROOT_DIR"

echo "🔍 Checking code formatting..."

if ! cargo fmt --all -- --check; then
    echo ""
    echo "❌ Code is not properly formatted!"
    echo "   Run 'cargo fmt --all' to fix."
    exit 1
fi

echo "✅ Code formatting OK"

