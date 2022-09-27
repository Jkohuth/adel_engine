use ash::vk;
use log;
use crate::adel_renderer_vulkan::utility::structures;

pub fn choose_swapchain_format(
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

pub fn choose_swapchain_present_mode(
    available_present_modes: &Vec<vk::PresentModeKHR>,
) -> vk::PresentModeKHR {
//    for &available_present_mode in available_present_modes.iter() {
//        if available_present_mode == vk::PresentModeKHR::MAILBOX {
//            return available_present_mode;
//        }
//    }
    let available_present_mode = available_present_modes.iter()
        .min_by_key( |present_mode|
            match **present_mode {
                vk::PresentModeKHR::MAILBOX => 0,
                vk::PresentModeKHR::FIFO => 1,
                vk::PresentModeKHR::FIFO_RELAXED => 2,
                vk::PresentModeKHR::IMMEDIATE => 3,
                _ => panic!("ERROR: Unknown present mode found {:?}", present_mode)
    }).unwrap();
    log::info!("Present mode: {:?}", &available_present_mode);
    *available_present_mode
}

pub fn choose_swapchain_extent(
    capabilities: &vk::SurfaceCapabilitiesKHR,
    window: &winit::window::Window,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::max_value() {
        capabilities.current_extent
    } else {
        use num::clamp;

        let window_size = window
            .inner_size();
        log::info!(
            "\t\tInner Window Size: ({}, {})",
            window_size.width, window_size.height
        );

        vk::Extent2D {
            width: clamp(
                window_size.width as u32,
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: clamp(
                window_size.height as u32,
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }
}

pub fn query_swapchain_support(
    physical_device: vk::PhysicalDevice,
    surface_info: &structures::SurfaceInfo,
) -> structures::SwapChainSupportDetail {
    unsafe {
        let capabilities = surface_info
            .surface_loader
            .get_physical_device_surface_capabilities(physical_device, surface_info.surface)
            .expect("Failed to query for surface capabilities.");
        let formats = surface_info
            .surface_loader
            .get_physical_device_surface_formats(physical_device, surface_info.surface)
            .expect("Failed to query for surface formats.");
        let present_modes = surface_info
            .surface_loader
            .get_physical_device_surface_present_modes(physical_device, surface_info.surface)
            .expect("Failed to query for surface present mode.");

        structures::SwapChainSupportDetail {
            capabilities,
            formats,
            present_modes,
        }
    }
}

pub fn create_swapchain(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    window: &winit::window::Window,
    surface_info: &structures::SurfaceInfo,
    queue_family: &structures::QueueFamilyIndices,
) -> structures::SwapChainInfo {
    let swapchain_support = query_swapchain_support(physical_device, surface_info);

    let surface_format = choose_swapchain_format(&swapchain_support.formats);
    let present_mode = choose_swapchain_present_mode(&swapchain_support.present_modes);
    let extent = choose_swapchain_extent(&swapchain_support.capabilities, window);

    let image_count = swapchain_support.capabilities.min_image_count + 1;
    let image_count = if swapchain_support.capabilities.max_image_count > 0 {
        image_count.min(swapchain_support.capabilities.max_image_count)
    } else {
        image_count
    };

    let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
        if queue_family.graphics_family != queue_family.present_family {
            (
                vk::SharingMode::CONCURRENT,
                2,
                vec![
                    queue_family.graphics_family.unwrap(),
                    queue_family.present_family.unwrap(),
                ],
            )
        } else {
            (vk::SharingMode::EXCLUSIVE, 0, vec![])
        };
    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface_info.surface)
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

    let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("ERROR: Failed to create swapchain")
    };
    let swapchain_images = unsafe {
        swapchain_loader
            .get_swapchain_images(swapchain)
            .expect("ERROR: Failed to get swapchain images")
    };

    structures::SwapChainInfo {
        swapchain_loader,
        swapchain,
        swapchain_format: surface_format.format,
        swapchain_extent: extent,
        swapchain_images,
    }
}
