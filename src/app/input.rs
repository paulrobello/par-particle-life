//! Input handling for the application.
//!
//! These structures are defined for mouse interaction features
//! including camera pan/zoom and brush tools.

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Brush tool types for user interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BrushTool {
    /// No active tool.
    #[default]
    None,
    /// Draw/add particles at brush position.
    Draw,
    /// Erase/remove particles at brush position.
    Erase,
    /// Attract particles toward brush position.
    Attract,
    /// Repel particles away from brush position.
    Repel,
}

impl BrushTool {
    /// Get all available brush tools.
    pub fn all() -> &'static [BrushTool] {
        &[
            BrushTool::None,
            BrushTool::Draw,
            BrushTool::Erase,
            BrushTool::Attract,
            BrushTool::Repel,
        ]
    }

    /// Get the display name for this tool.
    pub fn name(&self) -> &'static str {
        match self {
            BrushTool::None => "None",
            BrushTool::Draw => "Draw",
            BrushTool::Erase => "Erase",
            BrushTool::Attract => "Attract",
            BrushTool::Repel => "Repel",
        }
    }

    /// Get the icon for this tool (ASCII for font compatibility).
    pub fn icon(&self) -> &'static str {
        match self {
            BrushTool::None => "[X]",
            BrushTool::Draw => "[+]",
            BrushTool::Erase => "🧹",
            BrushTool::Attract => "[>]",
            BrushTool::Repel => "[<]",
        }
    }
}

/// Brush state for user interaction tools.
#[derive(Debug, Clone, Copy)]
pub struct BrushState {
    /// Current brush tool.
    pub tool: BrushTool,
    /// Brush position in world coordinates.
    pub position: Vec2,
    /// Brush velocity (for directional force).
    pub velocity: Vec2,
    /// Brush radius in world coordinates.
    pub radius: f32,
    /// Attraction force strength (0.0 - 100.0).
    pub attract_force: f32,
    /// Repulsion force strength (0.0 - 100.0).
    pub repel_force: f32,
    /// Directional force from brush movement (0.0 - 100.0).
    pub directional_force: f32,
    /// Number of particles to spawn per frame in Draw mode.
    pub draw_intensity: u32,
    /// Particle type to draw (-1 for random).
    pub draw_type: i32,
    /// Show brush circle indicator.
    pub show_circle: bool,
    /// Is brush currently active (mouse pressed)?
    pub is_active: bool,
    /// Target particle type for attract/repel/erase (-1 for all).
    pub target_type: i32,
}

impl Default for BrushState {
    fn default() -> Self {
        Self {
            tool: BrushTool::None,
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            radius: 50.0,
            attract_force: 50.0,
            repel_force: 50.0,
            directional_force: 40.0,
            draw_intensity: 50,
            draw_type: -1, // Random type
            show_circle: true,
            is_active: false,
            target_type: -1, // All types
        }
    }
}

impl BrushState {
    /// Update brush position and calculate velocity.
    pub fn update_position(&mut self, new_pos: Vec2, dt: f32) {
        if dt > 0.0 {
            self.velocity = (new_pos - self.position) / dt;
        }
        self.position = new_pos;
    }

    /// Get the signed force value (positive for attract, negative for repel).
    pub fn get_force(&self) -> f32 {
        match self.tool {
            BrushTool::Attract => self.attract_force * 10.0,
            BrushTool::Repel => -self.repel_force * 10.0,
            _ => 0.0,
        }
    }
}

/// Mouse button state.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MouseState {
    pub position: Vec2,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub middle_pressed: bool,
}

/// Keyboard modifier state.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

/// Input state for the application.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub mouse: MouseState,
    pub modifiers: ModifierState,
}

