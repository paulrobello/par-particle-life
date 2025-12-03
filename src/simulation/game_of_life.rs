//! Game of Life cellular automaton simulation.
//!
//! Implements Conway's Game of Life with customizable birth/survival rules
//! and various edge handling modes.

use serde::{Deserialize, Serialize};

/// Edge handling modes for the cellular automaton.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EdgeMode {
    /// Cells outside the grid are considered dead.
    #[default]
    Dead,
    /// Cells outside the grid are considered alive.
    Alive,
    /// Edges wrap around (toroidal topology).
    Mirror,
}

impl EdgeMode {
    /// Get all available edge modes.
    pub fn all() -> &'static [EdgeMode] {
        &[EdgeMode::Dead, EdgeMode::Alive, EdgeMode::Mirror]
    }

    /// Get the display name for this mode.
    pub fn display_name(&self) -> &'static str {
        match self {
            EdgeMode::Dead => "Dead Edges",
            EdgeMode::Alive => "Alive Edges",
            EdgeMode::Mirror => "Wrap Around",
        }
    }
}

/// Configuration for the Game of Life simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameOfLifeConfig {
    /// Grid width in cells.
    pub width: usize,
    /// Grid height in cells.
    pub height: usize,
    /// Cell render size in pixels.
    pub cell_size: usize,
    /// Birth conditions (number of neighbors that cause a dead cell to become alive).
    pub born: Vec<u8>,
    /// Survival conditions (number of neighbors that keep a cell alive).
    pub survives: Vec<u8>,
    /// Edge handling mode.
    pub edge_mode: EdgeMode,
    /// Animation speed in milliseconds per generation.
    pub speed_ms: u32,
}

impl Default for GameOfLifeConfig {
    fn default() -> Self {
        Self {
            width: 256,
            height: 256,
            cell_size: 4,
            born: vec![3],        // Standard Conway's Game of Life
            survives: vec![2, 3], // Standard Conway's Game of Life
            edge_mode: EdgeMode::Dead,
            speed_ms: 100,
        }
    }
}

impl GameOfLifeConfig {
    /// Create a configuration for Conway's standard Game of Life.
    pub fn conway() -> Self {
        Self::default()
    }

    /// Create a configuration for HighLife (B36/S23).
    pub fn highlife() -> Self {
        Self {
            born: vec![3, 6],
            survives: vec![2, 3],
            ..Default::default()
        }
    }

    /// Create a configuration for Day & Night (B3678/S34678).
    pub fn day_and_night() -> Self {
        Self {
            born: vec![3, 6, 7, 8],
            survives: vec![3, 4, 6, 7, 8],
            ..Default::default()
        }
    }

    /// Create a configuration for Seeds (B2/S).
    pub fn seeds() -> Self {
        Self {
            born: vec![2],
            survives: vec![],
            ..Default::default()
        }
    }

    /// Get the rule string in B/S notation (e.g., "B3/S23").
    pub fn rule_string(&self) -> String {
        let born: String = self.born.iter().map(|n| n.to_string()).collect();
        let survives: String = self.survives.iter().map(|n| n.to_string()).collect();
        format!("B{born}/S{survives}")
    }

    /// Parse a rule string in B/S notation.
    pub fn from_rule_string(rule: &str) -> Option<Self> {
        let parts: Vec<&str> = rule.split('/').collect();
        if parts.len() != 2 {
            return None;
        }

        let born_str = parts[0]
            .strip_prefix('B')
            .or_else(|| parts[0].strip_prefix('b'))?;
        let survives_str = parts[1]
            .strip_prefix('S')
            .or_else(|| parts[1].strip_prefix('s'))?;

        let born: Vec<u8> = born_str
            .chars()
            .filter_map(|c| c.to_digit(10).map(|d| d as u8))
            .collect();

        let survives: Vec<u8> = survives_str
            .chars()
            .filter_map(|c| c.to_digit(10).map(|d| d as u8))
            .collect();

        Some(Self {
            born,
            survives,
            ..Default::default()
        })
    }
}

