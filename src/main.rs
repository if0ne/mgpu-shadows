use std::{cell::RefCell, num::NonZero, thread::sleep, time::Duration};

use mgpu_shadows::{
    game_timer::GameTimer,
    graphics::{
        command_queue::{CommandQueue, Graphics},
        descriptor_heap::{DescriptorHeap, DsvHeapView, ResourceDescriptor, RtvHeapView},
        device::Device,
        fence::{LocalFence, SharedFence},
        resources::SharedResource,
        swapchain::{self, Swapchain},
    },
};
use oxidx::dx::{
    create_debug, create_factory, ClearValue, Debug3, Factory4, FactoryCreationFlags, Format,
    FrameBufferUsage, IDebug, IFactory4, Rect, ResourceBarrier, ResourceDesc, ResourceFlags,
    ResourceStates, SwapEffect, SwapchainDesc1, Viewport, OUTPUT_NONE, PSO_NONE,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, DeviceId, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::Window,
};

fn main() {
    run_sample::<Sample>();
}

#[derive(Debug)]
pub struct SwapchainContext {
    pub window: Window,
    pub hwnd: NonZero<isize>,

    pub swapchain: Swapchain,

    pub viewport: Viewport,
    pub rect: Rect,
}

#[derive(Debug)]
pub struct Base {
    pub factory: Factory4,
    pub gpu1: Device,
    pub gpu2: Device,

    pub queue1: CommandQueue<Graphics, LocalFence>,
    pub shared_fence: SharedFence,

    pub queue2: CommandQueue<Graphics, SharedFence>,

    pub rtv1: DescriptorHeap<RtvHeapView>,
    pub dsv1: DescriptorHeap<DsvHeapView>,
    pub rtv2: DescriptorHeap<RtvHeapView>,

    pub client_width: u32,
    pub client_height: u32,
    pub back_buffer_format: Format,
    pub depth_stencil_format: Format,

    pub title: String,
    pub app_paused: bool,

    pub context: Option<SwapchainContext>,
    pub timer: GameTimer,

    pub res1: SharedResource,
    pub res2: SharedResource,
    pub handle: ResourceDescriptor<RtvHeapView>,
}

impl Base {
    pub(crate) fn new() -> Self {
        let client_width = 1280;
        let client_height = 720;
        let back_buffer_format = Format::Bgra8Unorm;
        let depth_stencil_format = Format::D24UnormS8Uint;

        let factory: Factory4 = create_factory(FactoryCreationFlags::Debug).unwrap();
        let adapter1 = factory.enum_adapters(0).unwrap();
        let adapter2 = factory.enum_warp_adapters().unwrap();

        let debug: Debug3 = create_debug().unwrap();
        debug.enable_debug_layer();

        let gpu1 = Device::new(factory.clone(), adapter1);
        let gpu2 = Device::new(factory.clone(), adapter2);

        let mut rtv_heap = gpu1.create_rtv_descriptor_heap(8);

        let heap1 = gpu1.create_shared_heap(800 * 600 * 3);
        let heap2 = heap1.connect(gpu2.clone());

        let rtv1 = gpu1.create_rtv_descriptor_heap(8);
        let dsv1 = gpu1.create_dsv_descriptor_heap(8);

        let mut rtv2 = gpu2.create_rtv_descriptor_heap(8);

        let res1 = heap1.create_shared_resource(
            0,
            ResourceDesc::texture_2d(800, 600)
                .with_format(Format::R8Unorm)
                .with_flags(ResourceFlags::AllowRenderTarget)
                .with_mip_levels(1),
            ResourceStates::Common,
            ResourceStates::Common,
            None,
        );

        let res2 = res1.connect(
            &heap2,
            0,
            ResourceStates::Common,
            ResourceStates::Common,
            Some(&ClearValue::color(Format::R8Unorm, [0.5, 0.5, 0.5, 1.0])),
        );

        let handle = rtv2.push(res2.local_resource(), None);

        let fence1 = gpu1.create_shared_fence();
        let fence2 = fence1.connect(gpu2.clone());

        let fence = gpu1.create_fence();
        let present_queue = gpu1.create_graphics_command_queue(fence);

        let queue2 = gpu2.create_graphics_command_queue(fence2);

        Self {
            factory,
            context: None,

            title: "Dx Sample".to_string(),
            app_paused: false,

            client_width,
            client_height,

            back_buffer_format,
            depth_stencil_format,

            timer: Default::default(),
            gpu1,
            gpu2,
            queue1: present_queue,
            shared_fence: fence1,
            queue2,
            rtv2,
            rtv1,
            dsv1,
            res1,
            res2,
            handle,
        }
    }

