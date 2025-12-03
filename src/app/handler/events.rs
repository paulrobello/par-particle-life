//! Event handling for the application.

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{WindowAttributes, WindowId},
};

use super::AppHandler;
use crate::app::BrushTool;

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

            let window = Arc::new(
                event_loop
                    .create_window(window_attrs)
                    .expect("Failed to create window"),
            );

            self.init_gpu(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Let egui handle events first
        let mut egui_wants_pointer = false;
        if let Some(gpu) = &mut self.gpu {
            let response = gpu.egui_state.on_window_event(&gpu.context.window, &event);
            egui_wants_pointer = gpu.egui_ctx.wants_pointer_input();
            if response.consumed && egui_wants_pointer {
                // Only return early if egui actually wants the pointer (over UI)
                // But still update mouse position for smooth pan resumption
                if let WindowEvent::CursorMoved { position, .. } = &event {
                    self.camera.last_mouse_pos =
                        glam::Vec2::new(position.x as f32, position.y as f32);
                }
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested, exiting...");
                // Save UI states to app.config before saving the config
                self.app.config.ui_simulation_open = self.ui_simulation_open;
                self.app.config.ui_physics_open = self.ui_physics_open;
                self.app.config.ui_generators_open = self.ui_generators_open;
                self.app.config.ui_interaction_matrix_open = self.ui_interaction_matrix_open;
                self.app.config.ui_brush_tools_open = self.ui_brush_tools_open;
                self.app.config.ui_rendering_open = self.ui_rendering_open;
                self.app.config.ui_presets_open = self.ui_presets_open;
                self.app.config.ui_keyboard_shortcuts_open = self.ui_keyboard_shortcuts_open;

                // Persist current settings
                self.app.config.sim_num_particles = self.app.sim_config.num_particles;
                self.app.config.sim_num_types = self.app.sim_config.num_types;
                self.app.config.phys_force_factor = self.app.sim_config.force_factor;
                self.app.config.phys_friction = self.app.sim_config.friction;
                self.app.config.phys_repel_strength = self.app.sim_config.repel_strength;
                self.app.config.phys_max_velocity = self.app.sim_config.max_velocity;
                self.app.config.phys_boundary_mode = self.app.sim_config.boundary_mode;
                self.app.config.phys_wall_repel_strength = self.app.sim_config.wall_repel_strength;
                self.app.config.phys_mirror_wrap_count = self.app.sim_config.mirror_wrap_count;
                self.app.config.gen_rule = self.app.current_rule;
                self.app.config.gen_palette = self.app.current_palette;
                self.app.config.gen_pattern = self.app.current_pattern;
                self.app.config.render_particle_size = self.app.sim_config.particle_size;
                self.app.config.render_background_color = self.app.sim_config.background_color;
                self.app.config.render_glow_enabled = self.app.sim_config.enable_glow;
                self.app.config.render_glow_intensity = self.app.sim_config.glow_intensity;
                self.app.config.render_glow_size = self.app.sim_config.glow_size;
                self.app.config.render_glow_steepness = self.app.sim_config.glow_steepness;
                self.app.config.render_spatial_hash_cell_size =
                    self.app.sim_config.spatial_hash_cell_size;

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
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state.is_pressed() {
                    use winit::keyboard::{KeyCode, PhysicalKey};
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Space) => {
                            self.app.toggle_running();
                        }
                        PhysicalKey::Code(KeyCode::KeyR) => {
                            self.app.regenerate_particles();
                            self.sync_buffers();
                        }
                        PhysicalKey::Code(KeyCode::KeyM) => {
                            self.app.regenerate_rules();
                            self.sync_interaction_matrix();
                        }
                        PhysicalKey::Code(KeyCode::KeyH) => {
                            self.show_ui = !self.show_ui;
                        }
                        PhysicalKey::Code(KeyCode::KeyC) => {
                            // Reset camera
                            self.camera.reset();
                            self.update_camera();
                        }
                        PhysicalKey::Code(KeyCode::F11) => {
                            self.toggle_recording();
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
            }
            WindowEvent::MouseInput { state, button, .. } => {
                use winit::event::{ElementState, MouseButton};
                // Middle mouse button or right button for panning (only when not over UI)
                if button == MouseButton::Middle || button == MouseButton::Right {
                    if state == ElementState::Pressed && !egui_wants_pointer {
                        self.camera.is_panning = true;
                    } else if state == ElementState::Released {
                        self.camera.is_panning = false;
                    }
                }
                // Left mouse button for brush interaction
                if button == MouseButton::Left && self.brush.tool != BrushTool::None {
                    if state == ElementState::Pressed && !egui_wants_pointer {
                        self.brush.is_active = true;
                    } else if state == ElementState::Released {
                        self.brush.is_active = false;
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
