use anyhow::Result;
use std::sync::Arc;
use crate::gui::{GUIComponent, Transform2D, Grid, PanelComponent};
use crate::renderer::RenderContext;

/// A panel that can contain other components in a grid layout
pub struct ContainerPanel {
    background: PanelComponent,
    grid: Grid,
    transform: Transform2D,
}

impl GUIComponent for ContainerPanel {
    fn render(&self, ctx: &RenderContext, renderer: &mut crate::renderer::Renderer) -> Result<()> {
        // First render the background panel
        self.background.render(ctx, renderer)?;
        
        // Then render the grid contents
        self.grid.render(ctx, renderer)?;
        
        Ok(())
    }

    fn handle_mouse_down(&mut self, x: f32, y: f32) {
        self.grid.handle_mouse_down(x, y);
    }

    fn handle_mouse_up(&mut self, x: f32, y: f32) {
        self.grid.handle_mouse_up(x, y);
    }

    fn handle_mouse_move(&mut self, x: f32, y: f32) {
        self.grid.handle_mouse_move(x, y);
    }

    fn transform(&self) -> &Transform2D {
        &self.transform
    }
    
    fn transform_mut(&mut self) -> &mut Transform2D {
        &mut self.transform
    }

    fn destroy(&self, device: &ash::Device) {
        self.background.destroy(device);
        for row in &self.grid.rows {
            for component in &row.components {
                component.destroy(device);
            }
        }
    }
}

impl ContainerPanel {
    pub fn new(context: &Arc<crate::renderer::VulkanContext>, color: [f32; 3]) -> Result<Self> {
        Ok(ContainerPanel {
            background: PanelComponent::new(context, color)?,
            grid: Grid::new(),
            transform: Transform2D::new(),
        })
    }

    pub fn grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    /// Update the grid layout based on this container's current bounds
    /// Call this after the container's transform has been set by the parent layout
    pub fn update_grid_layout(&mut self) {
        // Sync background panel transform with container transform
        *self.background.transform_mut() = self.transform;
        
        // Get the container bounds from its transform
        let x = self.transform.position.x - (self.transform.scale.x / 2.0);
        let y = self.transform.position.y - (self.transform.scale.y / 2.0);
        let width = self.transform.scale.x;
        let height = self.transform.scale.y;
        
        // Apply internal grid layout within these bounds
        self.grid.set_bounds(x, y, width, height);
    }
}

