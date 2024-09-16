pub mod camera;
pub mod command_allocator;
pub mod command_queue;
pub mod constant_buffer;
pub mod csm;
pub mod fence;
pub mod fps_camera_controller;
pub mod frame_fence;
pub mod game_timer;
pub mod heap_view;
pub mod utils;
pub mod worker_thread;

/*

fn render() {

    std::thread::scope(|s| {
        s.spawn(|| {
            let worker_thread = self.renderer.gpu_secondary.acquire_direct_worker_thread();
            ... direct commands

            self.renderer.gpu_secondary.direct_command_queue.push(&worker_thread);
            self.renderer.gpu_secondary.direct_command_queue.execute(&self.shared_fence);
        });

        s.spawn(|| {
            self.renderer.gpu_primary.copy_command_queue.wait(&self.shared_fence);
            let worker_thread = self.renderer.gpu_primary.acquire_copy_worker_thread();

            ... copy commands

            self.renderer.gpu_primary.copy_command_queue.push(&worker_thread);
            self.renderer.gpu_primary.copy_command_queue.execute(&self.copy_fence);
        });

        s.spawn(|| {
            let worker_thread = self.renderer.gpu_primary.acquire_direct_worker_thread();

            ... draw commands

            self.renderer.gpu_primary.direct_command_queue.push(&worker_thread);
            self.renderer.gpu_primary.direct_command_queue.execute(&self.render_fence);

            ... draw commands

            worker_thread.reset();
            self.renderer.gpu_primary.direct_command_queue.push(&worker_thread);
            self.renderer.gpu_primary.direct_command_queue.execute(&self.render_fence);
        });
    });

    self.renderer.present();
}

*/
