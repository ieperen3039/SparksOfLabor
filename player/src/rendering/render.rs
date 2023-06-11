use std::{fs::File, ops::Neg, path::Path};

use glium::{Program, Surface};
use nalgebra::vector;
use simple_error::SimpleError;
use sol_voxel_lib::vector_alias::{self, Direction, Position};

use super::{
    camera::Camera,
    entity_shader::{AmbientLight, Color, DirectionalLight, EntityGraphics, EntityShader},
};

extern crate glium;

pub struct RenderEngine {
    display: glium::Display,
    event_loop: glium::glutin::event_loop::EventLoop<()>,
    camera: Camera,
    entity_shader: EntityShader,
}

impl RenderEngine {
    pub fn new(width: i32, height: i32) -> Result<RenderEngine, SimpleError> {
        let event_loop = glium::glutin::event_loop::EventLoop::new();
        // 2. Parameters for building the Window.
        let wb = glium::glutin::window::WindowBuilder::new()
            .with_inner_size(glium::glutin::dpi::LogicalSize::new(width, height))
            .with_title("Sparks of Labour");
        // 3. Parameters for building the OpenGL context.
        let cb = glium::glutin::ContextBuilder::new();
        // 4. Build the Display with the given window and OpenGL context parameters and register the
        //    window with the events_loop.
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();

        let entity_shader = EntityShader::new(&display)?;

        let (width, height) = display.get_framebuffer_dimensions();
        let mut camera = Camera::new(width, height);
        // camera.position = Position::new(4.0, 3.0, 2.0);
        // camera.direction = Direction::new_normalize(VEC_ZERO - camera.position);

        camera.position = Position::new(5.0, 0.0, 0.0);
        camera.direction = vector_alias::UNIT_X.neg();

        Ok(RenderEngine {
            display,
            event_loop,
            entity_shader,
            camera,
        })
    }

    pub fn run_until_close(mut self) {
        use glium::glutin;
        use glutin::event;

        let object_file = File::open("player/res/cube/cube.obj").unwrap();
        let texture_file = File::open("player/res/cube/cube.png").unwrap();
        let entity = EntityGraphics::new(&self.display, object_file, texture_file).unwrap();
        let game_start_time = std::time::Instant::now();

        self.event_loop.run(move |event, _target, control_flow| {
            let current_time = std::time::Instant::now();

            let mut frame = self.display.draw();
            frame.clear_color_and_depth((1.0, 0.0, 1.0, 0.0), 1.0);

            // update camera position
            let new_camera_state = self.camera.create_state();

            // update light position
            let sun_light = DirectionalLight {
                color: Color {
                    red: 200,
                    green: 200,
                    blue: 200,
                },
                direction: [-2.0, -1.0, -3.0],
                intensity: 1.0,
            };
            let ambient_light = AmbientLight {
                color: Color {
                    red: 20,
                    green: 20,
                    blue: 20,
                },
            };

            // draw with each shader
            let mut entity_shader_state =
                self.entity_shader
                    .start(&mut frame, new_camera_state, sun_light, ambient_light);

            let angle = (current_time - game_start_time).as_millis() as f32 / 1000.0;
            let rotation = nalgebra::UnitQuaternion::from_axis_angle(
                &Direction::new_normalize(vector![0.0, 0.0, 1.0]),
                angle,
            );

            // let transformation = nalgebra::Similarity3::identity();
            let transformation =
                nalgebra::Similarity3::rotation_wrt_point(rotation, vector_alias::VEC_ZERO, 1.0);
            entity_shader_state
                .draw(transformation, &entity)
                .expect("Failed to render entity");

            // draw GUI

            // update window
            frame.finish().expect("Failed to update frame");

            match event {
                event::Event::WindowEvent { event, .. } => match event {
                    event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                        return;
                    },
                    event::WindowEvent::Resized(new_size) => {
                        self.camera.set_view_port(new_size.width, new_size.height)
                    },
                    _ => return,
                },
                _ => {
                    // let next_frame_time = current_time + std::time::Duration::from_millis(250);
                    let next_frame_time =
                        current_time + std::time::Duration::from_nanos(16_666_667);
                    *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
                },
            }
        });
    }
}

// for dynamic loading of shaders
pub fn shader_program_from_directory(
    display: &glium::Display,
    directory: &Path,
) -> Result<Program, std::io::Error> {
    let vertex_shader = directory.join("vertex.glsl");
    let fragment_shader = directory.join("fragment.glsl");
    let geometry_shader = directory.join("geometry.glsl");

    if !vertex_shader.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            vertex_shader
                .to_str()
                .unwrap_or("<invalid characters in path>"),
        ));
    }

    if !fragment_shader.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            fragment_shader
                .to_str()
                .unwrap_or("<invalid characters in path>"),
        ));
    }

    let vertex_shader_str = vertex_shader.to_str().expect("Non-unicode in path");
    let fragment_shader_str = fragment_shader.to_str().expect("Non-unicode in path");

    let geometry_shader_str = geometry_shader
        .exists()
        .then(|| geometry_shader.to_str().expect("Non-unicode in path"));

    Program::from_source(
        display,
        vertex_shader_str,
        fragment_shader_str,
        geometry_shader_str,
    )
    .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err.to_string()))
}
