use mgpu_shadows::graphics::device::Device;
use oxidx::dx::{
    create_debug, create_factory, Debug3, Factory4, FactoryCreationFlags, Format, IDebug,
    IFactory4, ResourceBarrier, ResourceDesc, ResourceFlags, ResourceStates, PSO_NONE,
};

fn main() {
    let factory: Factory4 = create_factory(FactoryCreationFlags::Debug).unwrap();
    let adapter1 = factory.enum_adapters(0).unwrap();
    let adapter2 = factory.enum_warp_adapters().unwrap();

    let debug: Debug3 = create_debug().unwrap();
    debug.enable_debug_layer();

    let gpu1 = Device::new(factory.clone(), adapter1);
    let gpu2 = Device::new(factory.clone(), adapter2);

    let heap1 = gpu1.create_shared_heap(1920 * 1080 * 3);
    let heap2 = heap1.connect(gpu2.clone());

    let res1 = heap1.create_shared_resource(
        0,
        &ResourceDesc::texture_2d(1920, 1080)
            .with_format(Format::R8Unorm)
            .with_flags(ResourceFlags::AllowRenderTarget)
            .with_mip_levels(1),
    );

    dbg!(&res1);

    let res2 = res1.connect(&heap2, 0);

    dbg!(&res2);

    let fence1 = gpu1.create_shared_fence();
    let fence2 = fence1.connect(&gpu2);

    let queue1 = gpu1.create_graphics_command_queue(fence1);
    let queue2 = gpu2.create_graphics_command_queue(fence2);

    let mut desc2 = gpu2.create_rtv_descriptor_heap(8);
    let handle = desc2.push(res2.local_resource(), None);

    let worker = queue2.get_worker_thread(PSO_NONE);
    worker.barrier(&[ResourceBarrier::transition(
        res2.local_resource(),
        ResourceStates::Common,
        ResourceStates::RenderTarget,
    )]);
    worker.clear_rt(handle.cpu(), [0.5, 0.5, 0.5, 1.0]);
    worker.push_shared(&res2);
    queue2.push_worker(worker);
    queue2.execute();

    queue1.wait_other_queue_on_gpu(&queue2);
    let worker = queue1.get_worker_thread(PSO_NONE);
    worker.pull_shared(&res1);
    queue1.push_worker(worker);
    queue1.wait_on_cpu(queue1.execute());
}
