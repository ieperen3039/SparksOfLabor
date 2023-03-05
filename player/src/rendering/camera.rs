use sol_voxel_lib::vector_alias::{Position, Direction};

pub struct Camera {
    aspect_ratio: f32,
    field_of_view: f32,
    pub position: Position,
    pub direction: Direction,
}

const Z_NEAR : f32 = 0.05;
const Z_FAR : f32 = 1000.0;

impl Camera {
    pub fn new(
        view_width: u32,
        view_height: u32,
    ) -> Camera {
        Camera {
            aspect_ratio: (view_width as f32) / (view_height as f32),
            field_of_view: 3.14 / 2.0,
            position: Position::new(0.0, 0.0, 0.0),
            direction: Direction::new(1.0, 0.0, 0.0),
        }
    }

    pub fn get_view_projection(&self) -> nalgebra::Matrix4<f32> {
        let look_at_target = self.position + self.direction;
        let view = nalgebra::Matrix::face_towards(&self.position, &look_at_target, &Direction::z_axis());
        let projection = nalgebra::Matrix4::new_perspective(self.aspect_ratio, self.field_of_view, Z_NEAR, Z_FAR);
        projection * view
    }
}
