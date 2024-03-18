mod raytracer;
use raytracer::{Scene, State, Vec3};
use winit::{event::{ElementState, Event, KeyEvent, WindowEvent}, event_loop::EventLoopBuilder, keyboard::{KeyCode, PhysicalKey}, window::WindowBuilder};

#[derive(Debug, Clone, Copy)]
enum CustomEvent {
    Timer,
}

pub async fn run() {
    env_logger::init();

    let event_loop = EventLoopBuilder::<CustomEvent>::with_user_event()
        .build()
        .unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let event_loop_proxy = event_loop.create_proxy();

    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(17));
        event_loop_proxy.send_event(CustomEvent::Timer).ok();
    });

    // make the scene
    let mut scene = Scene::new(40, window.outer_size().width as f32, window.outer_size().height as f32);
    // scene.add_square(Vec3(0.0, 0.5, 0.0), 10.0, 10.0, Vec3(0.0, 1.0, 0.0), 0.0);
    scene.add_sphere(Vec3(0.0, 0.0, -1.0), Vec3(1.0, 0.0, 0.0), 0.5);
    // scene.add_object_mesh("assets/models/statue.obj");
    // scene.add_sphere(Vec3(1.5, 0.0, -1.0), Vec3(0.0, 1.0, 0.0), 0.5);
    // scene.add_sphere(Vec3(-1.5, 0.0, -1.0), Vec3(0.0, 0.0, 1.0), 0.5);
    scene.make_scene();
    

    let mut program_state: State<'_> = State::new(&window, scene).await;

    event_loop.run(move | event, elwt | match event {
        Event::UserEvent(..) => {
            program_state.window.request_redraw();
            program_state.scene.update();
        },

        Event::WindowEvent { window_id, ref event } if window_id == program_state.window.id() => match event {
            WindowEvent::Resized(physical_size) => program_state.resize(*physical_size),

            WindowEvent::CloseRequested 
            | WindowEvent::KeyboardInput { 
                event: 
                    KeyEvent { 
                        physical_key: PhysicalKey::Code(KeyCode::Escape), 
                        state: ElementState::Pressed, repeat: false, .. }, .. } => {
                println!("Goodbye see you!");
                elwt.exit();
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        ..
                    },
                ..
            } => {
                let key_code = match physical_key {
                    PhysicalKey::Code(code) => Some(code),
                    _ => None,
                };
                match state {
                    ElementState::Pressed => {
                        if let Some(code) = key_code {
                            program_state.scene.keys_pressed.insert(*code);
                        }
                    },
                    ElementState::Released => {
                        if let Some(code) = key_code {
                            program_state.scene.keys_pressed.remove(&code);
                        }
                    },
                }
            },                    

            WindowEvent::RedrawRequested => match program_state.render() {
                Ok(_) => {},
                Err(wgpu::SurfaceError::Lost) => program_state.resize(program_state.size),
                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                Err(e) => eprintln!("{:?}", e),
            }

            _ => (),

        },

        _ => {},
    }).expect("Error!");
}

fn main() {
    pollster::block_on(run());
}
