use crate::adel_camera::Camera;
use crate::adel_ecs::World;
use crate::adel_ecs::{RunStage, System};
use crate::adel_input::{InputConsumer, KeyboardHandler};
use crate::adel_renderer::RendererAsh;
use crate::adel_winit::WinitWindow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::mpsc;
use std::time;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

pub struct Application {
    pub world: World,
    pub systems: HashMap<String, Box<dyn System>>,
    event_loop: EventLoop<()>,
    transmitter: mpsc::Sender<(u32, u32)>,
}

impl Application {
    // Application is steadily growing with all of the initialization other systems need to implement
    // Perhaps make it a trait of systems to have an init function that can run
    pub fn new(mut world: World) -> Self {
        // WinitWindow handles boiler plate setup for windowing/event_loop
        let mut winit_window = WinitWindow::new();
        let event_loop: EventLoop<()> = winit_window.event_loop().unwrap();
        let window = winit_window.window().unwrap();
        // TODO: Tmp Values to send data to System without creating unique functions
        let (tx, rx): (mpsc::Sender<(u32, u32)>, mpsc::Receiver<(u32, u32)>) = mpsc::channel();

        let renderer_ash = RendererAsh::new(&window, rx).unwrap();
        world.insert_resource(window);

        // Create the input Consumer and keyboard handler
        let keyboard_handler = KeyboardHandler::new();
        let input_consumer = InputConsumer {
            pressed: HashSet::new(),
        };
        let camera = Camera::new();

        world.insert_resource::<InputConsumer>(input_consumer);
        world.insert_resource::<Camera>(camera);
        //log::info!("What is the value {:?}", keyboard.pressed);
        let mut systems: HashMap<String, Box<dyn System>> = HashMap::new();
        systems.insert(
            keyboard_handler.name().to_owned(),
            Box::new(keyboard_handler),
        );
        systems.insert(renderer_ash.name().to_owned(), Box::new(renderer_ash));
        systems.insert(winit_window.name().to_owned(), Box::new(winit_window));
        Self {
            world,
            systems,
            event_loop,
            transmitter: tx,
        }
    }

    pub fn main_loop(mut self) {
        #[allow(unused_imports)]
        use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
        let mut current_time = time::Instant::now();
        // Initialize everything that needs to at startup
        {
            for i in &mut self.systems.values_mut() {
                i.as_mut().startup(&mut self.world);
            }
        }
        self.event_loop.run(move |event, _, control_flow| {
            //*control_flow = ControlFlow::Wait;
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(window_size) => {
                        // Need to send a message into the Renderer Class that a resize has occured
                        self.transmitter
                            .send((window_size.width, window_size.height))
                            .expect("Failed to send data");
                    }
                    // Leave the close requested event here for now
                    WindowEvent::CloseRequested { .. } => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { ref input, .. } => {
                        // Special casing is bad design, but I'll leave this for now
                        if input.virtual_keycode.unwrap() == VirtualKeyCode::Escape {
                            *control_flow = ControlFlow::Exit;
                        } else if let Some(mut keyboard_input) =
                            self.world.get_resource_mut::<InputConsumer>()
                        {
                            keyboard_input.capture_keyboard_input(input);
                        }
                    }
                    _ => {
                        // Need to pass the pressed variable into a keyboard class
                        // Collect the various keyboard inputs and pass them into a class that can
                        // handle input
                        //println!("Keyboard Input Virtual_keycode: {:?}", event);
                    }
                },
                Event::MainEventsCleared => {
                    // Handle Time Step after user input
                    let new_time = time::Instant::now();
                    let frame_time = current_time.elapsed().as_secs_f32(); //new_time.duration_since(current_time).as_secs_f32();
                                                                           // TODO: Properly store deltaTime in world
                                                                           //println!("FrameTime {}", frame_time);
                    self.world.update_dt(frame_time);

                    current_time = new_time;
                    // TODO: This works, for now, with a small number of systems, iterating through them 3 times per update isn't
                    // terrible (right now) just not ideal, perhaps it'd be worth storing systems in different buckets?
                    // What can I do with a HashMap<Key, Vec<Systems>>??? something to think about
                    for i in &mut self.systems.values_mut() {
                        if i.get_run_stage() == RunStage::Update {
                            i.as_mut().run(&mut self.world);
                        }
                    }
                }
                Event::RedrawRequested(_window_id) => {
                    // Redraw frame
                    for i in &mut self.systems.values_mut() {
                        if i.get_run_stage() == RunStage::RedrawUpdate {
                            i.as_mut().run(&mut self.world);
                        }
                    }
                }
                Event::RedrawEventsCleared => {
                    for i in &mut self.systems.values_mut() {
                        if i.get_run_stage() == RunStage::LateUpdate {
                            i.as_mut().run(&mut self.world);
                        }
                    }
                }
                Event::LoopDestroyed => {
                    for i in &mut self.systems.values_mut() {
                        i.as_mut().shutdown(&mut self.world);
                    }
                }
                _ => (),
            }
        });
    }
}
