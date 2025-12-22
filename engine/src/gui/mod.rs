use crate::renderer::RenderContext;
use anyhow::Result;

mod button;
pub use button::ButtonComponent;

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
}



/// Simple triangle GUI component

/// GUI system that manages renderable components
pub struct UISystem {
    components: Vec<Box<dyn GUIComponent>>,
}
#[derive(Clone, Copy)]
pub struct ComponentHandle(usize);
impl UISystem {
    pub fn new() -> Self {
        UISystem {
            components: Vec::new(),
        }
    }

    // These three methods will be optimized later by using a grid or something
    pub fn handle_mouse_down(&mut self, x: f32, y: f32) {
        for component in &mut self.components {
            component.handle_mouse_down(x, y);
        }
    }

    pub fn handle_mouse_up(&mut self, x: f32, y: f32) {
        for component in &mut self.components {
            component.handle_mouse_up(x, y);
        }
    }

    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        for component in &mut self.components {
            component.handle_mouse_move(x, y);
        }
    }

    pub fn add_component(&mut self, component: Box<dyn GUIComponent>) -> ComponentHandle {
        let id = self.components.len();
        self.components.push(component);
        ComponentHandle(id)
    }

    pub fn render(
        &self,
        ctx: &RenderContext,
        renderer: &mut crate::renderer::Renderer,
    ) -> Result<()> {
        for component in &self.components {
            component.render(ctx, renderer)?;
        }
        Ok(())
    }

    pub fn get_component_mut(
        &mut self,
        handle: &ComponentHandle,
    ) -> Option<&mut Box<dyn GUIComponent>> {
        self.components.get_mut(handle.0)
    }
}

impl Default for UISystem {
    fn default() -> Self {
        Self::new()
    }
}
