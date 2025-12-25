// physics/src/collision/broad_phase.rs
use std::collections::{HashMap, HashSet};

use gravita_math::{AABB, Vec2};

use crate::body::RigidBody;

/// Interface for broad-phase collision strategies.
///
/// Implementations should cheaply cull non-interacting body pairs before
/// the narrow phase runs expensive shape tests.
pub trait BroadPhase {
    /// Update the spatial structure with the current body positions.
    fn update(&mut self, bodies: &[RigidBody]);
    /// Find all body indices that may overlap the given AABB.
    fn query(&self, aabb: &AABB) -> Vec<usize>;
    /// Get all potentially colliding body pairs.
    fn get_potential_pairs(&self) -> Vec<(usize, usize)>;
    /// Clear all spatial data.
    fn clear(&mut self);
}

/// Spatial hash grid for efficient broad phase collision detection.
pub struct SpatialHashGrid {
    /// Size of each cell in world units.
    cell_size: f32,
    /// Map from cell coordinates to body indices in that cell.
    grid: HashMap<(i32, i32), Vec<usize>>,
    /// Set of cells that contain at least one body.
    active_cells: HashSet<(i32, i32)>,
}

impl SpatialHashGrid {
    /// Create a new grid with the given cell size in world units.
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
            active_cells: HashSet::new(),
        }
    }

    fn hash_position(&self, position: Vec2) -> (i32, i32) {
        (
            (position.x / self.cell_size).floor() as i32,
            (position.y / self.cell_size).floor() as i32,
        )
    }

    fn get_cells_for_aabb(&self, aabb: &AABB) -> Vec<(i32, i32)> {
        let min_cell = self.hash_position(aabb.min);
        let max_cell = self.hash_position(aabb.max);

        let mut cells = Vec::new();
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                cells.push((x, y));
            }
        }
        cells
    }
}

impl BroadPhase for SpatialHashGrid {
    fn update(&mut self, bodies: &[RigidBody]) {
        self.clear();

        for (idx, body) in bodies.iter().enumerate() {
            let aabb = body.get_world_aabb();
            let cells = self.get_cells_for_aabb(&aabb);

            for cell in cells {
                self.grid.entry(cell).or_default().push(idx);
                self.active_cells.insert(cell);
            }
        }
    }

    fn query(&self, aabb: &AABB) -> Vec<usize> {
        let mut result = HashSet::new();
        let cells = self.get_cells_for_aabb(aabb);

        for cell in cells {
            if let Some(indices) = self.grid.get(&cell) {
                for idx in indices {
                    result.insert(*idx);
                }
            }
        }

        result.into_iter().collect()
    }

    fn get_potential_pairs(&self) -> Vec<(usize, usize)> {
        let mut pairs = HashSet::new();

        for cell in &self.active_cells {
            if let Some(indices) = self.grid.get(cell) {
                // Check all pairs within this cell
                for i in 0..indices.len() {
                    for j in (i + 1)..indices.len() {
                        let pair = if indices[i] < indices[j] {
                            (indices[i], indices[j])
                        } else {
                            (indices[j], indices[i])
                        };
                        pairs.insert(pair);
                    }
                }
            }
        }

        pairs.into_iter().collect()
    }

    fn clear(&mut self) {
        self.grid.clear();
        self.active_cells.clear();
    }
}

/// Simple O(n²) broad phase for testing and benchmarking.
pub struct NaiveBroadPhase;

impl BroadPhase for NaiveBroadPhase {
    fn update(&mut self, _bodies: &[RigidBody]) {}

    fn query(&self, _aabb: &AABB) -> Vec<usize> {
        Vec::new() // Not implemented for naive approach
    }

    fn get_potential_pairs(&self) -> Vec<(usize, usize)> {
        // This would be filled in by the detector
        Vec::new()
    }

    fn clear(&mut self) {}
}
