use ash::vk;

pub struct DeviceExtension {
    pub names: [&'static str; 1],
    //    pub raw_names: [*const i8; 1],
}

pub struct SurfaceInfo {
    pub surface_loader: ash::extensions::khr::Surface,
    pub surface: vk::SurfaceKHR,

    pub screen_width: u32,
    pub screen_height: u32,
}

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new() -> QueueFamilyIndices {
        QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}
use nalgebra;
#[repr(C)]
pub struct Vertex {
    pub position: nalgebra::Vector2::<f32>,
    pub color: nalgebra::Vector3::<f32>,
}
#[repr(C)]
#[derive(Debug)]
pub struct PushConstantData {
    pub transform: nalgebra::Matrix4<f32>,
    pub color: nalgebra::Vector3<f32>
}
pub unsafe fn as_bytes<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        std::mem::size_of::<T>()
    )
}