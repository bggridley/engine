use anyhow::Result;

pub mod triangle;
pub use triangle::{TriangleComponent, Vertex};

use glam::{Mat4, Vec2};
/// Base GUI component trait
pub trait GUIComponent: Send + Sync {
    fn render(&self, ctx: &crate::renderer::RenderContext, projection: glam::Mat4) -> Result<()>;
    fn set_pos(&mut self, _x: f32, _y: f32);
}

/// GUI system that manages all components
pub struct UISystem {
    components: Vec<Box<dyn GUIComponent>>,
    projection: glam::Mat4,
    window_size: glam::Vec2,
    scale_factor: f32,
}

impl UISystem {
    pub fn new() -> Self {
        UISystem {
            components: Vec::new(),
            projection: Mat4::IDENTITY,
            window_size: Vec2::ZERO,
            scale_factor: 1.0,
        }
    }

    pub fn update_size(&mut self, width: u32, height: u32, scale_factor: f32) {
        self.window_size = Vec2::new(width as f32, height as f32);
        self.scale_factor = scale_factor;

        // Orthographic projection: pixel-based coordinates
        let w = width as f32 / scale_factor;
        let h = height as f32 / scale_factor;
        self.projection = Mat4::orthographic_rh(0.0, w, h, 0.0, 0.1, 100.0);
    }

    pub fn add_component(&mut self, component: Box<dyn GUIComponent>) {
        self.components.push(component);
    }

    pub fn render(&self, ctx: &crate::renderer::RenderContext) -> Result<()> {
        for component in &self.components {
            component.render(ctx, self.projection)?;
        }
        Ok(())
    }
}

impl Default for UISystem {
    fn default() -> Self {
        Self::new()
    }
}
