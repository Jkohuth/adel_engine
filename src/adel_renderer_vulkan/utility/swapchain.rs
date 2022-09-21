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