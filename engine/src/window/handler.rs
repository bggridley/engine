
use anyhow::Result;
use winit::event::WindowEvent;
use std::{collections::HashMap, sync::Arc};
use winit::window::{WindowAttributes, WindowId, Window};
use winit::event_loop::ActiveEventLoop;

use crate::renderer::VulkanRenderer;


pub struct WindowHandler {
    renderers: HashMap<WindowId, VulkanRenderer>,
    windows: HashMap<WindowId, Arc<Window>>,
    primary_window_id: WindowId,
}

impl WindowHandler {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self> {
        let window = Arc::new(event_loop.create_window(WindowAttributes::default())?);
        let window_id = window.id();
        let windows = HashMap::from([(window_id, window)]);

        let renderers = windows.iter().map(|(id, window)| {
            let renderer = VulkanRenderer::new(window.clone()).unwrap();

            (*id, renderer)
        }).collect::<HashMap<_,_>>();

        Ok(Self {
            renderers,
            windows,
            primary_window_id: window_id,
        })
    }

    pub fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event:WindowEvent) {
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
}
