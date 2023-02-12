use super::constants::MAX_FRAMES_IN_FLIGHT;
use super::structures;
use super::{
    buffers::AshBuffers, context::AshContext, pipeline::AshPipeline, swapchain::AshSwapchain,
};
use anyhow::Result;
use ash::vk;
pub struct AshFrameInfo {
    pub framebuffers: Vec<vk::Framebuffer>,
    pub command_pool: vk::CommandPool,
    transient_command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    graphics_transient_queue: vk::Queue,

    depth_image: vk::Image,
    depth_image_memory: vk::DeviceMemory,
    depth_image_view: vk::ImageView,
}
impl AshFrameInfo {
    pub fn new(
        device: &ash::Device,
        context: &AshContext,
        swapchain: &AshSwapchain,
        pipeline: &AshPipeline,
    ) -> Result<Self> {
        let command_pool = AshFrameInfo::create_command_pool(
            &device,
            &context.queue_family,
            vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        )?;
        let command_buffers = AshFrameInfo::create_command_buffers(&device, command_pool)?;
        let graphics_transient_queue = swapchain.graphics_queue.clone();
        let (depth_image, depth_image_memory, depth_image_view) = AshFrameInfo::create_depth_image(
            &context,
            &device,
            swapchain.swapchain_info.swapchain_extent,
            &command_pool,
            graphics_transient_queue,
        )?;
        let framebuffers = AshFrameInfo::create_framebuffers(
            &device,
            pipeline.render_pass().clone(),
            &swapchain.image_views,
            depth_image_view,
            swapchain.extent(),
        )?;
        let transient_command_pool = AshFrameInfo::create_command_pool(
            &device,
            &context.queue_family,
            vk::CommandPoolCreateFlags::TRANSIENT,
        )?;

        Ok(Self {
            framebuffers,
            command_pool,
            transient_command_pool,
            command_buffers,
            graphics_transient_queue,
            depth_image,
            depth_image_memory,
            depth_image_view,
        })
    }
    fn create_framebuffers(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        image_views: &Vec<vk::ImageView>,
        depth_image_view: vk::ImageView,
        swapchain_extent: vk::Extent2D,
    ) -> Result<Vec<vk::Framebuffer>> {
        let mut framebuffers = vec![];

        for &image_view in image_views.iter() {
            let attachments = [image_view, depth_image_view];

            let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(swapchain_extent.width)
                .height(swapchain_extent.height)
                .layers(1)
                .build();

            let framebuffer = unsafe { device.create_framebuffer(&framebuffer_create_info, None)? };

            framebuffers.push(framebuffer);
        }

        Ok(framebuffers)
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

    pub fn create_depth_image(
        context: &AshContext,
        device: &ash::Device,
        extent: vk::Extent2D,
        command_pool: &vk::CommandPool,
        submit_queue: vk::Queue,
    ) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> {
        let format = AshFrameInfo::get_depth_format(context)?;
        let (depth_image, depth_image_memory) = AshBuffers::create_image(
            context,
            device,
            extent.width,
            extent.height,
            format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        let depth_image_view = AshSwapchain::create_image_view(
            device,
            depth_image,
            format,
            vk::ImageAspectFlags::DEPTH,
            1,
        )?;

        AshBuffers::transition_image_layout(
            device,
            depth_image,
            format,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            command_pool,
            submit_queue,
        )?;

        Ok((depth_image, depth_image_memory, depth_image_view))
    }
    pub fn recreate_framebuffers(
        &mut self,
        device: &ash::Device,
        render_pass: vk::RenderPass,
        image_views: &Vec<vk::ImageView>,
        depth_image_view: vk::ImageView,
        extent: vk::Extent2D,
    ) -> Result<()> {
        let framebuffers = AshFrameInfo::create_framebuffers(
            device,
            render_pass,
            image_views,
            depth_image_view,
            extent,
        )?;
        self.framebuffers = framebuffers;
        Ok(())
    }
    pub fn recreate_depth_image(
        &mut self,
        context: &AshContext,
        device: &ash::Device,
        extent: vk::Extent2D,
    ) -> Result<()> {
        let (depth_image, depth_image_memory, depth_image_view) = AshFrameInfo::create_depth_image(
            context,
            device,
            extent,
            &self.command_pool,
            self.graphics_transient_queue,
        )?;
        self.depth_image = depth_image;
        self.depth_image_memory = depth_image_memory;
        self.depth_image_view = depth_image_view;
        Ok(())
    }

    // Reference members of the struct functions
    pub fn framebuffers(&self) -> &Vec<vk::Framebuffer> {
        &self.framebuffers
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
    pub fn depth_image_view(&self) -> vk::ImageView {
        self.depth_image_view
    }
    pub fn get_depth_format(context: &AshContext) -> Result<vk::Format> {
        let candidates = &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ];

        context.get_supported_format(
            candidates,
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }

    // Destroyers, called by renderer Drop method
    pub unsafe fn destroy_framebuffers(&mut self, device: &ash::Device) {
        for &framebuffer in self.framebuffers.iter() {
            device.destroy_framebuffer(framebuffer, None);
        }
    }
    pub unsafe fn free_command_buffers(&mut self, device: &ash::Device) {
        device.free_command_buffers(self.command_pool, &self.command_buffers);
    }
    pub unsafe fn destroy_command_pools(&mut self, device: &ash::Device) {
        device.destroy_command_pool(self.command_pool, None);
        device.destroy_command_pool(self.transient_command_pool, None);
    }
    pub unsafe fn destroy_depth_image(&mut self, device: &ash::Device) {
        device.destroy_image(self.depth_image, None);
        device.destroy_image_view(self.depth_image_view, None);
        device.free_memory(self.depth_image_memory, None);
    }

    pub unsafe fn destroy_all(&mut self, device: &ash::Device) {
        self.destroy_depth_image(device);
        // Framebuffers, Commandbuffers, and CommandPool need cleanup
        // Framebuffers need to be separated as they are removed when recreating swapchain
        self.destroy_framebuffers(device);
        self.free_command_buffers(device);
        self.destroy_command_pools(device);
    }
}
