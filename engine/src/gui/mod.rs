use crate::renderer::{RenderContext, Renderable};
use anyhow::Result;

/// Simple triangle GUI component
pub struct TriangleComponent {
    pub renderer: crate::renderer::TriangleRenderer,
}

impl TriangleComponent {
    pub fn new(context: &std::sync::Arc<crate::renderer::VulkanContext>) -> Result<Self> {
        let renderer = crate::renderer::TriangleRenderer::new(context)?;
        Ok(TriangleComponent { renderer })
    }
}

impl Renderable for TriangleComponent {
    fn render(&self, ctx: &RenderContext) -> Result<()> {
        self.renderer.render_to_context(ctx)
    }
}

/// GUI system that manages renderable components
pub struct UISystem {
    components: Vec<Box<dyn Renderable>>,
}

impl UISystem {
    pub fn new() -> Self {
        UISystem {
            components: Vec::new(),
        }
    }

    pub fn add_component(&mut self, component: Box<dyn Renderable>) {
        self.components.push(component);
    }

    pub fn render(&self, ctx: &RenderContext) -> Result<()> {
        for component in &self.components {
            component.render(ctx)?;
        }
        Ok(())
    }
}

impl Default for UISystem {
    fn default() -> Self {
        Self::new()
    }
}
