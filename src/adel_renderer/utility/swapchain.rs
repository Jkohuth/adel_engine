use super::context::AshContext;
use super::structures::SurfaceInfo;
use anyhow::Result;
use ash::vk;
use log;

pub struct AshSwapchain {
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub swapchain_info: SwapChainInfo,
    pub image_views: Vec<vk::ImageView>,
}
impl AshSwapchain {
    pub fn new(
        context: &AshContext,
        device: &ash::Device,
        window_size: (u32, u32),
    ) -> Result<Self> {
        let graphics_queue =
            unsafe { device.get_device_queue(context.queue_family.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(context.queue_family.present_family.unwrap(), 0) };
        let swapchain_info = AshSwapchain::create_swapchain(context, device, window_size)?;
        let image_views = AshSwapchain::create_swapchain_image_views(
            &device,
            swapchain_info.swapchain_format,
            &swapchain_info.swapchain_images,
        )?;
        Ok(Self {
            graphics_queue,
            present_queue,
            swapchain_info,
            image_views,
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
                // NOTE: MAILBOX present mode seems to habe an issue rendering a triangle when using Intel
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
        //log::info!("Present mode: {:?}", &available_present_mode);
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

    pub unsafe fn destroy_swapchain(&mut self, device: &ash::Device) {
        for &image_view in self.image_views.iter() {
            device.destroy_image_view(image_view, None);
        }
        self.swapchain_info
            .swapchain_loader
            .destroy_swapchain(self.swapchain_info.swapchain, None);
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
