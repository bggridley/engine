use anyhow::Result;
use std::sync::Arc;

use crate::renderer::{Mesh, PipelineId, VertexBuffer};

#[repr(C)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

/// Triangle renderer - just holds the mesh geometry
/// Pipeline is managed centrally by PipelineManager
pub struct TriangleRenderer {
    mesh: Mesh<Vertex>,
}

impl TriangleRenderer {
    pub fn new(context: &Arc<crate::renderer::VulkanContext>) -> Result<Self> {
        // Define triangle vertices
        let vertices = [
            Vertex {
                position: [0.0, -0.5],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
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
    pub fn render(&self, ctx: &mut crate::renderer::RenderContext, pipeline: PipelineId) -> Result<()> {
        ctx.bind_pipeline_id(pipeline)?;
        self.mesh.draw(ctx)
    }
}
