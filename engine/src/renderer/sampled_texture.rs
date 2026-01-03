use anyhow::Result;
use ash::vk;
use std::sync::Arc;

use super::Texture;

/// A texture with sampler and descriptor sets ready for shader use
/// This encapsulates all the Vulkan boilerplate for texture sampling
pub struct SampledTexture {
    pub sampler: vk::Sampler,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_set: vk::DescriptorSet,
    device: Arc<ash::Device>,
}

/// Configuration for texture sampling
#[derive(Clone, Copy)]
pub struct SamplerConfig {
    pub mag_filter: vk::Filter,
    pub min_filter: vk::Filter,
    pub address_mode: vk::SamplerAddressMode,
    pub anisotropy: Option<f32>,  // None = disabled, Some(n) = enabled with max anisotropy n
}

impl SamplerConfig {
    /// Linear filtering with edge clamping - good for UI, text, sprites
    pub fn linear() -> Self {
        Self {
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            address_mode: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            anisotropy: None,
        }
    }

    /// Nearest filtering - for pixel art
    pub fn nearest() -> Self {
        Self {
            mag_filter: vk::Filter::NEAREST,
            min_filter: vk::Filter::NEAREST,
            address_mode: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            anisotropy: None,
        }
    }

    /// Linear with repeating - good for tiled 3D textures
    pub fn linear_repeat() -> Self {
        Self {
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            address_mode: vk::SamplerAddressMode::REPEAT,
            anisotropy: Some(16.0),  // Enable anisotropic filtering for 3D
        }
    }
}

impl SampledTexture {
    /// Create a sampled texture from a Texture
    /// 
    /// This sets up everything needed to use a texture in shaders:
    /// - Creates a sampler with the specified filtering
    /// - Creates descriptor pool and layout
    /// - Allocates and binds descriptor set
    pub fn new(
        texture: &Texture,
        config: SamplerConfig,
        device: &Arc<ash::Device>,
    ) -> Result<Self> {
        unsafe {
            // Create sampler
            let sampler_info = vk::SamplerCreateInfo::default()
                .mag_filter(config.mag_filter)
                .min_filter(config.min_filter)
                .address_mode_u(config.address_mode)
                .address_mode_v(config.address_mode)
                .address_mode_w(config.address_mode)
                .anisotropy_enable(config.anisotropy.is_some())
                .max_anisotropy(config.anisotropy.unwrap_or(1.0))
                .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
                .unnormalized_coordinates(false)
                .compare_enable(false)
                .compare_op(vk::CompareOp::ALWAYS)
                .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                .mip_lod_bias(0.0)
                .min_lod(0.0)
                .max_lod(0.0);
            
            let sampler = device.create_sampler(&sampler_info, None)?;

            // Create descriptor pool
            let pool_sizes = [
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::SAMPLED_IMAGE,
                    descriptor_count: 1,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::SAMPLER,
                    descriptor_count: 1,
                },
            ];

            let pool_info = vk::DescriptorPoolCreateInfo::default()
                .pool_sizes(&pool_sizes)
                .max_sets(1);

            let descriptor_pool = device.create_descriptor_pool(&pool_info, None)?;

            // Create descriptor set layout
            // Binding 0: SAMPLED_IMAGE (the texture)
            // Binding 1: SAMPLER (the sampling settings)
            let bindings = [
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
                vk::DescriptorSetLayoutBinding::default()
                    .binding(1)
                    .descriptor_type(vk::DescriptorType::SAMPLER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            ];
            
            let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
                .bindings(&bindings);
            
            let descriptor_set_layout = device.create_descriptor_set_layout(&layout_info, None)?;

            // Allocate descriptor set
            let layouts = [descriptor_set_layout];
            let alloc_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(descriptor_pool)
                .set_layouts(&layouts);

            let descriptor_set = device.allocate_descriptor_sets(&alloc_info)?[0];

            // Write descriptor set to bind the texture and sampler
            let image_info = [vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture.image_view)];

            let sampler_info_write = [vk::DescriptorImageInfo::default()
                .sampler(sampler)];

            let descriptor_writes = [
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_set)
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .image_info(&image_info),
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_set)
                    .dst_binding(1)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::SAMPLER)
                    .image_info(&sampler_info_write),
            ];

            device.update_descriptor_sets(&descriptor_writes, &[]);

            Ok(SampledTexture {
                sampler,
                descriptor_pool,
                descriptor_set_layout,
                descriptor_set,
                device: Arc::clone(device),
            })
        }
    }

    /// Destroy all Vulkan resources
    pub fn destroy(&self) {
        unsafe {
            self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.device.destroy_sampler(self.sampler, None);
        }
    }
}

impl Drop for SampledTexture {
    fn drop(&mut self) {
        self.destroy();
    }
}
