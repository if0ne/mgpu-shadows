use glam::Mat4;

#[derive(Clone, Debug)]
pub struct Camera {
    pub view: Mat4,
    pub far: f32,
    pub near: f32,
    pub fov: f32,
    pub aspect_ratio: f32,
}
