//! Video recording operations.

use super::AppHandler;
use crate::video_recorder::VideoRecorder;

impl AppHandler {
    /// Toggle video recording on/off.
    pub(crate) fn toggle_recording(&mut self) {
        if self.is_recording {
            self.stop_recording();
        } else {
            self.start_recording();
        }
    }

    /// Start video recording.
    pub(crate) fn start_recording(&mut self) {
        if self.is_recording {
            return;
        }

        let Some(gpu) = &self.gpu else {
            self.preset_status = "Cannot record: no GPU context".to_string();
            return;
        };

        // Ensure videos directory exists
        let videos_dir = match Self::ensure_videos_dir() {
            Ok(dir) => dir,
            Err(e) => {
                log::error!("Failed to create videos directory: {}", e);
                self.preset_status = format!("Recording failed: {}", e);
                return;
            }
        };

        let (width, height) = gpu.context.surface_size();
        let fps = 30; // Target framerate for recording

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!(
            "recording_{}_{:03}.{}",
            timestamp,
            self.video_counter,
            self.video_format.extension()
        );
        self.video_counter += 1;
        let filepath = videos_dir.join(&filename);
        let filepath_str = filepath.display().to_string();

        // Try ffmpeg-based recording first
        if self.use_ffmpeg {
            let mut recorder = VideoRecorder::new(width, height, fps, self.video_format);
            match recorder.start_recording(filepath_str.clone()) {
                Ok(()) => {
                    self.video_recorder = Some(recorder);
                    self.is_recording = true;
                    self.video_frame_counter = 0;
                    let format_name = self.video_format.name();
                    log::info!("Started {} recording: {}", format_name, filepath_str);
                    self.preset_status = format!("Recording {}... (F11 to stop)", format_name);
                    return;
                }
                Err(e) => {
                    log::warn!("ffmpeg not available: {}. Falling back to native GIF.", e);
                    // Fall through to native GIF recording
                }
            }
        }

        // Fallback to native GIF recording
        self.recorded_frames.clear();
        self.video_frame_counter = 0;
        self.is_recording = true;
        log::info!("Started native GIF recording");
        self.preset_status = "Recording GIF... (F11 to stop)".to_string();
    }

    /// Stop video recording and save the file.
    pub(crate) fn stop_recording(&mut self) {
        if !self.is_recording {
            return;
        }

        self.is_recording = false;

        // Check if using ffmpeg recorder
        if let Some(mut recorder) = self.video_recorder.take() {
            match recorder.stop_recording() {
                Ok(filename) => {
                    let format_name = self.video_format.name();
                    log::info!(
                        "{} saved: {} ({} frames)",
                        format_name,
                        filename,
                        recorder.frame_count()
                    );
                    self.preset_status = format!(
                        "{} saved: {} ({} frames)",
                        format_name,
                        filename,
                        recorder.frame_count()
                    );
                    self.last_capture_path = Some(filename);
                }
                Err(e) => {
                    log::error!("Failed to save video: {}", e);
                    self.preset_status = format!("Video save failed: {}", e);
                }
            }
            return;
        }

        // Native GIF recording
        log::info!(
            "Stopping native GIF recording with {} frames",
            self.recorded_frames.len()
        );

        if self.recorded_frames.is_empty() {
            self.preset_status = "No frames recorded".to_string();
            return;
        }

        self.save_native_gif();
    }

    /// Save recorded frames as native GIF (fallback when ffmpeg unavailable).
    pub(crate) fn save_native_gif(&mut self) {
        use color_quant::NeuQuant;
        use std::fs::File;

        if self.recorded_frames.is_empty() {
            return;
        }

        // Ensure videos directory exists
        let videos_dir = match Self::ensure_videos_dir() {
            Ok(dir) => dir,
            Err(e) => {
                log::error!("Failed to create videos directory: {}", e);
                self.preset_status = format!("GIF save failed: {}", e);
                return;
            }
        };

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("recording_{}_{:03}.gif", timestamp, self.video_counter);
        self.video_counter += 1;
        let filepath = videos_dir.join(&filename);

        let first_frame = &self.recorded_frames[0];
        let width = first_frame.width() as u16;
        let height = first_frame.height() as u16;

        // Build a global color palette from all frames for better color consistency
        // Sample pixels from all frames for palette generation
        let mut sample_pixels: Vec<u8> = Vec::new();
        let sample_rate = 4; // Sample every 4th pixel to speed up quantization
        for frame in &self.recorded_frames {
            for (i, chunk) in frame.as_raw().chunks(4).enumerate() {
                if i % sample_rate == 0 {
                    sample_pixels.extend_from_slice(&chunk[0..3]); // RGB only
                }
            }
        }

        // Use NeuQuant to generate optimal 256-color palette
        // samplefac: 1 = highest quality (slow), 30 = fastest (lower quality)
        let samplefac = 10; // Good balance of speed and quality
        let nq = NeuQuant::new(samplefac, 256, &sample_pixels);

        // Get the color palette (256 colors * 3 bytes = 768 bytes)
        let palette = nq.color_map_rgb();

        // Create GIF file
        let file = match File::create(&filepath) {
            Ok(f) => f,
            Err(e) => {
                log::error!("Failed to create GIF file: {}", e);
                self.preset_status = format!("GIF save failed: {}", e);
                return;
            }
        };

        // Create GIF encoder with global palette
        let mut encoder = match gif::Encoder::new(file, width, height, &palette) {
            Ok(e) => e,
            Err(e) => {
                log::error!("Failed to create GIF encoder: {}", e);
                self.preset_status = format!("GIF encode failed: {}", e);
                return;
            }
        };

        // Set repeat infinitely
        if let Err(e) = encoder.set_repeat(gif::Repeat::Infinite) {
            log::warn!("Failed to set GIF repeat: {}", e);
        }

        // Write frames
        let frame_count = self.recorded_frames.len();
        for (i, rgba_image) in self.recorded_frames.drain(..).enumerate() {
            // Convert RGBA to indexed color using the quantized palette
            let rgba_data = rgba_image.into_raw();
            let mut frame_data: Vec<u8> = Vec::with_capacity((width as usize) * (height as usize));

            for chunk in rgba_data.chunks(4) {
                // Map each pixel to its closest palette entry
                let idx = nq.index_of(&[chunk[0], chunk[1], chunk[2]]) as u8;
                frame_data.push(idx);
            }

            let frame = gif::Frame {
                width,
                height,
                delay: 3,      // 30ms delay (100 / 3 = ~33fps)
                palette: None, // Use global palette
                buffer: std::borrow::Cow::Owned(frame_data),
                ..Default::default()
            };

            if let Err(e) = encoder.write_frame(&frame) {
                log::error!("Failed to write GIF frame {}: {}", i, e);
                self.preset_status = format!("GIF frame {} failed: {}", i, e);
                return;
            }

            // Log progress for long recordings
            if frame_count > 50 && i % 50 == 0 {
                log::info!("GIF encoding progress: {}/{}", i, frame_count);
            }
        }

        let path_str = filepath.display().to_string();
        log::info!("GIF saved: {} ({} frames)", path_str, frame_count);
        self.preset_status = format!("GIF saved: {} ({} frames)", filename, frame_count);
        self.last_capture_path = Some(path_str);
    }
}
