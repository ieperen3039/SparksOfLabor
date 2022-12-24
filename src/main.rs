extern crate glfw;

pub mod rendering;
pub mod voxel_engine;
pub mod vector_alias;

fn main() {
    let render_engine = rendering::render::RenderEngine::new(800, 600);
}
