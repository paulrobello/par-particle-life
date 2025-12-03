//! Rendering module for GPU particle rendering.

pub mod gpu;

/// Renderer trait for different rendering backends.
pub trait Renderer {
    /// Render particles to the screen.
    fn render(&mut self) -> anyhow::Result<()>;

    /// Resize the render target.
    fn resize(&mut self, width: u32, height: u32);
}
