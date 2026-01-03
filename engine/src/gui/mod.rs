use crate::renderer::RenderContext;
use anyhow::Result;

mod button;
pub use button::ButtonComponent;

mod panel;
pub use panel::PanelComponent;

mod container;
pub use container::ContainerPanel;

mod grid;
pub use grid::{Grid, GridRow, LayoutConstraints};

mod layout;
pub use layout::{ComputedLayout, HAlign, LayoutSpec, SizeSpec, VAlign};

mod text;
pub use text::TextComponent;

pub use glam::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct Transform2D {
    pub position: glam::Vec2,
    pub rotation: f32,
    pub scale: glam::Vec2,
}

impl Transform2D {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
    
    pub fn contains_point(&self, point: Vec2) -> bool {
        let half_width = self.scale.x * 0.5;
        let half_height = self.scale.y * 0.5;
        
        let min_x = self.position.x - half_width;
        let max_x = self.position.x + half_width;
        let min_y = self.position.y - half_height;
        let max_y = self.position.y + half_height;
        
        point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::new()
    }
}

pub trait GUIComponent {
    fn render(&self, ctx: &RenderContext, renderer: &mut crate::renderer::Renderer) -> Result<()>;
    fn transform(&self) -> &Transform2D;
    fn transform_mut(&mut self) -> &mut Transform2D;
    fn handle_mouse_down(&mut self, x: f32, y: f32);
    fn handle_mouse_up(&mut self, x: f32, y: f32);
    fn handle_mouse_move(&mut self, x: f32, y: f32);
    /// Manually destroy Vulkan resources
    fn destroy(&self, device: &ash::Device);
}



/// Simple triangle GUI component

/// GUI system that manages renderable components via a grid layout
pub struct UISystem {
    pub grid: Grid,
}

impl UISystem {
    pub fn new() -> Self {
        UISystem {
            grid: Grid::new(),
        }
    }

    pub fn render(&self, ctx: &RenderContext, renderer: &mut crate::renderer::Renderer) -> anyhow::Result<()> {
        self.grid.render(ctx, renderer)
    }

    pub fn handle_mouse_down(&mut self, x: f32, y: f32) {
        self.grid.handle_mouse_down(x, y);
    }

    pub fn handle_mouse_up(&mut self, x: f32, y: f32) {
        self.grid.handle_mouse_up(x, y);
    }

    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        self.grid.handle_mouse_move(x, y);
    }

    /// Update layout for nested containers after main grid layout has been set
    pub fn update_nested_layouts(&mut self) {
        // This is a placeholder - the real implementation would require
        // the ability to downcast components to ContainerPanel
        // For now, containers will need to be updated manually
    }

    /// Manually destroy all GUI resources
    pub fn destroy(&self, device: &ash::Device) {
        for row in &self.grid.rows {
            for component in &row.components {
                component.destroy(device);
            }
        }
    }
}

impl Default for UISystem {
    fn default() -> Self {
        Self::new()
    }
}
