use anyhow::Result;
use ash::vk;

use super::constants::MAX_FRAMES_IN_FLIGHT;
use crate::adel_renderer::definitions::UniformBufferObject;
use crate::adel_renderer::utility::buffer::AshBuffer;

pub struct AshDescriptors {
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pub global_descriptor_sets: Vec<vk::DescriptorSet>,
}

impl AshDescriptors {
    pub fn new(device: &ash::Device, uniform_buffers: &Vec<AshBuffer>) -> Result<AshDescriptors> {
        let descriptor_set_layout = AshDescriptors::create_descriptor_set_layout_ubo(device)?;
        //let descriptor_pool = AshDescriptors::create_descriptor_pool_ubo_sampler(&device)?;
        let descriptor_pool = AshDescriptors::create_descriptor_pool_ubo(&device)?;
        let global_descriptor_sets = AshDescriptors::create_descriptor_sets_uniform(
            device,
            descriptor_pool,
            descriptor_set_layout,
            uniform_buffers,
        )?;
        Ok(Self {
            descriptor_pool,
            descriptor_set_layout: descriptor_set_layout.clone(),
            global_descriptor_sets,
        })
    }
    fn create_descriptor_set_layout_ubo(device: &ash::Device) -> Result<vk::DescriptorSetLayout> {
        let ubo_layout_bindings = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .build();
        let bindings = &[ubo_layout_bindings];
        let descriptor_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings)
            .build();
        let descriptor_set_layout =
            unsafe { device.create_descriptor_set_layout(&descriptor_layout_info, None)? };
        Ok(descriptor_set_layout)
    }
    #[allow(dead_code)]
    fn create_descriptor_pool_ubo(device: &ash::Device) -> Result<vk::DescriptorPool> {
        let uniform_size = vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32)
            .build();
        let pool_size = &[uniform_size];
        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(pool_size)
            .max_sets(MAX_FRAMES_IN_FLIGHT as u32)
            .build();
        let descriptor_pool =
            unsafe { device.create_descriptor_pool(&descriptor_pool_create_info, None)? };
        Ok(descriptor_pool)
    }
    #[allow(dead_code)]
    fn create_descriptor_pool_ubo_sampler(device: &ash::Device) -> Result<vk::DescriptorPool> {
        let uniform_size = vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32)
            .build();
        let sampler_size = vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32)
            .build();
        let pool_size = &[uniform_size, sampler_size];
        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(pool_size)
            .max_sets(MAX_FRAMES_IN_FLIGHT as u32)
            .build();
        let descriptor_pool =
            unsafe { device.create_descriptor_pool(&descriptor_pool_create_info, None)? };
        Ok(descriptor_pool)
    }

    pub fn create_descriptor_sets_uniform(
        device: &ash::Device,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
        uniform_buffers: &Vec<AshBuffer>,
    ) -> Result<Vec<vk::DescriptorSet>> {
        let layouts: Vec<vk::DescriptorSetLayout> =
            vec![descriptor_set_layout; MAX_FRAMES_IN_FLIGHT];

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts)
            .build();

        let descriptor_sets =
            unsafe { device.allocate_descriptor_sets(&descriptor_set_allocate_info)? };
        for (i, &descriptor_set) in descriptor_sets.iter().enumerate() {
            let descriptor_buffer_info = [vk::DescriptorBufferInfo::builder()
                .buffer(uniform_buffers[i].buffer())
                // TODO: As DescriptorSets may differ when using multiple these values will need to be passed in
                .range(uniform_buffers[i].buffer_size())
                //.range(std::mem::size_of::<UniformBufferObject>() as u64)
                .offset(0)
                .build()];
            log::info!("Descriptor Buffer Info {:?}", &descriptor_buffer_info);
            let ubo_write = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set)
                .dst_array_element(0)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&descriptor_buffer_info)
                .build();
            let descriptor_write_sets = [ubo_write];
            unsafe {
                device.update_descriptor_sets(&descriptor_write_sets, &[]);
            }
        }
        Ok(descriptor_sets)
    }
    pub fn create_descriptor_sets_uniform_texture(
        device: &ash::Device,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
        uniform_buffers: &Vec<vk::Buffer>,
        texture_image_view: vk::ImageView,
        texture_sampler: vk::Sampler,
    ) -> Result<Vec<vk::DescriptorSet>> {
        let layouts: Vec<vk::DescriptorSetLayout> =
            vec![descriptor_set_layout; MAX_FRAMES_IN_FLIGHT];

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts)
            .build();

        let descriptor_sets =
            unsafe { device.allocate_descriptor_sets(&descriptor_set_allocate_info)? };
        for (i, &descriptor_set) in descriptor_sets.iter().enumerate() {
            let descriptor_buffer_info = [vk::DescriptorBufferInfo::builder()
                .buffer(uniform_buffers[i])
                .range(std::mem::size_of::<UniformBufferObject>() as u64)
                .offset(0)
                .build()];
            let ubo_write = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set)
                .dst_array_element(0)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&descriptor_buffer_info)
                .build();
            let image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture_image_view)
                .sampler(texture_sampler)
                .build();
            let sampler_write = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set)
                .dst_array_element(0)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&[image_info])
                .build();
            let descriptor_write_sets = [ubo_write, sampler_write];
            unsafe {
                device.update_descriptor_sets(&descriptor_write_sets, &[]);
            }
        }
        Ok(descriptor_sets)
    }
    pub fn create_descriptor_sets_uniform_self(
        &self,
        device: &ash::Device,
        uniform_buffers: &Vec<AshBuffer>,
    ) -> Result<Vec<vk::DescriptorSet>> {
        AshDescriptors::create_descriptor_sets_uniform(
            device,
            self.descriptor_pool,
            self.descriptor_set_layout,
            uniform_buffers,
        )
    }
    pub fn create_descriptor_sets_uniform_texture_self(
        &self,
        device: &ash::Device,
        uniform_buffers: &Vec<vk::Buffer>,
        texture_image_view: vk::ImageView,
        texture_sampler: vk::Sampler,
    ) -> Result<Vec<vk::DescriptorSet>> {
        AshDescriptors::create_descriptor_sets_uniform_texture(
            device,
            self.descriptor_pool,
            self.descriptor_set_layout,
            uniform_buffers,
            texture_image_view,
            texture_sampler,
        )
    }
    pub fn descriptor_pool(&self) -> vk::DescriptorPool {
        self.descriptor_pool
    }
    pub fn descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set_layout
    }
    pub unsafe fn destroy_descriptor_pool(&mut self, device: &ash::Device) {
        device.destroy_descriptor_pool(self.descriptor_pool, None);
    }
    pub unsafe fn destroy_descriptor_set_layout(&mut self, device: &ash::Device) {
        device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
    }
    pub unsafe fn destroy_descriptors(&mut self, device: &ash::Device) {
        self.destroy_descriptor_pool(device);
        self.destroy_descriptor_set_layout(device);
    }
}
