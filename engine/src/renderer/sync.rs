use ash::{vk, Device};
use std::sync::Arc;

/// Manages frame synchronization with semaphores and fences
pub struct FrameSynchronizer {
    device: Arc<Device>,
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub current_frame: usize,
    pub max_frames_in_flight: usize,
}

impl FrameSynchronizer {
    /// Create a new frame synchronizer with the specified number of frames in flight
    pub fn new(device: &Arc<Device>, max_frames_in_flight: usize) -> Self {
        let mut image_available_semaphores = vec![];
        let mut render_finished_semaphores = vec![];
        let mut in_flight_fences = vec![];

        for _ in 0..max_frames_in_flight {
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();
            let image_available = unsafe {
                device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create image available semaphore!")
            };
            image_available_semaphores.push(image_available);

            let render_finished = unsafe {
                device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create render finished semaphore!")
            };
            render_finished_semaphores.push(render_finished);

            let fence_create_info = vk::FenceCreateInfo::default()
                .flags(vk::FenceCreateFlags::SIGNALED);
            let fence = unsafe {
                device
                    .create_fence(&fence_create_info, None)
                    .expect("Failed to create fence!")
            };
            in_flight_fences.push(fence);
        }

        FrameSynchronizer {
            device: device.clone(),
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame: 0,
            max_frames_in_flight,
        }
    }

    /// Get the current frame index
    pub fn current_frame_index(&self) -> usize {
        self.current_frame
    }

    /// Get the current frame's image available semaphore
    pub fn current_image_available_semaphore(&self) -> vk::Semaphore {
        self.image_available_semaphores[self.current_frame]
    }

    /// Get the current frame's render finished semaphore
    pub fn current_render_finished_semaphore(&self) -> vk::Semaphore {
        self.render_finished_semaphores[self.current_frame]
    }

    /// Get the current frame's in-flight fence
    pub fn current_in_flight_fence(&self) -> vk::Fence {
        self.in_flight_fences[self.current_frame]
    }

    /// Wait for the current frame's fence to be signaled and reset it
    pub fn begin_frame(&mut self) -> Result<(), vk::Result> {
        unsafe {
            self.device.wait_for_fences(&[self.current_in_flight_fence()], true, u64::MAX)?;
            self.device.reset_fences(&[self.current_in_flight_fence()])?;
        }
        Ok(())
    }

    /// Advance to the next frame
    pub fn end_frame(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.max_frames_in_flight;
    }

    /// Wait for the current frame's fence to be signaled
    pub fn wait_for_fence(&self) -> Result<(), vk::Result> {
        unsafe {
            self.device.wait_for_fences(
                &[self.current_in_flight_fence()],
                true,
                u64::MAX,
            )
        }
    }

    /// Reset the current frame's fence
    pub fn reset_fence(&self) -> Result<(), vk::Result> {
        unsafe {
            self.device.reset_fences(&[self.current_in_flight_fence()])
        }
    }
}

impl Drop for FrameSynchronizer {
    fn drop(&mut self) {
        unsafe {
            // Wait for all GPU work to complete before destroying synchronization objects
            let _ = self.device.device_wait_idle();
            
            for &semaphore in &self.image_available_semaphores {
                self.device.destroy_semaphore(semaphore, None);
            }
            for &semaphore in &self.render_finished_semaphores {
                self.device.destroy_semaphore(semaphore, None);
            }
            for &fence in &self.in_flight_fences {
                self.device.destroy_fence(fence, None);
            }
        }
    }
}
