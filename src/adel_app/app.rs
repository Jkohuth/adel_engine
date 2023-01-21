use crate::adel_ecs::World;
//use crate::adel_renderer::{VulkanoRenderer};
#[allow(unused_imports)]
use crate::adel_ecs::{RunStage, System};
use crate::adel_winit::WinitWindow;
use crate::adel_camera::Camera;
use crate::adel_input::{ KeyboardHandler, InputConsumer };
use crate::adel_renderer::RendererAsh;
use nalgebra::Vector3;
use std::collections::HashSet;
use std::time;
#[allow(unused_imports)]
use log;

use winit::{
    event::{
        Event,
        WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop },
    window::Window,
};
use std::rc::Rc;

pub struct Application {
    pub world: World,
    pub systems: Vec<Box<dyn System>>,
    pub window: Rc<Window>,
    event_loop: EventLoop<()>,
}

impl Application {
    // Application is steadily growing with all of the initialization other systems need to implement
    // Perhaps make it a trait of systems to have an init function that can run
    pub fn new(mut world: World) -> Self {
        let mut winit_window = WinitWindow::new();
        let event_loop: EventLoop<()> = winit_window.event_loop().unwrap();
        let renderer_ash = RendererAsh::new(winit_window.rc_clone_window());
        let app_window_ref = winit_window.rc_clone_window();
        //let renderer = VulkanoRenderer::new(winit_window.window().unwrap());
        // Create the input Consumer and keyboard handler
        let keyboard_handler = KeyboardHandler::new();
        let input_consumer = InputConsumer { pressed: HashSet::new() };
        let mut camera = Camera::new();
        //camera.set_orthographic_projection(-1.0, 1.0, -1.0, 1.0, 0.0, 10.0);
        //log::info!("Camera Info Position: {:?}\nProjection: {:?}", camera.position, camera.get_projection());
        //camera.set_orthographic_projection_pos(1.0, 1.0, 10.0);
        // TODO: Move this to camera startup script
        let mut dims;
        let mut aspect_ratio;
        {
            dims = app_window_ref.as_ref().inner_size();
            aspect_ratio = dims.width as f32 / dims.height as f32;
        }
        camera.set_perspective_projection((50.0f32).to_radians(), aspect_ratio, 0.1, 10.0);
        camera.set_view_target(nalgebra::Vector3::<f32>::new(2.0, 2.0, 2.0), nalgebra::Vector3::<f32>::new(0.0, 0.0, 0.0),
            Some(nalgebra::Vector3::<f32>::new(0.0, 0.0, 1.0)));
        //camera.set_view_direction(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.1, 0.0, 1.0), None);
        //camera.set_view_target(Vector3::<f32>::new(-1.0, 2.0, -2.0), Vector3::<f32>::new(0.0, 0.0, 2.5), None);
        // The current lack of depth buffering effects whether it can be rendered
        //camera.set_view_yxz(Vector3::new(0.0, 0.0, -2.5),  Vector3::new(0.0, 0.0, 0.0));
        world.insert_resource::<InputConsumer>(input_consumer);
        world.insert_resource::<Camera>(camera);
        //log::info!("What is the value {:?}", keyboard.pressed);
        let mut systems: Vec<Box<dyn System>> = Vec::new();
        systems.push(Box::new(keyboard_handler));
        systems.push(Box::new(renderer_ash));
        systems.push(Box::new(winit_window));
        Self {
            world,
            systems,
            window: app_window_ref,
            event_loop,
        }
    }

    pub fn main_loop(mut self) {
        #[allow(unused_imports)]
        use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
        let mut current_time = time::Instant::now();
        // Initialize everything that needs to at startup
        {
            for i in &mut self.systems {
                i.as_mut().startup(&mut self.world);
            }
        }
        self.event_loop.run(move |event, _, control_flow| {
            //*control_flow = ControlFlow::Wait;
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent {
                    event,
                    ..
                } => match event {
                    WindowEvent::Resized(_)  => {
                        //for i in &mut self.systems {
                        //    if i.name().eq("Renderer") {

                        //    }
                        //}
                    //    Need to send a message into the Renderer Class that a resize has occured
                    },
                    // Leave the close requested event here for now
                    WindowEvent::CloseRequested {
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { ref input, .. } => {
                        // Special casing is bad design, but I'll leave this for now
                        if input.virtual_keycode.unwrap() == VirtualKeyCode::Escape {
                            *control_flow = ControlFlow::Exit;
                        }
                        else if let Some(mut keyboard_input) = self.world.get_resource_mut::<InputConsumer>() {
                            keyboard_input.capture_keyboard_input(input);
                        }

                    },
                    _ => {
                        // Need to pass the pressed variable into a keyboard class
                        // Collect the various keyboard inputs and pass them into a class that can
                        // handle input
                        //println!("Keyboard Input Virtual_keycode: {:?}", event);
                    }
                }
                Event::MainEventsCleared => {
                    // Handle Time Step after user input
                    let new_time = time::Instant::now();
                    let frame_time = current_time.elapsed().as_secs_f32();//new_time.duration_since(current_time).as_secs_f32();
                    // TODO: Properly store deltaTime in world
                    //println!("FrameTime {}", frame_time);
                    self.world.update_dt(frame_time);

                    current_time = new_time;
                    // TODO: This works, for now, with a small number of systems, iterating through them 3 times per update isn't
                    // terrible (right now) just not ideal, perhaps it'd be worth storing systems in different buckets?
                    // What can I do with a HashMap<Key, Vec<Systems>>??? something to think about
                    for i in &mut self.systems {
                        if i.get_run_stage() == RunStage::Update {
                            i.as_mut().run(&mut self.world);
                        }
                    }

                } Event::RedrawRequested(_window_id) => {
                    // Redraw frame
                    for i in &mut self.systems {
                        if i.get_run_stage() == RunStage::RedrawUpdate {
                            i.as_mut().run(&mut self.world);
                        }
                    }

                },
                Event::RedrawEventsCleared => {
                    for i in &mut self.systems {
                        if i.get_run_stage() == RunStage::LateUpdate {
                            i.as_mut().run(&mut self.world);
                        }
                    }
                },
                Event::LoopDestroyed => {
                    for i in &mut self.systems {
                        i.as_mut().shutdown(&mut self.world);
                    }
                }
                _ => (),
            }
        });
    }
}