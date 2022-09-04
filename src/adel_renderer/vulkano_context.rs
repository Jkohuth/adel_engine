use log;
use std::sync::Arc;
use vulkano::{
    device::{
        Device, DeviceCreateInfo, DeviceExtensions,
        physical::{PhysicalDevice, PhysicalDeviceType},
        Queue,
        QueueCreateInfo,
    },
    image::{
        swapchain::SwapchainImage, ImageUsage, view::ImageView,
    },
    instance::{
        debug::{
            DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger,
            DebugUtilsMessengerCreateInfo,
        },
        Instance,
        InstanceCreateInfo,
        InstanceExtensions,
        Version,
    },
    swapchain::{
        PresentMode, 
        Surface, 
        SwapchainCreateInfo,
        Swapchain,
    },
    VulkanLibrary,
};

use winit::window::Window;

use crate::adel_renderer::FinalImageView;

//const VALIDATION_LAYERS: &[&str] =  &["VK_LAYER_KHRONOS_validation", "VK_LAYER_LUNARG_standard_validation"];
const VALIDATION_LAYERS: &[&str] =  &["VK_LAYER_KHRONOS_validation"];
const ENABLE_VALIDATION_LAYERS: bool = cfg!(debug_assertions);

pub struct VulkanoContext {
    instance: Arc<Instance>,
    _debug_callback: Option<DebugUtilsMessenger>,
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    device_name: String,
    device_type: PhysicalDeviceType
}

impl VulkanoContext {
    pub fn new() -> Self {    
        let library = VulkanLibrary::new().unwrap();
        let instance = Self::create_instance(library);
        let _debug_callback = unsafe { Self::setup_debug_callback(&instance) };
        let physical_device = PhysicalDevice::enumerate(&instance)
            .min_by_key(|p| {
                match p.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 0,
                    PhysicalDeviceType::IntegratedGpu => 1,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 3,
                    PhysicalDeviceType::Other => 4,
            }}).unwrap();
        //std::process::exit(0);
        let device_name = physical_device.properties().device_name.to_string();
        let device_type = physical_device.properties().device_type;

        log::info!("Device_Name {}, device_type {:?}", &device_name, &device_type);

