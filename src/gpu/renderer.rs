use std::{collections::VecDeque, sync::OnceLock};

use crate::gpu::{
    camera::GpuCamera,
    command::GpuCommand,
    mesh_registry::{GpuMesh, MeshId, get_mesh_registry},
    model_matrix::{ModelMatrix, ModelMatrixStack},
    texture_registry::{TextureId, get_texture_registry},
};
use fast_cell::FastCell;
use parking_lot::Mutex;

pub struct GpuRenderer {
    pub camera: GpuCamera,
    pub queue: VecDeque<GpuCommand>,
    pub stack: ModelMatrixStack,
    pub recordings: VecDeque<FastCell<GpuMesh>>,
}

impl GpuRenderer {
    pub fn new() -> Self {
        Self {
            camera: GpuCamera::new(),
            queue: VecDeque::new(),
            stack: ModelMatrixStack::new(64),
            recordings: VecDeque::new(),
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
                GpuCommand::PushRecord(primitive_type) => {
                    let mesh = get_mesh_registry().lock().create_mesh(primitive_type);
                    self.recordings.push_back(mesh);
                    // TODO: write GB_GPU_RECORD_ID
                }
                GpuCommand::PopRecord => {
                    self.recordings.pop_back();
                }
                GpuCommand::DrawRecorded(id) => {
                    let mesh = get_mesh_registry().lock().find_mesh(MeshId::from(id));
                    if let Some(mesh) = mesh {
                        let m = mesh.into_inner().unwrap();
                        macroquad::models::draw_mesh(&m.mesh);
                    } else {
                        log::error!("mesh with id {id} doesn't exist!");
                    }
                }
                GpuCommand::EmitVertex(vertex) => {
                    if let Some(recording) = self.recordings.back_mut() {
                        recording.get_mut().mesh.vertices.push(vertex);
                        // TODO: push indices?
                    } else {
                        // TODO: immediate mode, push vertex
                    }
                }
                GpuCommand::BindTexture(id) => {
                    let texture = get_texture_registry()
                        .lock()
                        .find_texture(TextureId::from(id));

                    if let Some(texture) = texture {
                        let t = texture.into_inner().unwrap().clone();

                        if let Some(recording) = self.recordings.back_mut() {
                            recording.get_mut().mesh.texture = Some(t);
                        } else {
                            // TODO: immediate mode, set texture
                        }
                    } else {
                        log::error!("texture not found in registry!");
                    }
                }
                GpuCommand::RegisterTexture { w, h, rgba } => {
                    get_texture_registry().lock().create_texture(w, h, &rgba);
                }
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
