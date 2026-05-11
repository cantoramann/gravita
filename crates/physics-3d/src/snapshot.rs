// physics-3d/src/snapshot.rs
//
// Deterministic snapshot/restore for the 3D `PhysicsWorld`. Mirrors the 2D
// surface in `gravita-physics::snapshot` but encodes 3D state (Vec3 vectors,
// quaternion orientation, Sphere/Aabb/Obb shapes).
//
// Format: little-endian binary with a `GR3D` magic header. No serde
// dependency; the byte layout is hand-rolled and versioned so saved
// snapshots survive minor crate changes.

use gravita_math::{Aabb3, Obb, Quat, Sphere, Vec3};

use crate::{
    body::{BodyType, CollisionShape, RigidBody},
    world::PhysicsWorld,
};

const MAGIC: &[u8; 4] = b"GR3D";
const FORMAT_VERSION: u16 = 1;

const SHAPE_SPHERE: u8 = 0;
const SHAPE_AABB: u8 = 1;
const SHAPE_OBB: u8 = 2;

const BODY_STATIC: u8 = 0;
const BODY_KINEMATIC: u8 = 1;
const BODY_DYNAMIC: u8 = 2;

/// A serialized 3D `PhysicsWorld` state. See [`PhysicsWorld::snapshot`] for the
/// determinism guarantee.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Snapshot {
    bytes: Vec<u8>,
}

impl Snapshot {
    /// Raw bytes of the snapshot.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Take ownership of the underlying byte buffer.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Wrap a previously saved byte buffer. Validation runs lazily on
    /// [`PhysicsWorld::restore_from`].
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

/// Reasons a 3D snapshot might fail to restore.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    /// Byte buffer is shorter than expected.
    Truncated,
    /// 4-byte magic header doesn't match `b"GR3D"`.
    BadMagic,
    /// Format version is newer than this build understands.
    UnsupportedVersion(u16),
    /// A type/shape tag byte was outside the known set.
    BadTag {
        /// Name of the field with the bad tag.
        field: &'static str,
        /// The unrecognized tag value.
        value: u8,
    },
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Truncated => f.write_str("3D snapshot byte buffer is truncated"),
            Self::BadMagic => f.write_str("3D snapshot magic header is not GR3D"),
            Self::UnsupportedVersion(v) => {
                write!(
                    f,
                    "3D snapshot format version {v} not supported by this build"
                )
            },
            Self::BadTag { field, value } => {
                write!(f, "3D snapshot has unknown tag {value} for field {field}")
            },
        }
    }
}

impl std::error::Error for SnapshotError {}

// ───────────────────────────────────────────────────────────────────────────
// Writer / Reader
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
    fn vec3(&mut self, v: Vec3) {
        self.f32(v.x);
        self.f32(v.y);
        self.f32(v.z);
    }
    fn quat(&mut self, q: Quat) {
        self.f32(q.x);
        self.f32(q.y);
        self.f32(q.z);
        self.f32(q.w);
    }
}

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
    fn vec3(&mut self) -> Result<Vec3, SnapshotError> {
        Ok(Vec3::new(self.f32()?, self.f32()?, self.f32()?))
    }
    fn quat(&mut self) -> Result<Quat, SnapshotError> {
        Ok(Quat::from_xyzw(
            self.f32()?,
            self.f32()?,
            self.f32()?,
            self.f32()?,
        ))
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
    w.vec3(b.position);
    w.quat(b.rotation);
    w.vec3(b.velocity);
    w.vec3(b.angular_velocity);
    w.vec3(b.force_accumulator);
    w.vec3(b.torque_accumulator);
    w.f32(b.mass());
    w.f32(b.inv_mass());
    w.vec3(b.inertia);
    w.vec3(b.inv_inertia());
    w.f32(b.restitution);
    w.f32(b.friction);
    w.f32(b.linear_damping);
    w.f32(b.angular_damping);
    w.f32(b.gravity_scale);

    match b.shape {
        CollisionShape::Sphere(s) => {
            w.u8(SHAPE_SPHERE);
            w.vec3(s.center);
            w.f32(s.radius);
        },
        CollisionShape::Aabb(a) => {
            w.u8(SHAPE_AABB);
            w.vec3(a.min);
            w.vec3(a.max);
        },
        CollisionShape::Obb(o) => {
            w.u8(SHAPE_OBB);
            w.vec3(o.center);
            w.vec3(o.half_extents);
            w.quat(o.rotation);
        },
    }

    w.u8(u8::from(b.is_sensor));
    w.u8(u8::from(b.enabled));
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
    let position = r.vec3()?;
    let rotation = r.quat()?;
    let velocity = r.vec3()?;
    let angular_velocity = r.vec3()?;
    let force_accumulator = r.vec3()?;
    let torque_accumulator = r.vec3()?;
    let mass = r.f32()?;
    let inv_mass = r.f32()?;
    let inertia = r.vec3()?;
    let inv_inertia = r.vec3()?;
    let restitution = r.f32()?;
    let friction = r.f32()?;
    let linear_damping = r.f32()?;
    let angular_damping = r.f32()?;
    let gravity_scale = r.f32()?;

    let shape = match r.u8()? {
        SHAPE_SPHERE => CollisionShape::Sphere(Sphere::new(r.vec3()?, r.f32()?)),
        SHAPE_AABB => CollisionShape::Aabb(Aabb3 {
            min: r.vec3()?,
            max: r.vec3()?,
        }),
        SHAPE_OBB => CollisionShape::Obb(Obb {
            center: r.vec3()?,
            half_extents: r.vec3()?,
            rotation: r.quat()?,
        }),
        v => {
            return Err(SnapshotError::BadTag {
                field: "shape",
                value: v,
            });
        },
    };

    let is_sensor = r.u8()? != 0;
    let enabled = r.u8()? != 0;

    Ok(RigidBody {
        id,
        body_type,
        position,
        rotation,
        velocity,
        angular_velocity,
        force_accumulator,
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
    })
}

