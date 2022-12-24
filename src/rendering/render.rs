
use core::time::Duration;

use glium::Surface;

extern crate glium;

struct Shader
{

}

struct RenderSequence
{
    shader: Shader,
}

pub struct Camera
{

}

pub struct RenderEngine
{
    display : glium::Display,
    camera : Camera,
    render_sequences : Vec<RenderSequence>,

}

impl RenderEngine
{
    pub fn new(width : i32, height : i32) -> RenderEngine {
        let mut events_loop = glium::glutin::event_loop::EventLoop::new();
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
            camera : Camera {},
            render_sequences : Vec::new()
        };
    }

    pub fn update_renderLoop(&self, current_time : Duration) {
        // if close requested, stop

        let mut frame = self.display.draw();

        frame.clear_all((1.0, 0.0, 1.0, 1.0), -1.0, 0);

        // enable GL_LINE_SMOOTH

        // update camera position
        // update light position

        for sequence in &self.render_sequences {
            // frame.draw(_, _, sequence.shader, uniforms, draw_parameters)
        }

        // draw GUI

        // update window
        frame.finish();
    }
}