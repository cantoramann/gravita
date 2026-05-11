// physics/src/snapshot.rs
//
// Deterministic snapshot/restore for `PhysicsWorld`.
//
// Bit-for-bit serialization of the simulation state — every dynamics field,
// every body, every contact-relevant tunable. Round-tripping a snapshot must
// produce a world that, when stepped with the same `dt`, yields exactly the
// same trajectory as a world that was never snapshotted.
//
// The byte layout is a hand-rolled little-endian binary format (versioned
// `GR2D` magic). No serde dependency, no encoding overhead.
//
// ## Determinism notes
//
// The step path is deterministic on a *fixed binary*: no `HashMap` iteration,
// no parallel solver, no system randomness. Across machines or compiler
// versions, FMA hardware (`f32::mul_add`) and float-store-precision quirks
// may differ; ship the same compiled artifact to all clients if you need
// lockstep across machines.
//
// ## What's NOT in a snapshot
//
// The `integrator` and `collision_detector` trait objects are not serialized.
// Reconstruct the world with the same dynamic dispatch choices *before*
// calling `restore_from`.

use gravita_math::{Aabb, Circle, Vec2};

use crate::{
    body::{BodyType, CollisionShape, RigidBody},
    world::PhysicsWorld,
};

const MAGIC: &[u8; 4] = b"GR2D";
const FORMAT_VERSION: u16 = 1;

const SHAPE_CIRCLE: u8 = 0;
const SHAPE_AABB: u8 = 1;

const BODY_STATIC: u8 = 0;
const BODY_KINEMATIC: u8 = 1;
const BODY_DYNAMIC: u8 = 2;

/// A serialized `PhysicsWorld` state.
///
/// Wraps an owned byte buffer. Pass to [`PhysicsWorld::restore_from`] to
/// rewind a world to this point, or save the bytes to disk for replays.
///
/// ```
/// # use gravita_math::{Circle, Vec2};
/// # use gravita_physics::{CollisionShape, PhysicsWorld, RigidBody};
/// let mut world = PhysicsWorld::new();
/// world.add_body(RigidBody::new(
///     0,
///     CollisionShape::Circle(Circle::new(Vec2::ZERO, 5.0)),
/// ));
/// for _ in 0..30 {
///     world.step(1.0 / 60.0);
/// }
///
/// let snapshot = world.snapshot();
/// for _ in 0..30 {
///     world.step(1.0 / 60.0);
/// }
/// world.restore_from(&snapshot).unwrap();
/// assert_eq!(world.bodies().len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Snapshot {
    bytes: Vec<u8>,
}

impl Snapshot {
    /// Raw bytes of the snapshot. Stable across runs of the same binary.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Take ownership of the underlying byte buffer.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Wrap a previously saved byte buffer. No validation runs until
    /// [`PhysicsWorld::restore_from`] is called.
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

/// Reasons a snapshot might fail to restore.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    /// The byte buffer is shorter than expected.
    Truncated,
    /// The 4-byte magic header doesn't match `b"GR2D"`.
    BadMagic,
    /// The format version is newer than this build understands.
    UnsupportedVersion(u16),
    /// A type/shape tag byte was outside the known set.
    BadTag {
        /// Name of the field with the bad tag (for debugging).
        field: &'static str,
        /// The unrecognized tag value.
        value: u8,
    },
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Truncated => f.write_str("snapshot byte buffer is truncated"),
            Self::BadMagic => f.write_str("snapshot magic header is not GR2D"),
            Self::UnsupportedVersion(v) => {
                write!(f, "snapshot format version {v} not supported by this build")
            },
            Self::BadTag { field, value } => {
                write!(f, "snapshot has unknown tag {value} for field {field}")
            },
        }
    }
}

impl std::error::Error for SnapshotError {}

// ───────────────────────────────────────────────────────────────────────────
// Writer
// ───────────────────────────────────────────────────────────────────────────

struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    fn new() -> Self {
        Self { buf: Vec::new() }
    }
    fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn f32(&mut self, v: f32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }
    fn vec2(&mut self, v: Vec2) {
        self.f32(v.x);
        self.f32(v.y);
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Reader
// ───────────────────────────────────────────────────────────────────────────

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], SnapshotError> {
        if self.pos + n > self.buf.len() {
            return Err(SnapshotError::Truncated);
        }
        let s = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, SnapshotError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, SnapshotError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, SnapshotError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, SnapshotError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32, SnapshotError> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn vec2(&mut self) -> Result<Vec2, SnapshotError> {
        let x = self.f32()?;
        let y = self.f32()?;
        Ok(Vec2::new(x, y))
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Body encoding
// ───────────────────────────────────────────────────────────────────────────

fn write_body(w: &mut Writer, b: &RigidBody) {
    w.u64(b.id as u64);
    w.u8(match b.body_type() {
        BodyType::Static => BODY_STATIC,
        BodyType::Kinematic => BODY_KINEMATIC,
        BodyType::Dynamic => BODY_DYNAMIC,
    });
    w.vec2(b.position);
    w.f32(b.rotation);
    w.vec2(b.velocity);
    w.vec2(b.acceleration());
    w.vec2(b.force_accumulator());
    w.f32(b.angular_velocity);
    w.f32(b.angular_acceleration());
    w.f32(b.torque_accumulator());
    w.f32(b.mass());
    w.f32(b.inv_mass());
    w.f32(b.inertia());
    w.f32(b.inv_inertia());
    w.f32(b.restitution);
    w.f32(b.friction);
    w.f32(b.linear_damping);
    w.f32(b.angular_damping);
    w.f32(b.gravity_scale);

    match b.shape {
        CollisionShape::Circle(c) => {
            w.u8(SHAPE_CIRCLE);
            w.vec2(c.center);
            w.f32(c.radius);
        },
        CollisionShape::Aabb(aabb) => {
            w.u8(SHAPE_AABB);
            w.vec2(aabb.min);
            w.vec2(aabb.max);
        },
    }

    w.u8(u8::from(b.is_sensor));
    w.u8(u8::from(b.enabled));
    w.u8(u8::from(b.fixed_rotation()));
}

fn read_body(r: &mut Reader<'_>) -> Result<RigidBody, SnapshotError> {
    let id = r.u64()? as usize;
    let body_type = match r.u8()? {
        BODY_STATIC => BodyType::Static,
        BODY_KINEMATIC => BodyType::Kinematic,
        BODY_DYNAMIC => BodyType::Dynamic,
        v => {
            return Err(SnapshotError::BadTag {
                field: "body_type",
                value: v,
            });
        },
    };
    let position = r.vec2()?;
    let rotation = r.f32()?;
    let velocity = r.vec2()?;
    let acceleration = r.vec2()?;
    let force_accumulator = r.vec2()?;
    let angular_velocity = r.f32()?;
    let angular_acceleration = r.f32()?;
    let torque_accumulator = r.f32()?;
    let mass = r.f32()?;
    let inv_mass = r.f32()?;
    let inertia = r.f32()?;
    let inv_inertia = r.f32()?;
    let restitution = r.f32()?;
    let friction = r.f32()?;
    let linear_damping = r.f32()?;
    let angular_damping = r.f32()?;
    let gravity_scale = r.f32()?;

    let shape = match r.u8()? {
        SHAPE_CIRCLE => CollisionShape::Circle(Circle::new(r.vec2()?, r.f32()?)),
        SHAPE_AABB => {
            let min = r.vec2()?;
            let max = r.vec2()?;
            CollisionShape::Aabb(Aabb { min, max })
        },
        v => {
            return Err(SnapshotError::BadTag {
                field: "shape",
                value: v,
            });
        },
    };

    let is_sensor = r.u8()? != 0;
    let enabled = r.u8()? != 0;
    let fixed_rotation = r.u8()? != 0;

    // Bypass `new` (which would zero accumulators and recompute inv_mass);
    // we want a bit-exact restore.
    Ok(RigidBody {
        id,
        body_type,
        position,
        rotation,
        velocity,
        acceleration,
        force_accumulator,
        angular_velocity,
        angular_acceleration,
        torque_accumulator,
        mass,
        inv_mass,
        inertia,
        inv_inertia,
        restitution,
        friction,
        linear_damping,
        angular_damping,
        gravity_scale,
        shape,
        is_sensor,
        enabled,
        fixed_rotation,
    })
}

// ───────────────────────────────────────────────────────────────────────────
// PhysicsWorld snapshot/restore
// ───────────────────────────────────────────────────────────────────────────

impl PhysicsWorld {
    /// Serialize the world state into an opaque byte buffer.
    ///
    /// Cheap — copies every body's fields into a flat `Vec<u8>` with no
    /// compression or encoding overhead. The result is bit-stable across runs
    /// of the same binary, so saving snapshots to disk for replays or running
    /// a determinism CI test against past snapshots both work.
    ///
    /// The integrator and collision-detector trait objects are NOT captured;
    /// the caller is responsible for reconstructing the world with the same
    /// choices before [`restore_from`](Self::restore_from).
    pub fn snapshot(&self) -> Snapshot {
        let mut w = Writer::new();
        w.buf.extend_from_slice(MAGIC);
        w.u16(FORMAT_VERSION);
        w.vec2(self.gravity());
        w.u32(self.velocity_iterations as u32);
        w.u32(self.position_iterations as u32);
        w.f32(self.position_correction);
        w.f32(self.sleep_threshold);
        w.u32(self.bodies().len() as u32);
        for body in self.bodies() {
            write_body(&mut w, body);
        }
        Snapshot { bytes: w.buf }
    }

    /// Restore the world to the state captured by `snapshot`.
    ///
    /// Replaces all bodies, gravity, and solver tunables. The integrator and
    /// collision-detector instances are left untouched — bring your own.
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotError`] when the byte buffer is malformed, truncated,
    /// or written by a format version this build doesn't understand.
    pub fn restore_from(&mut self, snapshot: &Snapshot) -> Result<(), SnapshotError> {
        let mut r = Reader::new(&snapshot.bytes);

        let magic = r.take(4)?;
        if magic != MAGIC {
            return Err(SnapshotError::BadMagic);
        }
        let version = r.u16()?;
        if version != FORMAT_VERSION {
            return Err(SnapshotError::UnsupportedVersion(version));
        }

        let gravity = r.vec2()?;
        let velocity_iterations = r.u32()? as usize;
        let position_iterations = r.u32()? as usize;
        let position_correction = r.f32()?;
        let sleep_threshold = r.f32()?;
        let body_count = r.u32()? as usize;

        let mut bodies = Vec::with_capacity(body_count);
        for _ in 0..body_count {
            bodies.push(read_body(&mut r)?);
        }

        // Atomic swap-in: only mutate `self` once every byte parsed cleanly.
        self.set_gravity(gravity);
        self.velocity_iterations = velocity_iterations;
        self.position_iterations = position_iterations;
        self.position_correction = position_correction;
        self.sleep_threshold = sleep_threshold;
        self.replace_bodies(bodies);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use gravita_math::{Aabb, Circle, Vec2};

    use super::*;
    use crate::body::{BodyType, CollisionShape, RigidBody};

    fn circle(r: f32) -> CollisionShape {
        CollisionShape::Circle(Circle::new(Vec2::ZERO, r))
    }
    fn boxshape(w: f32, h: f32) -> CollisionShape {
        CollisionShape::Aabb(Aabb::from_center_size(Vec2::ZERO, Vec2::new(w, h)))
    }

    fn busy_scene() -> PhysicsWorld {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec2::new(0.0, -500.0));
        world.add_body(
            RigidBody::new(0, boxshape(800.0, 50.0))
                .with_type(BodyType::Static)
                .with_position(Vec2::new(0.0, -300.0)),
        );
        for i in 0..12 {
            let x = (i as f32 - 6.0) * 30.0;
            world.add_body(
                RigidBody::new(0, circle(10.0))
                    .with_position(Vec2::new(x, (i as f32).mul_add(5.0, 200.0)))
                    .with_density(1.0)
                    .with_restitution(0.6)
                    .with_friction(0.3),
            );
        }
        world
    }

    fn drive(world: &mut PhysicsWorld, steps: usize, dt: f32) {
        for _ in 0..steps {
            world.step(dt);
        }
    }

    #[test]
    fn round_trip_preserves_state_exactly() {
        let mut world = busy_scene();
        drive(&mut world, 20, 1.0 / 60.0);

        let snap = world.snapshot();
        let mut other = PhysicsWorld::new();
        other.restore_from(&snap).unwrap();

        let re = other.snapshot();
        assert_eq!(snap.as_bytes(), re.as_bytes(), "snapshot must round-trip");
    }

    #[test]
    fn same_initial_state_steps_identically() {
        // Bit-for-bit determinism: same world, stepped twice, snapshots match.
        let mut a = busy_scene();
        let mut b = busy_scene();
        let snap_initial_a = a.snapshot();
        let snap_initial_b = b.snapshot();
        assert_eq!(snap_initial_a.as_bytes(), snap_initial_b.as_bytes());

        drive(&mut a, 200, 1.0 / 60.0);
        drive(&mut b, 200, 1.0 / 60.0);

        let snap_a = a.snapshot();
        let snap_b = b.snapshot();
        assert_eq!(snap_a.as_bytes(), snap_b.as_bytes());
    }

    #[test]
    fn restore_continues_identical_trajectory() {
        // Step a world to T=50; snapshot. Restore into a fresh world and step
        // to T=100. Trajectories must agree with a never-snapshotted run.
        let mut original = busy_scene();
        drive(&mut original, 50, 1.0 / 60.0);
        let mid_snap = original.snapshot();
        drive(&mut original, 50, 1.0 / 60.0);
        let final_original = original.snapshot();

        let mut rewound = PhysicsWorld::new();
        rewound.restore_from(&mid_snap).unwrap();
        drive(&mut rewound, 50, 1.0 / 60.0);
        let final_rewound = rewound.snapshot();

        assert_eq!(
            final_original.as_bytes(),
            final_rewound.as_bytes(),
            "post-restore trajectory must match"
        );
    }

    #[test]
    fn rejects_bad_magic() {
        let bad = Snapshot::from_bytes(b"XXXX\x01\x00".to_vec());
        let mut world = PhysicsWorld::new();
        assert_eq!(world.restore_from(&bad), Err(SnapshotError::BadMagic));
    }

    #[test]
    fn rejects_truncated_buffer() {
        let mut bytes = busy_scene().snapshot().into_bytes();
        bytes.truncate(bytes.len() - 1);
        let snap = Snapshot::from_bytes(bytes);
        let mut world = PhysicsWorld::new();
        assert_eq!(world.restore_from(&snap), Err(SnapshotError::Truncated));
    }

    #[test]
    fn rejects_unknown_version() {
        let mut bytes = busy_scene().snapshot().into_bytes();
        bytes[4..6].copy_from_slice(&u16::MAX.to_le_bytes());
        let snap = Snapshot::from_bytes(bytes);
        let mut world = PhysicsWorld::new();
        assert!(matches!(
            world.restore_from(&snap),
            Err(SnapshotError::UnsupportedVersion(_))
        ));
    }

    #[test]
    fn rejects_unknown_shape_tag() {
        // Corrupt the shape-tag byte for body 0. Offset = 34-byte header +
        // 93 bytes of body fields before the shape tag (id + body_type +
        // position + rotation + velocity + accel + force + angular_velocity
        // + angular_accel + torque + 4×mass-related f32s + 5×material f32s).
        // Any reorder of `write_body` invalidates this offset.
        let mut world = PhysicsWorld::new();
        world.add_body(RigidBody::new(0, circle(5.0)));
        let mut bytes = world.snapshot().into_bytes();
        bytes[34 + 93] = 99;
        let snap = Snapshot::from_bytes(bytes);
        let mut world = PhysicsWorld::new();
        assert!(matches!(
            world.restore_from(&snap),
            Err(SnapshotError::BadTag { field: "shape", .. })
        ));
    }
}
