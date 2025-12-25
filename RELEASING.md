# Releasing Gravita to crates.io

This document describes the release process for Gravita maintainers.

## Overview

Gravita uses **independent versioning** with an **umbrella crate**:

- Each crate (`gravita-math`, `gravita-physics`, etc.) has its own version
- The `gravita` umbrella crate re-exports all others
- When any child crate changes, the umbrella must also be bumped

## Prerequisites

```bash
# Install cargo-release
cargo install cargo-release

# Login to crates.io
cargo login

# Verify you own the crate names (first time only)
# Visit https://crates.io and check availability
```

## Crate Structure

```
gravita (umbrella) ──────────────────┐
    │                                │
    ├── gravita-math          ◄──────┤
    ├── gravita-physics       ◄──────┤
    ├── gravita-renderer      ◄──────┤
    ├── gravita-collections   ◄──────┤
    ├── gravita-engine-core   ◄──────┤
    ├── gravita-input         ◄──────┤
    └── gravita-assets        ◄──────┘
```

Users can depend on either:
- `gravita = "0.1"` — the umbrella (convenient)
- `gravita-physics = "0.1"` — individual crates (minimal)

## Quick Reference

| Action | Command |
|--------|---------|
| Bump single crate | `./scripts/version-bump.sh gravita-math patch` |
| Bump umbrella | `./scripts/version-bump.sh gravita patch` |
| Bump all crates | `./scripts/version-bump.sh --all minor` |
| Preview release | `cargo release -p gravita-math --dry-run` |
| Execute release | `cargo release -p gravita-math --execute` |
| Validate | `./scripts/pre-release.sh gravita-math 0.2.0` |

## Release Workflow

### Scenario 1: Bug Fix in gravita-math

```bash
# 1. Bump the changed crate
./scripts/version-bump.sh gravita-math patch

# 2. Also bump the umbrella to pick up the change
./scripts/version-bump.sh gravita patch

# 3. Update CHANGELOG.md for both crates

# 4. Commit
git add -A
git commit -m "chore(release): gravita-math v0.1.1, gravita v0.1.1"

# 5. Publish in order (math first, then umbrella)
cargo release -p gravita-math --execute

# Wait for crates.io to index (1-5 minutes)
sleep 120

cargo release -p gravita --execute
```

### Scenario 2: New Feature in gravita-physics

```bash
# 1. Bump physics (minor for new feature)
./scripts/version-bump.sh gravita-physics minor

# 2. Bump umbrella (at least patch, or minor if exposing new API)
./scripts/version-bump.sh gravita minor

# 3. Update changelogs, commit, release in order
```

### Scenario 3: Breaking Change

```bash
# 1. Bump affected crate with major
./scripts/version-bump.sh gravita-math major

# 2. Bump all dependents that expose the breaking change
./scripts/version-bump.sh gravita-physics major  # if it exposes math types
./scripts/version-bump.sh gravita major

# 3. Update changelogs with migration guide
```

### Scenario 4: Coordinated Release (All Crates)

```bash
# Bump everything
./scripts/version-bump.sh --all minor

# Release in dependency order
cargo release -p gravita-math --execute
sleep 120
cargo release -p gravita-physics --execute
cargo release -p gravita-renderer --execute
sleep 120
cargo release -p gravita-collections --execute
cargo release -p gravita-input --execute
cargo release -p gravita-assets --execute
sleep 120
cargo release -p gravita-engine-core --execute
sleep 120
cargo release -p gravita --execute  # ALWAYS LAST
```

## Publishing Order

**CRITICAL**: Always publish in dependency order. The umbrella `gravita` is LAST.

```
1. gravita-math         (no dependencies)
2. gravita-physics      (depends on math)
   gravita-renderer     (depends on math)
3. gravita-collections  (depends on math, renderer)
4. gravita-input        (standalone)
   gravita-assets       (standalone)
5. gravita-engine-core  (depends on math, physics, renderer)
6. gravita              (depends on ALL - ALWAYS LAST)
```

## Version Synchronization

When you bump a child crate, two things happen:

1. **Crate's Cargo.toml** gets new version
2. **Workspace Cargo.toml** dependency version is updated

```toml
# Before bump
[workspace.dependencies]
gravita-math = { version = "0.1.0", path = "crates/math" }

# After ./scripts/version-bump.sh gravita-math patch
[workspace.dependencies]
gravita-math = { version = "0.1.1", path = "crates/math" }
```

This ensures `cargo publish` uses the correct version.

## Tag Format

| Crate | Tag Example |
|-------|-------------|
| gravita-math v0.2.0 | `gravita-math-v0.2.0` |
| gravita-physics v0.1.1 | `gravita-physics-v0.1.1` |
| gravita v0.2.0 | `v0.2.0` (special: no prefix) |

## First-Time Publishing

Before first publish, reserve crate names:

```bash
# Publish placeholder versions (0.0.1) to reserve names
# Do this in dependency order

# Check each name is available at https://crates.io/crates/<name>
```

## Troubleshooting

### "version already exists"

The version was already published. Bump again:

```bash
./scripts/version-bump.sh gravita-math patch
```

### "crate not found" during publish

Dependent crate hasn't been indexed yet. Wait and retry:

```bash
sleep 300  # Wait 5 minutes
cargo release -p gravita --execute
```

### Need to yank a release

```bash
cargo yank --vers 0.2.0 gravita-math
```

### Check current versions

```bash
# Local versions
grep -r "^version = " crates/*/Cargo.toml

# Published versions
cargo search gravita
```

## Release Checklist

Before releasing:

- [ ] All tests pass (`./scripts/tests/run-all.sh`)
- [ ] CHANGELOG.md updated for each bumped crate
- [ ] Version bumped in crate's `Cargo.toml`
- [ ] Version bumped in workspace `[dependencies]`
- [ ] Umbrella `gravita` crate also bumped
- [ ] Pre-release validation passes
- [ ] Committed with appropriate message

After releasing:

- [ ] Tags pushed to GitHub
- [ ] Verify crate appears on crates.io
- [ ] Verify docs appear on docs.rs
- [ ] Create GitHub Release (optional)
- [ ] Announce release (optional)
