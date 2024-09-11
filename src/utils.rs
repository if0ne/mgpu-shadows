use glam::Vec4Swizzles;

pub fn align(value: u32, alignment: u32) -> u32 {
    (value + (alignment - 1)) & (!(alignment - 1))
}

pub trait MatrixExt {
    fn right(&self) -> glam::Vec3;
    fn up(&self) -> glam::Vec3;
    fn forward(&self) -> glam::Vec3;
}

impl MatrixExt for glam::Mat4 {
    #[inline]
    fn right(&self) -> glam::Vec3 {
        self.x_axis.xyz()
    }

    #[inline]
    fn up(&self) -> glam::Vec3 {
        self.y_axis.xyz()
    }

    #[inline]
    fn forward(&self) -> glam::Vec3 {
        self.z_axis.xyz()
    }
}

#[cfg(test)]
mod tests {
    use glam::{Mat4, Vec3};

    use crate::utils::MatrixExt;

    #[test]
    fn test_look_to_lh() {
        let matrix = Mat4::look_to_lh(Vec3::ZERO, Vec3::Z, Vec3::Y);

        assert_eq!(matrix.right(), Vec3::X);
        assert_eq!(matrix.up(), Vec3::Y);
        assert_eq!(matrix.forward(), Vec3::Z);
    }
}
