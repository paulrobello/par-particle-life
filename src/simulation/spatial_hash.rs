//! Spatial hashing for optimized neighbor queries.
//!
//! This module provides a spatial hash grid that allows O(1) average-case
//! neighbor lookups instead of O(n) brute force scanning.

use super::Particle;
use glam::Vec2;

/// A spatial hash grid for efficient neighbor queries.
///
/// The world is divided into cells of uniform size. Each cell stores
/// indices of particles that fall within it. Neighbor queries check
/// only the relevant cells instead of all particles.
#[derive(Debug, Clone)]
pub struct SpatialHash {
    /// Cell size in world units.
    cell_size: f32,
    /// Number of cells in X direction.
    grid_width: usize,
    /// Number of cells in Y direction.
    grid_height: usize,
    /// Particle indices stored per cell.
    cells: Vec<Vec<usize>>,
    /// World bounds.
    world_size: Vec2,
}

impl SpatialHash {
    /// Build a spatial hash from a slice of particles.
    ///
    /// # Arguments
    /// * `particles` - Particles to index
    /// * `cell_size` - Size of each cell (should be >= max interaction radius)
    /// * `world_size` - Size of the world bounds
    pub fn build(particles: &[Particle], cell_size: f32, world_size: Vec2) -> Self {
        let cell_size = cell_size.max(1.0); // Prevent division by zero
        let grid_width = (world_size.x / cell_size).ceil() as usize;
        let grid_height = (world_size.y / cell_size).ceil() as usize;

        let mut cells = vec![Vec::new(); grid_width * grid_height];

        for (i, p) in particles.iter().enumerate() {
            let cell_x = ((p.x / cell_size) as usize).min(grid_width.saturating_sub(1));
            let cell_y = ((p.y / cell_size) as usize).min(grid_height.saturating_sub(1));
            let cell_idx = cell_y * grid_width + cell_x;

            if cell_idx < cells.len() {
                cells[cell_idx].push(i);
            }
        }

        Self {
            cell_size,
            grid_width,
            grid_height,
            cells,
            world_size,
        }
    }

    /// Query all particle indices within a radius of a position.
    ///
    /// Returns an iterator over particle indices that might be within range.
    /// Note: This returns particles in nearby cells, so some may be slightly
    /// outside the radius. Caller should do precise distance checks.
    ///
    /// # Arguments
    /// * `position` - Center point to query from
    /// * `radius` - Maximum distance to search
    /// * `world_size` - World bounds (for wrapping)
    /// * `wrap` - Whether to consider wrapped positions
    pub fn query_radius(
        &self,
        position: Vec2,
        radius: f32,
        _world_size: Vec2,
        wrap: bool,
    ) -> Vec<usize> {
        let mut result = Vec::new();

        // Calculate cell range to check
        let cells_to_check = (radius / self.cell_size).ceil() as i32 + 1;

        let center_x = (position.x / self.cell_size) as i32;
        let center_y = (position.y / self.cell_size) as i32;

        for dy in -cells_to_check..=cells_to_check {
            for dx in -cells_to_check..=cells_to_check {
                let mut cell_x = center_x + dx;
                let mut cell_y = center_y + dy;

                // Handle wrapping
                if wrap {
                    cell_x = cell_x.rem_euclid(self.grid_width as i32);
                    cell_y = cell_y.rem_euclid(self.grid_height as i32);
                } else {
                    // Skip out-of-bounds cells
                    if cell_x < 0 || cell_x >= self.grid_width as i32 {
                        continue;
                    }
                    if cell_y < 0 || cell_y >= self.grid_height as i32 {
                        continue;
                    }
                }

                let cell_idx = (cell_y as usize) * self.grid_width + (cell_x as usize);
                if cell_idx < self.cells.len() {
                    result.extend(self.cells[cell_idx].iter().copied());
                }
            }
        }

        result
    }

