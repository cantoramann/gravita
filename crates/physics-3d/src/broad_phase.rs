// physics-3d/src/broad_phase.rs
//
// 3D analogue of the 2D `SpatialHashGrid`. Storage is a single sorted
// `Vec<(u64, u32)>` where the u64 packs three i32 cell coordinates (21 bits
// each, biased to map negative coordinates to the positive range). Integer
// compares only — no SipHash, no per-step `HashMap` allocations.

use gravita_math::{Aabb3, Vec3};

use crate::body::RigidBody;

/// Interface for 3D broad-phase strategies.
pub trait BroadPhase {
    /// Update with the current body positions.
    fn update(&mut self, bodies: &[RigidBody]);
    /// Append body indices that may overlap `aabb` into `out`. Output is
    /// deduplicated.
    fn query(&self, aabb: &Aabb3, out: &mut Vec<usize>);
    /// Append potentially colliding pairs into `out`. Output is deduplicated:
    /// each unordered pair appears at most once.
    fn get_potential_pairs(&mut self, out: &mut Vec<(usize, usize)>);
    /// Drop all spatial data.
    fn clear(&mut self);
}

/// 21 bits per axis = ±1,048,576 cells = ±67 km at 64 m/cell. Plenty.
const AXIS_BITS: u32 = 21;
const AXIS_MASK: u64 = (1 << AXIS_BITS) - 1;
const AXIS_BIAS: i64 = 1 << (AXIS_BITS - 1);

#[inline]
fn pack_cell(x: i32, y: i32, z: i32) -> u64 {
    let xb = (((x as i64) + AXIS_BIAS) as u64) & AXIS_MASK;
    let yb = (((y as i64) + AXIS_BIAS) as u64) & AXIS_MASK;
    let zb = (((z as i64) + AXIS_BIAS) as u64) & AXIS_MASK;
    (xb << (AXIS_BITS * 2)) | (yb << AXIS_BITS) | zb
}

/// 3D spatial hash grid.
pub struct SpatialHashGrid {
    cell_size: f32,
    entries: Vec<(u64, u32)>,
    pair_scratch: Vec<u64>,
}

impl SpatialHashGrid {
    /// Build with the given world-units cell size.
    #[must_use]
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            entries: Vec::new(),
            pair_scratch: Vec::new(),
        }
    }

    #[inline]
    fn cell_of(&self, p: Vec3) -> (i32, i32, i32) {
        (
            (p.x / self.cell_size).floor() as i32,
            (p.y / self.cell_size).floor() as i32,
            (p.z / self.cell_size).floor() as i32,
        )
    }

    /// Push `(packed_cell, body_idx)` for every cell the AABB touches.
    fn push_cells_for_aabb(&mut self, aabb: &Aabb3, body_idx: u32) {
        let (min_x, min_y, min_z) = self.cell_of(aabb.min);
        let (max_x, max_y, max_z) = self.cell_of(aabb.max);
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    self.entries.push((pack_cell(x, y, z), body_idx));
                }
            }
        }
    }
}

impl Default for SpatialHashGrid {
    fn default() -> Self {
        Self::new(2.0)
    }
}

impl BroadPhase for SpatialHashGrid {
    fn update(&mut self, bodies: &[RigidBody]) {
        self.entries.clear();
        for (idx, body) in bodies.iter().enumerate() {
            let aabb = body.world_aabb();
            self.push_cells_for_aabb(&aabb, idx as u32);
        }
        self.entries.sort_unstable_by_key(|&(cell, _)| cell);
    }

