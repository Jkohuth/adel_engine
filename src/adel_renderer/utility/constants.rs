use ash::vk;
use std::os::raw::c_char;
use super::structures::DeviceExtension;

// Constants
// Window title will change when this starts to interact with the rest of the code
pub const WINDOW_TITLE: &'static str = "Adel Engine";
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub const APPLICATION_VERSION: u32 = vk::make_api_version(0, 1, 0, 0);
pub const ENGINE_VERSION: u32 = vk::make_api_version(0, 1, 0, 0);
// TODO: Find out about API versioning here and what would work best
pub const API_VERSION: u32 = vk::make_api_version(0, 1, 3, 0);


pub const VALIDATION_LAYERS: &[&str] =  &["VK_LAYER_KHRONOS_validation"];
pub const ENABLE_VALIDATION_LAYERS: bool = cfg!(debug_assertions);

pub const DEVICE_EXTENSIONS: DeviceExtension = DeviceExtension {
    names: ["VK_KHR_swapchain"],
};
impl DeviceExtension {
    pub fn get_extensions_raw_names(&self) -> [*const c_char; 1] {
        [
            // currently just enable the Swapchain extension.
            ash::extensions::khr::Swapchain::name().as_ptr(),
        ]
    }
}
