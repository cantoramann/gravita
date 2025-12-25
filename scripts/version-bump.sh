#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# Version Bump Script for Gravita
# ═══════════════════════════════════════════════════════════════════════════════
#
# Usage:
#   ./scripts/version-bump.sh <crate> <bump-type> [--dry-run]
#   ./scripts/version-bump.sh --all <bump-type> [--dry-run]
#
# Examples:
#   ./scripts/version-bump.sh gravita-math patch           # 0.1.0 → 0.1.1
#   ./scripts/version-bump.sh gravita-physics minor        # 0.1.0 → 0.2.0
#   ./scripts/version-bump.sh gravita patch --dry-run      # Preview umbrella bump
#   ./scripts/version-bump.sh --all patch                  # Bump ALL crates
#
# IMPORTANT: When bumping a child crate, you should also bump the umbrella
#            `gravita` crate to include the updated dependency.
#
# ═══════════════════════════════════════════════════════════════════════════════

set -euo pipefail

# ─── Configuration ────────────────────────────────────────────────────────────

# Format: "crate-name:path"
CRATES=(
    "gravita-math:crates/math"
    "gravita-physics:crates/physics"
    "gravita-renderer:crates/renderer"
    "gravita-collections:crates/collections"
    "gravita-engine-core:crates/engine-core"
    "gravita-input:crates/input"
    "gravita-assets:crates/assets"
    "gravita:crates/gravita"
)

WORKSPACE_TOML="Cargo.toml"

# ─── Helper Functions ─────────────────────────────────────────────────────────

show_usage() {
    echo "Usage: $0 <crate-name> <patch|minor|major> [--dry-run]"
    echo "       $0 --all <patch|minor|major> [--dry-run]"
    echo ""
    echo "Available crates:"
    for entry in "${CRATES[@]}"; do
        name="${entry%%:*}"
        echo "  - $name"
    done
    echo ""
    echo "Bump types:"
    echo "  patch - Bug fixes (0.1.0 → 0.1.1)"
    echo "  minor - New features (0.1.0 → 0.2.0)"
    echo "  major - Breaking changes (0.1.0 → 1.0.0)"
    echo ""
    echo "Notes:"
    echo "  - When bumping a child crate, also bump 'gravita' umbrella"
    echo "  - Workspace dependency versions are updated automatically"
    exit 1
}

get_crate_path() {
    local crate_name="$1"
    for entry in "${CRATES[@]}"; do
        name="${entry%%:*}"
        path="${entry#*:}"
        if [[ "$name" == "$crate_name" ]]; then
            echo "$path"
            return 0
        fi
    done
    return 1
}

get_current_version() {
    local cargo_toml="$1/Cargo.toml"
    grep -m1 '^version = ' "$cargo_toml" | sed 's/.*"\(.*\)".*/\1/'
}

calculate_new_version() {
    local current="$1"
    local bump_type="$2"

    IFS='.' read -r major minor patch <<< "$current"

    case "$bump_type" in
        patch) echo "$major.$minor.$((patch + 1))" ;;
        minor) echo "$major.$((minor + 1)).0" ;;
        major) echo "$((major + 1)).0.0" ;;
    esac
}

update_workspace_dep_version() {
    local crate_name="$1"
    local new_version="$2"
    local dry_run="$3"

    # Update the version in [workspace.dependencies]
    # Pattern: gravita-math = { version = "0.1.0", path = "..." }
    if grep -q "^$crate_name = " "$WORKSPACE_TOML"; then
        if [[ "$dry_run" == "true" ]]; then
            echo "   Would update workspace dep: $crate_name = \"$new_version\""
        else
            # Use sed to update the version in the workspace dependency
            sed -i '' "s/^$crate_name = { version = \"[^\"]*\"/$crate_name = { version = \"$new_version\"/" "$WORKSPACE_TOML"
            echo "   ✅ Updated workspace dependency: $crate_name = \"$new_version\""
        fi
    fi
}

bump_crate() {
    local crate_name="$1"
    local bump_type="$2"
    local dry_run="$3"

    local crate_path
    crate_path=$(get_crate_path "$crate_name") || {
        echo "❌ Unknown crate: $crate_name"
        return 1
    }

    local current_version
    current_version=$(get_current_version "$crate_path")

    local new_version
    new_version=$(calculate_new_version "$current_version" "$bump_type")

    echo "📦 $crate_name: $current_version → $new_version ($bump_type)"

    if [[ "$dry_run" == "true" ]]; then
        echo "   (dry-run, no changes made)"
        update_workspace_dep_version "$crate_name" "$new_version" "$dry_run"
        return 0
    fi

    # Update Cargo.toml in crate directory
    local cargo_toml="$crate_path/Cargo.toml"
    sed -i '' "s/^version = \"$current_version\"/version = \"$new_version\"/" "$cargo_toml"
    echo "   ✅ Updated $cargo_toml"

    # Update workspace dependency version (for publishing)
    update_workspace_dep_version "$crate_name" "$new_version" "$dry_run"
}

# ─── Parse Arguments ──────────────────────────────────────────────────────────

if [[ $# -lt 2 ]]; then
    show_usage
fi

CRATE_ARG="$1"
BUMP_TYPE="$2"
DRY_RUN="false"

if [[ "${3:-}" == "--dry-run" ]]; then
    DRY_RUN="true"
fi

if [[ ! "$BUMP_TYPE" =~ ^(patch|minor|major)$ ]]; then
    echo "❌ Invalid bump type: $BUMP_TYPE"
    show_usage
fi

# ─── Execute ──────────────────────────────────────────────────────────────────

echo "═══════════════════════════════════════════════════════════════"
echo "🔄 Version Bump"
echo "═══════════════════════════════════════════════════════════════"
echo ""

if [[ "$CRATE_ARG" == "--all" ]]; then
    echo "⚠️  Bumping ALL crates..."
    echo ""
    for entry in "${CRATES[@]}"; do
        name="${entry%%:*}"
        bump_crate "$name" "$BUMP_TYPE" "$DRY_RUN"
    done
else
    bump_crate "$CRATE_ARG" "$BUMP_TYPE" "$DRY_RUN"

    # Remind about umbrella crate if not bumping it directly
    if [[ "$CRATE_ARG" != "gravita" ]]; then
        echo ""
        echo "💡 Reminder: Also bump the 'gravita' umbrella crate to include this change:"
        echo "   ./scripts/version-bump.sh gravita patch"
    fi
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"

if [[ "$DRY_RUN" == "true" ]]; then
    echo "🔍 Dry run complete. No files were modified."
    echo "   Run without --dry-run to apply changes."
else
    echo "✅ Version bump complete!"
    echo ""
    echo "📝 Next steps:"
    echo "   1. Update CHANGELOG.md with release notes"
    echo "   2. Commit: git add -A && git commit -m 'chore(release): bump versions'"
    echo "   3. Release: cargo release -p <crate> --execute"
fi
echo "═══════════════════════════════════════════════════════════════"
