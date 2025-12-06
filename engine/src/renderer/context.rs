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
    vk,
    Entry,
    Instance,
};
pub struct VulkanContext {
    pub entry: Entry,
    pub instance: Instance,
    pub graphics_devices: Vec<(vk::PhysicalDevice, Vec<u32>)>,
    pub surface_loader: ash::khr::surface::Instance,
    pub raw_display_handle: RawDisplayHandle,
    pub raw_window_handle: RawWindowHandle,
}

pub struct QueueFamilies {
    pub index: u32,
    pub properties: vk::QueueFamilyProperties,
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

            // let surface = ash_window::create_surface(
            //     &entry,
            //     &instance,
            //     raw_display_handle,
            //     raw_window_handle,
            //     None,
            // )?;
            let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

            // filters down to devices that support
            let graphics_devices: Vec<(vk::PhysicalDevice, Vec<u32>)> = instance
                .enumerate_physical_devices()
                .unwrap()
                .into_iter()
                .filter_map(|pdevice| {
                    // for each physical device, look at its queue families
                    let queue_families =
                        instance.get_physical_device_queue_family_properties(pdevice);

                    queue_families.iter().enumerate().find_map(|(_index, info)| {
                        if info.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                            let queue_count = info.queue_count;
                            // Store indices 0..queue_count
                            let queue_indices =
                                (0..queue_count).map(|i| i as u32).collect::<Vec<_>>();
                            Some((pdevice, queue_indices))
                        } else {
                            None
                        }
                    })
                })
                .collect();

            if graphics_devices.is_empty() {
                return Err(anyhow!("No graphics-capable devices found!"));
            }

            Ok(Self {
                entry,
                instance,
                graphics_devices,
                surface_loader,
                raw_display_handle,
                raw_window_handle,
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