// ───────────────────────────────────────────────────────────────────────────
// PhysicsWorld snapshot/restore
// ───────────────────────────────────────────────────────────────────────────

impl PhysicsWorld {
    /// Serialize the world state into an opaque byte buffer.
    ///
    /// Bit-stable across runs of the same compiled binary, so the bytes are
    /// safe to save to disk for replays or to round-trip through unit tests.
    /// The integrator trait object is NOT captured — bring your own when
    /// restoring.
    pub fn snapshot(&self) -> Snapshot {
        let mut w = Writer::new();
        w.buf.extend_from_slice(MAGIC);
        w.u16(FORMAT_VERSION);
        w.vec3(self.gravity());
        w.u32(self.velocity_iterations as u32);
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
    /// Replaces all bodies, gravity, and solver tunables. Returns
    /// [`SnapshotError`] on malformed input.
    ///
    /// # Errors
    ///
    /// See [`SnapshotError`] for the failure modes.
    pub fn restore_from(&mut self, snapshot: &Snapshot) -> Result<(), SnapshotError> {
        let mut r = Reader::new(&snapshot.bytes);

        if r.take(4)? != MAGIC {
            return Err(SnapshotError::BadMagic);
        }
        let version = r.u16()?;
        if version != FORMAT_VERSION {
            return Err(SnapshotError::UnsupportedVersion(version));
        }

        let gravity = r.vec3()?;
        let velocity_iterations = r.u32()? as usize;
        let position_correction = r.f32()?;
        let sleep_threshold = r.f32()?;
        let body_count = r.u32()? as usize;

        let mut bodies = Vec::with_capacity(body_count);
        for _ in 0..body_count {
            bodies.push(read_body(&mut r)?);
        }

        self.set_gravity(gravity);
        self.velocity_iterations = velocity_iterations;
        self.position_correction = position_correction;
        self.sleep_threshold = sleep_threshold;
        self.replace_bodies(bodies);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use gravita_math::{Sphere, Vec3};

    use super::*;
    use crate::body::{BodyType, CollisionShape, RigidBody};

    fn sphere(r: f32) -> CollisionShape {
        CollisionShape::Sphere(Sphere::new(Vec3::ZERO, r))
    }

    fn boxshape(w: f32, h: f32, d: f32) -> CollisionShape {
        CollisionShape::Aabb(Aabb3::from_center_size(Vec3::ZERO, Vec3::new(w, h, d)))
    }

    fn busy_scene() -> PhysicsWorld {
        let mut world = PhysicsWorld::new();
        world.set_gravity(Vec3::new(0.0, -9.81, 0.0));
        world.add_body(
            RigidBody::new(0, boxshape(20.0, 1.0, 20.0))
                .with_type(BodyType::Static)
                .with_position(Vec3::new(0.0, -0.5, 0.0)),
        );
        for i in 0..8 {
            let x = (i as f32 - 4.0) * 0.6;
            world.add_body(
                RigidBody::new(0, sphere(0.5))
                    .with_position(Vec3::new(x, (i as f32).mul_add(0.2, 3.0), 0.0))
                    .with_density(1.0)
                    .with_restitution(0.5)
                    .with_friction(0.2),
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
        assert_eq!(
            snap.as_bytes(),
            re.as_bytes(),
            "3D snapshot must round-trip"
        );
    }

    #[test]
    fn same_initial_state_steps_identically() {
        let mut a = busy_scene();
        let mut b = busy_scene();
        assert_eq!(a.snapshot().as_bytes(), b.snapshot().as_bytes());

        drive(&mut a, 200, 1.0 / 60.0);
        drive(&mut b, 200, 1.0 / 60.0);

        assert_eq!(a.snapshot().as_bytes(), b.snapshot().as_bytes());
    }

    #[test]
    fn restore_continues_identical_trajectory() {
        let mut original = busy_scene();
        drive(&mut original, 30, 1.0 / 60.0);
        let mid_snap = original.snapshot();
        drive(&mut original, 30, 1.0 / 60.0);
        let final_original = original.snapshot();

        let mut rewound = PhysicsWorld::new();
        rewound.restore_from(&mid_snap).unwrap();
        drive(&mut rewound, 30, 1.0 / 60.0);
        let final_rewound = rewound.snapshot();

        assert_eq!(final_original.as_bytes(), final_rewound.as_bytes());
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
    fn obb_round_trip() {
        let mut world = PhysicsWorld::new();
        world.add_body(
            RigidBody::new(
                0,
                CollisionShape::Obb(Obb {
                    center: Vec3::new(1.0, 2.0, 3.0),
                    half_extents: Vec3::new(0.5, 1.0, 1.5),
                    rotation: Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.7),
                }),
            )
            .with_position(Vec3::new(5.0, 5.0, 5.0)),
        );

        let snap = world.snapshot();
        let mut other = PhysicsWorld::new();
        other.restore_from(&snap).unwrap();

        let body = &other.bodies()[0];
        match body.shape {
            CollisionShape::Obb(o) => {
                assert_eq!(o.center, Vec3::new(1.0, 2.0, 3.0));
                assert_eq!(o.half_extents, Vec3::new(0.5, 1.0, 1.5));
            },
            _ => panic!("expected OBB shape after restore"),
        }
    }
}
