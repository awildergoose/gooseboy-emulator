use std::{collections::VecDeque, sync::OnceLock};

use crate::gpu::{
    camera::GpuCamera,
    command::GpuCommand,
    mesh_registry::{GpuMesh, MeshId, get_mesh_registry},
    model_matrix::{ModelMatrix, ModelMatrixStack},
    texture_registry::{TextureId, get_texture_registry},
};
use fast_cell::FastCell;
use macroquad::prelude::{Material, MaterialParams, ShaderSource, UniformDesc, load_material};
use parking_lot::Mutex;

pub struct GpuRenderer {
    pub camera: GpuCamera,
    pub queue: VecDeque<GpuCommand>,
    pub stack: ModelMatrixStack,
    pub recordings: VecDeque<FastCell<GpuMesh>>,
    pub gpu_material: Material,
}

impl GpuRenderer {
    pub fn new() -> Self {
        Self {
            camera: GpuCamera::new(),
            queue: VecDeque::new(),
            stack: ModelMatrixStack::new(64),
            recordings: VecDeque::new(),
            gpu_material: load_material(
                ShaderSource::Glsl {
                    vertex: include_str!("../shaders/vertex.glsl"),
                    fragment: include_str!("../shaders/fragment.glsl"),
                },
                MaterialParams {
                    uniforms: vec![UniformDesc::new(
                        "ModelView",
                        macroquad::prelude::UniformType::Mat4,
                    )],
                    textures: vec![],
                    ..Default::default()
                },
            )
            .expect("failed to load gpu shader"),
        }
    }

    pub fn set_uniforms(&self) {
        let model = self.stack.top().mat;
        self.gpu_material.set_uniform("ModelView", model);
    }

    pub fn queue_command(&mut self, command: GpuCommand) {
        self.queue.push_back(command);
    }

    pub fn execute_commands(&mut self) {
        macroquad::material::gl_use_material(&self.gpu_material);

        while let Some(command) = self.queue.pop_front() {
            // log::trace!("command: {command:?}");

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
                    if let Some(mut mesh) = mesh {
                        let m = mesh.get_mut();
                        self.set_uniforms();
                        macroquad::models::draw_mesh(&m.mesh);
                    } else {
                        log::error!("mesh with id {id} doesn't exist!");
                    }
                }
                GpuCommand::EmitVertex(vertex) => {
                    if let Some(recording) = self.recordings.back_mut() {
                        let mesh = &mut recording.get_mut().mesh;

                        mesh.vertices.push(vertex);

                        let len = mesh.vertices.len();
                        if len >= 3 {
                            let i = u16::try_from(len - 3).unwrap();
                            mesh.indices.extend_from_slice(&[i, i + 1, i + 2]);
                        }
                    } else {
                        // TODO: immediate mode, push vertex
                        todo!();
                    }
                }
                GpuCommand::BindTexture(id) => {
                    let registry = get_texture_registry().lock();
                    let texture = registry.find_texture(TextureId::from(id));

                    let mut texture = texture.unwrap_or_else(|| registry.get_default_texture());
                    drop(registry);
                    let t = texture.get_mut().clone();

                    if let Some(recording) = self.recordings.back_mut() {
                        recording.get_mut().mesh.texture = Some(t);
                    } else {
                        // TODO: immediate mode, set texture
                        // todo!();
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

        macroquad::material::gl_use_default_material();
    }
}

pub fn get_gpu_renderer() -> &'static Mutex<GpuRenderer> {
    static GPU_RENDERER: OnceLock<Mutex<GpuRenderer>> = OnceLock::new();
    GPU_RENDERER.get_or_init(|| Mutex::new(GpuRenderer::new()))
}
