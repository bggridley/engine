use ash::{vk, Device};
use std::sync::Arc;

pub struct Swapchain {
	pub swapchain: vk::SwapchainKHR,
	pub images: Vec<vk::Image>,
	pub image_views: Vec<vk::ImageView>,
	pub format: vk::Format,
	pub extent: vk::Extent2D,
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
			old_swapchain: vk::SwapchainKHR::null(),
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
		}
	}
}
