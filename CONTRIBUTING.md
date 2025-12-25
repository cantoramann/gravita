# Contributing to Gravita

Thank you for your interest in contributing to Gravita! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Code Style](#code-style)
- [Testing](#testing)
- [Benchmarks](#benchmarks)
- [Code Coverage](#code-coverage)
- [Documentation](#documentation)
- [Releasing](#releasing)
- [Submitting Changes](#submitting-changes)

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- **Rust nightly** (managed via `rust-toolchain.toml`)
- **Git** for version control
- A code editor with Rust support (VS Code + rust-analyzer recommended)

### Development Setup

1. **Fork and clone the repository**

   ```bash
   git clone https://github.com/cantoramann/gravita.git
   cd gravita
   ```

2. **Install git hooks** (recommended)

   ```bash
   ./scripts/setup-hooks.sh
   ```

   This installs pre-commit hooks that run formatting, linting, and tests before each commit.

3. **Verify the toolchain is set up**

   ```bash
   rustc --version
   # Should show nightly-2025-08-08 or later
   ```

4. **Build the project**

   ```bash
   cargo build --workspace
   ```

5. **Run the full test suite**

   ```bash
   ./scripts/tests/run-all.sh
   ```

   Or run individual checks:

   ```bash
   ./scripts/tests/check-format.sh   # Check code formatting
   ./scripts/tests/check-clippy.sh   # Run Clippy lints
   ./scripts/tests/run-unit-tests.sh # Run unit tests
   ./scripts/tests/check-docs.sh     # Check documentation
   ```

6. **Run an example to verify everything works**

   ```bash
   cargo run --example bouncing-balls
   ```

## Making Changes

### Branch Naming

Use descriptive branch names:

- `feature/polygon-collisions` - New features
- `fix/collision-detection-edge-case` - Bug fixes
- `docs/improve-physics-docs` - Documentation improvements
- `refactor/simplify-integrator` - Code refactoring

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `perf`: Performance improvements
- `chore`: Maintenance tasks

**Examples:**

```
feat(physics): add polygon collision shape support

fix(math): handle zero-length vector normalization

docs(readme): add usage examples for Vec2
```

## Code Style

### Formatting

The project uses `rustfmt` with custom settings in `rustfmt.toml`. Always format before committing:

```bash
cargo fmt --all
```

### Linting

Run Clippy and fix all warnings:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

### Style Guidelines

1. **Documentation**: All public items must have doc comments

   ```rust
   /// Compute the dot product of two vectors.
   ///
   /// # Examples
   ///
   /// ```
   /// use gravita_math::Vec2;
   ///
   /// let a = Vec2::new(1.0, 2.0);
   /// let b = Vec2::new(3.0, 4.0);
   /// assert_eq!(a.dot(b), 11.0);
   /// ```
   pub fn dot(&self, other: Vec2) -> f32 {
       self.x * other.x + self.y * other.y
   }
   ```

2. **Error Handling**: Use `Result` types for fallible operations

3. **Constants**: Use `SCREAMING_SNAKE_CASE`

   ```rust
   const MAX_VELOCITY: f32 = 1000.0;
   ```

4. **Avoid `unwrap()`**: Use proper error handling or `expect()` with a message

5. **Prefer composition**: Use traits and generics over inheritance-like patterns

## Testing

### Running Tests

Use the test scripts for consistency with CI:

```bash
# Run the full test suite (same as CI)
./scripts/tests/run-all.sh

# Run individual checks
./scripts/tests/check-format.sh   # Formatting
./scripts/tests/check-clippy.sh   # Lints
./scripts/tests/run-unit-tests.sh # Unit tests
./scripts/tests/check-docs.sh     # Documentation
```

Or use cargo directly:

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p gravita-physics

# Run a specific test
cargo test test_circle_collision

# Run tests with output
cargo test -- --nocapture
```

### Continuous Integration

All pull requests are automatically tested via GitHub Actions. The CI runs:

1. **Lint** — Format check and Clippy on Ubuntu
2. **Test** — Unit tests on Ubuntu, macOS, and Windows
3. **Docs** — Documentation build check
4. **Examples** — Build all examples
5. **MSRV** — Minimum Supported Rust Version check (nightly for edition 2024)
6. **Coverage** — Code coverage report via tarpaulin

Ensure your changes pass locally before pushing:

```bash
./scripts/tests/run-all.sh
```

### Writing Tests

1. **Unit tests** go in a `#[cfg(test)]` module in the same file:

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_vector_addition() {
           let a = Vec2::new(1.0, 2.0);
           let b = Vec2::new(3.0, 4.0);
           assert_eq!(a + b, Vec2::new(4.0, 6.0));
       }
   }
   ```

2. **Integration tests** go in a `tests/` directory at the crate root

3. **Use `approx` for floating-point comparisons**:

   ```rust
   use approx::assert_relative_eq;

   #[test]
   fn test_normalize() {
       let v = Vec2::new(3.0, 4.0);
       let n = v.normalize();
       assert_relative_eq!(n.length(), 1.0, epsilon = 1e-6);
   }
   ```

### Test Coverage Goals

- `math` crate: 90%+ coverage
- `physics` crate: 80%+ coverage
- Public APIs: 100% coverage

## Benchmarks

Benchmarks use [Criterion](https://bheisler.github.io/criterion.rs/book/) and live in each crate's `benches/` directory.

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench --workspace

# Run benchmarks for a specific crate
cargo bench -p gravita-math
cargo bench -p gravita-physics

# Filter by benchmark name
cargo bench -p gravita-math -- Vec2
cargo bench -p gravita-physics -- collision

# Using the script
./scripts/tests/run-benchmarks.sh
```

### Benchmark Structure

```
crates/
├── math/
│   └── benches/
│       └── math.rs          # Vec2, AABB, Circle, Ray, Transform
│
└── physics/
    └── benches/
        ├── simulation.rs    # Integrators, world step, body ops
        └── collision.rs     # Narrow/broad phase, scaling tests
```

### Writing Benchmarks

```rust
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn bench_vec2_ops(c: &mut Criterion) {
    c.bench_function("Vec2::dot", |b| {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        b.iter(|| black_box(v1).dot(black_box(v2)))
    });
}

criterion_group!(benches, bench_vec2_ops);
criterion_main!(benches);
```

## Code Coverage

Coverage reports are generated using [cargo-tarpaulin](https://github.com/xd009642/tarpaulin).

### Running Coverage Locally

```bash
# Install tarpaulin (first time only)
cargo install cargo-tarpaulin

# Generate HTML report
./scripts/tests/coverage.sh

# Generate XML for CI
./scripts/tests/coverage.sh --xml

# View the report
open target/coverage/tarpaulin-report.html
```

### Coverage Goals

| Crate | Target |
|-------|--------|
| `gravita-math` | 90%+ |
| `gravita-physics` | 80%+ |
| `gravita-renderer` | 70%+ |
| `gravita-collections` | 70%+ |

## Documentation

### Building Docs

```bash
cargo doc --workspace --no-deps --open
```

### Documentation Guidelines

1. **Crate-level docs**: Add `//!` comments at the top of `lib.rs`

2. **Module docs**: Document what the module provides

3. **Function docs**: Explain what, why, and how with examples

4. **Link related items**: Use `[`backticks`]` to link to other items

   ```rust
   /// See also [`PhysicsWorld::step`] for advancing the simulation.
   ```

## Submitting Changes

### Pull Request Process

1. **Create a feature branch** from `main`

2. **Make your changes** following the guidelines above

3. **Ensure all checks pass**:

   ```bash
   cargo fmt --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo doc --workspace --no-deps
   ```

4. **Push your branch** and create a Pull Request

5. **Fill out the PR template** with:
   - Description of changes
   - Related issues
   - Testing performed
   - Breaking changes (if any)

6. **Address review feedback** promptly

### PR Checklist

- [ ] Code follows the style guidelines
- [ ] Tests added for new functionality
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (for user-facing changes)
- [ ] All CI checks pass

## Architecture Notes

### Crate Dependencies

```
math (no deps)
  ↑
physics (depends on math)
  ↑
renderer (depends on math)
  ↑
collections (depends on math, renderer)
  ↑
engine-core (depends on math, physics, renderer)
  ↑
examples (depend on various crates)
```

### Key Design Decisions

1. **Minimal dependencies**: Core crates (`math`, `physics`) avoid external deps
2. **Pluggable integrators**: Use trait objects for different integration schemes
3. **Separation of concerns**: Physics doesn't know about rendering
4. **Fixed timestep**: Physics uses accumulator pattern for deterministic simulation

## Getting Help

- **Questions**: Open a Discussion on GitHub
- **Bugs**: Open an Issue with reproduction steps
- **Features**: Open an Issue to discuss before implementing

## Releasing

Gravita uses **Independent Versioning** with an **umbrella crate**:

- Each crate (`gravita-math`, `gravita-physics`, etc.) has its own version
- The `gravita` umbrella crate re-exports all others for convenience
- When any child crate changes, the umbrella should also be bumped

This follows [Semantic Versioning](https://semver.org/).

> **📚 For detailed release instructions, see [RELEASING.md](RELEASING.md)**

### Versioning Model

```
┌─────────────────────────────────────────────────────────────────┐
│                    INDEPENDENT VERSIONING                       │
├─────────────────────────────────────────────────────────────────┤
│  gravita-math       v0.1.0  ─┬─→  gravita-physics    v0.1.0    │
│                              │                                  │
│                              └─→  gravita-renderer   v0.1.0    │
│                                           │                     │
│                                           └─→  gravita-         │
│                                                collections      │
│                                                v0.1.0           │
│                                                                 │
│  gravita-input      v0.1.0  (standalone)                       │
│  gravita-assets     v0.1.0  (standalone)                       │
│  gravita-engine-core v0.1.0 (depends on math, physics, renderer)│
└─────────────────────────────────────────────────────────────────┘

When gravita-math bumps to v0.2.0:
  → gravita-physics, gravita-renderer automatically update their
    Cargo.toml to depend on gravita-math = "0.2.0"
  → BUT their versions stay the same (unless they have changes too)
```

### Version Bump Types

| Type | When | Example |
|------|------|---------|
| `patch` | Bug fixes, docs, internal refactors | 0.1.0 → 0.1.1 |
| `minor` | New features (backward-compatible) | 0.1.0 → 0.2.0 |
| `major` | Breaking API changes | 0.1.0 → 1.0.0 |

### Release Flow: Single Crate

When you've made changes to only one crate (e.g., `gravita-math`):

```bash
# 1. Preview the version bump
./scripts/version-bump.sh gravita-math patch --dry-run

# 2. Apply the bump
./scripts/version-bump.sh gravita-math patch

# 3. Update CHANGELOG.md with your changes

# 4. Commit
git add -A
git commit -m "chore(release): gravita-math v0.1.1"

# 5. Release with cargo-release (handles tagging and publishing)
cargo release -p gravita-math --execute
```

### Release Flow: Multiple Crates

When changes span multiple crates:

```bash
# 1. Bump each changed crate individually
./scripts/version-bump.sh gravita-math minor
./scripts/version-bump.sh gravita-physics patch

# 2. Update CHANGELOG.md for each crate

# 3. Commit all changes together
git add -A
git commit -m "chore(release): gravita-math v0.2.0, gravita-physics v0.1.1"

# 4. Release in dependency order
cargo release -p gravita-math --execute
cargo release -p gravita-physics --execute
```

### Release Flow: All Crates (Rare)

For major coordinated releases:

```bash
# Bump all crates
./scripts/version-bump.sh --all minor

# Release entire workspace
cargo release --workspace --execute
```

### Tag Format

Independent releases use crate-specific tags:

| Crate | Tag |
|-------|-----|
| gravita-math v0.2.0 | `gravita-math-v0.2.0` |
| gravita-physics v0.1.1 | `gravita-physics-v0.1.1` |

### Pre-release Validation

Validation runs automatically with `cargo-release`. Run manually with:

```bash
# Validate a specific crate
./scripts/pre-release.sh gravita-math 0.2.0

# Validate all crates
./scripts/pre-release.sh
```

This checks:
- ✅ Code formatting (`cargo fmt --check`)
- ✅ Clippy lints pass
- ✅ All tests pass
- ✅ Documentation builds
- ✅ Benchmarks compile

### Dependency Updates

When you bump a dependency crate, `cargo-release` with `dependent-version = "upgrade"`
automatically updates dependent crates' `Cargo.toml`:

```
Before: gravita-physics depends on gravita-math = "0.1.0"
After bumping gravita-math to 0.2.0:
        gravita-physics now depends on gravita-math = "0.2.0"
```

The dependent crate's **version number stays the same** unless it has its own changes.

### Publishing Order

Always publish in dependency order:

```
1. gravita-math       (no dependencies)
2. gravita-physics    (depends on math)
   gravita-renderer   (depends on math)
3. gravita-collections (depends on math, renderer)
4. gravita-input      (standalone)
   gravita-assets     (standalone)
5. gravita-engine-core (depends on math, physics, renderer)
```

### Checking Current Versions

```bash
# See all crate versions
grep -r "^version = " crates/*/Cargo.toml

# Or use cargo
cargo metadata --format-version=1 | jq '.packages[] | select(.name | startswith("gravita")) | {name, version}'
```

Thank you for contributing! 🚀
