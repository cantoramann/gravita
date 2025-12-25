#!/usr/bin/env bash
# scripts/tests/run-all.sh
# Run all tests in the workspace
#
# Usage: ./scripts/tests/run-all.sh
#
# This is the main entry point for CI and local testing.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$ROOT_DIR"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    Gravita Test Suite                        ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Run individual test scripts
"$SCRIPT_DIR/check-format.sh"
"$SCRIPT_DIR/check-clippy.sh"
"$SCRIPT_DIR/run-unit-tests.sh"
"$SCRIPT_DIR/check-docs.sh"

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "✅ All checks passed!"
echo "════════════════════════════════════════════════════════════════"

