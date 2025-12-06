use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

use crate::renderer::VulkanContext;
use crate::renderer::VulkanRenderer;

// enforce this a a singleton somehow later
pub struct WindowHandler {
    window: Arc<Window>,
    context: Arc<VulkanContext>,
}

impl WindowHandler {
    pub fn new(event_loop: &ActiveEventLoop, attributes: WindowAttributes) -> Result<Self> {
        let window = Arc::new(event_loop.create_window(attributes)?);

        // this is the global VulkanContext
        let context = Arc::new(VulkanContext::new(window.clone())?);
        let renderer = Arc::new(VulkanRenderer::new(window.clone(), context.clone())?);

        Ok(Self { window, context })
    }

    pub fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}
