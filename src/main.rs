use std::num::NonZero;

use mgpu_shadows::graphics::*;
use oxidx::dx::{
    create_debug, create_factory4, Debug, Factory4, FactoryCreationFlags, Format, IDebug,
    IDebugExt, IFactory4, Rect, Viewport,
};
use winit::{
    event::MouseButton,
    keyboard::KeyCode,
    window::Window,
};

fn main() {
    //run_sample::<Sample>();
    let factory: Factory4 = create_factory4(FactoryCreationFlags::Debug).unwrap();
    let adapter = factory.enum_adapters(0).unwrap();

    let debug: Debug = create_debug().unwrap();
    debug.enable_debug_layer();
    debug.set_callback(Box::new(|_, level, _, message| {
        println!("{:?} {}", level, message);
    }));

    let gpu1 = Device::new(factory, adapter);
    let heap1 = gpu1.create_heap(1920 * 1080, MemoryHeapType::Shared);
    let desc1 = gpu1.create_descriptor_allocator(8, 8, 8, 8);

    let _res1: SharedResource<Image> = gpu1.create_shared_image(
        &heap1,
        0,
        ImageDesc::new(1920, 1080, Format::R8Unorm).with_usage(TextureUsage::RenderTarget {
            color: None,
            srv: false,
            uav: false,
        }),
        ViewAccess(desc1.clone()),
        ResourceStates::RenderTarget,
        ResourceStates::CopyDst,
    );
}

/*#[derive(Debug)]
pub struct SwapchainContext {
    pub window: Window,
    pub hwnd: NonZero<isize>,

    pub swapchain: Swapchain,

    pub viewport: Viewport,
    pub rect: Rect,
}

#[derive(Debug)]
pub struct Base {}

impl Base {
    pub(crate) fn new() -> Self {
        todo!()
    }

    fn bind_window(&mut self, window: Window) {}

    fn on_resize(&mut self, new_width: u32, new_height: u32) {}

    fn create_swapchain(&mut self, hwnd: NonZero<isize>) -> Swapchain {
        todo!()
    }

    fn calculate_frame_stats(&self) {}

    pub fn aspect_ratio(&self) -> f32 {
        todo!()
    }
}

impl Drop for Base {
    fn drop(&mut self) {}
}

impl SwapchainContext {
    pub const BUFFER_COUNT: usize = 2;
}

pub trait DxSample {
    fn new(base: &mut Base) -> Self;
    fn init_resources(&mut self, base: &Base);
    fn update(&mut self, base: &Base);
    fn render(&mut self, base: &mut Base);
    fn on_resize(&mut self, base: &mut Base, width: u32, height: u32);

    fn on_key_down(&mut self, base: &Base, key: KeyCode, repeat: bool);
    fn on_key_up(&mut self, key: KeyCode);

    fn on_mouse_down(&mut self, btn: MouseButton);
    fn on_mouse_up(&mut self, btn: MouseButton);
    fn on_mouse_move(&mut self, x: f64, y: f64);
}

#[derive(Debug)]
pub(crate) struct SampleRunner<S: DxSample> {
    pub(crate) base: Base,
    pub(crate) sample: S,
}

impl<S: DxSample> ApplicationHandler for SampleRunner<S> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::Focused(focused) => {}
            WindowEvent::KeyboardInput { event, .. } => match event.state {
                winit::event::ElementState::Pressed => {
                    if let PhysicalKey::Code(code) = event.physical_key {
                        self.sample.on_key_down(&self.base, code, event.repeat);
                    }
                }
                winit::event::ElementState::Released => {
                    if event.physical_key == PhysicalKey::Code(KeyCode::Escape) {
                        event_loop.exit();
                    }

                    if let PhysicalKey::Code(code) = event.physical_key {
                        self.sample.on_key_up(code);
                    }
                }
            },
            WindowEvent::MouseInput { state, button, .. } => match state {
                winit::event::ElementState::Pressed => self.sample.on_mouse_down(button),
                winit::event::ElementState::Released => self.sample.on_mouse_up(button),
            },
            WindowEvent::Resized(size) => {}
            WindowEvent::RedrawRequested => {}
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => (),
        }
    }

    #[allow(clippy::single_match)]
    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => self.sample.on_mouse_move(delta.0, delta.0),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {}
}

pub struct Sample;

impl DxSample for Sample {
    fn new(base: &mut Base) -> Self {
        Sample
    }

    fn init_resources(&mut self, base: &Base) {}

    fn update(&mut self, base: &Base) {}

    fn render(&mut self, base: &mut Base) {}

    fn on_resize(&mut self, base: &mut Base, width: u32, height: u32) {}

    fn on_key_down(&mut self, base: &Base, key: KeyCode, repeat: bool) {}

    fn on_key_up(&mut self, key: KeyCode) {}

    fn on_mouse_down(&mut self, btn: MouseButton) {}

    fn on_mouse_up(&mut self, btn: MouseButton) {}

    fn on_mouse_move(&mut self, x: f64, y: f64) {}
}

pub fn run_sample<S: DxSample>() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut base = Base::new();
    let mut app = SampleRunner {
        sample: S::new(&mut base),
        base,
    };
    event_loop.run_app(&mut app).unwrap();
}
*/