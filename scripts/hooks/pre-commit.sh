#!/usr/bin/env bash
# scripts/hooks/pre-commit.sh
# Git pre-commit hook for Gravita
#
# Runs format check and unit tests before allowing a commit.
# Install with: ./scripts/setup-hooks.sh
#
# To bypass (use sparingly): git commit --no-verify

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

# If we're being run as a git hook, SCRIPT_DIR will be .git/hooks
# Detect and adjust path accordingly
if [[ "$SCRIPT_DIR" == *".git/hooks"* ]]; then
    ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
    TESTS_DIR="$ROOT_DIR/scripts/tests"
else
    TESTS_DIR="$SCRIPT_DIR/../tests"
fi

cd "$ROOT_DIR"

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                  Pre-commit Checks                           ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Fast checks first
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
"$TESTS_DIR/check-format.sh"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
"$TESTS_DIR/check-clippy.sh"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
"$TESTS_DIR/run-unit-tests.sh"

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "✅ Pre-commit checks passed! Proceeding with commit..."
echo "════════════════════════════════════════════════════════════════"
echo ""

