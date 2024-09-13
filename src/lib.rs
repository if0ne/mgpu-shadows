pub mod camera;
pub mod command_queue;
pub mod constant_buffer;
pub mod csm;
pub mod fps_camera_controller;
pub mod frame_command_allocator;
pub mod game_timer;
pub mod worker_thread;
pub mod heap_view;
pub mod utils;

/*

fn render() {
    
    std::thread::scope(|s| {
        s.spawn(|| {
            let worker_thread = self.renderer.gpu_secondary.acquire_direct_worker_thread();
            let cmd_list = worker_thread.acquire_cmd_list();

            ... direct commands

            self.renderer.gpu_secondary.direct_command_queue.push(&cmd_list);
            self.renderer.gpu_secondary.direct_command_queue.execute(&self.shared_fence);
        });

        s.spawn(|| {
            self.renderer.gpu_primary.copy_command_queue.wait(&self.shared_fence);
            let worker_thread = self.renderer.gpu_primary.acquire_copy_worker_thread();
            let cmd_list = worker_thread.acquire_cmd_list();

            ... copy commands

            self.renderer.gpu_primary.copy_command_queue.push(&cmd_list);
            self.renderer.gpu_primary.copy_command_queue.execute(&self.copy_fence);
        });

        s.spawn(|| {
            let worker_thread = self.renderer.gpu_primary.acquire_direct_worker_thread();
            let cmd_list = worker_thread.acquire_cmd_list();

            ... draw commands

            self.renderer.gpu_primary.direct_command_queue.push(&cmd_list);
            self.renderer.gpu_primary.direct_command_queue.execute(&self.render_fence);

            let cmd_list = worker_thread.acquire_cmd_list();

            ... draw commands

            self.renderer.gpu_primary.direct_command_queue.push(&cmd_list);
            self.renderer.gpu_primary.direct_command_queue.execute(&self.render_fence);
        });
    });

    self.renderer.present();
}

*/
