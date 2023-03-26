use std::{
    fs::File,
    io::BufReader,
};

use glium::{index::PrimitiveType, texture::srgb_texture2d::SrgbTexture2d, Surface};
use simple_error::SimpleError;

use super::camera::CameraState;

const SOL_OBJ_LOAD_OPTIONS: tobj::LoadOptions = tobj::LoadOptions {
    single_index: true,
    #[cfg(feature = "merging")]
    merge_identical_points: false,
    #[cfg(feature = "reordering")]
    reorder_data: false,
    triangulate: false,
    ignore_points: false,
    ignore_lines: false,
};

#[derive(Default)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
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
    texture: SrgbTexture2d,
}

impl EntityGraphics {
    pub fn new(
        display: &glium::Display,
        model_file: File,
        texture_file: File,
    ) -> Result<EntityGraphics, SimpleError> {
        let model_name = format!("{:?}", model_file);
        let mut reader = BufReader::new(model_file);
        let (models, _materials) = tobj::load_obj_buf(
            &mut reader,
            &SOL_OBJ_LOAD_OPTIONS,
            |_mat_path| Ok(Default::default()), // no materials
        )
        .map_err(|e| SimpleError::new(format!("Could not read OBJ file {model_name}: {e}")))?;

        let obj = &models
            .first()
            .ok_or_else(|| SimpleError::new(format!("No models in OBJ file {model_name}")))?
            .mesh;

        let mut vertices: Vec<Vertex> = Vec::new();
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
            .map_err(|e| SimpleError::new(format!("Could not create vertex buffer: {e}")))?;
        let indices = glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &obj.indices)
            .map_err(|e| SimpleError::new(format!("Could not create index buffer: {e}")))?;

        let image = image::load(BufReader::new(texture_file), image::ImageFormat::Png)
            .map_err(|e| SimpleError::new(format!("Could not load texture from file: {e}")))?
            .to_rgba8();
        
        let image_dimensions = image.dimensions();
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

        let texture = glium::texture::SrgbTexture2d::new(display, image)
            .map_err(|e| SimpleError::new(format!("Could not create texture from image file: {e}")))?;

        return Ok(EntityGraphics {
            vertices,
            indices,
            texture,
        });
    }
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
            draw_parameters: Default::default(),
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
        model: EntityGraphics,
    ) -> Result<(), glium::DrawError> {
        let transformation_matrix: nalgebra::Matrix4<f32> = transformation.to_homogeneous();
        let normal_matrix: nalgebra::Matrix3<f32> =
            transformation_matrix.fixed_slice::<3, 3>(0, 0).into();

        let uniforms = glium::uniform! {
            view_projection_matrix : *self.view_projection_matrix.as_ref(),
            model_matrix : *transformation_matrix.as_ref(),
            normal_matrix : *normal_matrix.as_ref(),
            camera_position : *self.camera_position.as_ref(),
            texture_sampler : model.texture,
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
