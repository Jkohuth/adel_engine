use crate::adel_ecs::World;
//use crate::adel_renderer::{VulkanoRenderer};
#[allow(unused_imports)]
use crate::adel_ecs::System;
use crate::adel_winit::WinitWindow;
use crate::adel_camera::Camera;
use crate::adel_input::{ KeyboardHandler, InputConsumer };
use crate::adel_renderer_vulkan::RendererAsh;
use glam::{Vec3};
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
};

pub struct Application {
    pub world: World,
    pub systems: Vec<Box<dyn System>>,
    event_loop: EventLoop<()>,
}

impl Application {
    // Application is steadily growing with all of the initialization other systems need to implement
    // Perhaps make it a trait of systems to have an init function that can run
    pub fn new(mut world: World) -> Self {
        let mut winit_window = WinitWindow::new();
        let event_loop: EventLoop<()> = winit_window.event_loop().unwrap();
        let renderer_ash = RendererAsh::new(winit_window);
        //let renderer = VulkanoRenderer::new(winit_window.window().unwrap());
        // Create the input Consumer and keyboard handler
        let keyboard_handler = KeyboardHandler::new();
        let input_consumer = InputConsumer { pressed: HashSet::new() };
        let mut camera = Camera::new();
        // TODO: Set up Fovy with radian angle
        //camera.set_perspective_projection((50.0f32).to_radians(), renderer.vulkano_window().aspect_ratio(), 0.1, 10.0);
        //camera.set_view_direction(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.1, 0.0, 1.0), None);
        //camera.set_view_target(Vec3::new(-1.0, 2.0, -2.0), Vec3::new(0.0, 0.0, 2.5), None);
        camera.set_view_yxz(Vec3::new(-1.0, 2.0, -2.0),
        Vec3::new(0.0, 0.0, 0.0));
        world.insert_resource::<InputConsumer>(input_consumer);
        world.insert_resource::<Camera>(camera);
        //log::info!("What is the value {:?}", keyboard.pressed);
        let mut systems: Vec<Box<dyn System>> = Vec::new();
        systems.push(Box::new(keyboard_handler));
        systems.push(Box::new(renderer_ash));
        Self {
            world,
            systems,
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
                    //    Need to send a message into the Renderer Class that a resize has occured
                    },
                    // Leave the close requested event here for now
                    WindowEvent::CloseRequested {
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { ref input, .. } => {
                        if let Some(mut keyboard_input) = self.world.get_resource_mut::<InputConsumer>() {
                            keyboard_input.keyboard_input_system(input);
                        }
                        // Special casing is bad design, but I'll leave this for now
                        if input.virtual_keycode.unwrap() == VirtualKeyCode::Escape {
                            *control_flow = ControlFlow::Exit;
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
                    //self.world.update_dt(0.05);

                    current_time = new_time;

                    // All events have been processed so it's time to draw,
                    // TODO: Make generic App.update, fix gametick
                    for i in &mut self.systems {
                        i.as_mut().run(&mut self.world);
                    }
                    // Currently all the systems are ran in order
                    //
                    // Perhaps request redraw here
                    //renderer_ash.window_ref().unwrap().request_redraw();

                } Event::RedrawRequested(_window_id) => {
                    // Redraw frame

                },
                Event::RedrawEventsCleared => {

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