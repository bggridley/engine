use anyhow::Result;
use std::sync::Arc;
use std::cell::RefCell;
use crate::renderer::{ColorVertex2D, Mesh, PipelineId, RenderContext, VertexBuffer};
use crate::gui::{GUIComponent, Transform2D, TextComponent};

use crate::renderer::PushConstants2D;

/// Button component with optional text
pub struct ButtonComponent {
    mesh: Mesh<ColorVertex2D>,
    transform: Transform2D,
    text: Option<RefCell<TextComponent>>,
    is_hovered: bool,
}

impl GUIComponent for ButtonComponent {
    /// Render the button and optional text
    fn render(&self, ctx: &RenderContext, renderer: &mut crate::renderer::Renderer) -> Result<()> {
        let pipeline = renderer.get_pipeline(PipelineId::BasicGeometry)?;
        let pipeline_layout = renderer.get_pipeline_layout(PipelineId::BasicGeometry)
            .ok_or_else(|| anyhow::anyhow!("Pipeline layout not found"))?;
        ctx.bind_pipeline(pipeline);

        // Set push constants (projection + transform + color modulation)
        let color_mod = if self.is_hovered {
            [0.7, 0.7, 0.7]  // 30% darker on hover
        } else {
            [1.0, 1.0, 1.0]  // Normal color
        };
        
        let push = PushConstants2D {
            projection: renderer.projection,  // Use ortho for 2D
            transform: 
            glam::Mat4::from_translation(glam::Vec3::new(self.transform.position.x, self.transform.position.y, 0.0)) *
            glam::Mat4::from_rotation_z(self.transform.rotation) * 
            glam::Mat4::from_scale(glam::Vec3::new(self.transform.scale.x, self.transform.scale.y, 1.0)),
            color_modulation: color_mod,
            _padding: 0.0,
        };

        ctx.push_constants(pipeline_layout, &push);
        
        self.mesh.draw(ctx)?;
        
        // Render text if present - position it at the button's center
        if let Some(text_cell) = &self.text {
            let mut text = text_cell.borrow_mut();
            // Position text at button center
            text.set_position(self.transform.position);
            drop(text);  // Release borrow
            
            // Now render
            let text = text_cell.borrow();
            text.render(ctx, renderer)?;
        }
        
        Ok(())
    }

    fn handle_mouse_down(&mut self, x: f32, y: f32) {
        if self.transform.contains_point(glam::Vec2::new(x, y)) {
            println!("Button clicked at ({}, {})", x, y);
        }
    }
    
    fn handle_mouse_up(&mut self, _x: f32, _y: f32) {
        
    }

    fn handle_mouse_move(&mut self, x: f32, y: f32) {
        self.is_hovered = self.transform.contains_point(glam::Vec2::new(x, y));
    }

    fn transform(&self) -> &Transform2D {
        &self.transform
    }
    
    fn transform_mut(&mut self) -> &mut Transform2D {
        &mut self.transform
    }

    fn destroy(&self, device: &ash::Device) {
        self.mesh.destroy(device);
        if let Some(text_cell) = &self.text {
            text_cell.borrow().destroy(device);
        }
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
            text: None,
            is_hovered: false,
        })
    }

    /// Set the button text
    pub fn set_text(&mut self, text: TextComponent) {
        self.text = Some(RefCell::new(text));
    }
    
    /// Update the button text content
    pub fn update_text(&mut self, new_text: &str, context: &Arc<crate::renderer::VulkanContext>) -> Result<()> {
        if let Some(text_cell) = &self.text {
            text_cell.borrow_mut().update_text(new_text, context)?;
        }
        Ok(())
    }
}
