#!/usr/bin/env bash
# scripts/tests/check-docs.sh
# Check that documentation builds without warnings
#
# Usage: ./scripts/tests/check-docs.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$ROOT_DIR"

echo "📚 Checking documentation..."

# Build docs and treat warnings as errors
# Note: We use RUSTDOCFLAGS to make warnings fail the build
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items 2>&1 || {
    echo ""
    echo "❌ Documentation has warnings or errors!"
    exit 1
}

echo "✅ Documentation builds cleanly"

