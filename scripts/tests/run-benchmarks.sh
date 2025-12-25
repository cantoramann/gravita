#!/usr/bin/env bash
# scripts/tests/run-benchmarks.sh
# Run performance benchmarks
#
# Usage: ./scripts/tests/run-benchmarks.sh [crate] [filter]
#
# Examples:
#   ./scripts/tests/run-benchmarks.sh              # Run all benchmarks
#   ./scripts/tests/run-benchmarks.sh math         # Run only math crate benchmarks
#   ./scripts/tests/run-benchmarks.sh physics      # Run only physics crate benchmarks
#   ./scripts/tests/run-benchmarks.sh math Vec2    # Run math benchmarks containing "Vec2"

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$ROOT_DIR"

CRATE="${1:-}"
FILTER="${2:-}"

echo "🏃 Running benchmarks..."
echo ""

case "$CRATE" in
    math)
        echo "Crate: gravita-math"
        if [[ -n "$FILTER" ]]; then
            echo "Filter: $FILTER"
            cargo bench -p gravita-math -- "$FILTER"
        else
            cargo bench -p gravita-math
        fi
        ;;
    physics)
        echo "Crate: gravita-physics"
        if [[ -n "$FILTER" ]]; then
            echo "Filter: $FILTER"
            cargo bench -p gravita-physics -- "$FILTER"
        else
            cargo bench -p gravita-physics
        fi
        ;;
    "")
        echo "Running all benchmarks..."
        cargo bench --workspace
        ;;
    *)
        # Treat as a filter for all benchmarks
        echo "Filter: $CRATE"
        cargo bench --workspace -- "$CRATE"
        ;;
esac

echo ""
echo "✅ Benchmarks complete!"
echo ""
echo "Results are saved in: target/criterion/"
echo "View HTML report: open target/criterion/report/index.html"

