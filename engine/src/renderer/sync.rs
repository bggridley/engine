use ash::{vk, Device};
use std::sync::Arc;

/// Manages frame synchronization with semaphores and fences
/// Uses frames-in-flight pattern with per-image render finished semaphores
pub struct FrameSynchronizer {
    device: Arc<Device>,
    /// Semaphores signaled when image is acquired (one per frame in flight)
    pub image_available_semaphores: Vec<vk::Semaphore>,
    /// Semaphores signaled when rendering is complete (one PER SWAPCHAIN IMAGE)
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    /// Fences to track CPU-GPU sync (one per frame in flight)
    pub in_flight_fences: Vec<vk::Fence>,
    /// Fences tracking which swapchain image is in use (one per swapchain image)
    pub images_in_flight: Vec<Option<vk::Fence>>,
    max_frames_in_flight: usize,
}

impl FrameSynchronizer {
    /// Create synchronization primitives for frames-in-flight rendering
    /// max_frames_in_flight: Usually 2 (double buffering) or 3 (triple buffering)
    /// swapchain_image_count: Number of images in the swapchain
    pub fn new(device: &Arc<Device>, max_frames_in_flight: usize, swapchain_image_count: usize) -> Self {
        let mut image_available_semaphores = vec![];
        let mut render_finished_semaphores = vec![];
        let mut in_flight_fences = vec![];

        // Create per-frame resources (for CPU-GPU synchronization)
        for _ in 0..max_frames_in_flight {
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();
            
            let image_available = unsafe {
                device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create image available semaphore!")
            };
            image_available_semaphores.push(image_available);

            let fence_create_info = vk::FenceCreateInfo::default()
                .flags(vk::FenceCreateFlags::SIGNALED);
            let fence = unsafe {
                device
                    .create_fence(&fence_create_info, None)
                    .expect("Failed to create fence!")
            };
            in_flight_fences.push(fence);
        }

        // Create per-image render finished semaphores (for presentation synchronization)
        for _ in 0..swapchain_image_count {
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();
            let render_finished = unsafe {
                device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create render finished semaphore!")
            };
            render_finished_semaphores.push(render_finished);
        }

        // Track which frame is using which swapchain image
        let images_in_flight = vec![None; swapchain_image_count];

        FrameSynchronizer {
            device: device.clone(),
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
            max_frames_in_flight,
        }
    }

    /// Wait for the current frame's fence and reset it
    pub fn wait_for_frame(&self, frame_index: usize) -> Result<(), vk::Result> {
        let fence = self.in_flight_fences[frame_index];
        unsafe {
            self.device.wait_for_fences(&[fence], true, u64::MAX)?;
        }
        Ok(())
    }

    /// Reset the current frame's fence
    pub fn reset_fence(&self, frame_index: usize) -> Result<(), vk::Result> {
        let fence = self.in_flight_fences[frame_index];
        unsafe {
            self.device.reset_fences(&[fence])?;
        }
        Ok(())
    }

    /// Get the current frame's fence
    pub fn get_fence(&self, frame_index: usize) -> vk::Fence {
        self.in_flight_fences[frame_index]
    }

    /// Get acquire semaphore for a given frame index
    pub fn get_acquire_semaphore(&self, frame_index: usize) -> vk::Semaphore {
        self.image_available_semaphores[frame_index]
    }

    /// Get render finished semaphore for a specific IMAGE index (not frame index)
    pub fn get_render_finished_semaphore(&self, image_index: u32) -> vk::Semaphore {
        self.render_finished_semaphores[image_index as usize]
    }

    /// Get the max frames in flight
    pub fn max_frames_in_flight(&self) -> usize {
        self.max_frames_in_flight
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
