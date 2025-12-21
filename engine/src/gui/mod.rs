use crate::renderer::{RenderContext};
use anyhow::Result;

mod triangle;
pub use triangle::{TriangleComponent};

pub trait GUIComponent {
    fn render(&self, ctx: &RenderContext, renderer: &mut crate::renderer::Renderer) -> Result<()>;
    fn set_position(&mut self, x: f32, y: f32);
    fn set_scale(&mut self, scale: f32);
}

/// Simple triangle GUI component

/// GUI system that manages renderable components
pub struct UISystem {
    components: Vec<Box<dyn GUIComponent>>,
}


impl UISystem {
    pub fn new() -> Self {
        UISystem {
            components: Vec::new(),
        }
    }

    pub fn add_component(&mut self, component: Box<dyn GUIComponent>) {
        self.components.push(component);
    }

    pub fn render(&self, ctx: &RenderContext, renderer: &mut crate::renderer::Renderer) -> Result<()> {
        for component in &self.components {
            component.render(ctx, renderer)?;
        }
        Ok(())
    }
}

impl Default for UISystem {
    fn default() -> Self {
        Self::new()
    }
}
