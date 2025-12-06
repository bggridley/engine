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
            let surface = ash_window::create_surface(
                &context.entry,
                &context.instance,
                context.raw_display_handle,
                context.raw_window_handle,
                None,
            )?;

            // Find a device that support the window surface
            let (physical_device, graphics_queue_indices) = context
                .graphics_devices
                .iter()
                .filter(|(pdevice, _queue_indices)| {
                    // Check if at least one queue family supports the surface
                    _queue_indices.iter().any(|&queue_family_index| {
                        context
                            .surface_loader
                            .get_physical_device_surface_support(
                                *pdevice,
                                queue_family_index,
                                surface,
                            )
                            .unwrap_or(false)
                    })
                })
                .max_by_key(|(pdevice, _)| {
                    VulkanContext::rate_device(window.id(), &context.instance, *pdevice)
                })
                .expect("Couldn't find a physical device.");

            let queue_families = context
                .instance
                .get_physical_device_queue_family_properties(*physical_device);

            // use copied because it's all references -- we don't want refs, we want actual values
            let graphics_family = graphics_queue_indices
                .iter()
                .copied()
                .find(|&i| {
                    queue_families[i as usize]
                        .queue_flags
                        .contains(vk::QueueFlags::GRAPHICS)
                })
                .expect("No graphics queue family found");

            let compute_family = graphics_queue_indices
                .iter()
                .copied()
                .find(|&i| {
                    queue_families[i as usize]
                        .queue_flags
                        .contains(vk::QueueFlags::COMPUTE)
                })
                .unwrap_or(graphics_family);

            let present_family = graphics_queue_indices
                .iter()
                .copied()
                .find(|&i| {
                    context
                        .surface_loader
                        .get_physical_device_surface_support(*physical_device, i, surface)
                        .unwrap_or(false)
                })
                .unwrap_or(graphics_family);

            let transfer_family = graphics_queue_indices
                .iter()
                .copied()
                .find(|&i| {
                    queue_families[i as usize]
                        .queue_flags
                        .contains(vk::QueueFlags::TRANSFER)
                })
                .unwrap_or(graphics_family);

            // Unique families
            let unique_families: std::collections::HashSet<u32> =
                [graphics_family, compute_family, present_family]
                    .iter()
                    .copied()
                    .collect();

            let mut queue_create_infos = vec![];
            let mut queue_priorities_storage: Vec<Vec<f32>> = vec![];

            for &family_index in &unique_families {
                let queue_count = queue_families[family_index as usize].queue_count;
                let priorities = vec![1.0f32; queue_count as usize];
                queue_priorities_storage.push(priorities);

                let mut queue_info = vk::DeviceQueueCreateInfo::default();
                queue_info.s_type = vk::StructureType::DEVICE_QUEUE_CREATE_INFO;
                queue_info.queue_family_index = family_index;
                queue_info.queue_count = queue_count; // create all queues in this family
                queue_info.p_queue_priorities = queue_priorities_storage.last().unwrap().as_ptr();
                queue_info.flags = vk::DeviceQueueCreateFlags::empty();

                queue_create_infos.push(queue_info);
            }

            // Create logical device
            let mut device_create_info = vk::DeviceCreateInfo::default();
            device_create_info.s_type = vk::StructureType::DEVICE_CREATE_INFO;
            device_create_info.queue_create_info_count = queue_create_infos.len() as u32;
            device_create_info.p_queue_create_infos = queue_create_infos.as_ptr();
            device_create_info.enabled_extension_count = 0;
            device_create_info.pp_enabled_extension_names = std::ptr::null();
            device_create_info.p_enabled_features = std::ptr::null();

            let device =
                context
                    .instance
                    .create_device(*physical_device, &device_create_info, None);

            println!("Got here somehow");

            Ok(Self {})
        }
    }
}
