
use glium::Surface;
use simple_error::SimpleError;

use super::{camera::CameraState, mesh::Mesh};

#[derive(Default)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}


struct GraphicSettings {
    ambient_light_color: Color,
}

#[derive(Default)]
pub struct DirectionalLight {
    pub color: Color,
    pub direction: [f32; 3],
    pub intensity: f32,
}

#[derive(Default)]
pub struct AmbientLight {
    pub color: Color,
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

        let shader_program =
            glium::Program::from_source(display, vertex_source, fragment_source, None)
                .map_err(|err| SimpleError::from(err))?;

        Ok(EntityShader { shader_program })
    }

    /**
     * create a new draw state, based on the given camera state.
     */
    pub fn start<'a>(
        &'a self,
        frame: &'a mut glium::Frame,
        camera: CameraState,
        sun_light: DirectionalLight,
        ambient_light: AmbientLight,
    ) -> EntityShaderDrawState<'a> {
        EntityShaderDrawState {
            frame,
            shader: self,
            camera_position: camera.position.coords,
            view_projection_matrix: camera.get_view_projection(),
            lights: LightData {
                ambient: ambient_light,
                directional: sun_light,
            },
            draw_parameters: glium::draw_parameters::DrawParameters {
                backface_culling: glium::BackfaceCullingMode::CullClockwise,
                ..Default::default()
            },
        }
    }
}

pub struct EntityShaderDrawState<'a> {
    frame: &'a mut glium::Frame,
    shader: &'a EntityShader,
    camera_position: nalgebra::Vector3<f32>,
    view_projection_matrix: nalgebra::Matrix4<f32>,
    lights: LightData,
    pub draw_parameters: glium::draw_parameters::DrawParameters<'a>,
}

impl EntityShaderDrawState<'_> {
    pub fn draw(
        &mut self,
        transformation: nalgebra::Similarity3<f32>,
        model: &Mesh,
    ) -> Result<(), glium::DrawError> {
        let transformation_matrix: nalgebra::Matrix4<f32> = transformation.to_homogeneous();
        let normal_matrix: nalgebra::Matrix3<f32> =
            transformation_matrix.fixed_slice::<3, 3>(0, 0).into();

        let uniforms = glium::uniform! {
            view_projection_matrix : *self.view_projection_matrix.as_ref(),
            model_matrix : *transformation_matrix.as_ref(),
            normal_matrix : *normal_matrix.as_ref(),
            camera_position : *self.camera_position.as_ref(),
            texture_sampler : &model.texture,
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
