use std::{collections::VecDeque, sync::OnceLock};

use crate::gpu::{
    camera::GpuCamera,
    command::GpuCommand,
    mesh_registry::{GpuMesh, MeshId, get_mesh_registry},
    model_matrix::{ModelMatrix, ModelMatrixStack},
    texture_registry::{TextureId, get_texture_registry},
    vertex::PrimitiveType,
};
use fast_cell::FastCell;
use macroquad::{
    models::Mesh,
    prelude::{Material, MaterialParams, ShaderSource, UniformDesc, load_material},
    texture::Texture2D,
};
use parking_lot::Mutex;

pub struct GpuRenderer {
    pub camera: GpuCamera,
    pub queue: VecDeque<GpuCommand>,
    pub stack: ModelMatrixStack,
    pub recordings: VecDeque<FastCell<GpuMesh>>,
    pub gpu_material: Material,

    // immediate-mode
    pub immediate_meshes: VecDeque<FastCell<GpuMesh>>,
    pub bound_texture: Option<Texture2D>,
}

const fn empty_mesh() -> Mesh {
    Mesh {
        indices: vec![],
        vertices: vec![],
        texture: None,
    }
}

impl GpuRenderer {
    pub fn new() -> Self {
        Self {
            camera: GpuCamera::new(),
            queue: VecDeque::new(),
            stack: ModelMatrixStack::new(64),
            recordings: VecDeque::new(),
            immediate_meshes: VecDeque::new(),
            bound_texture: None,
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

    // TODO: primitive_type
    pub fn get_immediate_mesh(&mut self) -> FastCell<GpuMesh> {
        let mut last = self.immediate_meshes.back();
        if last.is_none() {
            self.allocate_immediate_mesh();
            last = self.immediate_meshes.back();
        }
        last.unwrap().clone()
    }

    // TODO: primitive_type
    pub fn allocate_immediate_mesh(&mut self) {
        let m = FastCell::new(GpuMesh {
            mesh: empty_mesh(),
            primitive_type: PrimitiveType::Triangles,
        });
        self.immediate_meshes.push_back(m);
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
                        let mut m = mesh.get_mut();
                        let mut temp: GpuMesh;

                        if let Some(bound) = &self.bound_texture {
                            temp = m.clone();
                            temp.mesh.texture = Some(bound.clone());
                            m = &mut temp;
                        }

                        self.set_uniforms();
                        macroquad::models::draw_mesh(&m.mesh);
                    } else {
                        log::error!("mesh with id {id} doesn't exist!");
                    }
                }
                GpuCommand::EmitVertex(vertex) => {
                    let mut immediate = self.get_immediate_mesh();
                    let recording_opt = self.recordings.back_mut();
                    let recording: &mut FastCell<GpuMesh>;

                    // I'm sure this can be done a better way
                    if recording_opt.is_none() {
                        recording = &mut immediate;

                        if let Some(bound) = &self.bound_texture {
                            recording.get_mut().mesh.texture = Some(bound.clone());
                        }
                    } else {
                        recording = recording_opt.unwrap();
                    }

                    let primitive_type = recording.get_mut().primitive_type.clone();
                    let mesh = &mut recording.get_mut().mesh;
                    mesh.vertices.push(vertex);

                    let len = mesh.vertices.len();

                    match primitive_type {
                        PrimitiveType::Triangles => {
                            if len.is_multiple_of(3) {
                                let i = u16::try_from(len - 3).unwrap();
                                mesh.indices.extend_from_slice(&[i, i + 1, i + 2]);
                            }
                        }
                        PrimitiveType::Quads => {
                            if len.is_multiple_of(4) {
                                let i = u16::try_from(len - 4).unwrap();
                                mesh.indices
                                    .extend_from_slice(&[i, i + 1, i + 2, i, i + 2, i + 3]);
                            }
                        }
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
                        self.bound_texture = Some(t);
                        self.allocate_immediate_mesh();
                    }
                }
                GpuCommand::RegisterTexture { w, h, rgba } => {
                    get_texture_registry().lock().create_texture(w, h, &rgba);
                }

                // translations
                GpuCommand::Push => self.stack.push(),
                GpuCommand::Pop => self.stack.pop(),
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

        while let Some(mut mesh) = self.immediate_meshes.pop_front() {
            let m = mesh.get_mut();
            self.set_uniforms();
            macroquad::models::draw_mesh(&m.mesh);
        }

        self.bound_texture = None;
        macroquad::material::gl_use_default_material();
    }
}

pub fn get_gpu_renderer() -> &'static Mutex<GpuRenderer> {
    static GPU_RENDERER: OnceLock<Mutex<GpuRenderer>> = OnceLock::new();
    GPU_RENDERER.get_or_init(|| Mutex::new(GpuRenderer::new()))
}
