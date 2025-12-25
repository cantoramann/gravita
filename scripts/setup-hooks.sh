#!/usr/bin/env bash
# scripts/setup-hooks.sh
# Install git hooks for local development
#
# Usage: ./scripts/setup-hooks.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$ROOT_DIR/.git/hooks"

echo "🔧 Setting up git hooks..."

# Ensure hooks directory exists
mkdir -p "$HOOKS_DIR"

# Create symlink to pre-commit hook
# Using a symlink means updates to the script are automatically picked up
PRE_COMMIT_SRC="$SCRIPT_DIR/hooks/pre-commit.sh"
PRE_COMMIT_DST="$HOOKS_DIR/pre-commit"

if [[ -L "$PRE_COMMIT_DST" ]]; then
    echo "   Removing existing symlink..."
    rm "$PRE_COMMIT_DST"
elif [[ -f "$PRE_COMMIT_DST" ]]; then
    echo "   Backing up existing pre-commit hook..."
    mv "$PRE_COMMIT_DST" "$PRE_COMMIT_DST.backup"
fi

ln -s "$PRE_COMMIT_SRC" "$PRE_COMMIT_DST"
echo "   Installed pre-commit hook"

echo ""
echo "✅ Git hooks installed!"
echo ""
echo "The following hooks are now active:"
echo "  • pre-commit: Runs format check, clippy, and unit tests"
echo ""
echo "To bypass hooks (use sparingly): git commit --no-verify"

