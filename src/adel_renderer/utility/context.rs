use ash::vk;

use anyhow::{anyhow, Result};
use std::ffi::CString;

use super::{
    constants::*,
    debug, platforms,
    structures::{QueueFamilyIndices, SurfaceInfo},
    swapchain, tools,
};
use winit::window::Window;

pub struct AshContext {
    pub instance: ash::Instance,
    pub surface_info: SurfaceInfo,
    pub physical_device: vk::PhysicalDevice,
    pub physical_device_properties: vk::PhysicalDeviceProperties,
    pub queue_family: QueueFamilyIndices,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl AshContext {
    pub fn new(entry: &ash::Entry, window: &Window) -> Result<Self> {
        let instance = AshContext::create_instance(
            entry,
            ENABLE_VALIDATION_LAYERS,
            &VALIDATION_LAYERS.to_vec(),
        )?;
        let surface_info = AshContext::create_surface(entry, &instance, window)?;
        let physical_device = AshContext::pick_physical_device(&instance, &surface_info)?;
        let physical_device_properties =
            unsafe { instance.get_physical_device_properties(physical_device) };

        let queue_family =
            AshContext::find_queue_family(&instance, physical_device, &surface_info)?;
        let (debug_utils_loader, debug_messenger) =
            debug::setup_debug_utils(ENABLE_VALIDATION_LAYERS, &entry, &instance)?;
        Ok(Self {
            instance,
            surface_info,
            physical_device,
            physical_device_properties,
            queue_family,
            debug_utils_loader,
            debug_messenger,
        })
    }

    fn create_instance(
        entry: &ash::Entry,
        is_enable_debug: bool,
        required_validation_layers: &Vec<&str>,
    ) -> Result<ash::Instance> {
        if is_enable_debug
            && !AshContext::check_validation_layer_support(entry, required_validation_layers)?
        {
            panic!("Validation layers requested, but unavailable");
        }

        let app_name = CString::new(WINDOW_TITLE).unwrap();
        let engine_name = CString::new("Adel Engine").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(APPLICATION_VERSION)
            .engine_name(&engine_name)
            .api_version(APPLICATION_VERSION)
            .build();

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

        let instance: ash::Instance = unsafe { entry.create_instance(&create_info, None)? };

        Ok(instance)
    }

    fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &winit::window::Window,
    ) -> Result<SurfaceInfo> {
        let surface = unsafe {
            platforms::create_surface(entry, instance, window)
                .expect("Error: Failed to create Surface")
        };

        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);
        Ok(SurfaceInfo {
            surface_loader,
            surface,
            screen_width: window.inner_size().width,
            screen_height: window.inner_size().height,
        })
    }

    fn pick_physical_device(
        instance: &ash::Instance,
        surface_info: &SurfaceInfo,
    ) -> Result<vk::PhysicalDevice> {
        let physical_device = unsafe { instance.enumerate_physical_devices()? };

        let result = physical_device
            .iter()
            .filter(|physical_device| {
                AshContext::is_physical_device_suitable(instance, **physical_device, surface_info)
                    .unwrap_or(false)
            })
            .min_by_key(|physical_device| {
                let device_properties =
                    unsafe { instance.get_physical_device_properties(**physical_device) };
                let device_name = tools::vk_to_string(&device_properties.device_name);
                log::info!("Suitable GPU Found: {}", device_name);

                match device_properties.device_type {
                    vk::PhysicalDeviceType::DISCRETE_GPU => 0,
                    vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
                    vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
                    vk::PhysicalDeviceType::CPU => 3,
                    vk::PhysicalDeviceType::OTHER => 4,
                    _ => {
                        log::warn!("Found Device Type outside enumeration");
                        999
                    }
                }
            });

        match result {
            Some(p_physical_device) => {
                // TODO: Remove these extra calls
                let device_properties =
                    unsafe { instance.get_physical_device_properties(*p_physical_device) };
                let device_name = super::tools::vk_to_string(&device_properties.device_name);
                log::info!("Using GPU: {}", device_name);
                return Ok(*p_physical_device);
            }
            None => Err(anyhow!("Error: Failed to find a suitable GPU!")),
        }
    }

    fn is_physical_device_suitable(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_info: &SurfaceInfo,
    ) -> Result<bool> {
        let device_features = unsafe { instance.get_physical_device_features(physical_device) };
        if device_features.sampler_anisotropy != vk::TRUE {
            log::warn!("Physical Device does not support Sampler Anisotropy");
            return Ok(false);
        }
        let indices = AshContext::find_queue_family(instance, physical_device, surface_info)?;

        // Missing queue family, either graphics or present, return false
        if !indices.is_complete() {
            return Ok(false);
        }

        let is_device_extension_supported =
            AshContext::check_device_extension_support(instance, physical_device)?;
        let is_swapchain_supported = if is_device_extension_supported {
            let swapchain_support =
                swapchain::query_swapchain_support(physical_device, surface_info)?;
            !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
        } else {
            false
        };

        let is_support_sampler_anisotropy = device_features.sampler_anisotropy == 1;

        return Ok(is_device_extension_supported
            && is_swapchain_supported
            && is_support_sampler_anisotropy);
    }

    fn find_queue_family(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_info: &SurfaceInfo,
    ) -> Result<QueueFamilyIndices> {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut queue_family_indices = QueueFamilyIndices::new();

        let mut index = 0;
        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_family = Some(index);
            }
            let is_present_support = unsafe {
                surface_info
                    .surface_loader
                    .get_physical_device_surface_support(
                        physical_device,
                        index as u32,
                        surface_info.surface,
                    )?
            };

            if queue_family.queue_count > 0 && is_present_support {
                queue_family_indices.present_family = Some(index);
            }

            if queue_family_indices.is_complete() {
                break;
            }

            index += 1;
        }
        Ok(queue_family_indices)
    }

    fn check_device_extension_support(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> Result<bool> {
        let available_extensions =
            unsafe { instance.enumerate_device_extension_properties(physical_device)? };

        let mut available_extension_names = vec![];

        for extension in available_extensions.iter() {
            let extension_name = tools::vk_to_string(&extension.extension_name);
            available_extension_names.push(extension_name);
        }

        use std::collections::HashSet;
        let mut required_extensions = HashSet::new();
        for extension in DEVICE_EXTENSIONS.names.iter() {
            required_extensions.insert(extension.to_string());
        }

        for extension_name in available_extension_names.iter() {
            required_extensions.remove(extension_name);
        }

        return Ok(required_extensions.is_empty());
    }

    fn check_validation_layer_support(
        entry: &ash::Entry,
        required_validation_layers: &Vec<&str>,
    ) -> Result<bool> {
        let layer_properties = entry.enumerate_instance_layer_properties()?;

        if layer_properties.len() <= 0 {
            log::info!("No layers available");
            return Ok(false);
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
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn get_supported_format(
        &self,
        candidates: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Result<vk::Format> {
        let supported_formats = unsafe {
            candidates.iter().cloned().find(|f| {
                let properties = self
                    .instance()
                    .get_physical_device_format_properties(self.physical_device, *f);
                match tiling {
                    vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                    vk::ImageTiling::OPTIMAL => {
                        properties.optimal_tiling_features.contains(features)
                    }
                    _ => false,
                }
            })
        };
        match supported_formats {
            Some(format) => return Ok(format),
            None => {
                return Err(anyhow!(
                    "No supported formats found: Candidates: {:?}",
                    candidates
                ))
            }
        }
    }

    pub fn get_min_uniform_buffer_offset_alignment(&self) -> u64 {
        self.physical_device_properties.limits.min_uniform_buffer_offset_alignment
    }
    pub fn get_non_coherent_atom_size(&self) -> u64 {
        self.physical_device_properties.limits.non_coherent_atom_size
    }
    // Other structs require device to cleanup resources properly, I'm providing a cleanup function
    // here in order to properly remove it in the drop function
    pub unsafe fn destroy_context(&mut self) {
        self.surface_info
            .surface_loader
            .destroy_surface(self.surface_info.surface, None);

        if ENABLE_VALIDATION_LAYERS {
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_messenger, None);
        }
        self.instance.destroy_instance(None);
    }

    pub fn instance(&self) -> &ash::Instance {
        &self.instance
    }
}

// Since Device is used frequently in function calls, I'm going to keep this value stored top level
pub fn create_logical_device(
    context: &AshContext,
    required_validation_layers: &Vec<&str>,
) -> Result<ash::Device> {
    use std::collections::HashSet;
    let mut unique_queue_familes = HashSet::new();
    unique_queue_familes.insert(context.queue_family.graphics_family.unwrap());
    unique_queue_familes.insert(context.queue_family.present_family.unwrap());

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
    let physical_device_features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .build();

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
        context
            .instance
            .create_device(context.physical_device, &device_create_info, None)?
    };

    Ok(device)
}
