use ash::vk;
use std::ffi::CString;

use crate::adel_renderer_vulkan::utility::{
    structures,
    constants,
    debug,
    platforms,
    swapchain,
    tools
};

pub fn create_instance(
    entry: &ash::Entry,
    is_enable_debug: bool,
    required_validation_layers: &Vec<&str>
) -> ash::Instance {
    if is_enable_debug && !check_validation_layer_support(entry, required_validation_layers) {
        panic!("Validation layers requested, but unavailable");
    }

    let app_name = CString::new(constants::WINDOW_TITLE).unwrap();
    let engine_name = CString::new("Adel Engine").unwrap();
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(constants::APPLICATION_VERSION)
        .engine_name(&engine_name)
        .api_version(constants::APPLICATION_VERSION).build();

    let mut debug_utils_create_info = debug::populate_debug_messenger_create_info();

    let extension_names = platforms::required_extension_names();

    let requred_validation_layer_raw_names: Vec<CString> = required_validation_layers
        .iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect();
    let layer_names: Vec<*const i8> = requred_validation_layer_raw_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let create_info = if !constants::ENABLE_VALIDATION_LAYERS {
        vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names)
        .build()
    } else {
        vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names)
        .push_next(&mut debug_utils_create_info)
        .enabled_layer_names(&layer_names)
        .build()
    };

    let instance: ash::Instance = unsafe {
        entry
            .create_instance(&create_info, None)
            .expect("Error: Failed to create Instance")
    };

    instance
}
pub fn create_surface(
    entry: &ash::Entry,
    instance: &ash::Instance,
    window: &winit::window::Window,

) -> structures::SurfaceInfo {
    let surface = unsafe {
        platforms::create_surface(entry, instance, window).expect("Error: Failed to create Surface")
    };

    let surface_loader = ash::extensions::khr::Surface::new(entry, instance);
    structures::SurfaceInfo {
        surface_loader,
        surface,
        screen_width: window.inner_size().width,
        screen_height: window.inner_size().height,

    }
}

pub fn pick_physical_device(
    instance: &ash::Instance,
    surface_info: &structures::SurfaceInfo,
) -> vk::PhysicalDevice {
    let physical_device = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Error: Failed to enumerate Physical Devices")
    };

    let result = physical_device.iter().filter(|physical_device| {
        let is_suitable = is_physical_device_suitable(
            instance,
            **physical_device,
            surface_info
        );
        is_suitable
    }).min_by_key(|physical_device| {
        let device_properties = unsafe { instance.get_physical_device_properties(**physical_device) };
        let device_name = tools::vk_to_string(&device_properties.device_name);
        log::info!("Suitable GPU Found: {}", device_name);

        match device_properties.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU => 0,
            vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
            vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
            vk::PhysicalDeviceType::CPU => 3,
            vk::PhysicalDeviceType::OTHER => 4,
            _ => panic!("ERROR: Undefined behavior for device_type"),
        }
    });


    match result {
        Some(p_physical_device) => {
            // TODO: Remove these extra calls
            let device_properties = unsafe { instance.get_physical_device_properties(*p_physical_device) };
            let device_name = super::tools::vk_to_string(&device_properties.device_name);
            log::info!("Using GPU: {}", device_name);
            return *p_physical_device;
        },
        None => panic!("Error: Failed to find a suitable GPU!"),
    }
}

pub fn is_physical_device_suitable(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface_info: &structures::SurfaceInfo,
) -> bool {
    let device_features = unsafe { instance.get_physical_device_features(physical_device) };
    let indices = find_queue_family(instance, physical_device, surface_info);

    // Missing queue family, either graphics or present, return false
    if !indices.is_complete() { return false; }

    let is_device_extension_supported =
        check_device_extension_support(instance, physical_device);
    let is_swapchain_supported = if is_device_extension_supported {
        let swapchain_support = swapchain::query_swapchain_support(physical_device, surface_info);
        !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
    } else {
        false
    };

    let is_support_sampler_anisotropy = device_features.sampler_anisotropy == 1;

    return is_device_extension_supported
        && is_swapchain_supported
        && is_support_sampler_anisotropy;

}

pub fn find_queue_family(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface_info: &structures::SurfaceInfo,
)-> structures::QueueFamilyIndices {
    let queue_families = unsafe {
        instance.get_physical_device_queue_family_properties(physical_device)
    };

    let mut queue_family_indices = structures::QueueFamilyIndices::new();

    let mut index = 0;
    for queue_family in queue_families.iter() {
        if queue_family.queue_count > 0 &&
            queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        {
            queue_family_indices.graphics_family = Some(index);
        }
        let is_present_support = unsafe {
            surface_info.surface_loader.get_physical_device_surface_support(
                physical_device,
                index as u32,
                surface_info.surface,
            ).expect("ERROR: Failed to load surface device support")
        };

        if queue_family.queue_count > 0 && is_present_support {
            queue_family_indices.present_family = Some(index);
        }

        if queue_family_indices.is_complete() {
            break;
        }

        index += 1;
    }
    queue_family_indices
}

pub fn check_device_extension_support(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> bool {
    let available_extensions = unsafe {
        instance.
            enumerate_device_extension_properties(physical_device)
            .expect("ERROR: Failed to get device extension properties")
    };

    let mut available_extension_names = vec![];

    for extension in available_extensions.iter() {
        let extension_name = tools::vk_to_string(&extension.extension_name);
        available_extension_names.push(extension_name);
    }

    use std::collections::HashSet;
    let mut required_extensions = HashSet::new();
    for extension in constants::DEVICE_EXTENSIONS.names.iter() {
        required_extensions.insert(extension.to_string());
    }

    for extension_name in available_extension_names.iter() {
        required_extensions.remove(extension_name);
    }

    return required_extensions.is_empty();
}

pub fn check_validation_layer_support(
    entry: &ash::Entry,
    required_validation_layers: &Vec<&str>,
) -> bool {
    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("Error: Failed to enumerate Instance Layers Properties");

    if layer_properties.len() <= 0 {
        log::info!("No layers available");
        return false;
    }

    for required_layer_name in required_validation_layers.iter() {
        let mut is_layer_found = false;

        for layer_property in layer_properties.iter() {
            let test_layer_name = tools::vk_to_string(&layer_property.layer_name);
            if (*required_layer_name) == test_layer_name {
                is_layer_found = true;
                break;
            }
        }

        if is_layer_found == false {
            return false;
        }
    }
    true
}

