use std::{fs::File, io::BufReader};

use glium::index::PrimitiveType;
use simple_error::SimpleError;

const SOL_OBJ_LOAD_OPTIONS: tobj::LoadOptions = tobj::LoadOptions {
    single_index: true,
    triangulate: false,
    ignore_points: false,
    ignore_lines: false,
};

#[derive(Clone, Copy)]
pub struct Vertex {
    in_vertex_position: [f32; 3],
    in_vertex_normal: [f32; 3],
    in_texture_coord: [f32; 2],
}

pub struct Mesh {
    pub vertices: glium::VertexBuffer<Vertex>,
    pub indices: glium::IndexBuffer<u32>,
    pub texture: glium::texture::SrgbTexture2d,
}

glium::implement_vertex!(
    Vertex,
    in_vertex_position,
    in_vertex_normal,
    in_texture_coord
);

impl Mesh {
    pub fn from_file(
        display: &glium::Display,
        model_file: File,
        texture_file: File,
    ) -> Result<Mesh, SimpleError> {
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

        let num_vertices = obj.positions.len() / 3;

        let mut vertices: Vec<Vertex> = Vec::new();
        for vertex_index in 0..num_vertices {
            let position_index = vertex_index * 3;
            let normal_index = vertex_index * 3;
            let texture_index = vertex_index * 2;

            assert!(obj.positions.len() >= position_index + 3);
            assert!(obj.normals.len() >= normal_index + 3);
            assert!(obj.texcoords.len() >= texture_index + 2);

            vertices.push(Vertex {
                in_vertex_position: obj.positions[position_index..position_index + 3]
                    .try_into()
                    .unwrap(),
                in_vertex_normal: obj.normals[normal_index..normal_index + 3]
                    .try_into()
                    .unwrap(),
                in_texture_coord: obj.texcoords[texture_index..texture_index + 2]
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
        let image =
            glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

        let texture = glium::texture::SrgbTexture2d::new(display, image).map_err(|e| {
            SimpleError::new(format!("Could not create texture from image file: {e}"))
        })?;

        return Ok(Mesh {
            vertices,
            indices,
            texture,
        });
    }
}