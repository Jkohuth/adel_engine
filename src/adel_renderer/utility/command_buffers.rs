use super::constants::MAX_FRAMES_IN_FLIGHT;
use super::structures;
use super::{context::AshContext, pipeline::AshPipeline, swapchain::AshSwapchain};
use anyhow::Result;
use ash::vk;
pub struct AshCommandBuffers {
    pub command_pool: vk::CommandPool,
    transient_command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    graphics_transient_queue: vk::Queue,
}
impl AshCommandBuffers {
    pub fn new(
        device: &ash::Device,
        context: &AshContext,
        swapchain: &AshSwapchain,
    ) -> Result<Self> {
        log::info!("Creating Command Pool");
        let command_pool = AshCommandBuffers::create_command_pool(
            &device,
            &context.queue_family,
            vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        )?;
        let command_buffers = AshCommandBuffers::create_command_buffers(&device, command_pool)?;
        let graphics_transient_queue = swapchain.graphics_queue.clone();
        let transient_command_pool = AshCommandBuffers::create_command_pool(
            &device,
            &context.queue_family,
            vk::CommandPoolCreateFlags::TRANSIENT,
        )?;

        Ok(Self {
            command_pool,
            transient_command_pool,
            command_buffers,
            graphics_transient_queue,
        })
    }
    pub fn create_command_pool(
        device: &ash::Device,
        queue_families: &structures::QueueFamilyIndices,
        flags: vk::CommandPoolCreateFlags,
    ) -> Result<vk::CommandPool> {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(flags)
            .queue_family_index(queue_families.graphics_family.unwrap())
            .build();

        let command_pool = unsafe { device.create_command_pool(&command_pool_create_info, None)? };
        Ok(command_pool)
    }
    fn create_command_buffers(
        device: &ash::Device,
        command_pool: vk::CommandPool,
    ) -> Result<Vec<vk::CommandBuffer>> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32)
            .level(vk::CommandBufferLevel::PRIMARY)
            .build();

        let command_buffers =
            unsafe { device.allocate_command_buffers(&command_buffer_allocate_info)? };
        Ok(command_buffers)
    }
    pub fn recreate_command_buffers(&mut self, device: &ash::Device) -> Result<()> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32)
            .level(vk::CommandBufferLevel::PRIMARY)
            .build();

        self.command_buffers =
            unsafe { device.allocate_command_buffers(&command_buffer_allocate_info)? };

        Ok(())

    }
    pub fn command_buffers(&self) -> &Vec<vk::CommandBuffer> {
        &self.command_buffers
    }
    pub fn command_pool(&self) -> &vk::CommandPool {
        &self.command_pool
    }
    pub fn submit_queue(&self) -> vk::Queue {
        self.graphics_transient_queue
    }
    // Destroyers, called by renderer Drop method
    pub unsafe fn free_command_buffers(&mut self, device: &ash::Device) {
        device.free_command_buffers(self.command_pool, &self.command_buffers);
    }
    pub unsafe fn destroy_command_pools(&mut self, device: &ash::Device) {
        device.destroy_command_pool(self.command_pool, None);
        device.destroy_command_pool(self.transient_command_pool, None);
    }
    pub unsafe fn destroy_all(&mut self, device: &ash::Device) {
        // Commandbuffers, and CommandPool need cleanup
        self.free_command_buffers(device);
        self.destroy_command_pools(device);
    }
}
