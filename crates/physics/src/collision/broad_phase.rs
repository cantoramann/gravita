// physics/src/collision/broad_phase.rs
//
// Broad-phase data structures for culling non-overlapping body pairs before
// narrow-phase shape tests run.
//
// The default [`SpatialHashGrid`] uses a flat sorted `Vec<(packed_cell, body)>`
// rather than `HashMap<(i32,i32), Vec<usize>>`. The original implementation
// allocated several `Vec`s per body per step and used cryptographic `SipHash`
// for cell keys; this version reuses one growing buffer per phase and uses
// integer comparisons. The trade-off is one sort per `update`, which is
// O(N log N) — and dramatically faster in practice for the typical workload.

use gravita_math::{Aabb, Vec2};

use crate::body::RigidBody;

/// Interface for broad-phase collision strategies.
///
/// Implementations should cheaply cull non-interacting body pairs before
/// the narrow phase runs expensive shape tests. Output is written into
/// caller-provided buffers so steady-state simulation does not allocate.
pub trait BroadPhase {
    /// Update the spatial structure with the current body positions.
    fn update(&mut self, bodies: &[RigidBody]);
    /// Append all body indices that may overlap `aabb` into `out`.
    fn query(&self, aabb: &Aabb, out: &mut Vec<usize>);
    /// Append all potentially colliding body pairs into `out`. The output is
    /// deduplicated: each unordered pair appears at most once.
    ///
    /// `&mut self` because implementations may use internal scratch buffers to
    /// avoid per-call allocation.
    fn get_potential_pairs(&mut self, out: &mut Vec<(usize, usize)>);
    /// Clear all spatial data.
    fn clear(&mut self);
}

/// Pack two `i32` cell coordinates into a sortable `u64` key.
///
/// The high 32 bits are `x`, the low 32 bits are `y`, both biased by `i32::MIN`
/// so the unsigned ordering matches signed coordinate ordering.
#[inline]
fn pack_cell(x: i32, y: i32) -> u64 {
    let xu = (x as i64 - i32::MIN as i64) as u64;
    let yu = (y as i64 - i32::MIN as i64) as u64;
    (xu << 32) | yu
}

/// Spatial hash grid for efficient broad phase collision detection.
///
/// Stores `(packed_cell_key, body_index)` pairs in a single sorted `Vec` that
/// is reused across simulation steps. No `HashMap`/`HashSet` are involved.
pub struct SpatialHashGrid {
    cell_size: f32,
    /// Reusable entry buffer. After `update`, sorted by `packed_cell_key`.
    entries: Vec<(u64, u32)>,
    /// Scratch buffer used by `get_potential_pairs` to dedupe pairs.
    pair_scratch: Vec<u64>,
}

impl SpatialHashGrid {
    /// Create a new grid with the given cell size in world units.
    #[must_use]
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            entries: Vec::new(),
            pair_scratch: Vec::new(),
        }
    }

    #[inline]
    fn cell_of(&self, position: Vec2) -> (i32, i32) {
        (
            (position.x / self.cell_size).floor() as i32,
            (position.y / self.cell_size).floor() as i32,
        )
    }

    /// Push every `(cell_key, body_index)` pair covering `aabb` into `entries`.
    #[inline]
    fn push_cells_for_aabb(&mut self, aabb: &Aabb, body_idx: u32) {
        let (min_x, min_y) = self.cell_of(aabb.min);
        let (max_x, max_y) = self.cell_of(aabb.max);
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                self.entries.push((pack_cell(x, y), body_idx));
            }
        }
    }
}

impl Default for SpatialHashGrid {
    fn default() -> Self {
        Self::new(64.0)
    }
}

impl BroadPhase for SpatialHashGrid {
    fn update(&mut self, bodies: &[RigidBody]) {
        self.entries.clear();
        for (idx, body) in bodies.iter().enumerate() {
            let aabb = body.get_world_aabb();
            self.push_cells_for_aabb(&aabb, idx as u32);
        }
        // Sort by packed cell key so equal-cell runs are contiguous.
        self.entries.sort_unstable_by_key(|&(cell, _)| cell);
    }

