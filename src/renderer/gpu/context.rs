//! GPU context management using wgpu.
//!
//! This module handles the creation and management of the wgpu instance,
//! adapter, device, queue, and surface for GPU rendering.

use std::sync::Arc;

use anyhow::{Context, Result};
use wgpu::{
    Adapter, Device, Features, Instance, InstanceDescriptor, Limits, PresentMode, Queue, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages,
};
use winit::window::Window;

/// GPU context containing all wgpu resources.
///
/// This struct owns the core wgpu objects needed for rendering:
/// - Instance: Entry point to wgpu
/// - Adapter: Physical GPU device
/// - Device: Logical GPU device for creating resources
/// - Queue: Command submission queue
/// - Surface: Window surface for presenting frames
pub struct GpuContext {
    /// wgpu instance (entry point).
    pub instance: Instance,
    /// Physical GPU adapter.
    pub adapter: Adapter,
    /// Logical GPU device.
    pub device: Device,
    /// Command submission queue.
    pub queue: Queue,
    /// Window surface for rendering.
    pub surface: Surface<'static>,
    /// Surface configuration.
    pub surface_config: SurfaceConfiguration,
    /// Window reference.
    pub window: Arc<Window>,
}

impl GpuContext {
    /// Create a new GPU context for the given window.
    ///
    /// This will:
    /// 1. Create a wgpu instance
    /// 2. Create a surface from the window
    /// 3. Request a high-performance adapter
    /// 4. Request a device with appropriate features and limits
    /// 5. Configure the surface for presentation
    pub async fn new(window: Arc<Window>, vsync: bool) -> Result<Self> {
        // Create wgpu instance with all available backends
        let instance = Instance::new(&InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            ..Default::default()
        });

        // Create surface from window
        let surface = instance
            .create_surface(window.clone())
            .context("Failed to create surface")?;

        // Request high-performance GPU adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("Failed to find a suitable GPU adapter")?;

        log::info!("Using GPU: {:?}", adapter.get_info().name);
        log::info!("Backend: {:?}", adapter.get_info().backend);

