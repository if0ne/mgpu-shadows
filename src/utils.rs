use glam::Vec4Swizzles;
use gpu_allocator::{
    d3d12::{AllocationCreateDesc, ResourceCategory},
    MemoryLocation,
};
use oxidx::dx::{self, IDevice};

pub fn align(value: u32, alignment: u32) -> u32 {
    (value + (alignment - 1)) & (!(alignment - 1))
}

pub trait MatrixExt {
    fn right(&self) -> glam::Vec3;
    fn up(&self) -> glam::Vec3;
    fn forward(&self) -> glam::Vec3;
    fn create_shadow(shadow_plane: glam::Vec4, light_pos: glam::Vec4) -> Self;
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

    fn create_shadow(shadow_plane: glam::Vec4, l: glam::Vec4) -> Self {
        let d = shadow_plane.w;
        let n = shadow_plane.xyz().normalize();
        let nl = n.dot(l.xyz());

        glam::Mat4 {
            x_axis: glam::Vec4::new(nl + d * l.w - l.x * n.x, -l.x * n.y, -l.x * n.z, -l.x * d),
            y_axis: glam::Vec4::new(-l.y * n.x, nl + d * l.w - l.y * n.y, -l.y * n.z, -l.y * d),
            z_axis: glam::Vec4::new(-l.z * n.x, -l.z * n.y, nl + d * l.w - l.z * n.z, -l.z * d),
            w_axis: glam::Vec4::new(-l.w * n.x, -l.w * n.y, -l.w * n.z, nl),
        }
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

pub trait AllocationCreateDescExt {
    fn from_oxidx_d3d12_resource_desc(
        device: &dx::Device,
        desc: &dx::ResourceDesc,
        name: &str,
        location: MemoryLocation,
    ) -> Self;
}

impl<'a> AllocationCreateDescExt for AllocationCreateDesc<'a> {
    fn from_oxidx_d3d12_resource_desc(
        device: &dx::Device,
        desc: &dx::ResourceDesc,
        name: &str,
        location: MemoryLocation,
    ) -> Self {
        let allocation_info = device.get_resource_allocation_info(0, &[*desc]);
        let device = unsafe { std::mem::transmute_copy(device) };
        let desc = unsafe { std::mem::transmute(desc) };
        let resource_category: ResourceCategory = desc.into();

        AllocationCreateDesc {
            name,
            location,
            size: allocation_info.size(),
            alignment: allocation_info.alignment(),
            resource_category,
        }
    }
}

impl From<&dx::ResourceDesc> for ResourceCategory {
    fn from(desc: &dx::ResourceDesc) -> Self {
        if desc.dimension() == dx::ResourceDimension::Buffer {
            Self::Buffer
        } else if !(desc.flags()
            & (dx::ResourceFlags::AllowRenderTarget | dx::ResourceFlags::AllowDepthStencil))
            .is_empty()
        {
            Self::RtvDsvTexture
        } else {
            Self::OtherTexture
        }
    }
}
