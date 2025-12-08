use ash::{vk, Device};
use std::{mem::ManuallyDrop, sync::Arc};

pub struct Swapchain {
	pub swapchain: vk::SwapchainKHR,
	pub images: Vec<vk::Image>,
	pub image_views: Vec<vk::ImageView>,
	pub format: vk::Format,
	pub extent: vk::Extent2D,
	device: Arc<Device>,
	swapchain_loader: Arc<ash::khr::swapchain::Device>,
	surface: vk::SurfaceKHR,
	surface_format: vk::SurfaceFormatKHR,
	present_mode: vk::PresentModeKHR,
	min_image_count: u32,
	queue_family_indices: Vec<u32>,
}

impl Swapchain {
	pub fn new(
		device: &Arc<Device>,
		swapchain_loader: &ash::khr::swapchain::Device,
		surface_format: vk::SurfaceFormatKHR,
		extent: vk::Extent2D,
		surface: vk::SurfaceKHR,
		present_mode: vk::PresentModeKHR,
		min_image_count: u32,
		queue_family_indices: &[u32],
	) -> Swapchain {
		Self::create_swapchain_internal(
			device,
			swapchain_loader,
			surface_format,
			extent,
			surface,
			present_mode,
			min_image_count,
			queue_family_indices,
			vk::SwapchainKHR::null(),
		)
	}

	pub fn recreate(
		&mut self,
		extent: vk::Extent2D,
	) {
		// Wait for device to be idle before any recreation
		unsafe {
			let _ = self.device.device_wait_idle();
		}

		let old_swapchain = self.swapchain;
		let old_image_views = std::mem::take(&mut self.image_views);

		// Create new swapchain referencing the old one
		let new_swapchain = Self::create_swapchain_internal(
			&self.device,
			&self.swapchain_loader,
			self.surface_format,
			extent,
			self.surface,
			self.present_mode,
			self.min_image_count,
			&self.queue_family_indices,
			old_swapchain,
		);

		// Wrap in ManuallyDrop to prevent Drop from destroying the handles we're about to move
		let mut new_swapchain = ManuallyDrop::new(new_swapchain);
		
		// Update to new swapchain data BEFORE destroying old resources
		self.swapchain = new_swapchain.swapchain;
		self.images = new_swapchain.images.clone();
		self.image_views = new_swapchain.image_views.clone();
		self.format = new_swapchain.format;
		self.extent = new_swapchain.extent;

		// Manually drop the device Arc to release our extra reference
		// SAFETY: We're only dropping the Arc, not the Vulkan handles
		unsafe {
			std::ptr::drop_in_place(&mut new_swapchain.device);
			std::ptr::drop_in_place(&mut new_swapchain.swapchain_loader);
		}

		// NOW destroy old resources after ensuring device is idle again
		unsafe {
			let _ = self.device.device_wait_idle();
			for &image_view in &old_image_views {
				self.device.destroy_image_view(image_view, None);
			}
			self.swapchain_loader.destroy_swapchain(old_swapchain, None);
		}
	}

	fn create_swapchain_internal(
		device: &Arc<Device>,
		swapchain_loader: &ash::khr::swapchain::Device,
		surface_format: vk::SurfaceFormatKHR,
		extent: vk::Extent2D,
		surface: vk::SurfaceKHR,
		present_mode: vk::PresentModeKHR,
		min_image_count: u32,
		queue_family_indices: &[u32],
		old_swapchain: vk::SwapchainKHR,
	) -> Swapchain {
		println!("Creating swapchain with format: {:?}, extent: {}x{}", surface_format.format, extent.width, extent.height);
		
		let swapchain_create_info = vk::SwapchainCreateInfoKHR {
			surface,
			min_image_count,
			image_format: surface_format.format,
			image_color_space: surface_format.color_space,
			image_extent: extent,
			image_array_layers: 1,
			image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
			image_sharing_mode: vk::SharingMode::EXCLUSIVE,
			queue_family_index_count: queue_family_indices.len() as u32,
			p_queue_family_indices: if queue_family_indices.len() > 1 {
				queue_family_indices.as_ptr()
			} else {
				std::ptr::null()
			},
			pre_transform: vk::SurfaceTransformFlagsKHR::IDENTITY,
			composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
			present_mode,
			clipped: vk::TRUE,
			old_swapchain,
			..Default::default()
		};

		let swapchain = unsafe {
			swapchain_loader
				.create_swapchain(&swapchain_create_info, None)
				.expect("Failed to create swapchain!")
		};

		let images = unsafe {
			swapchain_loader
				.get_swapchain_images(swapchain)
				.expect("Failed to get swapchain images!")
		};

		let image_views = images
			.iter()
			.map(|&image| {
				let create_view_info = vk::ImageViewCreateInfo {
					image,
					view_type: vk::ImageViewType::TYPE_2D,
					format: surface_format.format,
					components: vk::ComponentMapping {
						r: vk::ComponentSwizzle::IDENTITY,
						g: vk::ComponentSwizzle::IDENTITY,
						b: vk::ComponentSwizzle::IDENTITY,
						a: vk::ComponentSwizzle::IDENTITY,
					},
					subresource_range: vk::ImageSubresourceRange {
						aspect_mask: vk::ImageAspectFlags::COLOR,
						base_mip_level: 0,
						level_count: 1,
						base_array_layer: 0,
						layer_count: 1,
					},
					..Default::default()
				};
				unsafe {
					device
						.create_image_view(&create_view_info, None)
						.expect("Failed to create image view!")
				}
			})
			.collect();

		Swapchain {
			swapchain,
			images,
			image_views,
			format: surface_format.format,
			extent,
			device: device.clone(),
			swapchain_loader: Arc::new(swapchain_loader.clone()),
			surface,
			surface_format,
			present_mode,
			min_image_count,
			queue_family_indices: queue_family_indices.to_vec(),
		}
	}
}

impl Drop for Swapchain {
	fn drop(&mut self) {
		unsafe {
			let _ = self.device.device_wait_idle();
			for &image_view in &self.image_views {
				self.device.destroy_image_view(image_view, None);
			}
			self.swapchain_loader.destroy_swapchain(self.swapchain, None);
		}
	}
}
