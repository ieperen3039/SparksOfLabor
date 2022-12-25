extern crate glfw;

pub mod rendering;
pub mod vector_alias;
pub mod voxel_engine;

fn main() {
    let render_engine = rendering::render::RenderEngine::new(800, 600);
}
