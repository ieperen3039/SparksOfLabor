use std::{fs::File, io::BufReader};

use glium::{index::PrimitiveType, Program, Surface};
use nalgebra::{Matrix3, Matrix4, Point3, Similarity3, Vector2, Vector3};
use simple_error::SimpleError;
use tobj::LoadOptions;

use super::camera::Camera;

const SOL_OBJ_LOAD_OPTIONS: tobj::LoadOptions = tobj::LoadOptions {
    single_index: true,
    ..Default::default()
};

#[derive(Default)]
struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Clone, Copy)]
struct Vertex {
    pos: [f32; 3],
    normal: [f32; 3],
    tex: [f32; 2],
}

glium::implement_vertex!(Vertex, pos, normal, tex);

pub struct EntityGraphics {
    vertices: glium::VertexBuffer<Vertex>,
    indices: glium::IndexBuffer<u32>,
}

impl EntityGraphics {
    pub fn new(
        display: &glium::Display,
        file: File,
    ) -> Result<EntityGraphics, SimpleError> {
        let mut reader = BufReader::new(file);
        let (models, materials) = tobj::load_obj_buf(
            &mut reader,
            &SOL_OBJ_LOAD_OPTIONS,
            |mat_path| Ok(Default::default()), // no materials
        )
        .map_err(|e| SimpleError::new(format!("Could not read OBJ file {:?}", file)))?;

        let obj = models
            .first()
            .ok_or_else(|| SimpleError::new(format!("No models in OBJ file {:?}", file)))
            .map(|m| m.mesh)?;

        let vertices: Vec<Vertex> = Vec::new();
        for vertex_index in 0..obj.indices.len() {
            let position_index = vertex_index * 3;
            let normal_index = vertex_index * 3;
            let texture_index = vertex_index * 2;

            assert!(obj.positions.len() >= position_index + 3);
            assert!(obj.normals.len() >= normal_index + 3);
            assert!(obj.texcoords.len() >= texture_index + 3);
            
            vertices.push(Vertex {
                pos: obj.positions[position_index..position_index + 3]
                    .try_into()
                    .unwrap(),
                normal: obj.normals[normal_index..normal_index + 3]
                    .try_into()
                    .unwrap(),
                tex: obj.texcoords[texture_index..texture_index + 3]
                    .try_into()
                    .unwrap(),
            });
        }

        let vertices = glium::VertexBuffer::new(display, &vertices)
            .map_err(|e| SimpleError::new("Could not create vertex buffer"))?;
        let indices = glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &obj.indices)
            .map_err(|e| SimpleError::new("Could not create index buffer"))?;

        return Ok(EntityGraphics { vertices, indices });
    }
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
            draw_parameters: Default::default(),
        }
    }
}

pub struct EntityShaderDrawState<'a> {
    frame: &'a mut glium::Frame,
    shader: &'a EntityShader,
    camera: Camera,
    view_projection_matrix: Matrix4<f32>,
    lights: LightData,
    pub draw_parameters: glium::draw_parameters::DrawParameters<'a>,
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
