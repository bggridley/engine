use ash::{vk, Device};
use std::sync::Arc;

pub struct CommandPool {
    pub pool: vk::CommandPool,
    pub buffers: Vec<vk::CommandBuffer>,
}

impl CommandPool {
    /// Create a command pool and allocate command buffers for the given queue family.
    ///
    /// # Arguments
    /// * `device` - The Vulkan logical device
    /// * `queue_family_index` - The queue family index (usually graphics family)
    /// * `buffer_count` - Number of command buffers to allocate
    pub fn new(
        device: &Arc<Device>,
        queue_family_index: u32,
        buffer_count: u32,
    ) -> Self {
        let pool_create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);

        let pool = unsafe {
            device
                .create_command_pool(&pool_create_info, None)
                .expect("Failed to create command pool!")
        };

        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(buffer_count);

        let buffers = unsafe {
            device
                .allocate_command_buffers(&allocate_info)
                .expect("Failed to allocate command buffers!")
        };

        CommandPool { pool, buffers }
    }

    pub fn allocate_buffers(
        &mut self,
        device: &Arc<Device>,
        count: u32,
    ) -> Vec<vk::CommandBuffer> {
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);

        let buffers = unsafe {
            device
                .allocate_command_buffers(&allocate_info)
                .expect("Failed to allocate command buffers!")
        };

        self.buffers.extend_from_slice(&buffers);
        buffers
    }

    /// Reset the command pool, freeing all allocated command buffers.
    pub fn reset(&self, device: &Arc<Device>) {
        unsafe {
            device
                .reset_command_pool(
                    self.pool,
                    vk::CommandPoolResetFlags::RELEASE_RESOURCES,
                )
                .expect("Failed to reset command pool!");
        }
    }

    /// Record a command buffer using the provided closure.
    pub fn begin_recording(
        device: &Arc<Device>,
        command_buffer: vk::CommandBuffer,
    ) -> Result<(), vk::Result> {
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe { device.begin_command_buffer(command_buffer, &begin_info) }
    }

    /// End recording for a command buffer.
    pub fn end_recording(
        device: &Arc<Device>,
        command_buffer: vk::CommandBuffer,
    ) -> Result<(), vk::Result> {
        unsafe { device.end_command_buffer(command_buffer) }
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        // Note: Device must still be valid when this is called.
        // In a real application, you'd want to ensure the device waits idle before dropping.
    }
}
