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
