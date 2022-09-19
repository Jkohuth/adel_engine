

use adel::renderer_ash::VulkanApp;
use winit::event_loop::EventLoop;

fn main() {

    simple_logger::SimpleLogger::new().env().init().unwrap();
    let event_loop = EventLoop::new();
    let _vulkan_app = VulkanApp::new(&event_loop);

}