    fn bind_window(&mut self, window: Window) {
        let Ok(RawWindowHandle::Win32(hwnd)) = window.window_handle().map(|h| h.as_raw()) else {
            panic!()
        };
        let hwnd = hwnd.hwnd;

        let swapchain = self.create_swapchain(hwnd);

        let viewport = Viewport::from_size((self.client_width as f32, self.client_height as f32));
        let rect = Rect::default().with_size((self.client_width as i32, self.client_height as i32));

        let context = SwapchainContext {
            window,
            hwnd,
            swapchain,
            viewport,
            rect,
        };

        self.context = Some(context);
    }

    fn on_resize(&mut self, new_width: u32, new_height: u32) {}

    fn create_swapchain(&mut self, hwnd: NonZero<isize>) -> Swapchain {
        let swapchain_desc = SwapchainDesc1::new(self.client_width, self.client_height)
            .with_buffer_count(SwapchainContext::BUFFER_COUNT as u32)
            .with_usage(FrameBufferUsage::RenderTargetOutput)
            .with_swap_effect(SwapEffect::FlipDiscard)
            .with_format(self.back_buffer_format);

        self.gpu1.create_swapchain(
            self.queue1.clone(),
            &mut self.rtv1,
            &mut self.dsv1,
            hwnd,
            swapchain_desc,
            SwapchainContext::BUFFER_COUNT,
        )
    }

    fn calculate_frame_stats(&self) {
        thread_local! {
            static FRAME_CNT: RefCell<i32> = Default::default();
            static TIME_ELAPSED: RefCell<f32> = Default::default();
        }

        FRAME_CNT.with_borrow_mut(|frame_cnt| {
            *frame_cnt += 1;
        });

        TIME_ELAPSED.with_borrow_mut(|time_elapsed| {
            if self.timer.total_time() - *time_elapsed > 1.0 {
                FRAME_CNT.with_borrow_mut(|frame_cnt| {
                    let fps = *frame_cnt as f32;
                    let mspf = 1000.0 / fps;

                    if let Some(ref context) = self.context {
                        context
                            .window
                            .set_title(&format!("{} Fps: {fps} Ms: {mspf}", self.title))
                    }

                    *frame_cnt = 0;
                    *time_elapsed += 1.0;
                });
            }
        })
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.client_width as f32 / self.client_height as f32
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
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        {
            let window_attributes = Window::default_attributes()
                .with_title(&self.base.title)
                .with_inner_size(PhysicalSize::new(
                    self.base.client_width,
                    self.base.client_height,
                ));
            let window = event_loop.create_window(window_attributes).unwrap();
            self.base.bind_window(window);
        }

        self.sample.init_resources(&self.base);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        self.base.timer.tick();
        match event {
            WindowEvent::Focused(focused) => {
                if focused {
                    self.base.app_paused = false;
                    self.base.timer.start();
                } else {
                    self.base.app_paused = true;
                    self.base.timer.stop();
                }
            }
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
            WindowEvent::Resized(size) => {
                let Some(ref mut context) = self.base.context else {
                    return;
                };

                if context.window.is_minimized().is_some_and(|minized| minized) {
                    self.base.app_paused = true;
                } else {
                    self.base.app_paused = false;
                    self.base.on_resize(size.width, size.height);
                    self.sample
                        .on_resize(&mut self.base, size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if self.base.app_paused {
                    sleep(Duration::from_millis(100));
                    return;
                }
                self.base.calculate_frame_stats();
                self.sample.update(&self.base);
                self.sample.render(&mut self.base);
                event_loop.exit();
            }
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

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(context) = self.base.context.as_ref() {
            context.window.request_redraw();
        }
    }
}

pub struct Sample;

impl DxSample for Sample {
    fn new(base: &mut Base) -> Self {
        Sample
    }

    fn init_resources(&mut self, base: &Base) {}

    fn update(&mut self, base: &Base) {}

    fn render(&mut self, base: &mut Base) {
        let worker = base.queue2.get_worker_thread(PSO_NONE);
        worker.barrier(&[ResourceBarrier::transition(
            base.res2.local_resource(),
            ResourceStates::Common,
            ResourceStates::RenderTarget,
        )]);
        worker.clear_rt(base.handle.cpu(), [0.5, 0.5, 0.5, 1.0]);
        worker.push_shared(&base.res2);
        base.queue2.push_worker(worker);
        base.queue2.execute();

        base.queue1.wait_other_queue_on_gpu(&base.queue2);
        let worker = base.queue1.get_worker_thread(PSO_NONE);
        worker.pull_shared(&base.res1);
        base.queue1.push_worker(worker);
        base.queue1.wait_on_cpu(base.queue1.execute());
        base.context.as_mut().unwrap().swapchain.present();
    }

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
