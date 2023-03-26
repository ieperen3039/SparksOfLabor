use sol_voxel_lib::vector_alias::{Direction, Position, self};

const Z_NEAR: f32 = 0.05;
const Z_FAR: f32 = 1000.0;

pub struct CameraState {
    pub position: Position,
    pub direction: Direction,
    projection: nalgebra::Matrix4<f32>,
}

impl CameraState {
    pub fn get_view_projection(&self) -> nalgebra::Matrix4<f32> {
        let look_at_target = self.position + self.direction.into_inner();
        let view = nalgebra::Matrix::face_towards(&self.position, &look_at_target, &vector_alias::UNIT_Z);
        self.projection * view
    }

    pub fn interpolate(
        &self,
        other: &CameraState,
        factor: f32,
    ) -> CameraState {
        let interpolated_position = self.position + (other.position - self.position) * factor;
        let interpolated_direction = self.direction.slerp(&other.direction, factor);
        CameraState {
            position: interpolated_position,
            direction: interpolated_direction,
            projection: self.projection,
        }
    }
}

pub struct Camera {
    pub position: Position,
    pub direction: Direction,
    projection: nalgebra::Matrix4<f32>,
    aspect_ratio: f32,
    field_of_view: f32,
}

impl Camera {
    pub fn new(
        view_width: u32,
        view_height: u32,
    ) -> Camera {
        let aspect_ratio = (view_width as f32) / (view_height as f32);
        let field_of_view = 3.14 / 2.0;
        let projection =
            nalgebra::Matrix4::new_perspective(aspect_ratio, field_of_view, Z_NEAR, Z_FAR);

        Camera {
            position: Position::new(0.0, 0.0, 0.0),
            direction: vector_alias::UNIT_X,
            projection,
            aspect_ratio,
            field_of_view,
        }
    }

    pub fn create_state(&self) -> CameraState {
        CameraState {
            position: self.position,
            direction: self.direction,
            projection: self.projection,
        }
    }
}
