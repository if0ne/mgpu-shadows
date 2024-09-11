use glam::Vec4Swizzles;
use smallvec::SmallVec;

use crate::camera::Camera;

#[derive(Debug)]
pub struct CascadedShadowMaps {
    cascade_proj_views: SmallVec<[glam::Mat4; 4]>,
    distances: SmallVec<[f32; 4]>,
    lamda: f32,
}

impl CascadedShadowMaps {
    pub fn new(count: usize, lamda: f32) -> Self {
        let mut cascade_proj_views = SmallVec::new();
        cascade_proj_views.resize(count, glam::Mat4::IDENTITY);

        let mut distances = SmallVec::new();
        distances.resize(count, 0.0);

        Self {
            cascade_proj_views,
            distances,
            lamda,
        }
    }

    pub fn update(&mut self, camera: &Camera, light_dir: glam::Vec3) {
        let cascade_count = self.distances.len();

        for (i, distance) in self.distances.iter_mut().enumerate() {
            let ratio = ((i + 1) as f32) / (cascade_count as f32);
            let clog = camera.near * camera.far.powf(ratio);
            let cuni = camera.near + (camera.far - camera.near) * ratio;
            *distance = self.lamda * clog + (1.0 - self.lamda) * cuni;
        }

        let mut cur_near = camera.near;

        for i in 0..cascade_count {
            let cur_far = self.distances[i];

            let mut corners = [
                glam::vec3(-1.0, -1.0, 0.0),
                glam::vec3(-1.0, -1.0, 1.0),
                glam::vec3(-1.0, 1.0, 0.0),
                glam::vec3(-1.0, 1.0, 1.0),
                glam::vec3(1.0, -1.0, 0.0),
                glam::vec3(1.0, -1.0, 1.0),
                glam::vec3(1.0, 1.0, 0.0),
                glam::vec3(1.0, 1.0, 1.0),
            ];

            let frust_proj =
                glam::Mat4::perspective_lh(camera.fov, camera.aspect_ratio, cur_near, cur_far);
            let cam_view = camera.view;

            let frust_proj_view = (frust_proj * cam_view).inverse();

            for corner in corners.iter_mut() {
                let temp = frust_proj_view * glam::vec4(corner.x, corner.y, corner.z, 1.0);
                let temp = temp / temp.w;

                *corner = temp.xyz();
            }

            let center = corners
                .into_iter()
                .fold(glam::Vec3::ZERO, |center, corner| center + corner)
                / 8.0;

            let light_view = glam::Mat4::look_at_lh(center, center + light_dir, glam::Vec3::Y);

            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;
            let mut min_z = f32::MAX;
            let mut max_z = f32::MIN;

            for corner in corners {
                let temp = light_view * glam::vec4(corner.x, corner.y, corner.z, 1.0);

                min_x = min_x.min(temp.x);
                max_x = max_x.max(temp.x);
                min_y = min_y.min(temp.y);
                max_y = max_y.max(temp.y);
                min_z = min_z.min(temp.z);
                max_z = max_z.max(temp.z);
            }

            let light_proj = glam::Mat4::orthographic_lh(min_x, max_x, min_y, max_y, min_z, max_z);

            self.cascade_proj_views[i] = light_proj * light_view;

            cur_near = cur_far;
        }
    }
}
