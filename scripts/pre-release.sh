#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# Pre-release Validation Script
# ═══════════════════════════════════════════════════════════════════════════════
#
# Called automatically by cargo-release before publishing.
# Can also be run manually to validate release readiness.
#
# Usage:
#   ./scripts/pre-release.sh <crate-name> <version>
#   ./scripts/pre-release.sh                         # Validate all
#
# Examples:
#   ./scripts/pre-release.sh gravita-math 0.2.0
#   ./scripts/pre-release.sh gravita-physics 0.1.1
#
# ═══════════════════════════════════════════════════════════════════════════════

set -euo pipefail

CRATE_NAME="${1:-all}"
VERSION="${2:-}"

echo "═══════════════════════════════════════════════════════════════"
if [[ "$CRATE_NAME" == "all" ]]; then
    echo "🚀 Pre-release checks (all crates)"
else
    echo "🚀 Pre-release checks: $CRATE_NAME v$VERSION"
fi
echo "═══════════════════════════════════════════════════════════════"
echo ""

# ─── Check for uncommitted changes ────────────────────────────────────────────

echo "📋 Checking for uncommitted changes..."
if ! git diff --quiet 2>/dev/null; then
    echo "⚠️  Warning: You have uncommitted changes"
    git status --short
    echo ""
fi

# ─── Run formatting check ─────────────────────────────────────────────────────

echo "📋 Checking code formatting..."
if ! cargo fmt --all -- --check 2>/dev/null; then
    echo "❌ Code is not formatted. Run: cargo fmt --all"
    exit 1
fi
echo "   ✅ Formatting OK"

# ─── Run clippy ───────────────────────────────────────────────────────────────

echo "📋 Running Clippy..."
if [[ "$CRATE_NAME" == "all" ]]; then
    if ! cargo clippy --workspace --lib -- -D warnings 2>/dev/null; then
        echo "❌ Clippy found issues"
        exit 1
    fi
else
    if ! cargo clippy -p "$CRATE_NAME" --lib -- -D warnings 2>/dev/null; then
        echo "❌ Clippy found issues in $CRATE_NAME"
        exit 1
    fi
fi
echo "   ✅ Clippy OK"

# ─── Run tests ────────────────────────────────────────────────────────────────

echo "📋 Running tests..."
if [[ "$CRATE_NAME" == "all" ]]; then
    if ! cargo test --workspace --lib 2>/dev/null; then
        echo "❌ Tests failed"
        exit 1
    fi
else
    if ! cargo test -p "$CRATE_NAME" --lib 2>/dev/null; then
        echo "❌ Tests failed for $CRATE_NAME"
        exit 1
    fi
fi
echo "   ✅ Tests pass"

# ─── Check documentation builds ───────────────────────────────────────────────

echo "📋 Checking documentation..."
if [[ "$CRATE_NAME" == "all" ]]; then
    if ! RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps 2>/dev/null; then
        echo "❌ Documentation has warnings"
        exit 1
    fi
else
    if ! RUSTDOCFLAGS="-D warnings" cargo doc -p "$CRATE_NAME" --no-deps 2>/dev/null; then
        echo "❌ Documentation has warnings for $CRATE_NAME"
        exit 1
    fi
fi
echo "   ✅ Documentation OK"

# ─── Verify benchmarks compile ────────────────────────────────────────────────

echo "📋 Verifying benchmarks compile..."
if ! cargo bench --workspace --no-run 2>/dev/null; then
    echo "❌ Benchmarks failed to compile"
    exit 1
fi
echo "   ✅ Benchmarks compile"

# ─── Done ─────────────────────────────────────────────────────────────────────

echo ""
echo "═══════════════════════════════════════════════════════════════"
if [[ "$CRATE_NAME" == "all" ]]; then
    echo "✅ All pre-release checks passed!"
else
    echo "✅ Pre-release checks passed for $CRATE_NAME v$VERSION"
fi
echo "═══════════════════════════════════════════════════════════════"