/// Game of Life simulation state.
pub struct GameOfLife {
    /// Current grid state (0 = dead, 1+ = alive with age).
    grid: Vec<u8>,
    /// Back buffer for double-buffering.
    back_buffer: Vec<u8>,
    /// Configuration.
    config: GameOfLifeConfig,
    /// Current generation count.
    generation: u64,
    /// Current population (number of alive cells).
    population: usize,
}

impl GameOfLife {
    /// Create a new Game of Life simulation with the given configuration.
    pub fn new(config: GameOfLifeConfig) -> Self {
        let size = config.width * config.height;
        Self {
            grid: vec![0; size],
            back_buffer: vec![0; size],
            config,
            generation: 0,
            population: 0,
        }
    }

    /// Create with default configuration.
    pub fn default_conway() -> Self {
        Self::new(GameOfLifeConfig::conway())
    }

    /// Get the grid width.
    pub fn width(&self) -> usize {
        self.config.width
    }

    /// Get the grid height.
    pub fn height(&self) -> usize {
        self.config.height
    }

    /// Get the current generation count.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Get the current population.
    pub fn population(&self) -> usize {
        self.population
    }

    /// Get the configuration.
    pub fn config(&self) -> &GameOfLifeConfig {
        &self.config
    }

    /// Get cell state at position (0 = dead, 1+ = alive).
    pub fn get_cell(&self, x: usize, y: usize) -> u8 {
        if x < self.config.width && y < self.config.height {
            self.grid[y * self.config.width + x]
        } else {
            0
        }
    }

    /// Set cell state at position.
    pub fn set_cell(&mut self, x: usize, y: usize, state: u8) {
        if x < self.config.width && y < self.config.height {
            self.grid[y * self.config.width + x] = state;
        }
    }

    /// Toggle cell state at position.
    pub fn toggle_cell(&mut self, x: usize, y: usize) {
        if x < self.config.width && y < self.config.height {
            let idx = y * self.config.width + x;
            self.grid[idx] = if self.grid[idx] > 0 { 0 } else { 1 };
        }
    }

    /// Clear the entire grid.
    pub fn clear(&mut self) {
        self.grid.fill(0);
        self.generation = 0;
        self.population = 0;
    }

    /// Randomize the grid with the given alive probability.
    pub fn randomize(&mut self, alive_probability: f32) {
        use rand::Rng;
        let mut rng = rand::rng();

        for cell in &mut self.grid {
            *cell = if rng.random::<f32>() < alive_probability {
                1
            } else {
                0
            };
        }

        self.generation = 0;
        self.update_population();
    }

