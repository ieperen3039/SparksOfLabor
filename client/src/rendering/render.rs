use core::time::Duration;

use glium::{Program, Surface};

extern crate glium;

pub struct Drawable {}

pub struct RenderSequence {
    // the shader to use
    shader: glium::Program,

    // How the fragment will interact with the depth buffer.
    pub depth: glium::draw_parameters::Depth,

    // How the fragment will interact with the stencil buffer.
    pub stencil: glium::draw_parameters::Stencil,

    // The effect that the GPU will use to merge the existing pixel with the pixel that is
    // being written.
    pub blend: glium::draw_parameters::Blend,

    // If specified, only pixels in this rect will be displayed. Default is `None`.
    //
    // This is different from a viewport. The image will stretch to fill the viewport, but
    // not the scissor box.
    pub scissor: Option<glium::Rect>,

    // If set, the time it took for the GPU to execute this draw command is added to the total
    // stored inside the `TimeElapsedQuery`.
    pub time_elapsed_query: Option<glium::draw_parameters::TimeElapsedQuery>,
}

pub struct Camera {}

pub struct RenderEngine {
    display: glium::Display,
    camera: Camera,
    render_sequences: Vec<RenderSequence>,
}

impl RenderEngine {
    pub fn new(
        width: i32,
        height: i32,
    ) -> RenderEngine {
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

        return RenderEngine {
            display,
            camera: Camera {},
            render_sequences: Vec::new(),
        };
    }

    pub fn add_sequence(
        mut self,
        shader: Program
    ) {
        self.render_sequences.push(RenderSequence {
            shader,
            depth: Default::default(),
            stencil: Default::default(),
            blend: glium::Blend::alpha_blending(),
            scissor: None,
            time_elapsed_query: None,
        });
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

        for sequence in &self.render_sequences {
            sequence.draw(&frame, &self.camera, current_time)
        }

        // draw GUI

        // update window
        frame.finish().expect("Failed to update frame");
    }
}

impl RenderSequence {
    pub fn new_from_directory(
        display: &glium::Display,
        directory: &std::path::Path,
    ) -> Result<RenderSequence, std::io::Error> {
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

        let shader = Program::from_source(
            display,
            vertex_shader_str,
            fragment_shader_str,
            geometry_shader_str,
        )
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err.to_string()))?;

        return Ok(RenderSequence {
            shader,
            depth: Default::default(),
            stencil: Default::default(),
            blend: glium::Blend::alpha_blending(),
            scissor: None,
            time_elapsed_query: None,
        });
    }

    fn draw(
        &self,
        mut frame: &glium::Frame,
        camera: &Camera,
        current_time: Duration,
    ) {
        // frame.draw(_, _, self.shader, self.uniforms, self.parameters)
    }
}
