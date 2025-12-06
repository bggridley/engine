use anyhow::Result;
use std::sync::Arc;
use winit::window::Window;

mod context;
pub use context::VulkanContext;

pub struct VulkanRenderer {
    context: Arc<VulkanContext>,
}

impl VulkanRenderer {
    pub fn new(window: Arc<Window>, context: Arc<VulkanContext>) -> Result<Self> {
       
        Ok(Self{
            context
        })
    }
}