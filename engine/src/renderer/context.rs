use anyhow::{anyhow, Result};
use std::ffi::CStr;
use std::{os::raw::c_char, sync::Arc};
use winit::{
    raw_window_handle::{HasDisplayHandle, RawDisplayHandle},
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::{Window, WindowId},
};

use ash::{
    // ext::debug_utils,
    khr::swapchain,
    vk,
    Entry,
    Instance,
};
pub struct VulkanContext {
    pub entry: Entry,
    pub instance: Instance,
    pub physical_device: vk::PhysicalDevice,
    pub surface_loader: ash::khr::surface::Instance,
    pub raw_display_handle: RawDisplayHandle,
    pub raw_window_handle: RawWindowHandle,
    pub device: std::sync::Arc<ash::Device>,
    pub surface: ash::vk::SurfaceKHR,
    pub queue_family_indices: Vec<u32>,
}

impl VulkanContext {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        unsafe {
            let entry = Entry::linked();
            let app_name = c"VulkanTriangle";

            let raw_display_handle = window.display_handle()?.as_raw();
            let raw_window_handle = window.window_handle()?.as_raw();

            let layer_names = [c"VK_LAYER_KHRONOS_validation"];
            let layers_names_raw: Vec<*const c_char> = layer_names
                .iter()
                .map(|raw_name| raw_name.as_ptr())
                .collect();

            let extension_names =
                ash_window::enumerate_required_extensions(raw_display_handle)?.to_vec();

            // extension_names.push(debug_utils::NAME.as_ptr());

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            {
                extension_names.push(ash::khr::portability_enumeration::NAME.as_ptr());
                extension_names.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
            }

            let appinfo = vk::ApplicationInfo::default()
                .application_name(app_name)
                .application_version(0)
                .engine_name(app_name)
                .engine_version(0)
                .api_version(vk::API_VERSION_1_3);

            let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
                vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
            } else {
                vk::InstanceCreateFlags::default()
            };

            let create_info = vk::InstanceCreateInfo::default()
                .application_info(&appinfo)
                .enabled_layer_names(&layers_names_raw)
                .enabled_extension_names(&extension_names)
                .flags(create_flags);

            let instance: Instance = entry.create_instance(&create_info, None)?;

            let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

            // filters down to devices that support
            let graphics_devices: Vec<(vk::PhysicalDevice, Vec<u32>)> = instance
                .enumerate_physical_devices()?
                .into_iter()
                .filter_map(|pdevice| {
                    let queue_families =
                        instance.get_physical_device_queue_family_properties(pdevice);

                    let graphics_families: Vec<u32> = queue_families
                        .iter()
                        .enumerate()
                        .filter_map(|(family_index, info)| {
                            if info.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                                Some(family_index as u32)
                            } else {
                                None
                            }
                        })
                        .collect();

                    if graphics_families.is_empty() {
                        None
                    } else {
                        Some((pdevice, graphics_families))
                    }
                })
                .collect();

            if graphics_devices.is_empty() {
                return Err(anyhow!("No graphics-capable devices found!"));
            }

            // window should outlive this
            let surface = ash_window::create_surface(
                &entry,
                &instance,
                raw_display_handle,
                raw_window_handle,
                None,
            )?;

            // pick the best physical device and make sure it supports the surface; graphics_queue_indices is &Vec<u32>
            let (physical_device, graphics_queue_indices) = graphics_devices
                .iter()
                .filter(|(pdevice, families)| {
                    families.iter().any(|&family| {
                        surface_loader
                            .get_physical_device_surface_support(*pdevice, family, surface)
                            .unwrap_or(false)
                    })
                })
                .max_by_key(|(pdevice, _)| {
                    VulkanContext::rate_device(window.id(), &instance, *pdevice)
                })
                .expect("Couldn't find a physical device.");

            let queue_families =
                instance.get_physical_device_queue_family_properties(*physical_device);

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
                    surface_loader
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

            let device_extension_names_raw = [
                swapchain::NAME.as_ptr(),
                #[cfg(any(target_os = "macos", target_os = "ios"))]
                ash::khr::portability_subset::NAME.as_ptr(),
            ];

            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };

            let mut dynamic_rendering_features =
                vk::PhysicalDeviceDynamicRenderingFeatures::default().dynamic_rendering(true);

            let mut buffer_device_features =
                vk::PhysicalDeviceBufferDeviceAddressFeatures::default()
                    .buffer_device_address(true);
            // Create logical device
            let device_create_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(&queue_create_infos)
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&features)
                .push_next(&mut dynamic_rendering_features)
                .push_next(&mut buffer_device_features);

            let device = instance
                .create_device(*physical_device, &device_create_info, None)
                .expect("Failed to create logical device");

            let graphics_queue = device.get_device_queue(graphics_family, 0);
            let present_queue = device.get_device_queue(present_family, 0);
            let compute_queue = device.get_device_queue(compute_family, 0);
            let transfer_queue = device.get_device_queue(transfer_family, 0);

            println!("Graphics queue:  {:?}", graphics_queue);
            println!("Present queue:   {:?}", present_queue);
            println!("Compute queue:   {:?}", compute_queue);
            println!("Transfer queue:  {:?}", transfer_queue);

            let device_arc = Arc::new(device);
            Ok(Self {
                entry,
                instance,
                physical_device: *physical_device,
                surface_loader,
                raw_display_handle,
                raw_window_handle,
                device: device_arc,
                surface,
                queue_family_indices: unique_families.iter().copied().collect(),
            })
        }
    }

    // source for this fn:
    // https://github.com/unknownue/vulkan-tutorial-rust/blob/master/src/utility/tools.rs
    pub fn vk_to_string(raw_string_array: &[c_char]) -> String {
        let raw_string = unsafe {
            let pointer = raw_string_array.as_ptr();
            CStr::from_ptr(pointer)
        };

        raw_string
            .to_str()
            .expect("Failed to convert vulkan raw string.")
            .to_owned()
    }

    pub fn rate_device(id: WindowId, instance: &ash::Instance, device: vk::PhysicalDevice) -> i32 {
        let props = unsafe { instance.get_physical_device_properties(device) };
        let mem_props = unsafe { instance.get_physical_device_memory_properties(device) };
        let device_type = match props.device_type {
            vk::PhysicalDeviceType::CPU => "Cpu",
            vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
            vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
            vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
            vk::PhysicalDeviceType::OTHER => "Unknown",
            _ => panic!(),
        };

        let mut score = 0;

        score += match props.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU => 1000,
            vk::PhysicalDeviceType::INTEGRATED_GPU => 100,
            _ => 0,
        };

        let total_mem: u64 = mem_props.memory_heaps[..mem_props.memory_heap_count as usize]
            .iter()
            .map(|heap| heap.size)
            .sum();
        score += (total_mem / (1024 * 1024)) as i32;

        let device_name = Self::vk_to_string(&props.device_name);
        println!(
            "Device for {:?}:\n\t {}, id: {}, type: {}",
            id, device_name, props.device_id, device_type
        );
        score
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}
