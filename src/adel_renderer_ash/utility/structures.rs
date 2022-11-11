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
impl SurfaceInfo {
    pub fn update_screen_width_height(&mut self, width: u32, height: u32) {
        self.screen_width = width;
        self.screen_height = height;
    }
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
#[derive(Debug)]
#[repr(C)]
pub struct Vertex2d {
    pub position: nalgebra::Vector2::<f32>,
    pub color: nalgebra::Vector3::<f32>,
}
pub struct TriangleComponent {
    pub verticies: Vec<Vertex2d>
}
impl TriangleComponent {
    pub fn new(verticies: Vec<Vertex2d>) -> Self {
        assert_eq!(verticies.len(), 3);
        Self {
            verticies
        }
    }
}
use ash::vk::{Buffer, DeviceMemory};
// TODO: Create separate files for Vertex specific structs
// TODO: Come up with a better name for this
pub struct VertexBuffer {
    pub buffer: Buffer,
    pub memory: DeviceMemory,
}
#[repr(C)]
#[derive(Debug)]
pub struct PushConstantData {
    pub transform: nalgebra::Matrix4<f32>,
    pub color: nalgebra::Vector3<f32>
}