        let (device, graphics_queue) = Self::create_device(physical_device);
        Self {
            instance,
            _debug_callback,
            device,
            graphics_queue,
            device_name,
            device_type
        }
    }

    fn create_instance(library: Arc<VulkanLibrary>) -> Arc<Instance> {
        let mut layers: Vec<String> = Vec::new();
        if ENABLE_VALIDATION_LAYERS {
            if Self::check_debug_layers(&library) {
                layers = VALIDATION_LAYERS.iter().map(|layer| layer.to_string()).collect::<Vec<String>>();
            } else {
                log::warn!("Requested validation layers are unavailable");
            }
        }
    
        let required_extensions = Self::required_extensions(&library);
    
        log::info!("Required Extensions: {:?}", required_extensions);
    
        Instance::new(library, 
            InstanceCreateInfo {
            enabled_extensions: required_extensions,
            enabled_layers: layers,
            application_name: Some("Adel Vulkano".into()),
            application_version: Version { major: 0, minor: 1, patch: 0 },
            engine_name: Some("Adel Engine".into()),
    
            ..Default::default()
        }).expect("Unable to create Vulkan instance")
    }
    
    fn check_debug_layers(library: &Arc<VulkanLibrary>) -> bool {
        let available_layers: Vec<String> = library.layer_properties().unwrap()
            .map(|layer| layer.name().to_owned())
            .collect();
    
        log::debug!("Available debug layers {:?}", available_layers);
    
        let available = VALIDATION_LAYERS.iter().all(|required_layer| {
            available_layers.contains(&required_layer.to_string())
        });
        available
    }
    
    fn required_extensions(library: &Arc<VulkanLibrary>) -> InstanceExtensions {
        let mut required_extensions = vulkano_win::required_extensions(library);
        if ENABLE_VALIDATION_LAYERS {
            required_extensions.ext_debug_utils = true;
        }
        required_extensions
    }
    
    pub unsafe fn setup_debug_callback(instance: &Arc<Instance>) -> Option<DebugUtilsMessenger> {
        let message_severity = DebugUtilsMessageSeverity {
            error: true,
            warning: true,
            information: true,
            verbose: true,
        };
    
        let message_type = DebugUtilsMessageType::all();

        DebugUtilsMessenger::new(instance.clone(), DebugUtilsMessengerCreateInfo {
            message_severity,
            message_type,
            ..DebugUtilsMessengerCreateInfo::user_callback(Arc::new(|msg|
                match msg.severity {
                    DebugUtilsMessageSeverity {error: true, ..} => {
                        log::error!("Vulkan Debug Callback: {:?}", msg.description);
                    }
                    DebugUtilsMessageSeverity {warning: true, ..} => {
                        log::warn!("Vulkan Debug Callback: {:?}", msg.description);
                    }
                    DebugUtilsMessageSeverity {information: true, ..} => {
                        log::info!("Vulkan Debug Callback: {:?}", msg.description);
                    }
                    DebugUtilsMessageSeverity {verbose: true, ..} => {
                        log::debug!("Vulkan Debug Callback: {:?}", msg.description);
                    }
                    _ => {
                        log::debug!("Vulkan Debug Callback: {:?}", msg.description);
                    }
                }
            ))
        })
        .ok()
    }

    fn device_extensions() -> DeviceExtensions {
        DeviceExtensions {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none()
        }
    }

    fn create_device(
        physical: PhysicalDevice, 
    ) -> (Arc<Device>, Arc<Queue>) {
        let (_gfx_index, queue_family) = physical
            .queue_families()
            .enumerate()
            .find(|&(_i, q)| q.supports_graphics())
            .unwrap();
        
        if !physical.supported_extensions().is_superset_of(&Self::device_extensions()) {
            panic!("Physical Device does not support required Extensions");
        }
        let (device, mut queues) = Device::new(
            physical,
            DeviceCreateInfo {
                enabled_extensions: Self::device_extensions(),
                queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
                ..Default::default()
            },
        ).unwrap();
        let gfx_queue = queues.next().unwrap();
        (device, gfx_queue)
    } 

    pub fn create_swapchain(
        &self, 
        surface: &Arc<Surface<Window>>
    ) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>, Vec<FinalImageView>) {
        let surface_capabilities = self.device.physical_device().surface_capabilities(&surface, Default::default()).unwrap();

        let image_format = Some(self.device.physical_device()
            .surface_formats(&surface, Default::default())
            .unwrap()[0].0);

        let surface_present_mode = self.device.physical_device().surface_present_modes(&surface).unwrap()
            .min_by_key(|k| {
                match k {
                    PresentMode::Mailbox => 0,
                    PresentMode::Fifo => 1,
                    PresentMode::FifoRelaxed => 2,
                    _ => 3
                }
            }).unwrap();
        log::info!("Present Mode {:?}", surface_present_mode);

        let (swapchain, images) = Swapchain::new(
            self.device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count,
                image_format,
                image_extent: surface.window().inner_size().into(),
                image_usage: ImageUsage::color_attachment(),
                present_mode: surface_present_mode,
                composite_alpha: surface_capabilities.supported_composite_alpha.iter().next().unwrap(),
                ..Default::default()
            },
        ).unwrap();

        let final_image = images.clone().into_iter().map(|image| ImageView::new_default(image).unwrap()).collect::<Vec<_>>();
        (swapchain, images, final_image)
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    pub fn device_type(&self) -> &PhysicalDeviceType {
        &self.device_type
    }

    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
    }
    
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn graphics_queue(&self) -> &Arc<Queue> {
        &self.graphics_queue
    }
    
}
