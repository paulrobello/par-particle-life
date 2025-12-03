//! UI rendering using egui.

use super::AppHandler;
use crate::app::{BrushTool, Preset};
use crate::generators::{
    colors::{PaletteType, generate_colors},
    positions::PositionPattern,
    rules::{RuleType, generate_rules},
};
use crate::simulation::{BoundaryMode, RadiusMatrix};
use crate::video_recorder::VideoFormat;

impl AppHandler {
    pub(crate) fn draw_ui(&mut self, ctx: &egui::Context) {
        if !self.show_ui {
            return;
        }

        egui::SidePanel::left("controls")
            .default_width(280.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Par Particle Life");
                    ui.separator();

                    // Stats
                    ui.horizontal(|ui| {
                        ui.label(format!("FPS: {:.1}", self.fps));
                        ui.separator();
                        ui.label(format!("EMA: {:.1}", self.fps_ema));
                        ui.separator();
                        ui.label(format!("Particles: {}", self.app.particles.len()));
                    });

                    if let Some(gpu) = &self.gpu
                        && gpu.gpu_total_ms > 0.0
                    {
                        ui.label(format!("GPU (spatial): {:.2} ms", gpu.gpu_total_ms));
                        ui.collapsing("GPU pass timings", |ui| {
                            for (label, ms) in &gpu.gpu_pass_ms {
                                ui.label(format!("{:<12} {:>6.3} ms", label, ms));
                            }
                        });
                    }
                    // Window and simulation dimensions
                    let (win_w, win_h) = self
                        .gpu
                        .as_ref()
                        .map(|g| g.context.surface_size())
                        .unwrap_or((
                            self.app.sim_config.world_size.x as u32,
                            self.app.sim_config.world_size.y as u32,
                        ));
                    ui.label(format!(
                        "Window: {}x{} | World: {:.0}x{:.0}",
                        win_w,
                        win_h,
                        self.app.sim_config.world_size.x,
                        self.app.sim_config.world_size.y
                    ));
                    ui.separator();

                    // Playback controls
                    ui.horizontal(|ui| {
                        if ui
                            .button(if self.app.running {
                                "⏸ Pause"
                            } else {
                                "▶ Play"
                            })
                            .clicked()
                        {
                            self.app.toggle_running();
                        }
                        if ui.button("🔄 Reset").clicked() {
                            self.app.regenerate_particles();
                            self.sync_buffers();
                        }
                        if ui.button("🎛 Toggle Controls (H)").clicked() {
                            self.show_ui = !self.show_ui;
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("📷 Screenshot (F12)").clicked() {
                            self.screenshot_requested = true;
                            log::info!("Screenshot requested via button");
                        }
                        let record_label = if self.is_recording {
                            "⏹ Stop Recording (F11)".to_string()
                        } else {
                            format!("🔴 Record {} (F11)", self.video_format.name())
                        };
                        if ui.button(record_label).clicked() {
                            self.toggle_recording();
                        }
                    });
                    ui.checkbox(&mut self.capture_hide_ui, "Hide UI for capture");

                    // Video format selection (only when not recording)
                    ui.horizontal(|ui| {
                        ui.label("Format:");
                        let enabled = !self.is_recording;
                        ui.add_enabled_ui(enabled, |ui| {
                            for format in VideoFormat::all() {
                                if ui
                                    .selectable_label(self.video_format == *format, format.name())
                                    .clicked()
                                {
                                    self.video_format = *format;
                                }
                            }
                        });
                    });

                    // Open last capture button
                    if let Some(ref path) = self.last_capture_path {
                        ui.horizontal(|ui| {
                            ui.label(format!("Last: {}", path));
                            if ui.button("📂 Open").clicked()
                                && let Err(e) = open::that(path)
                            {
                                log::error!("Failed to open file: {}", e);
                                self.preset_status = format!("Failed to open: {}", e);
                            }
                        });
                    }
                    ui.separator();

                    // Simulation settings
                    let response = egui::CollapsingHeader::new("Simulation")
                        .id_salt("simulation_header")
                        .default_open(self.ui_simulation_open)
                        .show(ui, |ui| {
                            let mut num_particles = self.app.sim_config.num_particles;
                            let particle_options =
                                [1000u32, 2000, 4000, 8000, 16000, 32000, 64000, 128000];
                            egui::ComboBox::from_label("Particles")
                                .selected_text(format!("{}", num_particles))
                                .show_ui(ui, |ui| {
                                    for &n in &particle_options {
                                        ui.selectable_value(
                                            &mut num_particles,
                                            n,
                                            format!("{}", n),
                                        );
                                    }
                                });
                            if num_particles != self.app.sim_config.num_particles {
                                self.app.sim_config.num_particles = num_particles;
                                self.app.config.sim_num_particles = num_particles;
                                self.app.rebalance_radii_for_density();
                                self.app.regenerate_particles();
                                self.sync_buffers();
                            }

                            let mut num_types = self.app.sim_config.num_types;
                            ui.add(egui::Slider::new(&mut num_types, 2..=16).text("Types"));
                            if num_types != self.app.sim_config.num_types {
                                self.app.sim_config.num_types = num_types;
                                self.app.config.sim_num_types = num_types;
                                self.app.radius_matrix =
                                    RadiusMatrix::default_for_size(num_types as usize);
                                self.app.rebalance_radii_for_density();
                                self.app.regenerate_rules();
                                self.app.regenerate_colors();
                                self.app.regenerate_particles();
                                self.sync_buffers();
                            }

                            let mut auto_scale = self.app.auto_scale_radii;
                            let auto_changed = ui
                            .checkbox(&mut auto_scale, "Auto-scale radii by density")
                            .on_hover_text(
                                "Keeps neighbor count stable as particle count/world size changes",
                            )
                            .changed();
                            if auto_changed {
                                self.app.auto_scale_radii = auto_scale;
                                self.app.config.auto_scale_radii = auto_scale;

                                if auto_scale {
                                    self.app.rebalance_radii_for_density();
                                } else {
                                    // Reset to defaults when disabling auto-scaling
                                    self.app.radius_matrix = RadiusMatrix::default_for_size(
                                        self.app.sim_config.num_types as usize,
                                    );
                                    let max_r = self.app.radius_matrix.max_interaction_radius();
                                    self.app.sim_config.spatial_hash_cell_size =
                                        self.app.config.render_spatial_hash_cell_size.max(max_r);
                                }

                                self.sync_buffers();
                            }
                        });
                    self.ui_simulation_open = response.openness > 0.5;

                    // Physics settings
                    let response = egui::CollapsingHeader::new("Physics")
                        .id_salt("physics_header")
                        .default_open(self.ui_physics_open)
                        .show(ui, |ui| {
                            ui.add(
                                egui::Slider::new(&mut self.app.sim_config.force_factor, 0.1..=5.0)
                                    .text("Force Factor")
                                    .logarithmic(true),
                            );
                            self.app.config.phys_force_factor = self.app.sim_config.force_factor;
                            ui.add(
                                egui::Slider::new(&mut self.app.sim_config.friction, 0.0..=1.0)
                                    .text("Friction"),
                            );
                            self.app.config.phys_friction = self.app.sim_config.friction;
                            ui.add(
                                egui::Slider::new(
                                    &mut self.app.sim_config.repel_strength,
                                    0.1..=4.0,
                                )
                                .text("Repel Strength"),
                            );
                            self.app.config.phys_repel_strength =
                                self.app.sim_config.repel_strength;
                            ui.add(
                                egui::Slider::new(
                                    &mut self.app.sim_config.max_velocity,
                                    1.0..=500.0,
                                )
                                .text("Max Velocity"),
                            );
                            self.app.config.phys_max_velocity = self.app.sim_config.max_velocity;

                            // Boundary mode
                            let boundary_modes = [
                                (BoundaryMode::Repel, "Repel"),
                                (BoundaryMode::Wrap, "Wrap"),
                                (BoundaryMode::MirrorWrap, "Mirror"),
                                (BoundaryMode::InfiniteWrap, "Infinite"),
                            ];
                            let old_boundary_mode = self.app.sim_config.boundary_mode;
                            egui::ComboBox::from_label("Boundary")
                                .selected_text(match self.app.sim_config.boundary_mode {
                                    BoundaryMode::Repel => "Repel",
                                    BoundaryMode::Wrap => "Wrap",
                                    BoundaryMode::MirrorWrap => "Mirror",
                                    BoundaryMode::InfiniteWrap => "Infinite",
                                })
                                .show_ui(ui, |ui| {
                                    for (mode, name) in boundary_modes {
                                        ui.selectable_value(
                                            &mut self.app.sim_config.boundary_mode,
                                            mode,
                                            name,
                                        );
                                    }
                                });

                            // If boundary mode changed, normalize particle positions
                            if self.app.sim_config.boundary_mode != old_boundary_mode {
                                self.sync_particles_from_gpu();
                                self.normalize_particle_positions();
                                self.sync_buffers();
                            }
                            self.app.config.phys_boundary_mode = self.app.sim_config.boundary_mode;

                            // Wall repel strength (only visible in Repel mode)
                            if self.app.sim_config.boundary_mode == BoundaryMode::Repel {
                                ui.add(
                                    egui::Slider::new(
                                        &mut self.app.sim_config.wall_repel_strength,
                                        0.0..=500.0,
                                    )
                                    .text("Wall Force"),
                                );
                                self.app.config.phys_wall_repel_strength =
                                    self.app.sim_config.wall_repel_strength;
                            }

                            // Mirror wrap count (only visible in Mirror mode)
                            if self.app.sim_config.boundary_mode == BoundaryMode::MirrorWrap {
                                let mirror_options = [(5u32, "5 copies"), (9u32, "9 copies")];
                                egui::ComboBox::from_label("Mirror Count")
                                    .selected_text(format!(
                                        "{} copies",
                                        self.app.sim_config.mirror_wrap_count
                                    ))
                                    .show_ui(ui, |ui| {
                                        for (count, label) in mirror_options {
                                            ui.selectable_value(
                                                &mut self.app.sim_config.mirror_wrap_count,
                                                count,
                                                label,
                                            );
                                        }
                                    });
                                self.app.config.phys_mirror_wrap_count =
                                    self.app.sim_config.mirror_wrap_count;
                            }
                        });
                    self.ui_physics_open = response.openness > 0.5;

                    // Generators
                    let response = egui::CollapsingHeader::new("Generators")
                        .id_salt("generators_header")
                        .default_open(self.ui_generators_open)
                        .show(ui, |ui| {
                            // Rule type
                            let rule_name = format!("{:?}", self.app.current_rule);
                            let mut new_rule = self.app.current_rule;
                            egui::ComboBox::from_label("Rules")
                                .selected_text(&rule_name)
                                .show_ui(ui, |ui| {
                                    for &rule in RuleType::all() {
                                        let name = format!("{:?}", rule);
                                        ui.selectable_value(&mut new_rule, rule, name);
                                    }
                                });
                            if new_rule != self.app.current_rule {
                                self.app.current_rule = new_rule;
                                self.app.config.gen_rule = new_rule;
                                self.app.regenerate_rules();
                                self.sync_interaction_matrix();
                            }

                            if ui.button("🎲 Randomize Rules").clicked() {
                                self.app.regenerate_rules();
                                self.sync_interaction_matrix();
                            }

                            ui.separator();

                            // Palette type
                            let palette_name = format!("{:?}", self.app.current_palette);
                            let mut new_palette = self.app.current_palette;
                            egui::ComboBox::from_label("Colors")
                                .selected_text(&palette_name)
                                .show_ui(ui, |ui| {
                                    for &palette in PaletteType::all() {
                                        let name = format!("{:?}", palette);
                                        ui.selectable_value(&mut new_palette, palette, name);
                                    }
                                });
                            if new_palette != self.app.current_palette {
                                self.app.current_palette = new_palette;
                                self.app.config.gen_palette = new_palette;
                                self.app.regenerate_colors();
                                self.sync_colors();
                            }

                            ui.separator();

                            // Position pattern
                            let pattern_name = format!("{:?}", self.app.current_pattern);
                            let mut new_pattern = self.app.current_pattern;
                            egui::ComboBox::from_label("Spawn Pattern")
                                .selected_text(&pattern_name)
                                .show_ui(ui, |ui| {
                                    for &pattern in PositionPattern::all() {
                                        let name = format!("{:?}", pattern);
                                        ui.selectable_value(&mut new_pattern, pattern, name);
                                    }
                                });
                            if new_pattern != self.app.current_pattern {
                                self.app.current_pattern = new_pattern;
                                self.app.config.gen_pattern = new_pattern;

                                // Update num_types if pattern requires a fixed number
                                if let Some(required) = new_pattern.required_types() {
                                    let required = required as u32;
                                    if self.app.sim_config.num_types != required {
                                        self.app.sim_config.num_types = required;
                                        self.app.config.sim_num_types = required;
                                        self.app.radius_matrix =
                                            RadiusMatrix::default_for_size(required as usize);
                                        self.app.interaction_matrix = generate_rules(
                                            self.app.current_rule,
                                            required as usize,
                                        );
                                        self.app.colors = generate_colors(
                                            self.app.current_palette,
                                            required as usize,
                                        );
                                    }
                                }

                                self.app.regenerate_particles();
                                self.sync_buffers();
                            }
                        });
                    self.ui_generators_open = response.openness > 0.5;

                    // Matrix editor
                    let response = egui::CollapsingHeader::new("Interaction Matrix")
                        .id_salt("interaction_matrix_header")
                        .default_open(self.ui_interaction_matrix_open)
                        .show(ui, |ui| {
                            self.draw_matrix_editor(ui);
                        });
                    self.ui_interaction_matrix_open = response.openness > 0.5;

                    // Brush Tools
                    let response = egui::CollapsingHeader::new("Brush Tools")
                        .id_salt("brush_tools_header")
                        .default_open(self.ui_brush_tools_open)
                        .show(ui, |ui| {
                            self.draw_brush_tools(ui);
                        });
                    self.ui_brush_tools_open = response.openness > 0.5;

                    // Rendering settings
                    let response = egui::CollapsingHeader::new("Rendering")
                        .id_salt("rendering_header")
                        .default_open(self.ui_rendering_open)
                        .show(ui, |ui| {
                            self.draw_rendering_settings(ui);
                        });
                    self.ui_rendering_open = response.openness > 0.5;

                    // Presets
                    let response = egui::CollapsingHeader::new("Presets")
                        .id_salt("presets_header")
                        .default_open(self.ui_presets_open)
                        .show(ui, |ui| {
                            self.draw_presets_ui(ui);
                        });
                    self.ui_presets_open = response.openness > 0.5;

                    ui.separator();

                    // Keyboard shortcuts help
                    let response = egui::CollapsingHeader::new("Keyboard Shortcuts")
                        .id_salt("keyboard_shortcuts_header")
                        .default_open(self.ui_keyboard_shortcuts_open)
                        .show(ui, |ui| {
                            ui.label("Space - Pause/Resume");
                            ui.label("R - Regenerate Particles");
                            ui.label("M - New Interaction Matrix");
                            ui.label("H - Toggle UI");
                            ui.label("Escape - Quit");
                        });
                    self.ui_keyboard_shortcuts_open = response.openness > 0.5;
                });
            });
    }

    fn draw_brush_tools(&mut self, ui: &mut egui::Ui) {
        // Tool selection
        ui.horizontal(|ui| {
            for &tool in BrushTool::all() {
                let selected = self.brush.tool == tool;
                let text = if tool == BrushTool::Erase {
                    format!("{} {}", tool.icon(), tool.name())
                } else {
                    tool.name().to_string()
                };
                if ui.selectable_label(selected, text).clicked() {
                    self.brush.tool = tool;
                }
            }
        });

        if self.brush.tool != BrushTool::None {
            ui.separator();

            // Brush radius
            ui.add(
                egui::Slider::new(&mut self.brush.radius, 20.0..=500.0)
                    .text("Radius")
                    .logarithmic(true),
            );

            // Force settings
            if self.brush.tool == BrushTool::Attract {
                ui.add(
                    egui::Slider::new(&mut self.brush.attract_force, 1.0..=100.0)
                        .text("Attract Force"),
                );
            } else if self.brush.tool == BrushTool::Repel {
                ui.add(
                    egui::Slider::new(&mut self.brush.repel_force, 1.0..=100.0).text("Repel Force"),
                );
            } else if self.brush.tool == BrushTool::Draw {
                ui.add(
                    egui::Slider::new(&mut self.brush.draw_intensity, 1..=200).text("Intensity"),
                );

                // Type selector for Draw tool
                let num_types = self.app.sim_config.num_types as i32;
                let type_label = if self.brush.draw_type < 0 {
                    "Random".to_string()
                } else {
                    format!("Type {}", self.brush.draw_type)
                };
                egui::ComboBox::from_label("Draw Type")
                    .selected_text(type_label)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.brush.draw_type, -1, "Random");
                        for i in 0..num_types {
                            // Show color swatch with type number
                            let color = self.app.colors[i as usize];
                            let label = format!("Type {}", i);
                            ui.horizontal(|ui| {
                                let size = egui::vec2(12.0, 12.0);
                                let (response, painter) =
                                    ui.allocate_painter(size, egui::Sense::hover());
                                painter.rect_filled(
                                    response.rect,
                                    2.0,
                                    egui::Color32::from_rgb(
                                        (color[0] * 255.0) as u8,
                                        (color[1] * 255.0) as u8,
                                        (color[2] * 255.0) as u8,
                                    ),
                                );
                                if ui
                                    .selectable_label(self.brush.draw_type == i, label)
                                    .clicked()
                                {
                                    self.brush.draw_type = i;
                                }
                            });
                        }
                    });
            } else if self.brush.tool == BrushTool::Erase {
                // Type selector for Erase tool
                let num_types = self.app.sim_config.num_types as i32;
                let type_label = if self.brush.target_type < 0 {
                    "All".to_string()
                } else {
                    format!("Type {}", self.brush.target_type)
                };
                egui::ComboBox::from_label("Target Type")
                    .selected_text(type_label)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.brush.target_type, -1, "All");
                        for i in 0..num_types {
                            // Show color swatch with type number
                            let color = self.app.colors[i as usize];
                            let label = format!("Type {}", i);
                            ui.horizontal(|ui| {
                                let size = egui::vec2(12.0, 12.0);
                                let (response, painter) =
                                    ui.allocate_painter(size, egui::Sense::hover());
                                painter.rect_filled(
                                    response.rect,
                                    2.0,
                                    egui::Color32::from_rgb(
                                        (color[0] * 255.0) as u8,
                                        (color[1] * 255.0) as u8,
                                        (color[2] * 255.0) as u8,
                                    ),
                                );
                                if ui
                                    .selectable_label(self.brush.target_type == i, label)
                                    .clicked()
                                {
                                    self.brush.target_type = i;
                                }
                            });
                        }
                    });
            }

            // Directional force (for attract/repel)
            if matches!(self.brush.tool, BrushTool::Attract | BrushTool::Repel) {
                ui.add(
                    egui::Slider::new(&mut self.brush.directional_force, 0.0..=100.0)
                        .text("Directional"),
                );
            }

            // Show circle toggle
            ui.checkbox(&mut self.brush.show_circle, "Show Circle");

            ui.separator();
            ui.label("Left-click to use brush");
        }
    }

    fn draw_rendering_settings(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::Slider::new(&mut self.app.sim_config.particle_size, 0.1..=2.0)
                .text("Particle Size"),
        );
        self.app.config.render_particle_size = self.app.sim_config.particle_size;

        ui.horizontal(|ui| {
            ui.label("Background");
            ui.color_edit_button_rgb(&mut self.app.sim_config.background_color);
        });
        self.app.config.render_background_color = self.app.sim_config.background_color;

        let vsync_changed = ui
            .checkbox(&mut self.app.config.vsync, "VSync (present)")
            .changed();
        if vsync_changed {
            self.pending_vsync = Some(self.app.config.vsync);
        }

        ui.separator();

        // Spatial hashing is mandatory
        self.app.sim_config.use_spatial_hash = true;
        ui.horizontal(|ui| {
            ui.label("Spatial Hash (always on)");
            ui.label("(O(n·k))");
        });

        // Cell size must be >= max interaction radius for correct spatial hashing
        let min_cell_size = self.app.radius_matrix.max_interaction_radius().max(20.0);
        ui.add(
            egui::Slider::new(
                &mut self.app.sim_config.spatial_hash_cell_size,
                min_cell_size..=200.0,
            )
            .text("Cell Size"),
        );
        self.app.config.render_spatial_hash_cell_size = self.app.sim_config.spatial_hash_cell_size;

        ui.separator();

        // Glow effect toggle
        ui.checkbox(&mut self.app.sim_config.enable_glow, "Glow Effect");
        self.app.config.render_glow_enabled = self.app.sim_config.enable_glow;

        if self.app.sim_config.enable_glow {
            ui.add(
                egui::Slider::new(&mut self.app.sim_config.glow_intensity, 0.1..=2.0)
                    .text("Intensity"),
            );
            self.app.config.render_glow_intensity = self.app.sim_config.glow_intensity;
            ui.add(
                egui::Slider::new(&mut self.app.sim_config.glow_size, 1.0..=8.0).text("Glow Size"),
            );
            self.app.config.render_glow_size = self.app.sim_config.glow_size;
            ui.add(
                egui::Slider::new(&mut self.app.sim_config.glow_steepness, 0.5..=4.0)
                    .text("Steepness"),
            );
            self.app.config.render_glow_steepness = self.app.sim_config.glow_steepness;
        }
    }

    fn draw_presets_ui(&mut self, ui: &mut egui::Ui) {
        // Status message
        if !self.preset_status.is_empty() {
            ui.label(&self.preset_status);
            ui.separator();
        }

        // Load section
        ui.label("Load preset:");
        ui.horizontal(|ui| {
            let selected = if self.selected_preset.is_empty() {
                "Select..."
            } else {
                &self.selected_preset
            };

            egui::ComboBox::from_id_salt("preset_select")
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    for preset_name in &self.preset_list.clone() {
                        ui.selectable_value(
                            &mut self.selected_preset,
                            preset_name.clone(),
                            preset_name,
                        );
                    }
                });

            if ui.button("Load").clicked() && !self.selected_preset.is_empty() {
                let name = self.selected_preset.clone();
                self.load_preset(&name);
            }
        });

        if ui.button("🔄 Refresh List").clicked() {
            self.refresh_presets();
        }

        ui.separator();

        // Save section
        ui.label("Save preset:");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.save_preset_name);
            if ui.button("Save").clicked() && !self.save_preset_name.is_empty() {
                let name = self.save_preset_name.clone();
                self.save_preset(&name);
            }
        });

        ui.separator();

        // Show presets directory
        if ui.button("📁 Open Presets Folder").clicked() {
            let dir = Preset::presets_dir();
            if let Err(e) = Preset::ensure_presets_dir() {
                self.preset_status = format!("Error: {}", e);
            } else {
                #[cfg(target_os = "macos")]
                let _ = std::process::Command::new("open").arg(&dir).spawn();
                #[cfg(target_os = "linux")]
                let _ = std::process::Command::new("xdg-open").arg(&dir).spawn();
                #[cfg(target_os = "windows")]
                let _ = std::process::Command::new("explorer").arg(&dir).spawn();
            }
        }

        ui.separator();

        if ui.button("Reset All Settings to Defaults").clicked() {
            self.reset_to_defaults();
        }
    }

    pub(crate) fn draw_matrix_editor(&mut self, ui: &mut egui::Ui) {
        let num_types = self.app.sim_config.num_types as usize;
        let cell_size = 18.0;
        let spacing = 2.0;

        ui.label("Scroll over cells to edit attraction/repulsion:");
        ui.add_space(4.0);

        // Calculate total size
        let total_size = (cell_size + spacing) * num_types as f32 + 20.0; // +20 for labels

        // Matrix grid
        let (response, painter) =
            ui.allocate_painter(egui::vec2(total_size, total_size), egui::Sense::click());

        let rect = response.rect;
        let origin = rect.min + egui::vec2(20.0, 20.0); // Offset for labels

        // Draw column labels (colors)
        for j in 0..num_types {
            let x = origin.x + (j as f32) * (cell_size + spacing) + cell_size / 2.0;
            let y = origin.y - 10.0;
            let color = self.app.colors[j];
            let egui_color = egui::Color32::from_rgba_unmultiplied(
                (color[0] * 255.0) as u8,
                (color[1] * 255.0) as u8,
                (color[2] * 255.0) as u8,
                255,
            );
            painter.circle_filled(egui::pos2(x, y), 5.0, egui_color);
        }

        // Draw row labels (colors)
        for i in 0..num_types {
            let x = origin.x - 10.0;
            let y = origin.y + (i as f32) * (cell_size + spacing) + cell_size / 2.0;
            let color = self.app.colors[i];
            let egui_color = egui::Color32::from_rgba_unmultiplied(
                (color[0] * 255.0) as u8,
                (color[1] * 255.0) as u8,
                (color[2] * 255.0) as u8,
                255,
            );
            painter.circle_filled(egui::pos2(x, y), 5.0, egui_color);
        }

        // Track if we need to update the matrix
        let mut matrix_changed = false;

        // Draw cells and handle clicks
        for i in 0..num_types {
            for j in 0..num_types {
                let x = origin.x + (j as f32) * (cell_size + spacing);
                let y = origin.y + (i as f32) * (cell_size + spacing);
                let cell_rect =
                    egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_size, cell_size));

                // Get interaction value (-1 to 1)
                let value = self.app.interaction_matrix.get(i, j);

                // Color based on value: red for negative, green for positive, gray for zero
                let cell_color = if value > 0.0 {
                    // Green for attraction
                    let intensity = (value * 200.0) as u8;
                    egui::Color32::from_rgb(0, 80 + intensity, 0)
                } else if value < 0.0 {
                    // Red for repulsion
                    let intensity = (-value * 200.0) as u8;
                    egui::Color32::from_rgb(80 + intensity, 0, 0)
                } else {
                    // Gray for neutral
                    egui::Color32::from_gray(60)
                };

                painter.rect_filled(cell_rect, 2.0, cell_color);

                // Highlight on hover
                if cell_rect.contains(response.hover_pos().unwrap_or(egui::pos2(-100.0, -100.0))) {
                    painter.rect_stroke(
                        cell_rect,
                        2.0,
                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                        egui::StrokeKind::Outside,
                    );

                    // Handle scroll wheel to change value
                    // Cycles through -1 -> 0 -> 1 so neutral (0) is between attract and repel
                    let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
                    if scroll_delta != 0.0 {
                        let new_value = if scroll_delta > 0.0 {
                            // Scroll up: -1 -> 0 -> 1
                            if value < -0.5 {
                                0.0
                            } else {
                                1.0 // Already at 0 or max
                            }
                        } else {
                            // Scroll down: 1 -> 0 -> -1
                            if value > 0.5 {
                                0.0
                            } else {
                                -1.0 // Already at 0 or min
                            }
                        };
                        self.app.interaction_matrix.set(i, j, new_value);
                        matrix_changed = true;
                    }

                    // Show tooltip using on_hover_ui
                    response.clone().on_hover_ui_at_pointer(|ui| {
                        ui.label(format!("Type {} -> Type {}", i, j));
                        ui.label(format!("Value: {:.2}", value));
                        ui.label("Scroll to change value");
                    });
                }
            }
        }

        // Update GPU buffers if matrix changed
        if matrix_changed {
            self.sync_interaction_matrix();
        }

        ui.add_space(4.0);

        // Legend
        ui.horizontal(|ui| {
            let legend_size = egui::vec2(12.0, 12.0);
            ui.label("Legend:");

            let (rect, _) = ui.allocate_exact_size(legend_size, egui::Sense::hover());
            ui.painter()
                .rect_filled(rect, 2.0, egui::Color32::from_rgb(0, 200, 0));
            ui.label("Attract");

            let (rect, _) = ui.allocate_exact_size(legend_size, egui::Sense::hover());
            ui.painter()
                .rect_filled(rect, 2.0, egui::Color32::from_rgb(200, 0, 0));
            ui.label("Repel");
        });
    }
}
