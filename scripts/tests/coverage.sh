#!/usr/bin/env bash
# Code coverage with cargo-tarpaulin
# Install: cargo install cargo-tarpaulin
#
# Usage:
#   ./scripts/tests/coverage.sh           # Generate HTML report
#   ./scripts/tests/coverage.sh --xml     # Generate Cobertura XML (for CI)
#   ./scripts/tests/coverage.sh --lcov    # Generate LCOV format

set -euo pipefail

OUTPUT_FORMAT="${1:-html}"

echo "═══════════════════════════════════════════════════════════════"
echo "📊 Running code coverage analysis..."
echo "═══════════════════════════════════════════════════════════════"

# Check if tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "❌ cargo-tarpaulin not found."
    echo "   Install with: cargo install cargo-tarpaulin"
    exit 1
fi

# Create coverage output directory
mkdir -p target/coverage

case "$OUTPUT_FORMAT" in
    --html|html)
        echo "📄 Generating HTML coverage report..."
        cargo tarpaulin \
            --workspace \
            --exclude-files "examples/*" \
            --exclude-files "benches/*" \
            --ignore-tests \
            --out Html \
            --output-dir target/coverage \
            --timeout 120 \
            --skip-clean
        echo ""
        echo "✅ Coverage report: target/coverage/tarpaulin-report.html"
        ;;
    --xml|xml)
        echo "📄 Generating Cobertura XML report..."
        cargo tarpaulin \
            --workspace \
            --exclude-files "examples/*" \
            --exclude-files "benches/*" \
            --ignore-tests \
            --out Xml \
            --output-dir target/coverage \
            --timeout 120 \
            --skip-clean
        echo ""
        echo "✅ Coverage report: target/coverage/cobertura.xml"
        ;;
    --lcov|lcov)
        echo "📄 Generating LCOV report..."
        cargo tarpaulin \
            --workspace \
            --exclude-files "examples/*" \
            --exclude-files "benches/*" \
            --ignore-tests \
            --out Lcov \
            --output-dir target/coverage \
            --timeout 120 \
            --skip-clean
        echo ""
        echo "✅ Coverage report: target/coverage/lcov.info"
        ;;
    *)
        echo "Unknown format: $OUTPUT_FORMAT"
        echo "Valid options: html, --html, xml, --xml, lcov, --lcov"
        exit 1
        ;;
esac

echo "═══════════════════════════════════════════════════════════════"

