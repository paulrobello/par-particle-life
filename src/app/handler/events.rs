//! Event handling for the application.

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Fullscreen, WindowAttributes, WindowId},
};

use super::{AppHandler, RuleSelection};
use crate::app::BrushTool;
use crate::simulation::{MAX_OBSTACLES, Obstacle, ObstacleShape};

impl AppHandler {
    pub(crate) fn fullscreen_target(is_fullscreen: bool) -> Option<Fullscreen> {
        if is_fullscreen {
            None
        } else {
            Some(Fullscreen::Borderless(None))
        }
    }

    pub(crate) fn toggle_fullscreen(&mut self) {
        if let Some(gpu) = &self.gpu {
            let is_fullscreen = gpu.context.window.fullscreen().is_some();
            gpu.context
                .window
                .set_fullscreen(Self::fullscreen_target(is_fullscreen));
            self.preset_status = if is_fullscreen {
                "Exited fullscreen".to_string()
            } else {
                "Entered fullscreen".to_string()
            };
        }
    }
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_none() {
            // Load window icon
            let window_icon = Self::load_window_icon();

            // Create window
            let mut window_attrs = WindowAttributes::default()
                .with_title(&self.app.config.title)
                .with_inner_size(LogicalSize::new(
                    self.app.config.window_width,
                    self.app.config.window_height,
                ));

            if let Some(icon) = window_icon {
                window_attrs = window_attrs.with_window_icon(Some(icon));
            }

            // ARC-013: window creation failure used to `.expect`-panic. The
            // winit `ApplicationHandler::resumed` signature returns `()`, so
            // we cannot `?`-propagate; instead log + exit the loop cleanly so
            // the user sees an error message instead of an unfriendly abort.
            let window = match event_loop.create_window(window_attrs) {
                Ok(w) => Arc::new(w),
                Err(e) => {
                    log::error!(
                        "Failed to create window: {e}. Exiting event loop."
                    );
                    event_loop.exit();
                    return;
                }
            };

            // ARC-013: GPU init failures used to `.expect`-panic inside
            // `init_gpu`. Now they propagate as `AppError`; log + exit
            // cleanly instead of bringing down the whole process.
            if let Err(e) = self.init_gpu(window) {
                log::error!("GPU initialization failed: {e}. Exiting event loop.");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Let egui handle events first
        if let Some(gpu) = &mut self.gpu {
            let response = gpu.egui_state.on_window_event(&gpu.context.window, &event);
            let egui_wants_pointer = gpu.egui_ctx.egui_wants_pointer_input();

            // Track whether the cursor is over the egui UI panel (updated on move)
            // Use screen X position vs stored panel right edge — egui_wants_pointer
            // is unreliable (always true regardless of cursor location).
            if let WindowEvent::CursorMoved { position, .. } = &event {
                self.cursor_over_ui = self.show_ui && position.x as f32 <= self.ui_panel_right_edge;
            }

            if let WindowEvent::DroppedFile(path) = &event {
                self.load_dropped_preset(path);
                return;
            }

            if response.consumed && egui_wants_pointer {
                if self.brush.tool != BrushTool::None && !self.cursor_over_ui {
                    // Fall through to brush handlers when cursor is in simulation area
                } else {
                    if let WindowEvent::CursorMoved { position, .. } = &event {
                        self.camera.last_mouse_pos =
                            glam::Vec2::new(position.x as f32, position.y as f32);
                    }
                    return;
                }
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested, exiting...");
                // UI panel open/closed state lives on AppHandler — push it into
                // app.config before snapshotting so it survives alongside the
                // runtime-mirrored sim/physics/generator fields.
                self.app.config.ui_simulation_open = self.ui_simulation_open;
                self.app.config.ui_physics_open = self.ui_physics_open;
                self.app.config.ui_generators_open = self.ui_generators_open;
                self.app.config.ui_interaction_matrix_open = self.ui_interaction_matrix_open;
                self.app.config.ui_brush_tools_open = self.ui_brush_tools_open;
                self.app.config.ui_rendering_open = self.ui_rendering_open;
                self.app.config.ui_presets_open = self.ui_presets_open;
                self.app.config.ui_keyboard_shortcuts_open = self.ui_keyboard_shortcuts_open;
                self.app.config.ui_obstacles_open = self.ui_obstacles_open;

                // ARC-003/009: single-source-of-truth snapshot replaces the
                // ~30-line hand-mirror block. Previously this forgot to mirror
                // `phys_temperature` and `phys_velocity_coupling`, so both
                // silently reset to their defaults on every normal quit.
                let snapshot = self.app.snapshot_config();
                self.app.config = snapshot;

                if let Err(e) = self.app.config.save() {
                    log::error!("Failed to save app config: {}", e);
                }
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.context.resize(new_size.width, new_size.height);
                    gpu.render.update_camera(
                        &gpu.context.queue,
                        self.app.sim_config.world_size.x,
                        self.app.sim_config.world_size.y,
                        new_size.width as f32,
                        new_size.height as f32,
                    );
                }
            }
            WindowEvent::RedrawRequested => {
                self.update();
                self.render();

                // Request another frame
                if let Some(gpu) = &self.gpu {
                    gpu.context.window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } if event.state.is_pressed() => {
                use winit::keyboard::{KeyCode, PhysicalKey};
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Space) => {
                        self.app.toggle_running();
                    }
                    PhysicalKey::Code(KeyCode::KeyR) => {
                        self.app.regenerate_particles();
                        self.sync_buffers();
                    }
                    PhysicalKey::Code(KeyCode::KeyM) => match &self.rule_selection {
                        RuleSelection::BuiltIn(_) => {
                            self.app.regenerate_rules();
                            self.sync_interaction_matrix();
                        }
                        RuleSelection::Custom(idx) => match self.app.generate_custom_rules(*idx) {
                            Ok(matrix) => {
                                self.app.interaction_matrix = matrix;
                                self.app.capture_matrix_variation_base();
                                self.sync_interaction_matrix();
                                self.preset_status.clear();
                            }
                            Err(e) => {
                                self.preset_status = format!("Custom generator error: {e}");
                            }
                        },
                    },
                    PhysicalKey::Code(KeyCode::KeyH) => {
                        self.show_ui = !self.show_ui;
                        if !self.show_ui {
                            self.ui_panel_right_edge = 0.0;
                        }
                    }
                    PhysicalKey::Code(KeyCode::KeyC) => {
                        // Reset camera
                        self.camera.reset();
                        self.update_camera();
                    }
                    PhysicalKey::Code(KeyCode::F5) => {
                        self.toggle_recording();
                    }
                    PhysicalKey::Code(KeyCode::F11) => {
                        self.toggle_fullscreen();
                    }
                    PhysicalKey::Code(KeyCode::F12) => {
                        self.screenshot_requested = true;
                        log::info!("Screenshot requested");
                    }
                    PhysicalKey::Code(KeyCode::Escape) => {
                        event_loop.exit();
                    }
                    _ => {}
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                use winit::event::{ElementState, MouseButton};
                // Middle mouse button or right button for panning (only when not over UI)
                if button == MouseButton::Middle || button == MouseButton::Right {
                    if state == ElementState::Pressed && !self.cursor_over_ui {
                        self.camera.is_panning = true;
                    } else if state == ElementState::Released {
                        self.camera.is_panning = false;
                    }
                }
                // Left mouse button for brush interaction (except Obstacle tool)
                if button == MouseButton::Left
                    && self.brush.tool != BrushTool::None
                    && self.brush.tool != BrushTool::Obstacle
                {
                    if state == ElementState::Pressed && !self.cursor_over_ui {
                        self.brush.is_active = true;
                    } else if state == ElementState::Released {
                        self.brush.is_active = false;
                    }
                }
                // Left mouse button for obstacle tool
                if button == MouseButton::Left
                    && self.brush.tool == BrushTool::Obstacle
                    && !self.cursor_over_ui
                {
                    if state == ElementState::Pressed {
                        let hit = self.hit_test_obstacle(self.brush.position);
                        if hit >= 0 {
                            // Select existing obstacle and start drag
                            self.selected_obstacle = hit;
                            self.obstacle_dragging = true;
                            let obs = &self.app.obstacles[hit as usize];
                            self.obstacle_drag_offset = glam::Vec2::new(
                                self.brush.position.x - obs.x,
                                self.brush.position.y - obs.y,
                            );
                        } else if self.app.obstacles.len() < MAX_OBSTACLES {
                            // Place new obstacle at cursor
                            let obs = Obstacle {
                                x: self.brush.position.x,
                                y: self.brush.position.y,
                                shape: self.obstacle_tool_shape,
                                ..Obstacle::default()
                            };
                            self.app.obstacles.push(obs);
                            self.selected_obstacle = (self.app.obstacles.len() - 1) as i32;
                            self.sync_obstacles();
                        }
                    } else if state == ElementState::Released {
                        self.obstacle_dragging = false;
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = glam::Vec2::new(position.x as f32, position.y as f32);
                self.mouse_screen_pos = new_pos;

                if let Some(gpu) = &self.gpu {
                    // Convert screen position to world coordinates
                    let screen_width = gpu.context.surface_config.width as f32;
                    let screen_height = gpu.context.surface_config.height as f32;
                    let world_width = self.app.sim_config.world_size.x;
                    let world_height = self.app.sim_config.world_size.y;

                    let world_pos = self.camera.screen_to_world(
                        new_pos,
                        glam::Vec2::new(screen_width, screen_height),
                        self.app.sim_config.world_size,
                    );

                    // Update brush position with velocity calculation
                    // Use a fixed dt estimate for velocity calculation
                    self.brush.update_position(world_pos, 1.0 / 60.0);

                    // Handle obstacle dragging
                    if self.obstacle_dragging
                        && self.selected_obstacle >= 0
                        && (self.selected_obstacle as usize) < self.app.obstacles.len()
                    {
                        let idx = self.selected_obstacle as usize;
                        self.app.obstacles[idx].x =
                            self.brush.position.x - self.obstacle_drag_offset.x;
                        self.app.obstacles[idx].y =
                            self.brush.position.y - self.obstacle_drag_offset.y;
                        self.sync_obstacles();
                    }

                    if self.camera.is_panning {
                        let delta = new_pos - self.camera.last_mouse_pos;
                        // Convert screen delta to world delta
                        // Screen Y is inverted relative to world Y
                        let world_delta = glam::Vec2::new(
                            -delta.x / self.camera.zoom * (world_width / screen_width),
                            -delta.y / self.camera.zoom * (world_height / screen_height),
                        );
                        self.camera.pan(world_delta);
                        self.update_camera();
                    }
                }
                self.camera.last_mouse_pos = new_pos;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                use winit::event::MouseScrollDelta;
                let scroll_amount = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 50.0,
                };

                // Resize selected obstacle when Obstacle tool is active
                if self.brush.tool == BrushTool::Obstacle
                    && self.selected_obstacle >= 0
                    && !self.cursor_over_ui
                {
                    let idx = self.selected_obstacle as usize;
                    if idx < self.app.obstacles.len() {
                        let resize_factor = 1.0 + scroll_amount * 0.1;
                        let obs = &mut self.app.obstacles[idx];
                        obs.width = (obs.width * resize_factor).clamp(10.0, 500.0);
                        if obs.shape == ObstacleShape::Rectangle {
                            obs.height = (obs.height * resize_factor).clamp(10.0, 500.0);
                        }
                        self.sync_obstacles();
                        return;
                    }
                }

                // Zoom factor: positive scroll = zoom in
                let zoom_factor = 1.0 + scroll_amount * 0.1;
                self.camera.zoom_center(zoom_factor);
                self.update_camera();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request redraw for continuous rendering
        if let Some(gpu) = &self.gpu {
            gpu.context.window.request_redraw();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppHandler;

    #[test]
    fn fullscreen_target_enters_borderless_when_windowed() {
        assert!(AppHandler::fullscreen_target(false).is_some());
    }

    #[test]
    fn fullscreen_target_exits_when_already_fullscreen() {
        assert!(AppHandler::fullscreen_target(true).is_none());
    }
}
