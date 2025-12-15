use anyhow::Result;
use std::sync::Arc;

use crate::renderer::{ColorVertex2D, Mesh, PipelineId, RenderContext, VertexBuffer};

/// Triangle renderer - just holds the mesh geometry
/// Pipeline is managed centrally by PipelineManager
pub struct TriangleRenderer {
    mesh: Mesh<ColorVertex2D>,
}

impl TriangleRenderer {
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

        Ok(TriangleRenderer {
            mesh: Mesh::new(vertex_buffer),
        })
    }

    /// Render the triangle using the specified pipeline
    pub fn render(&self, ctx: &RenderContext, renderer: &mut crate::renderer::Renderer) -> Result<()> {
        let pipeline = renderer.get_pipeline(PipelineId::BasicGeometry)?;
        ctx.bind_pipeline(pipeline);
        self.mesh.draw(ctx)
    }
}