/// Camera state for pan and zoom.
#[derive(Debug, Clone, Copy)]
pub struct CameraState {
    /// Camera offset in world coordinates (pan).
    pub offset: Vec2,
    /// Zoom level (1.0 = default, >1 = zoomed in, <1 = zoomed out).
    pub zoom: f32,
    /// Is the user currently panning?
    pub is_panning: bool,
    /// Last mouse position for pan delta calculation.
    pub last_mouse_pos: Vec2,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
            is_panning: false,
            last_mouse_pos: Vec2::ZERO,
        }
    }
}

impl CameraState {
    /// Reset camera to default view.
    pub fn reset(&mut self) {
        self.offset = Vec2::ZERO;
        self.zoom = 1.0;
    }

    /// Apply zoom centered on a screen position.
    /// `screen_pos` is in normalized device coordinates (-1 to 1).
    /// `world_center` is the current center in world coords.
    /// `world_size` is the visible world size.
    pub fn zoom_at(&mut self, factor: f32, screen_pos: Vec2, world_center: Vec2, world_size: Vec2) {
        let old_zoom = self.zoom;
        self.zoom = (self.zoom * factor).clamp(0.1, 10.0);

        // Adjust offset to keep the point under cursor stationary
        let zoom_ratio = self.zoom / old_zoom;
        let world_pos = world_center + screen_pos * world_size * 0.5;
        self.offset = world_pos - (world_pos - self.offset) * zoom_ratio;
    }

    /// Simple zoom that keeps center fixed.
    pub fn zoom_center(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(0.1, 10.0);
    }

    /// Pan by a delta in world coordinates.
    pub fn pan(&mut self, delta: Vec2) {
        self.offset += delta;
    }

    /// Convert screen coordinates to world coordinates.
    /// `screen_pos`: Screen position (0,0 at top-left).
    /// `screen_size`: Screen dimensions.
    /// `world_size`: World dimensions.
    pub fn screen_to_world(&self, screen_pos: Vec2, screen_size: Vec2, world_size: Vec2) -> Vec2 {
        // Screen to normalized (-1 to 1)
        let normalized_x = (screen_pos.x / screen_size.x) * 2.0 - 1.0;
        let normalized_y = (screen_pos.y / screen_size.y) * 2.0 - 1.0;

        // Account for camera zoom and offset
        let world_x = (normalized_x / self.zoom + 1.0) * 0.5 * world_size.x + self.offset.x;
        let world_y = (normalized_y / self.zoom + 1.0) * 0.5 * world_size.y + self.offset.y;

        Vec2::new(world_x, world_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_to_world_mapping() {
        let camera = CameraState::default();
        let screen_size = Vec2::new(800.0, 600.0);
        let world_size = Vec2::new(1600.0, 1200.0);

        // Top-left screen (0, 0) should map to top-left world (0, 0) with default camera
        let screen_top_left = Vec2::new(0.0, 0.0);
        let world_top_left = camera.screen_to_world(screen_top_left, screen_size, world_size);
        assert_eq!(world_top_left, Vec2::new(0.0, 0.0));

        // Bottom-right screen (800, 600) should map to bottom-right world (1600, 1200)
        let screen_bottom_right = Vec2::new(800.0, 600.0);
        let world_bottom_right =
            camera.screen_to_world(screen_bottom_right, screen_size, world_size);
        assert_eq!(world_bottom_right, Vec2::new(1600.0, 1200.0));

        // Center screen (400, 300) should map to center world (800, 600)
        let screen_center = Vec2::new(400.0, 300.0);
        let world_center = camera.screen_to_world(screen_center, screen_size, world_size);
        assert_eq!(world_center, Vec2::new(800.0, 600.0));
    }

    #[test]
    fn test_screen_to_world_with_zoom() {
        let mut camera = CameraState::default();
        camera.zoom = 2.0;
        let screen_size = Vec2::new(800.0, 600.0);
        let world_size = Vec2::new(1600.0, 1200.0);

        // With 2x zoom, center should still be center
        let screen_center = Vec2::new(400.0, 300.0);
        let world_center = camera.screen_to_world(screen_center, screen_size, world_size);
        assert_eq!(world_center, Vec2::new(800.0, 600.0));
    }
}
