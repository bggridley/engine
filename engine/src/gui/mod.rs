use crate::renderer::Renderable;
use anyhow::Result;
use std::any::Any;

/// Base GUI component trait
pub trait GUIComponent: Renderable + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

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
    fn render(&self, ctx: &crate::renderer::RenderContext) -> Result<()> {
        self.renderer.render_to_context(ctx)?;
        Ok(())
    }
}

impl GUIComponent for TriangleComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// GUI system that manages all components
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

    pub fn render(&self, ctx: &crate::renderer::RenderContext) -> Result<()> {
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
