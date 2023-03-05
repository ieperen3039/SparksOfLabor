use glium::{Program, Surface};
use nalgebra::{Matrix3, Matrix4, Point3, Similarity3, Vector2, Vector3};
use simple_error::SimpleError;

use super::camera::Camera;

#[derive(Default)]
struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Clone, Copy)]
struct Vertex {
    pos: Point3<f32>,
    normal: Vector3<f32>,
    tex: Vector2<f32>,
}

struct EntityGraphics {
    vertices: glium::VertexBuffer<Vertex>,
    indices: glium::IndexBuffer<u32>,
}

struct GraphicSettings {
    ambient_light_color: Color,
}

#[derive(Default)]
struct DirectionalLight {
    color: Color,
    direction: [f32; 3],
    intensity: f32,
}

#[derive(Default)]
struct AmbientLight {
    color: Color,
}

struct CameraData {
    position: [f32; 3],
}

#[derive(Default)]
struct LightData {
    ambient: AmbientLight,
    directional: DirectionalLight,
}

pub struct EntityShader {
    shader_program: glium::Program,
}

impl EntityShader {
    pub fn new(display: &glium::Display) -> Result<Self, SimpleError> {
        let vertex_source = include_str!("shaders/entity/vertex.glsl");
        let fragment_source = include_str!("shaders/entity/fragment.glsl");
        // let geometry_source = include_str!("shaders/entity/geometry.glsl");

        let shader_program = Program::from_source(display, vertex_source, fragment_source, None)
            .map_err(|err| SimpleError::from(err))?;

        Ok(EntityShader { shader_program })
    }

    /**
     * create a new draw state, based on the given camera state.
     */
    pub fn start<'a>(
        &'a self,
        frame: &'a mut glium::Frame,
        camera: Camera,
        sun_light: DirectionalLight,
        ambient_light: AmbientLight,
    ) -> EntityShaderDrawState<'a> {
        let view_projection_matrix = camera.get_view_projection();
        EntityShaderDrawState {
            frame,
            shader: self,
            camera,
            view_projection_matrix,
            lights: LightData {
                ambient: ambient_light,
                directional: sun_light,
            },
            draw_parameters : Default::default()
        }
    }
}

pub struct EntityShaderDrawState<'a> {
    frame: &'a mut glium::Frame,
    shader: &'a EntityShader,
    camera: Camera,
    view_projection_matrix: Matrix4<f32>,
    lights: LightData,
    pub draw_parameters : glium::draw_parameters::DrawParameters<'a>
}

impl EntityShaderDrawState<'_> {
    pub fn draw(
        &mut self,
        transformation: Similarity3<f32>,
        model: EntityGraphics,
        texture: glium::Texture2d,
    ) -> Result<(), glium::DrawError> {
        let transformation_matrix: Matrix4<f32> = transformation.to_homogeneous();
        let normal_matrix: Matrix3<f32> = transformation_matrix.fixed_slice::<3, 3>(0, 0).into();
        let camera_position: Vector3<f32> = self.camera.position.coords;

        let uniforms = glium::uniform! {
            view_projection_matrix : *self.view_projection_matrix.as_ref(),
            model_matrix : *transformation_matrix.as_ref(),
            normal_matrix : *normal_matrix.as_ref(),
            camera_position : *camera_position.as_ref(),
            texture_sampler : texture,
        };

        self.frame.draw(
            &model.vertices,
            &model.indices,
            &self.shader.shader_program,
            &uniforms,
            &self.draw_parameters,
        )
    }
}
