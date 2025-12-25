#!/usr/bin/env bash
# scripts/tests/check-clippy.sh
# Run Clippy linter with strict settings
#
# Usage: ./scripts/tests/check-clippy.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$ROOT_DIR"

echo "🔍 Running Clippy lints..."

# Run clippy with warnings as errors for CI
# Allow some warnings that are acceptable in a game engine context
cargo clippy --workspace --all-targets -- \
    -D warnings \
    -A clippy::too_many_arguments \
    -A clippy::type_complexity

echo "✅ Clippy checks passed"

