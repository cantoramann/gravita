# gravita-wasm

JavaScript-friendly WebAssembly bindings for the Gravita physics engine.

Bindings are **handwritten** — every method is shaped for JS readers (plain numeric arguments, `Float32Array` returns), not the verbose builder pattern an auto-generated `wasm-bindgen` surface would produce.

## Build

```bash
# One-time:
rustup target add wasm32-unknown-unknown
cargo install wasm-pack

# Build the JS package:
wasm-pack build crates/wasm --target web --release
```

You'll get `crates/wasm/pkg/` containing the `.wasm` binary, a JS loader, and TypeScript `.d.ts` definitions.

## Quick start (2D)

```html
<script type="module">
import init, { World2D, BodyKind } from "./pkg/gravita_wasm.js";

await init();

const world = new World2D(0, -500);

// Static floor.
const floor = world.addBox(0, 0, 800, 50);
world.setBodyKind(floor, BodyKind.Static);

// A bouncy ball.
const ball = world.addCircle(0, 300, 20);
world.setBodyRestitution(ball, 0.8);

for (let i = 0; i < 600; i++) world.step(1 / 60);

const pos = world.bodyPosition(ball);   // Float32Array [x, y]
console.log("Ball ended at", pos[0], pos[1]);
</script>
```

## Quick start (3D)

```js
import init, { World3D } from "./pkg/gravita_wasm.js";

await init();

const world = new World3D(0, -9.81, 0);
const ball = world.addSphere(0, 5, 0, 0.5);
world.setBodyRestitution(ball, 0.6);

for (let i = 0; i < 120; i++) world.step(1 / 60);

console.log(world.bodyPosition(ball)); // Float32Array [x, y, z]
console.log(world.bodyRotation(ball)); // Float32Array [qx, qy, qz, qw]
```

## API surface

### `World2D`

- `new World2D(gravity_x, gravity_y)`
- `setGravity(gx, gy)`
- `addCircle(x, y, radius)` → `id`
- `addBox(x, y, width, height)` → `id`
- `setBodyKind(id, kind)` — `kind` is one of `BodyKind.Dynamic`, `BodyKind.Kinematic`, `BodyKind.Static`
- `setBodyVelocity(id, vx, vy)`
- `setBodyRestitution(id, restitution)`
- `setBodyFriction(id, friction)`
- `step(dt)`
- `bodyPosition(id)` → `Float32Array [x, y]`
- `bodyVelocity(id)` → `Float32Array [vx, vy]`
- `bodyRotation(id)` → `number` (radians)
- `disableBody(id)` / `enableBody(id)`
- `bodyCount()` → `number`
- `allPositions()` → `Float32Array [x0, y0, x1, y1, …]` — one bridge call to read every body's position
- `snapshot()` → `Uint8Array` — full simulation state. Bit-stable across runs of the same binary.
- `restoreFrom(bytes)` — restore from a previously captured `Uint8Array`. Throws on malformed input.

### `World3D`

Same surface as `World2D` with `[x, y, z]` vectors. Differences:

- `new World3D(gx, gy, gz)` — three-component gravity
- `addSphere(x, y, z, radius)` and `addBox(x, y, z, w, h, d)`
- `bodyRotation(id)` returns `Float32Array [qx, qy, qz, qw]` (unit quaternion)
- `allPositions()` returns `[x0, y0, z0, x1, y1, z1, …]`
- `snapshot()` / `restoreFrom(bytes)` work identically to the 2D surface — the bytes are tagged `GR3D` and not interchangeable with 2D snapshots.

## Why handwritten bindings?

Auto-generated `wasm-bindgen` exports of Gravita's internal Rust types would force JS users to write Rust-shaped code in JavaScript:

```js
// What auto-generated bindings would force you to write:
const desc = new RigidBodyDesc();
desc.with_position(new Vec2(0, 100));
desc.with_radius(10);
const body = world.create_rigid_body(desc);
```

The handwritten layer here flattens that to:

```js
const body = world.addCircle(0, 100, 10);
```

Same engine underneath; the ergonomic difference is real.

## License

MIT — see [../../LICENSE](../../LICENSE).
