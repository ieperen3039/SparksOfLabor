use core::time::Duration;
use std::{fs::File, path::Path};

use glium::{Program, Surface};
use simple_error::SimpleError;

use super::{
    camera::Camera,
    entity_shader::{AmbientLight, Color, DirectionalLight, EntityGraphics, EntityShader},
};

extern crate glium;

pub struct RenderEngine {
    camera: Camera,
    display: glium::Display,
    entity_shader: EntityShader,
}

impl RenderEngine {
    pub fn new(
        width: i32,
        height: i32,
    ) -> Result<RenderEngine, SimpleError> {
        let events_loop = glium::glutin::event_loop::EventLoop::new();
        // 2. Parameters for building the Window.
        let wb = glium::glutin::window::WindowBuilder::new()
            .with_inner_size(glium::glutin::dpi::LogicalSize::new(width, height))
            .with_title("Sparks of Labor");
        // 3. Parameters for building the OpenGL context.
        let cb = glium::glutin::ContextBuilder::new();
        // 4. Build the Display with the given window and OpenGL context parameters and register the
        //    window with the events_loop.
        let display = glium::Display::new(wb, cb, &events_loop).unwrap();

        let entity_shader = EntityShader::new(&display)?;

        let (width, height) = display.get_framebuffer_dimensions();
        let camera = Camera::new(width, height);

        Ok(RenderEngine {
            display,
            entity_shader,
            camera,
        })
    }

    pub fn update_render_loop(
        &self,
        current_time: Duration,
    ) {
        // if close requested, stop

        let mut frame = self.display.draw();

        // frame.clear_all((1.0, 0.0, 1.0, 1.0), -1.0, 0);
        frame.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

        // enable GL_LINE_SMOOTH

        // update camera position
        let new_camera_state = self.camera.create_state();

        // update light position
        let sun_light = DirectionalLight {
            color: Color {
                red: 200,
                green: 200,
                blue: 200,
            },
            direction: [-4.0, -3.0, -2.0],
            intensity: 1.0,
        };
        let ambient_light = AmbientLight {
            color: Color {
                red: 20,
                green: 20,
                blue: 20,
            },
        };

        let object_file = File::open("res/models/cube.obj").unwrap();
        let texture_file = File::open("res/textures/cube.png").unwrap();
        let entity = EntityGraphics::new(&self.display, object_file, texture_file).unwrap();

        // draw with each shader
        let mut entity_shader_state =
            self.entity_shader
                .start(&mut frame, new_camera_state, sun_light, ambient_light);

        let transformation = nalgebra::Similarity3::identity();
        entity_shader_state
            .draw(transformation, entity)
            .expect("Failed to render entity");

        // draw GUI

        // update window
        frame.finish().expect("Failed to update frame");
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
