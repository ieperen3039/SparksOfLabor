use core::time::Duration;
use std::path::Path;

use glium::{Program, Surface};
use simple_error::SimpleError;

use super::entity_shader::EntityShader;

extern crate glium;

pub struct RenderEngine {
    display: glium::Display,
    entity_shader : EntityShader,
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

        Ok(RenderEngine { display, entity_shader })
    }

    pub fn update_render_loop(
        &self,
        current_time: Duration,
    ) {
        // if close requested, stop

        let mut frame = self.display.draw();

        frame.clear_all((1.0, 0.0, 1.0, 1.0), -1.0, 0);

        // enable GL_LINE_SMOOTH

        // update camera position
        // update light position

        // draw with each shader
        let entity_shader_state = self.entity_shader.start(&mut frame, camera, sun_light, ambient_light);
        entity_shader_state.draw(transformation, model, texture);

        // draw GUI

        // update window
        frame.finish().expect("Failed to update frame");
    }
}

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
    .map(|shader| Ok(shader))
    .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err.to_string()))?
}
