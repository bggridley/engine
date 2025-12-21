use anyhow::Result;
use std::sync::Arc;
use crate::renderer::{ColorVertex2D, Mesh, PipelineId, RenderContext, VertexBuffer};
use crate::gui::GUIComponent;

/// Triangle renderer - just holds the mesh geometry
/// Pipeline is managed centrally by PipelineManager
pub struct TriangleComponent {
    mesh: Mesh<ColorVertex2D>,
    position: glam::Vec2,
    scale: f32,
}

impl GUIComponent for TriangleComponent {
    /// Render the triangle using the specified pipeline
    fn render(&self, ctx: &RenderContext, renderer: &mut crate::renderer::Renderer) -> Result<()> {
        let pipeline = renderer.get_pipeline(PipelineId::BasicGeometry)?;
        ctx.bind_pipeline(pipeline);
        self.mesh.draw(ctx)
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.position.x = x;
        self.position.y = y;
    }

    fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }
}

impl TriangleComponent {
    pub fn new(context: &Arc<crate::renderer::VulkanContext>) -> Result<Self> {
        // Define triangle vertices
        let vertices = [
            ColorVertex2D {
                position: [0.0, -0.5],
                color: [1.0, 0.0, 0.0],
            },
            ColorVertex2D {
                position: [0.5, 0.5],
                color: [0.0, 1.0, 0.0],
            },
            ColorVertex2D {
                position: [-0.5, 0.5],
                color: [0.0, 0.0, 1.0],
            },
        ];

        // Create vertex buffer
        let vertex_buffer = VertexBuffer::new(
            &context.device,
            context.physical_device,
            &context.instance,
            &vertices,
        )?;

        Ok(TriangleComponent {
            mesh: Mesh::new(vertex_buffer),
            position: glam::Vec2::ZERO,
            scale: 1.0,
        })
    }
}
