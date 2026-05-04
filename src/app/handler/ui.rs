//! UI rendering using egui.

use super::{AppHandler, RuleSelection};
use crate::app::{BrushTool, Preset};
use crate::generators::{
    colors::PaletteType,
    positions::PositionPattern,
    rules::{RuleType, generate_rules},
};
use crate::simulation::{
    BoundaryMode, IntegrationMethod, MatrixVariationMode, ObstacleShape, RadiusMatrix,
};
use crate::video_recorder::VideoFormat;

impl AppHandler {
    pub(crate) fn draw_ui(&mut self, ctx: &egui::Context) {
        if !self.show_ui {
            return;
        }

        #[allow(deprecated)]
        egui::Panel::left("controls")
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
                        if ui.button("⛶ Fullscreen (F11)").clicked() {
                            self.toggle_fullscreen();
                        }
                    });
                    ui.horizontal(|ui| {
                        let record_label = if self.is_recording {
                            "⏹ Stop Recording (F5)".to_string()
                        } else {
                            format!("🔴 Record {} (F5)", self.video_format.name())
                        };
                        if ui.button(record_label).clicked() {
                            self.toggle_recording();
                        }
                    });
                    ui.checkbox(&mut self.capture_hide_ui, "Hide UI for capture");

                    // Speed control
                    ui.add(
                        egui::Slider::new(&mut self.app.sim_config.time_scale, 0.1..=5.0)
                            .text("Speed")
                            .logarithmic(true),
                    );
                    self.app.config.phys_time_scale = self.app.sim_config.time_scale;

                    // Step button (only when paused)
                    if !self.app.running && ui.button("⏭ Step").clicked() {
                        self.step_requested = true;
                    }

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
                                })
                                .response
                                .on_hover_text("Total number of particles in the simulation");
                            if num_particles != self.app.sim_config.num_particles {
                                self.hot_swap_particle_count(num_particles);
                            }

                            let mut num_types = self.app.sim_config.num_types;
                            ui.add(egui::Slider::new(&mut num_types, 2..=16).text("Types"))
                                .on_hover_text("Number of distinct particle types");
                            if num_types != self.app.sim_config.num_types {
                                self.app.sim_config.num_types = num_types;
                                self.app.config.sim_num_types = num_types;
                                self.app.radius_matrix =
                                    RadiusMatrix::default_for_size(num_types as usize);
                                self.app.rebalance_radii_for_density();
                                self.app.regenerate_rules();
                                self.app.regenerate_colors();
                                self.app.regenerate_particles();
                                self.app.resize_per_type_arrays();
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
                            )
                            .on_hover_text("Global multiplier for all inter-particle forces");
                            self.app.config.phys_force_factor = self.app.sim_config.force_factor;
                            ui.add(
                                egui::Slider::new(&mut self.app.sim_config.friction, 0.0..=1.0)
                                    .text("Friction"),
                            )
                            .on_hover_text("Velocity damping per frame: 0 = full stop, 1 = no friction");
                            self.app.config.phys_friction = self.app.sim_config.friction;
                            ui.add(
                                egui::Slider::new(
                                    &mut self.app.sim_config.repel_strength,
                                    0.1..=4.0,
                                )
                                .text("Repel Strength"),
                            )
                            .on_hover_text("Short-range repulsion to prevent particle overlap");
                            self.app.config.phys_repel_strength =
                                self.app.sim_config.repel_strength;
                            ui.add(
                                egui::Slider::new(
                                    &mut self.app.sim_config.max_velocity,
                                    1.0..=500.0,
                                )
                                .text("Max Velocity"),
                            )
                            .on_hover_text("Maximum particle speed per frame");
                            self.app.config.phys_max_velocity = self.app.sim_config.max_velocity;

                            // Temperature slider
                            ui.add(
                                egui::Slider::new(&mut self.app.sim_config.temperature, 0.0..=50.0)
                                    .text("Temperature"),
                            )
                            .on_hover_text("Random jitter added to particle velocities each frame");
                            self.app.config.phys_temperature = self.app.sim_config.temperature;

                            ui.add(
                                egui::Slider::new(
                                    &mut self.app.sim_config.velocity_coupling,
                                    0.0..=1.0,
                                )
                                .text("Velocity Coupling"),
                            )
                            .on_hover_text(
                                "Boid-like velocity alignment with nearby particles: 0 = none, 1 = strong flocking",
                            );
                            self.app.config.phys_velocity_coupling =
                                self.app.sim_config.velocity_coupling;

                            egui::ComboBox::from_label("Integration")
                                .selected_text(self.app.sim_config.integration_method.display_name())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.app.sim_config.integration_method,
                                        IntegrationMethod::Euler,
                                        IntegrationMethod::Euler.display_name(),
                                    );
                                    ui.selectable_value(
                                        &mut self.app.sim_config.integration_method,
                                        IntegrationMethod::VelocityVerlet,
                                        IntegrationMethod::VelocityVerlet.display_name(),
                                    );
                                })
                                .response
                                .on_hover_text("Velocity Verlet uses the average of old and updated velocity for smoother position updates");
                            self.app.config.phys_integration_method =
                                self.app.sim_config.integration_method;

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

                            // Per-type mass
                            let num_types = self.app.sim_config.num_types as usize;
                            let mass_response = egui::CollapsingHeader::new("Per-Type Mass")
                                .id_salt("per_type_mass")
                                .default_open(false)
                                .show(ui, |ui| {
                                    let mut changed = false;
                                    for i in 0..num_types {
                                        if i < self.app.type_masses.len() {
                                            let r = ui.add(
                                                egui::Slider::new(
                                                    &mut self.app.type_masses[i],
                                                    0.1..=10.0,
                                                )
                                                .text(format!("Type {}", i)),
                                            );
                                            changed = changed || r.changed();
                                        }
                                    }
                                    changed
                                });
                            if mass_response.body_returned.unwrap_or(false) {
                                self.sync_type_masses();
                            }
                        });
                    self.ui_physics_open = response.openness > 0.5;

                    // Generators
                    let response = egui::CollapsingHeader::new("Generators")
                        .id_salt("generators_header")
                        .default_open(self.ui_generators_open)
                        .show(ui, |ui| {
                            // Rule type
                            let rule_label = match &self.rule_selection {
                                RuleSelection::BuiltIn(rt) => rt.display_name().to_owned(),
                                RuleSelection::Custom(idx) => {
                                    self.app.custom_generators.get(*idx)
                                        .map(|g| g.name.clone())
                                        .unwrap_or_else(|| "Custom (missing)".into())
                                }
                            };

                            let mut selection_changed = false;
                            let mut new_selection = self.rule_selection.clone();

                            egui::ComboBox::from_label("Rules")
                                .selected_text(&rule_label)
                                .show_ui(ui, |ui| {
                                    for &rule in RuleType::all() {
                                        let name = rule.display_name();
                                        let selected = matches!(&self.rule_selection, RuleSelection::BuiltIn(rt) if *rt == rule);
                                        if ui.selectable_label(selected, name).clicked() {
                                            new_selection = RuleSelection::BuiltIn(rule);
                                            selection_changed = true;
                                        }
                                    }

                                    if !self.app.custom_generators.is_empty() {
                                        ui.separator();
                                        for (idx, generator) in self.app.custom_generators.iter().enumerate() {
                                            let selected = matches!(&self.rule_selection, RuleSelection::Custom(i) if *i == idx);
                                            if ui.selectable_label(selected, &generator.name).clicked() {
                                                new_selection = RuleSelection::Custom(idx);
                                                selection_changed = true;
                                            }
                                        }
                                    }
                                });

                            if selection_changed {
                                self.rule_selection = new_selection;
                                match &self.rule_selection {
                                    RuleSelection::BuiltIn(rule) => {
                                        self.app.current_rule = *rule;
                                        self.app.config.gen_rule = *rule;
                                        self.app.regenerate_rules();
                                        self.sync_interaction_matrix();
                                    }
                                    RuleSelection::Custom(idx) => {
                                        match self.app.generate_custom_rules(*idx) {
                                            Ok(matrix) => {
                                                self.app.interaction_matrix = matrix;
                                                self.app.capture_matrix_variation_base();
                                                self.sync_interaction_matrix();
                                                self.preset_status.clear();
                                            }
                                            Err(e) => {
                                                self.preset_status = format!("Custom generator error: {e}");
                                            }
                                        }
                                    }
                                }
                            }

                            if ui.button("🎲 Randomize Rules").clicked() {
                                self.app.regenerate_rules();
                                self.sync_interaction_matrix();
                            }

                            ui.collapsing("Time Variation", |ui| {
                                let was_enabled = self.app.matrix_variation.enabled;
                                if ui
                                    .checkbox(&mut self.app.matrix_variation.enabled, "Animate matrix")
                                    .on_hover_text("Continuously varies a preserved base interaction matrix")
                                    .changed()
                                {
                                    self.app.config.gen_matrix_variation_enabled =
                                        self.app.matrix_variation.enabled;
                                    if self.app.matrix_variation.enabled && !was_enabled {
                                        self.app.capture_matrix_variation_base();
                                    }
                                    if !self.app.matrix_variation.enabled {
                                        self.app.interaction_matrix =
                                            self.app.matrix_variation_base.clone();
                                        self.sync_interaction_matrix();
                                    }
                                }

                                let mut mode = self.app.matrix_variation.mode;
                                egui::ComboBox::from_label("Mode")
                                    .selected_text(mode.display_name())
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut mode,
                                            MatrixVariationMode::Oscillate,
                                            MatrixVariationMode::Oscillate.display_name(),
                                        );
                                        ui.selectable_value(
                                            &mut mode,
                                            MatrixVariationMode::Drift,
                                            MatrixVariationMode::Drift.display_name(),
                                        );
                                    });
                                if mode != self.app.matrix_variation.mode {
                                    self.app.matrix_variation.mode = mode;
                                    self.app.config.gen_matrix_variation_mode = mode;
                                }

                                ui.add(
                                    egui::Slider::new(
                                        &mut self.app.matrix_variation.amplitude,
                                        0.0..=0.75,
                                    )
                                    .text("Amplitude"),
                                );
                                ui.add(
                                    egui::Slider::new(&mut self.app.matrix_variation.speed, 0.01..=3.0)
                                        .text("Speed"),
                                );
                                self.app.config.gen_matrix_variation_amplitude =
                                    self.app.matrix_variation.amplitude;
                                self.app.config.gen_matrix_variation_speed =
                                    self.app.matrix_variation.speed;

                                if ui.button("Use current matrix as base").clicked() {
                                    self.app.capture_matrix_variation_base();
                                }
                            });

                            ui.horizontal(|ui| {
                                if ui.button("Open Custom Generators").clicked()
                                    && let Ok(dir) =
                                        crate::generators::custom::CustomGenerator::ensure_dir()
                                {
                                    #[cfg(target_os = "macos")]
                                    let _ = std::process::Command::new("open").arg(&dir).spawn();
                                    #[cfg(target_os = "linux")]
                                    let _ = std::process::Command::new("xdg-open")
                                        .arg(&dir)
                                        .spawn();
                                    #[cfg(target_os = "windows")]
                                    let _ = std::process::Command::new("explorer")
                                        .arg(&dir)
                                        .spawn();
                                }
                                if ui.button("Reload").clicked() {
                                    self.app.custom_generators = crate::generators::custom::CustomGenerator::list()
                                        .unwrap_or_default();
                                }
                            });

                            ui.separator();

                            // Palette type
                            let palette_name = self
                                .app
                                .active_custom_palette
                                .as_ref()
                                .map(|palette| format!("Custom: {}", palette.name))
                                .unwrap_or_else(|| self.app.current_palette.display_name().to_string());
                            let mut new_palette = self.app.current_palette;
                            egui::ComboBox::from_label("Colors")
                                .selected_text(&palette_name)
                                .show_ui(ui, |ui| {
                                    for &palette in PaletteType::all() {
                                        ui.selectable_value(
                                            &mut new_palette,
                                            palette,
                                            palette.display_name(),
                                        );
                                    }
                                });
                            if new_palette != self.app.current_palette {
                                self.app.current_palette = new_palette;
                                self.app.config.gen_palette = new_palette;
                                self.app.clear_custom_palette();
                                self.sync_colors();
                            }
                            if self.app.active_custom_palette.is_some()
                                && ui.button("Use Built-in Palette").clicked()
                            {
                                self.app.clear_custom_palette();
                                self.sync_colors();
                            }

                            ui.collapsing("Custom Palettes", |ui| {
                                let mut changed = false;
                                for i in 0..self.app.colors.len() {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("Type {i}"));
                                        let mut rgb = [
                                            self.app.colors[i][0],
                                            self.app.colors[i][1],
                                            self.app.colors[i][2],
                                        ];
                                        if ui.color_edit_button_rgb(&mut rgb).changed() {
                                            self.app.colors[i][0] = rgb[0];
                                            self.app.colors[i][1] = rgb[1];
                                            self.app.colors[i][2] = rgb[2];
                                            self.app.colors[i][3] = 1.0;
                                            changed = true;
                                        }
                                    });
                                }
                                if changed {
                                    self.app.active_custom_palette = crate::generators::colors::CustomPalette::new(
                                        "Unsaved Custom Palette",
                                        self.app.colors.clone(),
                                    )
                                    .ok();
                                    self.sync_colors();
                                }

                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.text_edit_singleline(&mut self.save_custom_palette_name);
                                    if ui.button("Save Palette").clicked()
                                        && !self.save_custom_palette_name.is_empty()
                                    {
                                        let name = self.save_custom_palette_name.clone();
                                        self.save_custom_palette(&name);
                                    }
                                });

                                ui.horizontal(|ui| {
                                    let selected = if self.selected_custom_palette.is_empty() {
                                        "Select palette..."
                                    } else {
                                        &self.selected_custom_palette
                                    };
                                    egui::ComboBox::from_id_salt("custom_palette_select")
                                        .selected_text(selected)
                                        .show_ui(ui, |ui| {
                                            for palette in &self.app.custom_palettes.clone() {
                                                ui.selectable_value(
                                                    &mut self.selected_custom_palette,
                                                    palette.name.clone(),
                                                    &palette.name,
                                                );
                                            }
                                        });

                                    if ui.button("Load Palette").clicked()
                                        && !self.selected_custom_palette.is_empty()
                                    {
                                        let name = self.selected_custom_palette.clone();
                                        self.load_custom_palette(&name);
                                    }
                                    if ui.button("Reload").clicked() {
                                        self.refresh_custom_palettes();
                                    }
                                });
                            });

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
                                        self.app.capture_matrix_variation_base();
                                        self.app.regenerate_colors();
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

                    // Obstacles
                    if !self.app.obstacles.is_empty() {
                    let response = egui::CollapsingHeader::new("Obstacles")
                        .id_salt("obstacles_header")
                        .default_open(self.ui_obstacles_open)
                        .show(ui, |ui| {
                            if self.app.obstacles.is_empty() {
                                ui.label(
                                    egui::RichText::new("Select Obstacle tool to place")
                                        .small(),
                                );
                            }

                            let mut to_delete: Option<usize> = None;
                            let mut changed = false;
                            for (i, obs) in self.app.obstacles.iter_mut().enumerate() {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        let is_selected = self.selected_obstacle == i as i32;
                                        let label_prefix = if is_selected { "> " } else { "  " };
                                        let shape_name = match obs.shape {
                                            ObstacleShape::Circle => "Circle",
                                            ObstacleShape::Rectangle => "Rect",
                                        };
                                        // Draw shape icon
                                        let icon_size = egui::vec2(14.0, 14.0);
                                        let (icon_rect, _icon_resp) =
                                            ui.allocate_exact_size(icon_size, egui::Sense::hover());
                                        let painter = ui.painter();
                                        let icon_color = if is_selected {
                                            egui::Color32::from_rgb(255, 220, 100)
                                        } else {
                                            egui::Color32::from_rgb(255, 100, 100)
                                        };
                                        match obs.shape {
                                            ObstacleShape::Circle => {
                                                painter.circle_filled(
                                                    icon_rect.center(),
                                                    icon_rect.width() / 2.0 - 1.0,
                                                    icon_color,
                                                );
                                            }
                                            ObstacleShape::Rectangle => {
                                                let inner = icon_rect.shrink(1.0);
                                                painter.rect_filled(
                                                    inner,
                                                    1.0,
                                                    icon_color,
                                                );
                                            }
                                        }
                                        ui.label(format!(
                                            "{}{}. {}",
                                            label_prefix, i, shape_name
                                        ));
                                        if ui.small_button("✕").clicked() {
                                            to_delete = Some(i);
                                        }
                                    });
                                    changed |= ui
                                        .add(
                                            egui::Slider::new(&mut obs.bounce, 0.0..=1.0)
                                                .text("Bounce"),
                                        )
                                        .on_hover_text(
                                            "Restitution: 0 = absorb impact, 1 = full bounce",
                                        )
                                        .changed();
                                });
                            }
                            if let Some(idx) = to_delete {
                                self.app.obstacles.remove(idx);
                                // Adjust selection after deletion
                                if self.selected_obstacle == idx as i32 {
                                    self.selected_obstacle = -1;
                                } else if self.selected_obstacle > idx as i32 {
                                    self.selected_obstacle -= 1;
                                }
                                changed = true;
                            }
                            changed
                        });
                    self.ui_obstacles_open = response.openness > 0.5;
                    if response.body_returned.unwrap_or(false) {
                        self.sync_obstacles();
                    }
                    }

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
                            ui.label("F5 - Start/Stop Recording");
                            ui.label("F11 - Toggle Fullscreen");
                            ui.label("F12 - Screenshot");
                            ui.label("Escape - Quit");
                        });
                    self.ui_keyboard_shortcuts_open = response.openness > 0.5;
                });
            });

        // Draw obstacle overlays on top of GPU render
        let show_obstacle_overlay =
            !self.app.obstacles.is_empty() || self.brush.tool == BrushTool::Obstacle;
        if show_obstacle_overlay {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("obstacles"),
            ));

            let screen_size = self
                .gpu
                .as_ref()
                .map(|g| {
                    let (w, h) = g.context.surface_size();
                    glam::Vec2::new(w as f32, h as f32)
                })
                .unwrap_or(glam::Vec2::new(
                    self.app.sim_config.world_size.x,
                    self.app.sim_config.world_size.y,
                ));
            let world_size = self.app.sim_config.world_size;

            for (i, obs) in self.app.obstacles.iter().enumerate() {
                // World to screen conversion (inverse of CameraState::screen_to_world)
                let norm_x =
                    ((obs.x - self.camera.offset.x) * 2.0 / world_size.x - 1.0) * self.camera.zoom;
                let norm_y =
                    ((obs.y - self.camera.offset.y) * 2.0 / world_size.y - 1.0) * self.camera.zoom;
                let sx = (norm_x + 1.0) * 0.5 * screen_size.x;
                let sy = (norm_y + 1.0) * 0.5 * screen_size.y;
                let center = egui::pos2(sx, sy);

                let is_selected =
                    self.brush.tool == BrushTool::Obstacle && self.selected_obstacle == i as i32;
                let fill = if is_selected {
                    egui::Color32::from_rgba_unmultiplied(255, 200, 50, 100)
                } else {
                    egui::Color32::from_rgba_unmultiplied(255, 50, 50, 80)
                };
                let stroke = if is_selected {
                    egui::Color32::from_rgba_unmultiplied(255, 220, 100, 200)
                } else {
                    egui::Color32::from_rgba_unmultiplied(255, 100, 100, 120)
                };
                let stroke_width = if is_selected { 3.0 } else { 2.0 };

                match obs.shape {
                    ObstacleShape::Circle => {
                        let screen_radius =
                            (obs.width / 2.0) * self.camera.zoom * screen_size.x / world_size.x;
                        painter.circle_filled(center, screen_radius, fill);
                        painter.circle_stroke(
                            center,
                            screen_radius,
                            egui::Stroke::new(stroke_width, stroke),
                        );
                    }
                    ObstacleShape::Rectangle => {
                        let sw = obs.width * self.camera.zoom * screen_size.x / world_size.x;
                        let sh = obs.height * self.camera.zoom * screen_size.y / world_size.y;
                        let rect = egui::Rect::from_center_size(center, egui::vec2(sw, sh));
                        painter.rect_filled(rect, 2.0, fill);
                        painter.rect_stroke(
                            rect,
                            2.0,
                            egui::Stroke::new(stroke_width, stroke),
                            egui::StrokeKind::Outside,
                        );
                    }
                }
            }

            // Draw shape preview cursor for Obstacle tool placement
            if self.brush.tool == BrushTool::Obstacle
                && !self.cursor_over_ui
                && !self.obstacle_dragging
            {
                let cursor_world = self.brush.position;
                let norm_cx = ((cursor_world.x - self.camera.offset.x) * 2.0 / world_size.x - 1.0)
                    * self.camera.zoom;
                let norm_cy = ((cursor_world.y - self.camera.offset.y) * 2.0 / world_size.y - 1.0)
                    * self.camera.zoom;
                let cx = (norm_cx + 1.0) * 0.5 * screen_size.x;
                let cy = (norm_cy + 1.0) * 0.5 * screen_size.y;
                let center = egui::pos2(cx, cy);
                let preview_color = egui::Color32::from_rgba_unmultiplied(255, 200, 50, 150);
                let preview_stroke = egui::Color32::from_rgba_unmultiplied(255, 220, 100, 220);
                let default_half = 50.0_f32; // half of default width (100)
                let screen_scale = self.camera.zoom * screen_size.x / world_size.x;
                let preview_r = default_half * screen_scale;

                match self.obstacle_tool_shape {
                    ObstacleShape::Circle => {
                        painter.circle_filled(center, preview_r, preview_color);
                        painter.circle_stroke(
                            center,
                            preview_r,
                            egui::Stroke::new(2.0, preview_stroke),
                        );
                    }
                    ObstacleShape::Rectangle => {
                        let rect = egui::Rect::from_center_size(
                            center,
                            egui::vec2(preview_r * 2.0, preview_r * 2.0),
                        );
                        painter.rect_filled(rect, 2.0, preview_color);
                        painter.rect_stroke(
                            rect,
                            2.0,
                            egui::Stroke::new(2.0, preview_stroke),
                            egui::StrokeKind::Outside,
                        );
                    }
                }
            }
        }

        // Store the panel's right edge in screen pixels for cursor tracking.
        // Uses the available rect (area not occupied by the panel).
        #[allow(deprecated)]
        let available = ctx.available_rect();
        let pixels_per_point = ctx.pixels_per_point();
        self.ui_panel_right_edge = available.left() * pixels_per_point;
    }

    fn draw_brush_tools(&mut self, ui: &mut egui::Ui) {
        // Tool selection
        ui.horizontal(|ui| {
            for &tool in BrushTool::all() {
                let selected = self.brush.tool == tool;
                let text = tool.name().to_string();
                let resp = ui
                    .selectable_label(selected, text)
                    .on_hover_text(match tool {
                        BrushTool::None => "No brush tool active",
                        BrushTool::Draw => "Add particles at cursor",
                        BrushTool::Erase => "Remove particles at cursor",
                        BrushTool::Attract => "Pull particles toward cursor",
                        BrushTool::Repel => "Push particles away from cursor",
                        BrushTool::Obstacle => "Place and edit obstacles",
                    });
                if resp.clicked() {
                    self.brush.tool = tool;
                    self.selected_obstacle = -1;
                    self.obstacle_dragging = false;
                }
            }
        });

        if self.brush.tool != BrushTool::None {
            ui.separator();

            // Brush radius (not for Obstacle tool)
            if self.brush.tool != BrushTool::Obstacle {
                ui.add(
                    egui::Slider::new(&mut self.brush.radius, 20.0..=500.0)
                        .text("Radius")
                        .logarithmic(true),
                )
                .on_hover_text("Brush area of effect radius in world units");
            }

            // Force settings
            if self.brush.tool == BrushTool::Attract {
                ui.add(
                    egui::Slider::new(&mut self.brush.attract_force, 1.0..=100.0)
                        .text("Attract Force"),
                )
                .on_hover_text("Strength of attraction toward brush center");
            } else if self.brush.tool == BrushTool::Repel {
                ui.add(
                    egui::Slider::new(&mut self.brush.repel_force, 1.0..=100.0).text("Repel Force"),
                )
                .on_hover_text("Strength of repulsion away from brush center");
            } else if self.brush.tool == BrushTool::Draw {
                ui.add(
                    egui::Slider::new(&mut self.brush.draw_intensity, 1..=200).text("Intensity"),
                )
                .on_hover_text("Number of particles spawned per frame");

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
            } else if self.brush.tool == BrushTool::Obstacle {
                // Shape selector
                egui::ComboBox::from_label("Shape")
                    .selected_text(match self.obstacle_tool_shape {
                        ObstacleShape::Circle => "Circle",
                        ObstacleShape::Rectangle => "Rectangle",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.obstacle_tool_shape,
                            ObstacleShape::Circle,
                            "Circle",
                        );
                        ui.selectable_value(
                            &mut self.obstacle_tool_shape,
                            ObstacleShape::Rectangle,
                            "Rectangle",
                        );
                    })
                    .response
                    .on_hover_text("Shape for the next obstacle placed by clicking");

                // Bounce slider for selected obstacle
                if self.selected_obstacle >= 0 {
                    let idx = self.selected_obstacle as usize;
                    if idx < self.app.obstacles.len() {
                        let obs = &mut self.app.obstacles[idx];
                        if ui
                            .add(egui::Slider::new(&mut obs.bounce, 0.0..=1.0).text("Bounce"))
                            .on_hover_text("Restitution: 0 = absorb impact, 1 = full bounce")
                            .changed()
                        {
                            self.sync_obstacles();
                        }
                    }
                } else if self.app.obstacles.len() >= crate::simulation::MAX_OBSTACLES {
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        format!(
                            "Max obstacles reached ({})",
                            crate::simulation::MAX_OBSTACLES
                        ),
                    );
                }
            }

            // Directional force (for attract/repel)
            if matches!(self.brush.tool, BrushTool::Attract | BrushTool::Repel) {
                ui.add(
                    egui::Slider::new(&mut self.brush.directional_force, 0.0..=100.0)
                        .text("Directional"),
                )
                .on_hover_text("Force applied in the direction of brush movement");
            }

            // Show circle toggle (not for Obstacle tool)
            if self.brush.tool != BrushTool::Obstacle {
                ui.checkbox(&mut self.brush.show_circle, "Show Circle")
                    .on_hover_text("Toggle the brush area indicator overlay");
            }

            ui.separator();
            match self.brush.tool {
                BrushTool::Obstacle => {
                    ui.label("Click to place/select, drag to move");
                    ui.label("Scroll to resize selected obstacle");
                }
                _ => {
                    ui.label("Left-click to use brush");
                }
            }
        }
    }

    fn draw_rendering_settings(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::Slider::new(&mut self.app.sim_config.particle_size, 0.1..=2.0)
                .text("Particle Size"),
        )
        .on_hover_text("Base size of rendered particles");
        self.app.config.render_particle_size = self.app.sim_config.particle_size;

        // Per-type size multipliers
        let num_types = self.app.sim_config.num_types as usize;
        let size_response = egui::CollapsingHeader::new("Per-Type Size")
            .id_salt("per_type_size")
            .default_open(false)
            .show(ui, |ui| {
                let mut changed = false;
                for i in 0..num_types {
                    if i < self.app.type_sizes.len() {
                        let r = ui.add(
                            egui::Slider::new(&mut self.app.type_sizes[i], 0.1..=5.0)
                                .text(format!("Type {}", i)),
                        );
                        changed = changed || r.changed();
                    }
                }
                changed
            });
        if size_response.body_returned.unwrap_or(false) {
            self.sync_type_sizes();
        }

        ui.horizontal(|ui| {
            ui.label("Background");
            ui.color_edit_button_rgb(&mut self.app.sim_config.background_color);
        });
        self.app.config.render_background_color = self.app.sim_config.background_color;

        let vsync_changed = ui
            .checkbox(&mut self.app.config.vsync, "VSync (present)")
            .on_hover_text("Sync frames to display refresh rate (reduces tearing)")
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
        ui.label("Tip: drag and drop any preset .json file onto the window to load it.");
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
                    let scroll_delta = ui.input(|i| i.smooth_scroll_delta().y);
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
            self.app.capture_matrix_variation_base();
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
