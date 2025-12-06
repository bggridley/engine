use anyhow::Result;
use std::{sync::Arc, os::raw::c_char};

use winit::{
    raw_window_handle::HasDisplayHandle, raw_window_handle::HasWindowHandle, window::Window,
};

use ash::{
    // ext::debug_utils,
    vk, 
    Entry, Instance
};
pub struct VulkanContext {
     entry: Entry,
     instance: Instance,
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

            let surface = ash_window::create_surface(
                &entry,
                &instance,
                raw_display_handle,
                raw_window_handle,
                None,
            )?;

            let physical_devices = instance.enumerate_physical_devices()?;

            Ok(Self {entry, instance})
        }

    }
}

// impl Drop for Context {

// }