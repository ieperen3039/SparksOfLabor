use sol_voxel_lib::vector_alias::{self, Direction, Position};

const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 100.0;

pub struct CameraState {
    pub position: Position,
    pub direction: Direction,
    projection: nalgebra::Matrix4<f32>,
}

impl CameraState {
    pub fn get_view_projection(&self) -> nalgebra::Matrix4<f32> {
        let look_at_target = self.position + self.direction.into_inner();
        let view =
            nalgebra::Matrix::look_at_rh(&self.position, &look_at_target, &vector_alias::UNIT_Z);
        self.projection * view
    }

    pub fn interpolate(&self, other: &CameraState, factor: f32) -> CameraState {
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
    pub fn new(view_width: u32, view_height: u32) -> Camera {
        let aspect_ratio = (view_width as f32) / (view_height as f32);
        let field_of_view = f32::to_radians(60.0);
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

    fn recalculate_projection(&self) -> nalgebra::Matrix4<f32> {
        nalgebra::Matrix4::new_perspective(self.aspect_ratio, self.field_of_view, Z_NEAR, Z_FAR)
    }

    pub fn set_view_port(&mut self, view_width: u32, view_height: u32) {
        self.aspect_ratio = (view_width as f32) / (view_height as f32);
        self.projection = self.recalculate_projection();
    }

    pub fn set_fov(&mut self, field_of_view: f32) {
        self.field_of_view = field_of_view;
        self.projection = self.recalculate_projection();
    }

    pub fn create_state(&self) -> CameraState {
        CameraState {
            position: self.position,
            direction: self.direction,
            projection: self.projection,
        }
    }
}
