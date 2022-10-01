

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

fn main() {

    simple_logger::SimpleLogger::new().env().init().unwrap();
    let event_loop: EventLoop<()> = EventLoop::new();
    let mut window = WindowBuilder::new()
            .with_title("Test Window")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
            .build(&event_loop)
            .expect("Failed: Create window");
    let mut vulkan_app = RendererAsh::new(window);
    main_loop(vulkan_app, event_loop);
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
                    app.window.request_redraw();
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