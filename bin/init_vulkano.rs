
use adel::renderer::{VulkanoRenderer, Vertex2d};
use winit::{
    event::{
        Event,
        WindowEvent,
    },
    event_loop::{
        EventLoop,
        ControlFlow,
    },
    window::{
        Window,
        WindowBuilder,
    },
};

const WINDOW_TITLE: &'static str = "adel Engine";
const WINDOW_WIDTH: u32 = 400;
const WINDOW_HEIGHT: u32 = 300;

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let verticies = vec![    Vertex2d { position: [0.5, 0.5], color: [0.0, 1.0, 0.0], },
                             Vertex2d { position: [0.0, -0.5], color: [0.0, 1.0, 0.0], },
                             Vertex2d { position: [-0.5, 0.5], color: [0.0, 1.0, 0.0], } ];
    let event_loop: EventLoop<()> = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_inner_size(winit::dpi::LogicalSize::new(
            WINDOW_WIDTH as f32,
            WINDOW_HEIGHT as f32,
        )).build(&event_loop).unwrap();

    let verticies = vec![   Vertex2d { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0], },
                            Vertex2d { position: [0.5, -0.5], color: [0.0, 1.0, 0.0], },
                            Vertex2d { position: [-0.5, 0.5], color: [0.0, 1.0, 0.0], } ];

    let mut renderer = VulkanoRenderer::new(window);
    use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
    renderer.vulkano_window().window().set_visible(true);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event,
                ..
            } => match event {
                WindowEvent::Resized(_)  => {
                    renderer.vulkano_window().set_recreate_swapchain(true);
                },
                WindowEvent::CloseRequested {
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput  {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {
                    //println!("Jakob Event is {:?}", event);
                }
            } Event::RedrawEventsCleared => {
                // Removed as this entire class may be
                //renderer.render(verticies.clone());
            }
            _ => (),
        } 
    });
    
}