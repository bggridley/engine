use anyhow::Result;
use std::sync::Arc;
use winit::window::Window;

pub struct VulkanRenderer {

}

impl VulkanRenderer {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        Ok(Self{})
    }
}