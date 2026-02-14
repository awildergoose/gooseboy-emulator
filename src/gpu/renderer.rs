use std::{collections::VecDeque, sync::OnceLock};

use crate::gpu::{camera::GpuCamera, command::GpuCommand};
use parking_lot::Mutex;

pub struct GpuRenderer {
    pub camera: GpuCamera,
    pub queue: VecDeque<GpuCommand>,
}

impl GpuRenderer {
    pub fn new() -> Self {
        Self {
            camera: GpuCamera::new(),
            queue: VecDeque::new(),
        }
    }

    pub fn queue_command(&mut self, command: GpuCommand) {
        self.queue.push_back(command);
    }

    pub fn execute_commands(&mut self) {
        // pop and read commands, and do stuff idk
        while let Some(command) = self.queue.pop_front() {
            match command {
                GpuCommand::Push => self.camera.t_push(),
                GpuCommand::Pop => self.camera.t_pop(),
                GpuCommand::PushRecord(primitive_type) => todo!(),
                GpuCommand::PopRecord => todo!(),
                GpuCommand::DrawRecorded(id) => todo!(),
                GpuCommand::EmitVertex(vertex) => todo!(),
                GpuCommand::BindTexture(id) => todo!(),
                GpuCommand::RegisterTexture { w, h, rgba } => todo!(),
                GpuCommand::Translate { x, y, z } => self.camera.t_translate(x, y, z),
                GpuCommand::RotateAxis { x, y, z, angle } => {
                    self.camera.t_rotate_axis(x, y, z, angle);
                }
                GpuCommand::RotateEuler { yaw, pitch, roll } => {
                    self.camera.t_rotate_euler(yaw, pitch, roll);
                }
                GpuCommand::Scale { x, y, z } => self.camera.t_scale(x, y, z),
                GpuCommand::LoadMatrix(m) => self.camera.t_load_matrix(m),
                GpuCommand::MulMatrix(m) => self.camera.t_mul_matrix(m),
                GpuCommand::Identity => self.camera.t_identity(),
            }
        }
    }
}

pub fn get_gpu_renderer() -> &'static Mutex<GpuRenderer> {
    static GPU_RENDERER: OnceLock<Mutex<GpuRenderer>> = OnceLock::new();
    GPU_RENDERER.get_or_init(|| Mutex::new(GpuRenderer::new()))
}
