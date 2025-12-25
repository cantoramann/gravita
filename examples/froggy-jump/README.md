# 🐸 Froggy Jump

A minimal Doodle Jump-style game built with Gravita, running in the browser via WebAssembly.

## Play

### Quick Start (with wasm-pack)

```bash
# Install wasm-pack if you haven't
cargo install wasm-pack

# Build the WASM module
cd examples/froggy-jump
wasm-pack build --target web

# Serve the files
python3 -m http.server 8080
# Or: npx serve .

# Open in browser
open http://localhost:8080
```

### Alternative (with cargo)

```bash
# Install wasm32 target
rustup target add wasm32-unknown-unknown

# Build
cargo build --target wasm32-unknown-unknown --release -p froggy-jump

# You'll need to use wasm-bindgen-cli to generate JS bindings
```

## Controls

| Key | Action |
|-----|--------|
| ← / A | Move left |
| → / D | Move right |
| Space | Restart (after game over) |

## Features

- 🎮 Simple one-button gameplay
- 🌟 Procedurally generated platforms
- 📊 Score tracking with high score
- 🎨 Cute blob character
- 🌙 Space theme with stars
- 📱 Responsive canvas

## How It Works

The game uses:
- `gravita-math` for `Vec2` operations
- Pure Canvas 2D rendering via `web-sys`
- `wasm-bindgen` for JavaScript interop
- `requestAnimationFrame` for smooth 60fps gameplay

## Code Structure

```
froggy-jump/
├── Cargo.toml      # WASM dependencies
├── index.html      # Entry point
├── README.md       # This file
└── src/
    └── lib.rs      # Game logic + WASM bindings
```

## Building for Production

```bash
wasm-pack build --target web --release

# The output will be in pkg/
# - froggy_jump_bg.wasm (game binary)
# - froggy_jump.js (JS bindings)
```

## Browser Compatibility

Works in all modern browsers with WebAssembly support:
- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

