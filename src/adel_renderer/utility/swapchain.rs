use super::buffer::AshBuffer;
use super::context::AshContext;
use super::structures::{QueueFamilyIndices, SurfaceInfo};
use anyhow::Result;
use ash::vk;
use log;

pub struct AshSwapchain {
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub swapchain_info: SwapChainInfo,
    pub image_views: Vec<vk::ImageView>,
    single_submit_command_pool: vk::CommandPool,

    depth_image: vk::Image,
    depth_image_memory: vk::DeviceMemory,
    depth_image_view: vk::ImageView,
    depth_format: vk::Format,
    pub render_pass: vk::RenderPass,
    pub frame_buffers: Vec<vk::Framebuffer>,
}

impl AshSwapchain {
    pub fn new(
        context: &AshContext,
        device: &ash::Device,
        window_size: (u32, u32),
    ) -> Result<Self> {
        log::info!("Context graphics Family {:?}\tContext present Family {:?}", context.queue_family.graphics_family, context.queue_family.present_family);
        let graphics_queue =
            unsafe { device.get_device_queue(context.queue_family.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(context.queue_family.present_family.unwrap(), 0) };
        log::info!("Graphics Queue {:?}\t Present Queue {:?}", graphics_queue, present_queue);
        let swapchain_info = AshSwapchain::create_swapchain(context, device, window_size)?;
        let image_views = AshSwapchain::create_swapchain_image_views(
            &device,
            swapchain_info.swapchain_format,
            &swapchain_info.swapchain_images,
        )?;

        let single_submit_command_pool = AshSwapchain::create_command_pool(
            device,
            &context.queue_family,
            vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        )?;
        let (depth_image, depth_image_memory, depth_image_view) = AshSwapchain::create_depth_image(
            &context,
            &device,
            swapchain_info.swapchain_extent,
            &single_submit_command_pool,
            graphics_queue,
        )?;
        let depth_format = AshSwapchain::get_depth_format(context)?;
        let render_pass = AshSwapchain::create_render_pass(
            &device,
            swapchain_info.swapchain_format,
            depth_format,
        )?;
        let frame_buffers = AshSwapchain::create_framebuffers(
            device,
            render_pass,
            &image_views,
            depth_image_view,
            swapchain_info.swapchain_extent,
        )?;

        Ok(Self {
            graphics_queue,
            present_queue,
            swapchain_info,
            image_views,
            single_submit_command_pool,

            depth_image,
            depth_image_memory,
            depth_image_view,
            depth_format,
            render_pass,
            frame_buffers,
        })
    }
    fn choose_swapchain_format(
        available_formats: &Vec<vk::SurfaceFormatKHR>,
    ) -> vk::SurfaceFormatKHR {
        for available_format in available_formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return available_format.clone();
            }
        }

