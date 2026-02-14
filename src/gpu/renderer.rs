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
    window::get_internal_gl,
};
use parking_lot::Mutex;

pub struct GpuRenderer {
    pub camera: GpuCamera,
    pub queue: VecDeque<GpuCommand>,
    pub stack: ModelMatrixStack,
    pub recordings: VecDeque<FastCell<GpuMesh>>,
    pub gpu_material: Material,
    pub bound_texture: Option<Texture2D>,
    pub immediate_mesh: Option<GpuMesh>,
}

impl GpuRenderer {
    pub fn new() -> Self {
        Self {
            camera: GpuCamera::new(),
            queue: VecDeque::new(),
            stack: ModelMatrixStack::new(64),
            recordings: VecDeque::new(),
            bound_texture: None,
            immediate_mesh: None,
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

    #[allow(clippy::too_many_lines)]
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
                        let gl = unsafe { get_internal_gl().quad_gl };
                        let m = mesh.get_mut();

                        self.set_uniforms();

                        if let Some(bound) = &self.bound_texture {
                            gl.texture(Some(bound));
                        } else {
                            gl.texture(m.mesh.texture.as_ref());
                        }

                        gl.draw_mode(macroquad::prelude::DrawMode::Triangles);
                        gl.geometry(&m.mesh.vertices, &m.mesh.indices);
                    } else {
                        log::error!("mesh with id {id} doesn't exist!");
                    }
                }
                GpuCommand::EmitVertex(vertex) => {
                    let mut primitive_type = PrimitiveType::Triangles;
                    let indices: &mut Vec<u16>;
                    let len: usize;

                    if let Some(recording) = self.recordings.back_mut() {
                        let rec = recording.get_mut();

                        if let Some(bound) = &self.bound_texture {
                            rec.mesh.texture = Some(bound.clone());
                        }

                        primitive_type = rec.primitive_type.clone();
                        let mesh = &mut rec.mesh;
                        mesh.vertices.push(vertex);
                        len = mesh.vertices.len();
                        indices = &mut mesh.indices;
                    } else {
                        let texture_registry = get_texture_registry().lock();
                        let mut missing_tex = texture_registry.get_default_texture();
                        let missing_tex_m = missing_tex.get_mut();
                        let texture = self
                            .bound_texture
                            .clone()
                            .unwrap_or_else(|| missing_tex_m.clone());
                        drop(texture_registry);
                        if self.immediate_mesh.is_none() {
                            self.immediate_mesh = Some(GpuMesh {
                                mesh: Mesh {
                                    indices: vec![],
                                    vertices: vec![],
                                    texture: Some(texture),
                                },
                                primitive_type: PrimitiveType::Triangles,
                            });
                        }

                        let imm = self.immediate_mesh.as_mut().unwrap();
                        imm.mesh.vertices.push(vertex);

                        indices = &mut imm.mesh.indices;
                        len = imm.mesh.vertices.len();
                    }

                    match primitive_type {
                        PrimitiveType::Triangles => {
                            if len.is_multiple_of(3) {
                                let i = u16::try_from(len - 3).unwrap();
                                indices.extend_from_slice(&[i, i + 1, i + 2]);
                            }
                        }
                        PrimitiveType::Quads => {
                            if len.is_multiple_of(4) {
                                let i = u16::try_from(len - 4).unwrap();
                                indices.extend_from_slice(&[i, i + 1, i + 2, i, i + 2, i + 3]);
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
                        self.bound_texture = Some(t);
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

        if let Some(imm) = self.immediate_mesh.as_ref() {
            let gl = unsafe { get_internal_gl().quad_gl };
            gl.texture(imm.mesh.texture.as_ref());
            gl.draw_mode(macroquad::prelude::DrawMode::Triangles);
            gl.geometry(&imm.mesh.vertices, &imm.mesh.indices);
        }

        self.bound_texture = None;
        self.immediate_mesh = None;
        macroquad::material::gl_use_default_material();
    }
}

pub fn get_gpu_renderer() -> &'static Mutex<GpuRenderer> {
    static GPU_RENDERER: OnceLock<Mutex<GpuRenderer>> = OnceLock::new();
    GPU_RENDERER.get_or_init(|| Mutex::new(GpuRenderer::new()))
}