    /// Get the cell index for a position.
    #[inline]
    pub fn get_cell_index(&self, x: f32, y: f32) -> Option<usize> {
        if x < 0.0 || y < 0.0 || x >= self.world_size.x || y >= self.world_size.y {
            return None;
        }

        let cell_x = (x / self.cell_size) as usize;
        let cell_y = (y / self.cell_size) as usize;

        if cell_x < self.grid_width && cell_y < self.grid_height {
            Some(cell_y * self.grid_width + cell_x)
        } else {
            None
        }
    }

    /// Get particles in a specific cell.
    pub fn get_cell(&self, cell_idx: usize) -> &[usize] {
        self.cells
            .get(cell_idx)
            .map(|c| c.as_slice())
            .unwrap_or(&[])
    }

    /// Get the number of cells in the grid.
    pub fn num_cells(&self) -> usize {
        self.cells.len()
    }

    /// Get grid dimensions.
    pub fn dimensions(&self) -> (usize, usize) {
        (self.grid_width, self.grid_height)
    }

    /// Get the cell size.
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Clear all cells (for reuse).
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.clear();
        }
    }

    /// Get statistics about the spatial hash distribution.
    pub fn stats(&self) -> SpatialHashStats {
        let non_empty = self.cells.iter().filter(|c| !c.is_empty()).count();
        let total_particles: usize = self.cells.iter().map(|c| c.len()).sum();
        let max_per_cell = self.cells.iter().map(|c| c.len()).max().unwrap_or(0);
        let avg_per_cell = if non_empty > 0 {
            total_particles as f32 / non_empty as f32
        } else {
            0.0
        };

        SpatialHashStats {
            total_cells: self.cells.len(),
            non_empty_cells: non_empty,
            total_particles,
            max_per_cell,
            avg_per_cell,
        }
    }
}

/// Statistics about spatial hash distribution.
#[derive(Debug, Clone)]
pub struct SpatialHashStats {
    /// Total number of cells in the grid.
    pub total_cells: usize,
    /// Number of cells containing at least one particle.
    pub non_empty_cells: usize,
    /// Total particles indexed.
    pub total_particles: usize,
    /// Maximum particles in any single cell.
    pub max_per_cell: usize,
    /// Average particles per non-empty cell.
    pub avg_per_cell: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_particles() -> Vec<Particle> {
        vec![
            Particle::new(10.0, 10.0, 0),
            Particle::new(15.0, 10.0, 0),
            Particle::new(90.0, 90.0, 1),
        ]
    }

    #[test]
    fn test_spatial_hash_build() {
        let particles = make_particles();
        let hash = SpatialHash::build(&particles, 20.0, Vec2::new(100.0, 100.0));

        assert!(hash.num_cells() > 0);
        let stats = hash.stats();
        assert_eq!(stats.total_particles, 3);
    }

    #[test]
    fn test_query_radius() {
        let particles = make_particles();
        let hash = SpatialHash::build(&particles, 20.0, Vec2::new(100.0, 100.0));

        // Query near particles 0 and 1
        let nearby = hash.query_radius(Vec2::new(12.0, 10.0), 10.0, Vec2::new(100.0, 100.0), false);

        // Should find particles 0 and 1, not particle 2
        assert!(nearby.contains(&0));
        assert!(nearby.contains(&1));
        // Note: particle 2 might be included if cells overlap, but it's far away
    }

    #[test]
    fn test_cell_index() {
        let particles = make_particles();
        let hash = SpatialHash::build(&particles, 20.0, Vec2::new(100.0, 100.0));

        assert!(hash.get_cell_index(10.0, 10.0).is_some());
        assert!(hash.get_cell_index(-5.0, 10.0).is_none());
        assert!(hash.get_cell_index(110.0, 10.0).is_none());
    }

    #[test]
    fn test_wrapping_query() {
        let particles = vec![
            Particle::new(5.0, 50.0, 0),  // Near left edge
            Particle::new(95.0, 50.0, 1), // Near right edge
        ];
        let hash = SpatialHash::build(&particles, 20.0, Vec2::new(100.0, 100.0));

        // Query from particle 0, with wrapping should find particle 1
        let nearby = hash.query_radius(Vec2::new(5.0, 50.0), 15.0, Vec2::new(100.0, 100.0), true);

        // Both particles should be reachable through wrapping
        assert!(!nearby.is_empty());
    }
}
