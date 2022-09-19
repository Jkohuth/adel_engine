
use ash::vk;
use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};

use log;
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_void};
use std::ptr;

use crate::adel_renderer_vulkan::platforms;
use crate::adel_renderer_vulkan::utility;
use crate::adel_renderer_vulkan::structures::*;
use crate::adel_renderer_vulkan::constants::*;

pub struct VulkanApp {
    // vulkan stuff
    _entry: ash::Entry,
    instance: ash::Instance,
    surface_loader: ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    window: winit::window::Window,
}

impl VulkanApp {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let window = winit::window::WindowBuilder::new()
            .with_title("Test Window")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
            .build(event_loop)
            .expect("Failed: Create window");

        // init vulkan stuff
        let entry = unsafe { ash::Entry::load().expect("Error: Failed to create Ash Entry") };
        let instance = create_instance(&entry, ENABLE_VALIDATION_LAYERS, &VALIDATION_LAYERS.to_vec());
        let (debug_utils_loader, debug_messenger) = setup_debug_utils(ENABLE_VALIDATION_LAYERS, &entry, &instance);
        let surface_info = create_surface(&entry, &instance, &window);
        let physical_device = pick_physical_device(&instance, &surface_info);
        Self {
            _entry: entry,
            instance,
            surface_loader: surface_info.surface_loader,
            surface: surface_info.surface,
            debug_utils_loader,
            debug_messenger,
            window,
        }

    }

}

pub fn create_instance(
    entry: &ash::Entry,
    is_enable_debug: bool,
    required_validation_layers: &Vec<&str>
) -> ash::Instance {
    if is_enable_debug && !check_validation_layer_support(entry, required_validation_layers) {
        panic!("Validation layers requested, but unavailable");
    }

    let app_name = CString::new(WINDOW_TITLE).unwrap();
    let engine_name = CString::new("Adel Engine").unwrap();
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(APPLICATION_VERSION)
        .engine_name(&engine_name)
        .api_version(APPLICATION_VERSION).build();

    let mut debug_utils_create_info = populate_debug_messenger_create_info();

    let mut extension_names = platforms::required_extension_names();

    let requred_validation_layer_raw_names: Vec<CString> = required_validation_layers
        .iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect();
    let layer_names: Vec<*const i8> = requred_validation_layer_raw_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let create_info = if !ENABLE_VALIDATION_LAYERS {
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

) -> SurfaceInfo {
    let surface = unsafe {
        platforms::create_surface(entry, instance, window).expect("Error: Failed to create Surface")
    };

    let surface_loader = ash::extensions::khr::Surface::new(entry, instance);
    SurfaceInfo {
        surface_loader,
        surface,
        screen_width: window.inner_size().width,
        screen_height: window.inner_size().height,

    }
}

pub fn pick_physical_device(
    instance: &ash::Instance,
    surface_info: &SurfaceInfo,
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
        let device_name = vk_to_string(&device_properties.device_name);
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
            let device_name = vk_to_string(&device_properties.device_name);
            log::info!("Using GPU: {}", device_name);
            return *p_physical_device;
        },
        None => panic!("Error: Failed to find a suitable GPU!"),
    }
}

pub fn is_physical_device_suitable(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface_info: &SurfaceInfo,
) -> bool {
    let device_features = unsafe { instance.get_physical_device_features(physical_device) };
    let indices = find_queue_family(instance, physical_device, surface_info);

    // Missing queue family, either graphics or present, return false
    if !indices.is_complete() { return false; }

    let is_device_extension_supported =
        check_device_extension_support(instance, physical_device);
    let is_swapchain_supported = if is_device_extension_supported {
        let swapchain_support = query_swapchain_support(physical_device, surface_info);
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
    surface_info: &SurfaceInfo,
)-> QueueFamilyIndices {
    let queue_families = unsafe {
        instance.get_physical_device_queue_family_properties(physical_device)
    };

    let mut queue_family_indices = QueueFamilyIndices::new();

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
        println!("JAKOB queue family count {}", &queue_family.queue_count);
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
        let extension_name = vk_to_string(&extension.extension_name);
        available_extension_names.push(extension_name);
    }

    use std::collections::HashSet;
    let mut required_extensions = HashSet::new();
    for extension in DEVICE_EXTENSIONS {
        required_extensions.insert(extension.to_string());
    }

    for extension_name in available_extension_names.iter() {
        required_extensions.remove(extension_name);
    }

    return required_extensions.is_empty();
}

pub fn query_swapchain_support(
    physical_device: vk::PhysicalDevice,
    surface_info: &SurfaceInfo,
) -> SwapChainSupportDetail {
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

        SwapChainSupportDetail {
            capabilities,
            formats,
            present_modes,
        }
    }
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
            let test_layer_name = vk_to_string(&layer_property.layer_name);
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

pub fn vk_to_string(raw_string_array: &[c_char]) -> String {
    let raw_string = unsafe {
        let pointer = raw_string_array.as_ptr();
        CStr::from_ptr(pointer)
    };

    raw_string
        .to_str()
        .expect("Failed to convert vulkan raw string.")
        .to_owned()
}
pub fn setup_debug_utils(
    is_enable_debug: bool,
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> (ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT) {
    let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);

    if is_enable_debug == false {
        (debug_utils_loader, ash::vk::DebugUtilsMessengerEXT::null())
    } else {
        let messenger_ci = populate_debug_messenger_create_info();

        let utils_messenger = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&messenger_ci, None)
                .expect("Debug Utils Callback")
        };

        (debug_utils_loader, utils_messenger)
    }
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);
    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!("Vulkan Debug Callback: Type: {}, Msg: {:?}", types, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!("Vulkan Debug Callback: Type: {}, {:?}", types, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            log::info!("Vulkan Debug Callback: Type: {} {:?}", types, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            log::debug!("Vulkan Debug Callback: Type: {}, {:?}", types, message);
        }
        _ => {
            log::debug!("[UNKNOWN] Vulkan Debug Callback: Type: {}, {:?}", types, message);
        }
    }
    vk::FALSE
}
pub fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
            // vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
            // vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(vulkan_debug_utils_callback),
        p_user_data: ptr::null_mut(),
    }
}