use std::sync::Arc;
use std::cell::RefCell;
use anyhow::Result;

use super::{GUIComponent, Transform, ButtonComponent, ContainerPanel};
use crate::renderer::{RenderContext, Renderer};

/// A reference-counted, interior-mutable wrapper for GUI components
/// This allows external code to hold references and mutate components
/// while they're also owned by the UI grid
pub struct ComponentRef<T> {
    inner: Arc<RefCell<T>>,
    cached_transform: Transform,
}

impl<T> ComponentRef<T> {
    pub fn new(component: T) -> (Self, Arc<RefCell<T>>) {
        let arc = Arc::new(RefCell::new(component));
        let wrapper = ComponentRef {
            inner: arc.clone(),
            cached_transform: Transform::new(),
        };
        (wrapper, arc)
    }
    
    /// Get a reference for external mutation
    pub fn handle(&self) -> Arc<RefCell<T>> {
        self.inner.clone()
    }
}

// Macro to reduce boilerplate for implementing GUIComponent
macro_rules! impl_component_ref {
    ($component_type:ty, $pre_render:expr) => {
        impl GUIComponent for ComponentRef<$component_type> {
            fn render(&self, ctx: &RenderContext, renderer: &mut Renderer) -> Result<()> {
                let mut component = self.inner.borrow_mut();
                *component.transform_mut() = self.cached_transform;
                $pre_render(&mut component);
                component.render(ctx, renderer)
            }
            
            fn transform(&self) -> &Transform {
                &self.cached_transform
            }
            
            fn transform_mut(&mut self) -> &mut Transform {
                &mut self.cached_transform
            }
            
            fn handle_mouse_down(&mut self, x: f32, y: f32) {
                self.inner.borrow_mut().handle_mouse_down(x, y);
            }
            
            fn handle_mouse_up(&mut self, x: f32, y: f32) {
                self.inner.borrow_mut().handle_mouse_up(x, y);
            }
            
            fn handle_mouse_move(&mut self, x: f32, y: f32) {
                self.inner.borrow_mut().handle_mouse_move(x, y);
            }
            
            fn destroy(&self, device: &ash::Device) {
                self.inner.borrow().destroy(device);
            }
        }
    };
}

// ButtonComponent - no pre-render needed
impl_component_ref!(ButtonComponent, |_: &mut ButtonComponent| {});

// ContainerPanel - needs grid layout update
impl_component_ref!(ContainerPanel, |c: &mut ContainerPanel| {
    c.update_grid_layout();
});
