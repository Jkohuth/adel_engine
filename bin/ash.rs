

use adel::renderer_ash::RendererAsh;
use winit::{
    event_loop::EventLoop,
    window::{
        WindowBuilder,
        Window
    }
};

use winit::event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};
use winit::event_loop::{ControlFlow};
use adel::window::WinitWindow;
fn main() {

    simple_logger::SimpleLogger::new().env().init().unwrap();
    let window = WinitWindow::new();
    let mut vulkan_app = RendererAsh::new(window);
    let event_loop = vulkan_app.window.event_loop();
    main_loop(vulkan_app, event_loop.unwrap());
}

    pub fn main_loop(mut app: RendererAsh, event_loop: EventLoop<()>) {

        event_loop.run(move |event, _, control_flow| {

            match event {
                | Event::WindowEvent { event, .. } => {
                    match event {
                        | WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit
                        },
                        | WindowEvent::KeyboardInput { input, .. } => {
                            match input {
                                | KeyboardInput { virtual_keycode, state, .. } => {
                                    match (virtual_keycode, state) {
                                        | (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                            *control_flow = ControlFlow::Exit
                                        },
                                        | _ => {},
                                    }
                                },
                            }
                        },
                        | _ => {},
                    }
                },
                | Event::MainEventsCleared => {
                    app.window.window_ref().unwrap().request_redraw();
                },
                | Event::RedrawRequested(_window_id) => {
                    app.draw_frame();
                },
                | Event::LoopDestroyed => {
                    unsafe {
                        app.device.device_wait_idle()
                            .expect("Failed to wait device idle!")
                    };
                },
                _ => (),
            }

        })
    }