    fn query(&self, aabb: &Aabb3, out: &mut Vec<usize>) {
        let (min_x, min_y, min_z) = self.cell_of(aabb.min);
        let (max_x, max_y, max_z) = self.cell_of(aabb.max);

        let start = out.len();
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let key = pack_cell(x, y, z);
                    let lo = self.entries.partition_point(|&(k, _)| k < key);
                    for &(k, idx) in &self.entries[lo..] {
                        if k != key {
                            break;
                        }
                        out.push(idx as usize);
                    }
                }
            }
        }

        // Dedup the freshly-appended slice. A body straddling multiple queried
        // cells lands in the output once per cell; users want it once total.
        let added = &mut out[start..];
        added.sort_unstable();
        let mut write = 0usize;
        for read in 0..added.len() {
            if read == 0 || added[read] != added[read - 1] {
                added[write] = added[read];
                write += 1;
            }
        }
        out.truncate(start + write);
    }

    fn get_potential_pairs(&mut self, out: &mut Vec<(usize, usize)>) {
        let scratch = &mut self.pair_scratch;
        scratch.clear();

        let mut i = 0;
        while i < self.entries.len() {
            let cell = self.entries[i].0;
            let mut j = i + 1;
            while j < self.entries.len() && self.entries[j].0 == cell {
                j += 1;
            }
            let run = &self.entries[i..j];
            for a in 0..run.len() {
                for b in (a + 1)..run.len() {
                    let mut ai = run[a].1;
                    let mut bi = run[b].1;
                    if ai > bi {
                        std::mem::swap(&mut ai, &mut bi);
                    }
                    scratch.push((u64::from(ai) << 32) | u64::from(bi));
                }
            }
            i = j;
        }

        scratch.sort_unstable();
        scratch.dedup();
        out.reserve(scratch.len());
        for &packed in scratch.iter() {
            let bi = (packed & 0xffff_ffff) as u32;
            let ai = (packed >> 32) as u32;
            out.push((ai as usize, bi as usize));
        }
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use gravita_math::Sphere;

    use super::*;
    use crate::body::CollisionShape;

    fn sphere_body(id: usize, position: Vec3, radius: f32) -> RigidBody {
        RigidBody::new(id, CollisionShape::Sphere(Sphere::new(Vec3::ZERO, radius)))
            .with_position(position)
    }

    fn normalize_pair(p: (usize, usize)) -> (usize, usize) {
        if p.0 < p.1 { p } else { (p.1, p.0) }
    }

    #[test]
    fn pack_cell_distinct_for_distinct_coords() {
        let keys = [
            pack_cell(0, 0, 0),
            pack_cell(1, 0, 0),
            pack_cell(0, 1, 0),
            pack_cell(0, 0, 1),
            pack_cell(-1, -1, -1),
            pack_cell(-1000, 1000, 12345),
        ];
        let set: HashSet<_> = keys.iter().copied().collect();
        assert_eq!(set.len(), keys.len());
    }

    #[test]
    fn empty_grid_yields_no_pairs() {
        let mut grid = SpatialHashGrid::new(2.0);
        grid.update(&[]);
        let mut pairs = Vec::new();
        grid.get_potential_pairs(&mut pairs);
        assert!(pairs.is_empty());
    }

    #[test]
    fn far_apart_bodies_yield_no_pairs() {
        let bodies = vec![
            sphere_body(0, Vec3::ZERO, 0.5),
            sphere_body(1, Vec3::splat(100.0), 0.5),
        ];
        let mut grid = SpatialHashGrid::new(2.0);
        grid.update(&bodies);
        let mut pairs = Vec::new();
        grid.get_potential_pairs(&mut pairs);
        assert!(pairs.is_empty());
    }

    #[test]
    fn overlapping_bodies_yield_pair() {
        let bodies = vec![
            sphere_body(0, Vec3::ZERO, 1.0),
            sphere_body(1, Vec3::new(0.5, 0.0, 0.0), 1.0),
        ];
        let mut grid = SpatialHashGrid::new(2.0);
        grid.update(&bodies);
        let mut pairs = Vec::new();
        grid.get_potential_pairs(&mut pairs);
        assert!(pairs.into_iter().map(normalize_pair).any(|p| p == (0, 1)));
    }

    #[test]
    fn potential_pairs_deduplicated_across_cells() {
        // Both spheres straddle several cells.
        let bodies = vec![
            sphere_body(0, Vec3::new(0.5, 0.5, 0.5), 3.0),
            sphere_body(1, Vec3::new(1.0, 1.0, 1.0), 3.0),
        ];
        let mut grid = SpatialHashGrid::new(2.0);
        grid.update(&bodies);
        let mut pairs = Vec::new();
        grid.get_potential_pairs(&mut pairs);
        let unique: HashSet<_> = pairs.iter().map(|&p| normalize_pair(p)).collect();
        assert_eq!(unique.len(), pairs.len());
        assert!(unique.contains(&(0, 1)));
    }

    #[test]
    fn pairs_within_same_cell_emit_all_combinations() {
        // 3 bodies in the same cell → C(3,2) = 3 pairs.
        let bodies = vec![
            sphere_body(0, Vec3::splat(1.0), 0.3),
            sphere_body(1, Vec3::splat(1.1), 0.3),
            sphere_body(2, Vec3::splat(0.9), 0.3),
        ];
        let mut grid = SpatialHashGrid::new(10.0);
        grid.update(&bodies);
        let mut pairs = Vec::new();
        grid.get_potential_pairs(&mut pairs);
        let unique: HashSet<_> = pairs.iter().map(|&p| normalize_pair(p)).collect();
        assert_eq!(unique.len(), 3);
        assert!(unique.contains(&(0, 1)));
        assert!(unique.contains(&(0, 2)));
        assert!(unique.contains(&(1, 2)));
    }

    #[test]
    fn query_returns_overlapping_indices() {
        let bodies = vec![
            sphere_body(0, Vec3::ZERO, 0.5),
            sphere_body(1, Vec3::new(50.0, 0.0, 0.0), 0.5),
            sphere_body(2, Vec3::new(0.5, 0.5, 0.5), 0.5),
        ];
        let mut grid = SpatialHashGrid::new(2.0);
        grid.update(&bodies);
        let q = Aabb3::from_center_size(Vec3::ZERO, Vec3::splat(2.0));
        let mut hits = Vec::new();
        grid.query(&q, &mut hits);
        let set: HashSet<_> = hits.into_iter().collect();
        assert!(set.contains(&0));
        assert!(set.contains(&2));
        assert!(!set.contains(&1));
    }

    #[test]
    fn query_dedupes_body_across_cells() {
        let bodies = vec![sphere_body(0, Vec3::ZERO, 6.0)];
        let mut grid = SpatialHashGrid::new(2.0);
        grid.update(&bodies);
        let q = Aabb3::from_center_size(Vec3::ZERO, Vec3::splat(20.0));
        let mut hits = Vec::new();
        grid.query(&q, &mut hits);
        assert_eq!(hits, vec![0]);
    }

    #[test]
    fn clear_resets_state() {
        let bodies = vec![
            sphere_body(0, Vec3::ZERO, 1.0),
            sphere_body(1, Vec3::new(0.5, 0.0, 0.0), 1.0),
        ];
        let mut grid = SpatialHashGrid::new(2.0);
        grid.update(&bodies);
        let mut pairs = Vec::new();
        grid.get_potential_pairs(&mut pairs);
        assert!(!pairs.is_empty());
        grid.clear();
        pairs.clear();
        grid.get_potential_pairs(&mut pairs);
        assert!(pairs.is_empty());
    }
}
