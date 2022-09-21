
use ash::vk;
//use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
//use winit::event_loop::{EventLoop, ControlFlow};

use log;
use std::ffi::{CString};
use std::os::raw::c_char;

// TODO: Create a prelude and add these to it
use crate::adel_renderer_vulkan::utility::{
    constants::*,
    debug,
    platforms,
    structures,
    tools,
    functions,
};

pub struct VulkanApp {
    // vulkan stuff
    _entry: ash::Entry,
    instance: ash::Instance,
    surface_loader: ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,

    _physical_device: vk::PhysicalDevice,
    device: ash::Device,

    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
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
        let instance = functions::create_instance(&entry, ENABLE_VALIDATION_LAYERS, &VALIDATION_LAYERS.to_vec());
        let (debug_utils_loader, debug_messenger) = debug::setup_debug_utils(ENABLE_VALIDATION_LAYERS, &entry, &instance);
        let surface_info = functions::create_surface(&entry, &instance, &window);
        let physical_device = functions::pick_physical_device(&instance, &surface_info);
        let (device, family_indices) = create_logical_device(&instance, physical_device, &surface_info, &VALIDATION_LAYERS.to_vec());
        let graphics_queue =
            unsafe { device.get_device_queue(family_indices.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(family_indices.present_family.unwrap(), 0) };
        Self {
            _entry: entry,
            instance,
            surface_loader: surface_info.surface_loader,
            surface: surface_info.surface,
            debug_utils_loader,
            debug_messenger,
            _physical_device: physical_device,
            device,
            graphics_queue,
            present_queue,
            window,
        }

    }

}

pub fn create_logical_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface_info: &structures::SurfaceInfo,
    required_validation_layers: &Vec<&str>
) ->  (ash::Device, structures::QueueFamilyIndices) {
    let indices = functions::find_queue_family(instance, physical_device, surface_info);

    use std::collections::HashSet;
    let mut unique_queue_familes = HashSet::new();
    unique_queue_familes.insert(indices.graphics_family.unwrap());
    unique_queue_familes.insert(indices.present_family.unwrap());

    let queue_priorities = [1.0_f32];
    let mut queue_create_infos = vec![];
    for &queue_family in unique_queue_familes.iter() {
        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
        .flags(vk::DeviceQueueCreateFlags::empty())
        .queue_family_index(queue_family)
        .queue_priorities(&queue_priorities)
        .build();
        queue_create_infos.push(queue_create_info);
    }
    let physical_device_features = vk::PhysicalDeviceFeatures::builder().build();

    let requred_validation_layer_raw_names: Vec<CString> = required_validation_layers
        .iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect();
    let layer_names: Vec<*const i8> = requred_validation_layer_raw_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

        let enabled_extension_names = DEVICE_EXTENSIONS.get_extensions_raw_names();


    let device_create_info = if ENABLE_VALIDATION_LAYERS {
     vk::DeviceCreateInfo::builder()
        .flags(vk::DeviceCreateFlags::empty())
        .queue_create_infos(&queue_create_infos)
        .enabled_layer_names(&layer_names)
        .enabled_extension_names(&enabled_extension_names)
        .enabled_features(&physical_device_features)
        .build()
    } else {
        vk::DeviceCreateInfo::builder()
        .flags(vk::DeviceCreateFlags::empty())
        .queue_create_infos(&queue_create_infos)
        .enabled_extension_names(&enabled_extension_names)
        .enabled_features(&physical_device_features)
        .build()
    };
    let device: ash::Device = unsafe {
        instance
            .create_device(physical_device, &device_create_info, None)
            .expect("ERROR: Failed to create logical device")
    };

    (device, indices)
}
