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

mod component_ref;
pub use component_ref::ComponentRef;

pub use glam::Vec2;

pub use crate::math::Transform;

pub trait GUIComponent {
    fn render(&self, ctx: &RenderContext, renderer: &mut crate::renderer::Renderer) -> Result<()>;
    fn transform(&self) -> &Transform;
    fn transform_mut(&mut self) -> &mut Transform;
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
