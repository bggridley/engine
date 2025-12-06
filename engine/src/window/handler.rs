use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

use crate::renderer::VulkanRenderer;
use crate::renderer::VulkanContext;

pub struct WindowHandler {
    renderers: HashMap<WindowId, VulkanRenderer>,
    windows: HashMap<WindowId, Arc<Window>>,
    primary_window_id: WindowId,
    context: Arc<VulkanContext>,
}

impl WindowHandler {
    pub fn new(event_loop: &ActiveEventLoop, attributes: WindowAttributes) -> Result<Self> {

        let window = Arc::new(event_loop.create_window(attributes)?);
        let primary_window_id = window.id();

        // this is the global VulkanContext
        let context = Arc::new(VulkanContext::new(window.clone())?);      
        let renderer = VulkanRenderer::new(window.clone(), context.clone())?;

        let windows: HashMap<WindowId, Arc<Window>> = HashMap::from([(primary_window_id, window)]);
        let renderers: HashMap<WindowId, VulkanRenderer> = HashMap::from([(primary_window_id, renderer)]);

        Ok(Self{renderers, windows, primary_window_id, context})
    }

    pub fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                if window_id == self.primary_window_id {
                    event_loop.exit();
                } else {
                    self.windows.remove(&window_id);
                    self.renderers.remove(&window_id);
                }
            }
            _ => {}
        }
    }

    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        attributes: WindowAttributes,
    ) -> Result<WindowId> {
        let window = Arc::new(event_loop.create_window(attributes)?);
        let window_id = window.id();
        self.windows.insert(window_id, window.clone()); // window is moved into map??

        let renderer = VulkanRenderer::new(window, self.context.clone())?;
        self.renderers.insert(window_id, renderer);

        Ok(window_id)
    }
}
