use std::{collections::VecDeque, sync::OnceLock};

use crate::gpu::{
    camera::GpuCamera,
    command::GpuCommand,
    model_matrix::{ModelMatrix, ModelMatrixStack},
};
use parking_lot::Mutex;

pub struct GpuRenderer {
    pub camera: GpuCamera,
    pub queue: VecDeque<GpuCommand>,
    pub stack: ModelMatrixStack,
}

impl GpuRenderer {
    pub fn new() -> Self {
        Self {
            camera: GpuCamera::new(),
            queue: VecDeque::new(),
            stack: ModelMatrixStack::new(64),
        }
    }

    pub fn queue_command(&mut self, command: GpuCommand) {
        self.queue.push_back(command);
    }

    pub fn execute_commands(&mut self) {
        // pop and read commands, and do stuff idk
        while let Some(command) = self.queue.pop_front() {
            match command {
                GpuCommand::Push => self.stack.push(),
                GpuCommand::Pop => self.stack.pop(),
                GpuCommand::PushRecord(primitive_type) => {}
                GpuCommand::PopRecord => {}
                GpuCommand::DrawRecorded(id) => {}
                GpuCommand::EmitVertex(vertex) => {}
                GpuCommand::BindTexture(id) => {}
                GpuCommand::RegisterTexture { w, h, rgba } => {}
                GpuCommand::Translate { x, y, z } => self.stack.top_mut().translate(x, y, z),
                GpuCommand::RotateAxis { x, y, z, angle } => {
                    self.stack.top_mut().rotate_axis(x, y, z, angle);
                }
                GpuCommand::RotateEuler { yaw, pitch, roll } => {
                    self.stack.top_mut().rotate_euler(yaw, pitch, roll);
                }
                GpuCommand::Scale { x, y, z } => self.stack.top_mut().scale(x, y, z),
                GpuCommand::LoadMatrix(m) => self.stack.set_top_from_cols(m),
                GpuCommand::MulMatrix(m) => self.stack.mul_top_by_cols(m),
                GpuCommand::Identity => self
                    .stack
                    .set_top_from_cols(ModelMatrix::identity().as_cols_array()),
            }
        }
    }
}

pub fn get_gpu_renderer() -> &'static Mutex<GpuRenderer> {
    static GPU_RENDERER: OnceLock<Mutex<GpuRenderer>> = OnceLock::new();
    GPU_RENDERER.get_or_init(|| Mutex::new(GpuRenderer::new()))
}