    /// Count alive neighbors for a cell.
    fn count_neighbors(&self, x: usize, y: usize) -> u8 {
        let mut count = 0u8;
        let w = self.config.width as i32;
        let h = self.config.height as i32;

        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                let alive = match self.config.edge_mode {
                    EdgeMode::Dead => {
                        if nx >= 0 && nx < w && ny >= 0 && ny < h {
                            self.grid[(ny as usize) * self.config.width + (nx as usize)] > 0
                        } else {
                            false
                        }
                    }
                    EdgeMode::Alive => {
                        if nx >= 0 && nx < w && ny >= 0 && ny < h {
                            self.grid[(ny as usize) * self.config.width + (nx as usize)] > 0
                        } else {
                            true
                        }
                    }
                    EdgeMode::Mirror => {
                        let wrapped_x = nx.rem_euclid(w) as usize;
                        let wrapped_y = ny.rem_euclid(h) as usize;
                        self.grid[wrapped_y * self.config.width + wrapped_x] > 0
                    }
                };

                if alive {
                    count += 1;
                    // Early exit optimization: max useful count is 8
                    if count > 8 {
                        return count;
                    }
                }
            }
        }

        count
    }

    /// Advance the simulation by one generation.
    pub fn step(&mut self) {
        let mut population = 0usize;

        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let idx = y * self.config.width + x;
                let neighbors = self.count_neighbors(x, y);
                let currently_alive = self.grid[idx] > 0;

                let will_live = if currently_alive {
                    self.config.survives.contains(&neighbors)
                } else {
                    self.config.born.contains(&neighbors)
                };

                self.back_buffer[idx] = if will_live {
                    // Increment age if already alive, otherwise set to 1
                    if currently_alive {
                        self.grid[idx].saturating_add(1)
                    } else {
                        1
                    }
                } else {
                    0
                };

                if will_live {
                    population += 1;
                }
            }
        }

        std::mem::swap(&mut self.grid, &mut self.back_buffer);
        self.generation += 1;
        self.population = population;
    }

    /// Update population count from current grid.
    fn update_population(&mut self) {
        self.population = self.grid.iter().filter(|&&c| c > 0).count();
    }

    /// Get a reference to the raw grid data.
    pub fn grid(&self) -> &[u8] {
        &self.grid
    }

    /// Load a pattern at the given position.
    pub fn load_pattern(&mut self, pattern: &[&str], offset_x: usize, offset_y: usize) {
        for (y, row) in pattern.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                if ch == 'O' || ch == '*' || ch == '#' {
                    let px = offset_x + x;
                    let py = offset_y + y;
                    if px < self.config.width && py < self.config.height {
                        self.set_cell(px, py, 1);
                    }
                }
            }
        }
        self.update_population();
    }

    /// Load a glider pattern at the center.
    pub fn load_glider(&mut self) {
        let cx = self.config.width / 2;
        let cy = self.config.height / 2;
        let pattern = [".O.", "..O", "OOO"];
        self.load_pattern(&pattern, cx, cy);
    }

    /// Load a Gosper glider gun pattern.
    pub fn load_glider_gun(&mut self) {
        let pattern = [
            "........................O...........",
            "......................O.O...........",
            "............OO......OO............OO",
            "...........O...O....OO............OO",
            "OO........O.....O...OO..............",
            "OO........O...O.OO....O.O...........",
            "..........O.....O.......O...........",
            "...........O...O....................",
            "............OO......................",
        ];
        self.load_pattern(&pattern, 10, 10);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = GameOfLife::new(GameOfLifeConfig::default());
        assert_eq!(game.width(), 256);
        assert_eq!(game.height(), 256);
        assert_eq!(game.generation(), 0);
        assert_eq!(game.population(), 0);
    }

    #[test]
    fn test_set_get_cell() {
        let mut game = GameOfLife::new(GameOfLifeConfig::default());
        game.set_cell(10, 20, 1);
        assert_eq!(game.get_cell(10, 20), 1);
        assert_eq!(game.get_cell(0, 0), 0);
    }

    #[test]
    fn test_toggle_cell() {
        let mut game = GameOfLife::new(GameOfLifeConfig::default());
        assert_eq!(game.get_cell(5, 5), 0);
        game.toggle_cell(5, 5);
        assert_eq!(game.get_cell(5, 5), 1);
        game.toggle_cell(5, 5);
        assert_eq!(game.get_cell(5, 5), 0);
    }

    #[test]
    fn test_blinker_oscillation() {
        let mut config = GameOfLifeConfig::default();
        config.width = 5;
        config.height = 5;
        let mut game = GameOfLife::new(config);

        // Create a blinker (vertical line)
        game.set_cell(2, 1, 1);
        game.set_cell(2, 2, 1);
        game.set_cell(2, 3, 1);

        // Step should turn it horizontal
        game.step();

        // Cells are alive if > 0 (implementation tracks age)
        assert!(game.get_cell(1, 2) > 0);
        assert!(game.get_cell(2, 2) > 0);
        assert!(game.get_cell(3, 2) > 0);
        assert_eq!(game.get_cell(2, 1), 0);
        assert_eq!(game.get_cell(2, 3), 0);

        // Step again should return to vertical
        game.step();

        assert!(game.get_cell(2, 1) > 0);
        assert!(game.get_cell(2, 2) > 0);
        assert!(game.get_cell(2, 3) > 0);
    }

    #[test]
    fn test_rule_string() {
        let config = GameOfLifeConfig::conway();
        assert_eq!(config.rule_string(), "B3/S23");

        let highlife = GameOfLifeConfig::highlife();
        assert_eq!(highlife.rule_string(), "B36/S23");
    }

    #[test]
    fn test_parse_rule_string() {
        let config = GameOfLifeConfig::from_rule_string("B36/S23").unwrap();
        assert_eq!(config.born, vec![3, 6]);
        assert_eq!(config.survives, vec![2, 3]);
    }
}