    fn query(&self, aabb: &Aabb, out: &mut Vec<usize>) {
        let (min_x, min_y) = self.cell_of(aabb.min);
        let (max_x, max_y) = self.cell_of(aabb.max);

        let start_idx = out.len();
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                let key = pack_cell(x, y);
                // Binary search for the start of this cell's run.
                let lo = self.entries.partition_point(|&(k, _)| k < key);
                for &(k, body_idx) in &self.entries[lo..] {
                    if k != key {
                        break;
                    }
                    out.push(body_idx as usize);
                }
            }
        }

        // Deduplicate (a body straddling multiple queried cells could appear
        // more than once). Sort + dedup is cheaper than maintaining a HashSet.
        let added = &mut out[start_idx..];
        added.sort_unstable();
        let unique_len = {
            let new_slice = added;
            // Simulating Vec::dedup on a slice: count unique consecutive elems.
            let mut write = 0usize;
            for read in 0..new_slice.len() {
                if read == 0 || new_slice[read] != new_slice[read - 1] {
                    new_slice[write] = new_slice[read];
                    write += 1;
                }
            }
            write
        };
        out.truncate(start_idx + unique_len);
    }

    fn get_potential_pairs(&mut self, out: &mut Vec<(usize, usize)>) {
        let scratch = &mut self.pair_scratch;
        scratch.clear();

        // Walk runs of equal cell keys and emit body-pair combos within each cell.
        let mut i = 0;
        while i < self.entries.len() {
            let cell = self.entries[i].0;
            let mut j = i + 1;
            while j < self.entries.len() && self.entries[j].0 == cell {
                j += 1;
            }
            // entries[i..j] all share `cell`. Emit C(len, 2) pairs.
            let run = &self.entries[i..j];
            for a in 0..run.len() {
                for b in (a + 1)..run.len() {
                    let mut ai = run[a].1;
                    let mut bi = run[b].1;
                    if ai > bi {
                        std::mem::swap(&mut ai, &mut bi);
                    }
                    // Pack (ai, bi) into a u64 for cheap sort+dedup.
                    scratch.push(((ai as u64) << 32) | bi as u64);
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

/// Simple O(n²) broad phase for testing and benchmarking.
///
/// Returns no pairs from `get_potential_pairs` — callers (typically benches)
/// use this when they want to drive every pair through the narrow phase
/// directly.
pub struct NaiveBroadPhase;

impl BroadPhase for NaiveBroadPhase {
    fn update(&mut self, _bodies: &[RigidBody]) {}

    fn query(&self, _aabb: &Aabb, _out: &mut Vec<usize>) {}

    fn get_potential_pairs(&mut self, _out: &mut Vec<(usize, usize)>) {}

    fn clear(&mut self) {}
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use gravita_math::Circle;

    use super::*;
    use crate::body::CollisionShape;

    fn circle_body_at(id: usize, position: Vec2, radius: f32) -> RigidBody {
        RigidBody::new(id, CollisionShape::Circle(Circle::new(Vec2::ZERO, radius)))
            .with_position(position)
    }

    fn normalize_pair(p: (usize, usize)) -> (usize, usize) {
        if p.0 < p.1 { p } else { (p.1, p.0) }
    }

    #[test]
    fn empty_grid_yields_no_pairs() {
        let mut grid = SpatialHashGrid::new(50.0);
        grid.update(&[]);
        let mut pairs = Vec::new();
        grid.get_potential_pairs(&mut pairs);
        assert!(pairs.is_empty());
    }

    #[test]
    fn far_apart_bodies_yield_no_pairs() {
        let bodies = vec![
            circle_body_at(0, Vec2::ZERO, 5.0),
            circle_body_at(1, Vec2::new(1000.0, 1000.0), 5.0),
        ];
        let mut grid = SpatialHashGrid::new(50.0);
        grid.update(&bodies);
        let mut pairs = Vec::new();
        grid.get_potential_pairs(&mut pairs);
        assert!(pairs.is_empty());
    }

    #[test]
    fn overlapping_bodies_yield_pair() {
        let bodies = vec![
            circle_body_at(0, Vec2::ZERO, 10.0),
            circle_body_at(1, Vec2::new(5.0, 0.0), 10.0),
        ];
        let mut grid = SpatialHashGrid::new(50.0);
        grid.update(&bodies);
        let mut pairs = Vec::new();
        grid.get_potential_pairs(&mut pairs);
        assert!(pairs.into_iter().map(normalize_pair).any(|p| p == (0, 1)));
    }

    #[test]
    fn potential_pairs_are_deduplicated_across_cells() {
        // Two bodies that span multiple shared cells should still produce a single pair entry.
        let bodies = vec![
            circle_body_at(0, Vec2::new(25.0, 25.0), 30.0),
            circle_body_at(1, Vec2::new(30.0, 30.0), 30.0),
        ];
        let mut grid = SpatialHashGrid::new(50.0);
        grid.update(&bodies);
        let mut pairs = Vec::new();
        grid.get_potential_pairs(&mut pairs);
        let unique: HashSet<_> = pairs.iter().map(|&p| normalize_pair(p)).collect();
        assert_eq!(unique.len(), pairs.len(), "pairs should be unique");
        assert!(unique.contains(&(0, 1)));
    }

    #[test]
    fn pairs_within_same_cell_are_emitted() {
        // 3 bodies in the same cell -> C(3,2) = 3 pairs
        let bodies = vec![
            circle_body_at(0, Vec2::new(10.0, 10.0), 5.0),
            circle_body_at(1, Vec2::new(12.0, 12.0), 5.0),
            circle_body_at(2, Vec2::new(15.0, 15.0), 5.0),
        ];
        let mut grid = SpatialHashGrid::new(100.0);
        grid.update(&bodies);
        let mut pairs = Vec::new();
        grid.get_potential_pairs(&mut pairs);
        let unique: HashSet<_> = pairs.iter().map(|&p| normalize_pair(p)).collect();
        assert!(unique.contains(&(0, 1)));
        assert!(unique.contains(&(0, 2)));
        assert!(unique.contains(&(1, 2)));
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn query_returns_overlapping_body_indices() {
        let bodies = vec![
            circle_body_at(0, Vec2::ZERO, 10.0),
            circle_body_at(1, Vec2::new(1000.0, 0.0), 10.0),
            circle_body_at(2, Vec2::new(5.0, 5.0), 10.0),
        ];
        let mut grid = SpatialHashGrid::new(50.0);
        grid.update(&bodies);
        let query_aabb = Aabb::from_center_size(Vec2::ZERO, Vec2::new(40.0, 40.0));
        let mut hits = Vec::new();
        grid.query(&query_aabb, &mut hits);
        let hits: HashSet<usize> = hits.into_iter().collect();
        assert!(hits.contains(&0));
        assert!(hits.contains(&2));
        assert!(!hits.contains(&1));
    }

    #[test]
    fn query_dedupes_body_across_cells() {
        // Body straddles multiple cells; query should return it once.
        let bodies = vec![circle_body_at(0, Vec2::new(0.0, 0.0), 60.0)];
        let mut grid = SpatialHashGrid::new(50.0);
        grid.update(&bodies);
        let mut hits = Vec::new();
        let query_aabb = Aabb::from_center_size(Vec2::ZERO, Vec2::new(200.0, 200.0));
        grid.query(&query_aabb, &mut hits);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0], 0);
    }

    #[test]
    fn clear_resets_grid_state() {
        let bodies = vec![
            circle_body_at(0, Vec2::ZERO, 10.0),
            circle_body_at(1, Vec2::new(5.0, 0.0), 10.0),
        ];
        let mut grid = SpatialHashGrid::new(50.0);
        grid.update(&bodies);
        let mut pairs = Vec::new();
        grid.get_potential_pairs(&mut pairs);
        assert!(!pairs.is_empty());
        grid.clear();
        pairs.clear();
        grid.get_potential_pairs(&mut pairs);
        assert!(pairs.is_empty());
    }

    #[test]
    fn pack_cell_round_trip_for_negative_coordinates() {
        // Sanity: distinct (x, y) → distinct packed key, including negatives.
        let keys = [
            pack_cell(0, 0),
            pack_cell(-1, 0),
            pack_cell(0, -1),
            pack_cell(-1, -1),
            pack_cell(i32::MIN, i32::MIN),
            pack_cell(i32::MAX, i32::MAX),
        ];
        let unique: HashSet<u64> = keys.iter().copied().collect();
        assert_eq!(unique.len(), keys.len());
    }
}
