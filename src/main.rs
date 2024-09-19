use mgpu_shadows::graphics::{device::Device, resource::SharedResource};
use oxidx::dx::{
    create_debug, create_factory, Debug3, Factory4, FactoryCreationFlags, Format, IDebug,
    IFactory4, ResourceDesc,
};

fn main() {
    let factory: Factory4 = create_factory(FactoryCreationFlags::Debug).unwrap();
    let adapter1 = factory.enum_adapters(0).unwrap();
    let adapter2 = factory.enum_warp_adapters().unwrap();

    let debug: Debug3 = create_debug().unwrap();
    debug.enable_debug_layer();

    let gpu1 = Device::new(adapter1);
    let gpu2 = Device::new(adapter2);

    let heap1 = gpu1.create_shared_heap(1920 * 1080 * 3);
    let heap2 = heap1.connect(gpu2.clone());

    let res1 = heap2.create_shared_resource(
        0,
        &ResourceDesc::texture_2d(1920, 1080)
            .with_format(Format::R8Unorm)
            .with_mip_levels(1),
    );

    let res2 = res1.connect(&heap1, 0);
}
