extern crate glfw;

pub mod rendering;
pub mod voxel_engine;

fn main() {
    let window = rendering::window::Window::new(800, 600);
}