        return available_formats.first().unwrap().clone();
    }

    fn choose_swapchain_present_mode(
        available_present_modes: &Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        //    for &available_present_mode in available_present_modes.iter() {
        //        if available_present_mode == vk::PresentModeKHR::MAILBOX {
        //            return available_present_mode;
        //        }
        //    }
        let available_present_mode = available_present_modes
            .iter()
            .min_by_key(|present_mode| match **present_mode {
                // NOTE: MAILBOX present mode seems to have an issue rendering a triangle when using Intel
                vk::PresentModeKHR::MAILBOX => 0,
                vk::PresentModeKHR::FIFO => 1,
                vk::PresentModeKHR::FIFO_RELAXED => 2,
                vk::PresentModeKHR::IMMEDIATE => 3,
                _ => {
                    log::warn!("Unknown present mode found {:?}", present_mode);
                    999
                }
            })
            .unwrap();
        log::info!("Present mode: {:?}", &available_present_mode);
        *available_present_mode
    }

    fn choose_swapchain_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        window_size: (u32, u32), //window: &winit::window::Window,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::max_value() {
            capabilities.current_extent
        } else {
            use num::clamp;

            vk::Extent2D {
                width: clamp(
                    window_size.0,
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: clamp(
                    window_size.1,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    pub fn create_swapchain(
        context: &AshContext,
        device: &ash::Device,
        window_size: (u32, u32), //window: &winit::window::Window,
    ) -> Result<SwapChainInfo> {
        let swapchain_support =
            query_swapchain_support(context.physical_device, &context.surface_info)?;

        let surface_format = AshSwapchain::choose_swapchain_format(&swapchain_support.formats);
        let present_mode =
            AshSwapchain::choose_swapchain_present_mode(&swapchain_support.present_modes);
        let extent =
            AshSwapchain::choose_swapchain_extent(&swapchain_support.capabilities, window_size);

        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
        } else {
            image_count
        };
        log::info!("Image Count {:?}", image_count);

        let (_image_sharing_mode, _queue_family_index_count, queue_family_indices) =
            if context.queue_family.graphics_family != context.queue_family.present_family {
                (
                    vk::SharingMode::CONCURRENT,
                    2,
                    vec![
                        context.queue_family.graphics_family.unwrap(),
                        context.queue_family.present_family.unwrap(),
                    ],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, 0, vec![])
            };
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(context.surface_info.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_array_layers(1)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .build();

        let swapchain_loader = ash::extensions::khr::Swapchain::new(&context.instance, device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };
        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };

        Ok(SwapChainInfo {
            swapchain_loader,
            swapchain,
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
            swapchain_images,
        })
    }

    fn create_swapchain_image_views(
        device: &ash::Device,
        surface_format: vk::Format,
        images: &Vec<vk::Image>,
    ) -> Result<Vec<vk::ImageView>> {
        let image_views: Result<Vec<vk::ImageView>> = images
            .iter()
            .map(|&image| -> Result<vk::ImageView> {
                AshSwapchain::create_image_view(
                    device,
                    image,
                    surface_format,
                    vk::ImageAspectFlags::COLOR,
                    1,
                )
            })
            .collect();
        image_views
    }
    pub fn create_image_view(
        device: &ash::Device,
        image: vk::Image,
        format: vk::Format,
        aspect_flags: vk::ImageAspectFlags,
        mip_levels: u32,
    ) -> Result<vk::ImageView> {
        let image_view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(
                vk::ComponentMapping::builder()
                    .r(vk::ComponentSwizzle::IDENTITY)
                    .g(vk::ComponentSwizzle::IDENTITY)
                    .b(vk::ComponentSwizzle::IDENTITY)
                    .a(vk::ComponentSwizzle::IDENTITY)
                    .build(),
            )
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(aspect_flags)
                    .base_mip_level(0)
                    .level_count(mip_levels)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .build();

        let image_view = unsafe { device.create_image_view(&image_view_info, None)? };

        Ok(image_view)
    }
    // Might want to remove this for clean up and call it from somewhere else
    pub fn create_command_pool(
        device: &ash::Device,
        queue_families: &QueueFamilyIndices,
        flags: vk::CommandPoolCreateFlags,
    ) -> Result<vk::CommandPool> {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(flags)
            .queue_family_index(queue_families.graphics_family.unwrap())
            .build();

        let command_pool = unsafe { device.create_command_pool(&command_pool_create_info, None)? };
        Ok(command_pool)
    }

    pub fn create_render_pass(
        device: &ash::Device,
        surface_format: vk::Format,
        depth_format: vk::Format,
    ) -> Result<vk::RenderPass> {
        let color_attachment = vk::AttachmentDescription::builder()
            .format(surface_format)
            .flags(vk::AttachmentDescriptionFlags::empty())
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();
        let depth_stencil_attachment = vk::AttachmentDescription::builder()
            .format(depth_format)
            .flags(vk::AttachmentDescriptionFlags::empty())
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let depth_stencil_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let subpasses = [vk::SubpassDescription::builder()
            .color_attachments(&[color_attachment_ref])
            .depth_stencil_attachment(&depth_stencil_attachment_ref)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .build()];

        let render_pass_attachments = [color_attachment, depth_stencil_attachment];

        let subpass_dependencies = [vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            )
            .dependency_flags(vk::DependencyFlags::empty())
            .build()];

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&render_pass_attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies)
            .build();

        let render_pass = unsafe { device.create_render_pass(&renderpass_create_info, None)? };
        Ok(render_pass)
    }

    pub fn create_depth_image(
        context: &AshContext,
        device: &ash::Device,
        extent: vk::Extent2D,
        command_pool: &vk::CommandPool,
        submit_queue: vk::Queue,
    ) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> {
        let format = AshSwapchain::get_depth_format(context)?;
        let (depth_image, depth_image_memory) = AshBuffer::create_image(
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

        AshBuffer::transition_image_layout(
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

    pub fn recreate_swapchain(
        &mut self,
        context: &AshContext,
        device: &ash::Device,
        window_size: (u32, u32),
        //window: &Window,
    ) -> Result<()> {
        let swapchain_info = AshSwapchain::create_swapchain(context, device, window_size)?;
        let image_views = AshSwapchain::create_swapchain_image_views(
            &device,
            swapchain_info.swapchain_format,
            &swapchain_info.swapchain_images,
        )?;
        self.swapchain_info = swapchain_info;
        self.image_views = image_views;

        // Returns empty result indicating that internal functions were successful
        Ok(())
    }
    pub fn recreate_framebuffers(&mut self, device: &ash::Device) -> Result<()> {
        let framebuffers = AshSwapchain::create_framebuffers(
            device,
            self.render_pass,
            &self.image_views,
            self.depth_image_view,
            self.swapchain_info.swapchain_extent,
        )?;
        self.frame_buffers = framebuffers;
        Ok(())
    }
    pub fn recreate_depth_image(
        &mut self,
        context: &AshContext,
        device: &ash::Device,
    ) -> Result<()> {
        let (depth_image, depth_image_memory, depth_image_view) = AshSwapchain::create_depth_image(
            context,
            device,
            self.swapchain_info.swapchain_extent,
            &self.single_submit_command_pool,
            self.graphics_queue,
        )?;
        self.depth_image = depth_image;
        self.depth_image_memory = depth_image_memory;
        self.depth_image_view = depth_image_view;
        Ok(())
    }

    // Reference members of the struct functions
    pub fn recreate_render_pass(&mut self, device: &ash::Device) -> Result<()> {
        let render_pass = AshSwapchain::create_render_pass(
            &device,
            self.swapchain_info.swapchain_format,
            self.depth_format,
        )?;
        self.render_pass = render_pass;
        Ok(())
    }

    // Getters for nested values

    pub fn format(&self) -> vk::Format {
        self.swapchain_info.swapchain_format
    }
    pub fn extent(&self) -> vk::Extent2D {
        self.swapchain_info.swapchain_extent
    }
    pub fn swapchain(&self) -> vk::SwapchainKHR {
        self.swapchain_info.swapchain
    }
    pub fn swapchain_loader(&self) -> &ash::extensions::khr::Swapchain {
        &self.swapchain_info.swapchain_loader
    }
    pub fn image_views(&self) -> &Vec<vk::ImageView> {
        &self.image_views
    }
    pub fn render_pass(&self) -> vk::RenderPass {
        self.render_pass
    }
    pub fn frame_buffers(&self) -> &Vec<vk::Framebuffer> {
        &self.frame_buffers
    }
    pub fn single_submit_command_pool(&self) -> &vk::CommandPool {
        &self.single_submit_command_pool
    }
    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }
    pub unsafe fn destroy_render_pass(&mut self, device: &ash::Device) {
        device.destroy_render_pass(self.render_pass, None);
    }
    pub unsafe fn destroy_swapchain(&mut self, device: &ash::Device) {
        for &image_view in self.image_views.iter() {
            device.destroy_image_view(image_view, None);
        }
        self.swapchain_info
            .swapchain_loader
            .destroy_swapchain(self.swapchain_info.swapchain, None);
    }
    pub unsafe fn destroy_frame_buffers(&mut self, device: &ash::Device) {
        for &frame_buffer in self.frame_buffers.iter() {
            device.destroy_framebuffer(frame_buffer, None);
        }
    }
    pub unsafe fn destroy_depth_data(&mut self, device: &ash::Device) {
        device.destroy_image(self.depth_image, None);
        device.destroy_image_view(self.depth_image_view, None);
        device.free_memory(self.depth_image_memory, None);
    }

    pub unsafe fn destroy_ashswapchain(&mut self, device: &ash::Device) {
        self.destroy_frame_buffers(device);
        self.destroy_render_pass(device);
        self.destroy_depth_data(device);
        device.destroy_command_pool(self.single_submit_command_pool, None);
        self.destroy_swapchain(device);
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
}

pub fn query_swapchain_support(
    physical_device: vk::PhysicalDevice,
    surface_info: &SurfaceInfo,
) -> Result<SwapChainSupportDetail> {
    unsafe {
        let capabilities = surface_info
            .surface_loader
            .get_physical_device_surface_capabilities(physical_device, surface_info.surface)?;
        let formats = surface_info
            .surface_loader
            .get_physical_device_surface_formats(physical_device, surface_info.surface)?;
        let present_modes = surface_info
            .surface_loader
            .get_physical_device_surface_present_modes(physical_device, surface_info.surface)?;

        Ok(SwapChainSupportDetail {
            capabilities,
            formats,
            present_modes,
        })
    }
}

pub struct SwapChainInfo {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
}

pub struct SwapChainSupportDetail {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}