        // Request device
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Device"),
                required_features: Self::required_features(&adapter),
                required_limits: Self::required_limits(&adapter),
                memory_hints: wgpu::MemoryHints::Performance,
                ..Default::default()
            })
            .await?;

        // Configure the surface
        let window_size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);

        // Prefer sRGB format for correct color rendering
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        log::info!("Surface format: {:?}", surface_format);

        let present_mode = Self::select_present_mode(&adapter, &surface, vsync);

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            format: surface_format,
            width: window_size.width.max(1),
            height: window_size.height.max(1),
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface,
            surface_config,
            window,
        })
    }

    /// Select the best present mode for the vsync flag.
    fn select_present_mode(adapter: &Adapter, surface: &Surface, vsync: bool) -> PresentMode {
        let caps = surface.get_capabilities(adapter);

        if vsync {
            return caps
                .present_modes
                .iter()
                .find(|m| **m == PresentMode::Fifo)
                .copied()
                .unwrap_or(caps.present_modes[0]);
        }

        // Uncapped preference: Mailbox > Immediate > FifoRelaxed > fallback
        caps.present_modes
            .iter()
            .find(|m| **m == PresentMode::Mailbox)
            .or_else(|| {
                caps.present_modes
                    .iter()
                    .find(|m| **m == PresentMode::Immediate)
            })
            .or_else(|| {
                caps.present_modes
                    .iter()
                    .find(|m| **m == PresentMode::FifoRelaxed)
            })
            .copied()
            .unwrap_or(caps.present_modes[0])
    }

    /// Get required GPU features for particle simulation.
    fn required_features(adapter: &Adapter) -> Features {
        let available = adapter.features();
        let mut features = Features::empty();

        // Enable GPU timestamps when fully supported so we can profile passes.
        if available.contains(Features::TIMESTAMP_QUERY)
            && available.contains(Features::TIMESTAMP_QUERY_INSIDE_PASSES)
        {
            features |= Features::TIMESTAMP_QUERY | Features::TIMESTAMP_QUERY_INSIDE_PASSES;
        }

        // Enable SHADER_F16 if available for bandwidth optimization.
        if available.contains(Features::SHADER_F16) {
            log::info!("Enabling SHADER_F16 feature");
            features |= Features::SHADER_F16;
        }

        // Keep other optional features off for maximum compatibility.
        features
    }

    /// Get required GPU limits for particle simulation.
    fn required_limits(adapter: &Adapter) -> Limits {
        // Start with adapter's supported limits and ensure minimum requirements
        let limits = adapter.limits();

        Limits {
            // Need enough storage buffer size for particles
            // 1M particles * 32 bytes = 32MB
            max_storage_buffer_binding_size: limits.max_storage_buffer_binding_size.max(128 << 20),
            // Need at least 4 storage buffers (particles, forces, interaction matrix, radius)
            max_storage_buffers_per_shader_stage: limits
                .max_storage_buffers_per_shader_stage
                .max(8),
            // Keep other limits at adapter defaults
            ..limits
        }
    }

    /// Resize the surface for a new window size.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
            log::debug!("Resized surface to {}x{}", width, height);
        }
    }

    /// Get the current surface texture format.
    pub fn surface_format(&self) -> TextureFormat {
        self.surface_config.format
    }

    /// Get the current surface dimensions.
    pub fn surface_size(&self) -> (u32, u32) {
        (self.surface_config.width, self.surface_config.height)
    }

    /// Get the current frame surface texture for rendering.
    ///
    /// Returns `None` if the surface is not ready (e.g., minimized).
    pub fn get_current_texture(&self) -> Option<wgpu::SurfaceTexture> {
        match self.surface.get_current_texture() {
            Ok(frame) => Some(frame),
            Err(wgpu::SurfaceError::Timeout) => {
                log::warn!("Surface timeout");
                None
            }
            Err(wgpu::SurfaceError::Outdated) => {
                log::warn!("Surface outdated, reconfiguring");
                self.surface.configure(&self.device, &self.surface_config);
                None
            }
            Err(wgpu::SurfaceError::Lost) => {
                log::warn!("Surface lost, reconfiguring");
                self.surface.configure(&self.device, &self.surface_config);
                None
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                log::error!("Out of GPU memory");
                None
            }
            Err(wgpu::SurfaceError::Other) => {
                log::error!("Surface error: unknown");
                None
            }
        }
    }

    /// Submit a command buffer to the GPU.
    pub fn submit(&self, command_buffer: wgpu::CommandBuffer) {
        self.queue.submit(std::iter::once(command_buffer));
    }

    /// Update present mode to match the vsync flag and reconfigure the surface if needed.
    pub fn set_vsync(&mut self, vsync: bool) {
        let desired = Self::select_present_mode(&self.adapter, &self.surface, vsync);
        if desired != self.surface_config.present_mode {
            self.surface_config.present_mode = desired;
            self.surface.configure(&self.device, &self.surface_config);
            log::info!("Present mode updated to {:?} (vsync={})", desired, vsync);
        }
    }

    /// Submit multiple command buffers to the GPU.
    pub fn submit_multiple(&self, command_buffers: impl IntoIterator<Item = wgpu::CommandBuffer>) {
        self.queue.submit(command_buffers);
    }

    /// Create a command encoder for recording GPU commands.
    pub fn create_encoder(&self, label: &str) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) })
    }

    /// Capture the current frame to an RGBA image.
    ///
    /// This copies the frame texture to a staging buffer and reads the pixel data.
    /// Note: This is a blocking operation that waits for the GPU.
    pub fn capture_frame(&self, frame_texture: &wgpu::Texture) -> Option<image::RgbaImage> {
        let (width, height) = self.surface_size();

        // Calculate buffer size with proper row alignment (256 bytes for wgpu)
        let bytes_per_pixel = 4u32; // RGBA
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
        let buffer_size = (padded_bytes_per_row * height) as u64;

        // Create staging buffer for reading back pixels
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Screenshot Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Copy texture to staging buffer
        let mut encoder = self.create_encoder("Screenshot Copy Encoder");
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: frame_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.submit(encoder.finish());

        // Map the buffer and read the data
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });

        // Wait for the GPU to finish
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());

        // Check if mapping succeeded
        if receiver.recv().ok()?.is_err() {
            log::error!("Failed to map screenshot buffer");
            return None;
        }

        // Read the pixel data
        let data = buffer_slice.get_mapped_range();

        // Remove row padding and convert to image
        let mut pixels = Vec::with_capacity((width * height * bytes_per_pixel) as usize);
        for y in 0..height {
            let start = (y * padded_bytes_per_row) as usize;
            let end = start + (width * bytes_per_pixel) as usize;
            pixels.extend_from_slice(&data[start..end]);
        }

        drop(data);
        staging_buffer.unmap();

        // Handle sRGB format conversion if needed
        // The surface is typically in sRGB format, but the raw bytes are linear
        // For screenshots, we want to preserve the displayed colors

        image::RgbaImage::from_raw(width, height, pixels)
    }
}

#[cfg(test)]
mod tests {
    // GPU context tests require a window which is hard to create in unit tests.
    // Integration tests would be more appropriate for GPU context testing.
}
