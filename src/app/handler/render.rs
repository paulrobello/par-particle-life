//! Rendering operations for the application.

use super::AppHandler;
use crate::simulation::BoundaryMode;

impl AppHandler {
    pub(crate) fn render(&mut self) {
        // Extract what we need from gpu first to avoid borrow issues
        let gpu = match &mut self.gpu {
            Some(gpu) => gpu,
            None => return,
        };

        // Get current frame
        let frame = match gpu.context.get_current_texture() {
            Some(f) => f,
            None => return,
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Clone egui context (cheap - it's an Arc internally)
        let egui_ctx = gpu.egui_ctx.clone();
        let raw_input = gpu.egui_state.take_egui_input(&gpu.context.window);

        // Run egui - now we can access self freely
        let full_output = egui_ctx.run(raw_input, |ctx| {
            self.draw_ui(ctx);
        });

        // Get gpu back
        let gpu = self.gpu.as_mut().unwrap();

        gpu.egui_state
            .handle_platform_output(&gpu.context.window, full_output.platform_output);

        let clipped_primitives = gpu
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        // Update egui textures
        for (id, image_delta) in &full_output.textures_delta.set {
            gpu.egui_renderer.update_texture(
                &gpu.context.device,
                &gpu.context.queue,
                *id,
                image_delta,
            );
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [
                gpu.context.surface_config.width,
                gpu.context.surface_config.height,
            ],
            pixels_per_point: full_output.pixels_per_point,
        };

        let mut encoder = gpu.context.create_encoder("Render Encoder");

        // Update egui buffers
        gpu.egui_renderer.update_buffers(
            &gpu.context.device,
            &gpu.context.queue,
            &mut encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        // Clear background
        {
            let bg = self.app.sim_config.background_color;
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: bg[0] as f64,
                            g: bg[1] as f64,
                            b: bg[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            // Pass ends here, just clears the background
        }

        // Render glow effect first (if enabled)
        if self.app.sim_config.enable_glow {
            // Update glow params
            gpu.render
                .update_glow(&gpu.context.queue, &self.app.sim_config);

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Glow Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Don't clear, load existing content
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&gpu.render.glow_pipeline);
            render_pass.set_bind_group(0, &gpu.glow_bind_group, &[]);
            render_pass.draw(0..4, 0..gpu.buffers.num_particles);
        }

        // Render solid particles on top
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Particle Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Don't clear, load existing content (glow)
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            match self.app.sim_config.boundary_mode {
                BoundaryMode::Repel | BoundaryMode::Wrap => {
                    // Standard rendering - one instance per particle
                    render_pass.set_pipeline(&gpu.render.particle_pipeline);
                    render_pass.set_bind_group(0, &gpu.render_bind_group, &[]);
                    render_pass.draw(0..4, 0..gpu.buffers.num_particles);
                }
                BoundaryMode::MirrorWrap => {
                    // Mirror wrap rendering - multiple copies per particle
                    // Update mirror params
                    gpu.render
                        .update_mirror(&gpu.context.queue, &self.app.sim_config);
                    render_pass.set_pipeline(&gpu.render.mirror_pipeline);
                    render_pass.set_bind_group(0, &gpu.mirror_bind_group, &[]);
                    // Draw 4 vertices per particle copy, num_particles * mirror_copies instances
                    let num_copies = self.app.sim_config.mirror_wrap_count;
                    render_pass.draw(0..4, 0..(gpu.buffers.num_particles * num_copies));
                }
                BoundaryMode::InfiniteWrap => {
                    // Infinite wrap rendering - tiled copies based on camera view
                    // Calculate camera center from world center + offset
                    let world_w = self.app.sim_config.world_size.x;
                    let world_h = self.app.sim_config.world_size.y;
                    let camera_center_x = world_w / 2.0 + self.camera.offset.x;
                    let camera_center_y = world_h / 2.0 + self.camera.offset.y;

                    // Calculate how many tiles are visible
                    let infinite_params =
                        crate::renderer::gpu::RenderPipelines::get_infinite_params(
                            world_w,
                            world_h,
                            camera_center_x,
                            camera_center_y,
                            self.camera.zoom,
                        );
                    gpu.render.update_infinite(
                        &gpu.context.queue,
                        world_w,
                        world_h,
                        camera_center_x,
                        camera_center_y,
                        self.camera.zoom,
                    );
                    render_pass.set_pipeline(&gpu.render.infinite_pipeline);
                    render_pass.set_bind_group(0, &gpu.infinite_bind_group, &[]);
                    // Draw 4 vertices per particle copy, num_particles * total_copies instances
                    let total_copies = infinite_params.total_copies();
                    render_pass.draw(0..4, 0..(gpu.buffers.num_particles * total_copies));
                }
            }
        }

        // Render brush circle indicator (if visible)
        {
            // Update brush render params
            gpu.brush_pipelines.update_render(
                &gpu.context.queue,
                &self.brush,
                self.app.sim_config.world_size.x,
                self.app.sim_config.world_size.y,
                self.camera.zoom,
                self.camera.offset.x,
                self.camera.offset.y,
            );

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Brush Circle Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&gpu.brush_pipelines.circle_pipeline);
            render_pass.set_bind_group(0, &gpu.brush_pipelines.circle_bind_group, &[]);
            render_pass.draw(0..4, 0..1);
        }

        // Capture frame without UI if needed (before egui render)
        let need_capture_without_ui =
            self.capture_hide_ui && (self.screenshot_requested || self.is_recording);

        let frame_without_ui = if need_capture_without_ui {
            // Submit current encoder to get the frame without UI
            gpu.context.submit(encoder.finish());
            let captured = gpu.context.capture_frame(&frame.texture);
            // Create new encoder for egui render
            encoder = gpu.context.create_encoder("Egui Encoder");
            captured
        } else {
            None
        };

        // Render egui on top
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Use forget_lifetime to satisfy the static lifetime requirement
            let mut render_pass = render_pass.forget_lifetime();
            gpu.egui_renderer
                .render(&mut render_pass, &clipped_primitives, &screen_descriptor);
        }

        // Free egui textures
        for id in &full_output.textures_delta.free {
            gpu.egui_renderer.free_texture(id);
        }

        gpu.context.submit(encoder.finish());

        // Capture screenshot if requested
        if self.screenshot_requested {
            self.screenshot_requested = false;
            // Use pre-captured frame without UI, or capture now with UI
            let image = if self.capture_hide_ui {
                frame_without_ui.clone()
            } else {
                gpu.context.capture_frame(&frame.texture)
            };
            if let Some(image) = image {
                // Ensure screenshots directory exists
                match Self::ensure_screenshots_dir() {
                    Ok(dir) => {
                        // Generate filename with timestamp and counter
                        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                        let filename = format!(
                            "screenshot_{}_{:03}.png",
                            timestamp, self.screenshot_counter
                        );
                        self.screenshot_counter += 1;
                        let filepath = dir.join(&filename);

                        // Save to screenshots directory
                        match image.save(&filepath) {
                            Ok(()) => {
                                let path_str = filepath.display().to_string();
                                log::info!("Screenshot saved: {}", path_str);
                                self.preset_status = format!("Screenshot saved: {}", filename);
                                self.last_capture_path = Some(path_str);
                            }
                            Err(e) => {
                                log::error!("Failed to save screenshot: {}", e);
                                self.preset_status = format!("Screenshot failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to create screenshots directory: {}", e);
                        self.preset_status = format!("Screenshot failed: {}", e);
                    }
                }
            } else {
                log::error!("Failed to capture screenshot");
                self.preset_status = "Screenshot capture failed".to_string();
            }
        }

        // Capture frame for video/GIF recording
        if self.is_recording {
            self.video_frame_counter += 1;
            // Skip frames to reduce file size
            if self.video_frame_counter >= self.video_frame_skip {
                self.video_frame_counter = 0;

                // Use pre-captured frame without UI, or capture now with UI
                let image = if self.capture_hide_ui {
                    frame_without_ui.clone()
                } else {
                    gpu.context.capture_frame(&frame.texture)
                };

                if let Some(image) = image {
                    // Check if using ffmpeg video recorder
                    if let Some(ref mut recorder) = self.video_recorder {
                        // Send raw RGBA data to ffmpeg
                        let frame_data = image.into_raw();
                        if let Err(e) = recorder.add_frame(frame_data) {
                            log::error!("Failed to add frame to video: {}", e);
                        } else {
                            self.preset_status = format!(
                                "Recording: {} frames (F11 to stop)",
                                recorder.frame_count()
                            );
                        }
                    } else {
                        // Native GIF recording - limit frames to prevent memory exhaustion
                        const MAX_FRAMES: usize = 300;
                        if self.recorded_frames.len() < MAX_FRAMES {
                            self.recorded_frames.push(image);
                            self.preset_status = format!(
                                "Recording: {} frames (F11 to stop)",
                                self.recorded_frames.len()
                            );
                        } else {
                            // Auto-stop when max frames reached - set flag, will stop after frame
                            log::info!("Max GIF frames reached, auto-stopping");
                            self.pending_stop_recording = true;
                        }
                    }
                }
            }
        }

        frame.present();

        // Safe to reconfigure surface now that the frame is dropped
        if let Some(vsync) = self.pending_vsync.take() {
            gpu.context.set_vsync(vsync);
        }

        // Handle deferred stop recording (after gpu borrow is released)
        if self.pending_stop_recording {
            self.pending_stop_recording = false;
            self.stop_recording();
        }
    }
}
