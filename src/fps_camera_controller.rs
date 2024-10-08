use crate::{camera::Camera, utils::MatrixExt};

pub struct FpsController {
    sensivity: f32,
    speed: f32,
    yaw: f32,
    pitch: f32,

    position: glam::Vec3,
}

impl FpsController {
    pub fn new(sensivity: f32, speed: f32) -> Self {
        Self {
            sensivity,
            speed,
            yaw: 0.0,
            pitch: 0.0,
            position: glam::Vec3::ZERO,
        }
    }

    pub fn update_position(&mut self, dt: f32, camera: &mut Camera, direction: glam::Vec3) {
        let rot_mat = glam::Mat4::from_euler(glam::EulerRot::YXZ, self.yaw, self.pitch, 0.0);

        let dir = direction.normalize();

        let direction = rot_mat.forward() * dir.x + glam::Vec3::Y * dir.y + rot_mat.right() * dir.z;

        let direction = if direction.length() != 0.0 {
            direction.normalize()
        } else {
            direction
        };

        self.position += direction * self.speed * dt;

        camera.view = glam::Mat4::look_at_lh(
            self.position,
            self.position + rot_mat.forward(),
            rot_mat.up(),
        );
    }

    pub fn update_yaw_pitch(&mut self, camera: &mut Camera, x: f32, y: f32) {
        self.yaw += x * 0.003 * self.sensivity;
        self.pitch -= y * 0.003 * self.sensivity;

        let rot_mat = glam::Mat4::from_euler(glam::EulerRot::YXZ, self.yaw, self.pitch, 0.0);

        camera.view = glam::Mat4::look_at_lh(
            self.position,
            self.position + rot_mat.forward(),
            rot_mat.up(),
        );
    }
}
