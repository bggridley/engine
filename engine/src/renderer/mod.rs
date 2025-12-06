use anyhow::Result;
use std::sync::Arc;

use winit::{
    raw_window_handle::HasDisplayHandle, raw_window_handle::HasWindowHandle, window::Window,
};

use ash::{
    khr::{surface, swapchain},
    vk, Device,
};

mod context;
pub use context::VulkanContext;

pub struct VulkanRenderer {}

impl VulkanRenderer {
    pub fn new(window: Arc<Window>, context: Arc<VulkanContext>) -> Result<Self> {
        unsafe {
            

            Ok(Self {})
        }
    }
}
