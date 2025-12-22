use anyhow::Result;
use std::sync::Arc;
use crate::renderer::{ColorVertex2D, Mesh, PipelineId, RenderContext, VertexBuffer};
use crate::gui::{GUIComponent, Transform2D};

use crate::renderer::PushConstants2D;

/// Triangle renderer - just holds the mesh geometry
/// Pipeline is managed centrally by PipelineManager
pub struct ButtonComponent {
    mesh: Mesh<ColorVertex2D>,
    transform: Transform2D,
}

impl GUIComponent for ButtonComponent {
    /// Render the triangle using the specified pipeline
    fn render(&self, ctx: &RenderContext, renderer: &mut crate::renderer::Renderer) -> Result<()> {
        let pipeline = renderer.get_pipeline(PipelineId::BasicGeometry)?;
        let pipeline_layout = renderer.get_pipeline_layout(PipelineId::BasicGeometry).unwrap();
        ctx.bind_pipeline(pipeline);

        // Set push constants (projection + transform)
        let push = PushConstants2D {
            projection: renderer.projection,  // Use ortho for 2D
            transform: 
            glam::Mat4::from_translation(glam::Vec3::new(self.transform.position.x, self.transform.position.y, 0.0)) *
            glam::Mat4::from_rotation_z(self.transform.rotation) * 
            glam::Mat4::from_scale(glam::Vec3::new(self.transform.scale.x, self.transform.scale.y, 1.0)),
        };

        ctx.push_constants(pipeline_layout, &push);
        
        self.mesh.draw(ctx)?;
        
        Ok(())
    }

    fn handle_mouse_down(&mut self, x: f32, y: f32) {
        if self.transform.contains_point(glam::Vec2::new(x, y)) {
            println!("Button clicked at ({}, {})", x, y);
        }
    }
    
    fn handle_mouse_up(&mut self, _x: f32, _y: f32) {
        
    }

    fn handle_mouse_move(&mut self, _x: f32, _y: f32) {

    }

    fn transform(&self) -> &Transform2D {
        &self.transform
    }
    
    fn transform_mut(&mut self) -> &mut Transform2D {
        &mut self.transform
    }
}

impl ButtonComponent {
    pub fn new(context: &Arc<crate::renderer::VulkanContext>) -> Result<Self> {
        // Define triangle vertices
        let vertices = [
            ColorVertex2D {
                position: [-0.5, 0.5],
                color: [1.0, 0.0, 0.0],
            },
            ColorVertex2D {
                position: [-0.5, -0.5],
                color: [1.0, 0.0, 0.0],
            },
            ColorVertex2D {
                position: [0.5, -0.5],
                color: [0.0, 0.0, 1.0],
            },
            ColorVertex2D {
                position: [0.5, -0.5],
                color: [0.0, 0.0, 1.0],
            },
            ColorVertex2D {
                position: [0.5, 0.5],
                color: [0.0, 0.0, 1.0],
            },
            ColorVertex2D {
                position: [-0.5, 0.5],
                color: [1.0, 0.0, 0.0],
            },
        ];

        // Create vertex buffer
        let vertex_buffer = VertexBuffer::new(
            &context.device,
            context.physical_device,
            &context.instance,
            &vertices,
        )?;

        Ok(ButtonComponent {
            mesh: Mesh::new(vertex_buffer),
            transform: Transform2D::new(),
        })
    }
}
