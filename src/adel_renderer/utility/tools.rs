use anyhow::Result;
use ash::vk;
use std::ffi::CStr;
use std::os::raw::c_char;

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

pub fn create_shader_module(device: &ash::Device, code: &[u32]) -> Result<vk::ShaderModule> {
    let shader_module_create_info = vk::ShaderModuleCreateInfo::builder().code(&code).build();

    // Call to graphics card to build shader
    let shader_module = unsafe { device.create_shader_module(&shader_module_create_info, None)? };
    Ok(shader_module)
}
