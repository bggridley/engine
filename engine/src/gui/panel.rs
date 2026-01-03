use anyhow::Result;
use std::sync::Arc;
use crate::renderer::{ColorVertex2D, Mesh, PipelineId, RenderContext, VertexBuffer};
use crate::gui::{GUIComponent, Transform2D};
use crate::renderer::PushConstants2D;

/// A panel is a rectangular container that can render a background and hold other components
pub struct PanelComponent {
    mesh: Mesh<ColorVertex2D>,
    transform: Transform2D,
    color: [f32; 3],
}

impl GUIComponent for PanelComponent {
    fn render(&self, ctx: &RenderContext, renderer: &mut crate::renderer::Renderer) -> Result<()> {
        let pipeline = renderer.get_pipeline(PipelineId::BasicGeometry)?;
        let pipeline_layout = renderer.get_pipeline_layout(PipelineId::BasicGeometry).unwrap();
        ctx.bind_pipeline(pipeline);

        let push = PushConstants2D {
            projection: renderer.projection,
            transform: 
            glam::Mat4::from_translation(glam::Vec3::new(self.transform.position.x, self.transform.position.y, 0.0)) *
            glam::Mat4::from_rotation_z(self.transform.rotation) * 
            glam::Mat4::from_scale(glam::Vec3::new(self.transform.scale.x, self.transform.scale.y, 1.0)),
        };

        ctx.push_constants(pipeline_layout, &push);
        
        self.mesh.draw(ctx)?;
        
        Ok(())
    }

    fn handle_mouse_down(&mut self, _x: f32, _y: f32) {}
    fn handle_mouse_up(&mut self, _x: f32, _y: f32) {}
    fn handle_mouse_move(&mut self, _x: f32, _y: f32) {}

    fn transform(&self) -> &Transform2D {
        &self.transform
    }
    
    fn transform_mut(&mut self) -> &mut Transform2D {
        &mut self.transform
    }

    fn destroy(&self, device: &ash::Device) {
        self.mesh.destroy(device);
    }
}

impl PanelComponent {
    pub fn new(
        context: &Arc<crate::renderer::VulkanContext>,
        color: [f32; 3],
    ) -> Result<Self> {
        // Define quad vertices (0.5 units = 50% of width/height from center)
        let vertices = [
            ColorVertex2D {
                position: [-0.5, 0.5],
                color,
            },
            ColorVertex2D {
                position: [-0.5, -0.5],
                color,
            },
            ColorVertex2D {
                position: [0.5, -0.5],
                color,
            },
            ColorVertex2D {
                position: [0.5, -0.5],
                color,
            },
            ColorVertex2D {
                position: [0.5, 0.5],
                color,
            },
            ColorVertex2D {
                position: [-0.5, 0.5],
                color,
            },
        ];

        let vertex_buffer = VertexBuffer::new(
            &context.device,
            context.physical_device,
            &context.instance,
            &vertices,
        )?;

        Ok(PanelComponent {
            mesh: Mesh::new(vertex_buffer),
            transform: Transform2D::new(),
            color,
        })
    }

    pub fn set_color(&mut self, color: [f32; 3]) {
        self.color = color;
    }

    pub fn color(&self) -> [f32; 3] {
        self.color
    }
}
