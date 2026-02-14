use dashmap::DashMap;
use fast_cell::FastCell;
use macroquad::models::Mesh;
use parking_lot::Mutex;
use std::{fmt::Debug, sync::OnceLock};

use crate::gpu::{texture_registry::get_texture_registry, vertex::PrimitiveType};

pub type MeshId = u64;

pub struct GpuMesh {
    pub mesh: Mesh,
    pub primitive_type: PrimitiveType,
}

pub struct MeshRegistry {
    meshes: DashMap<MeshId, FastCell<GpuMesh>>,
    last_id: MeshId,
}

impl MeshRegistry {
    pub fn new() -> Self {
        Self {
            last_id: 0,
            meshes: DashMap::new(),
        }
    }

    pub fn create_mesh(&mut self, primitive_type: PrimitiveType) -> FastCell<GpuMesh> {
        let texture_registry = get_texture_registry().lock();
        let mesh = GpuMesh {
            mesh: Mesh {
                indices: vec![],
                texture: Some(texture_registry.get_default_texture().get_mut().clone()),
                vertices: vec![],
            },
            primitive_type,
        };
        drop(texture_registry);
        let mesh = FastCell::new(mesh);
        self.meshes.insert(self.last_id, mesh.clone());
        log::info!("registered mesh with id {}", self.last_id);
        self.last_id += 1;
        mesh
    }

    pub fn find_mesh(&self, id: MeshId) -> Option<FastCell<GpuMesh>> {
        self.meshes.get(&id).map(|f| f.value().clone())
    }
}

pub fn get_mesh_registry() -> &'static Mutex<MeshRegistry> {
    static MESH_REGISTRY: OnceLock<Mutex<MeshRegistry>> = OnceLock::new();
    MESH_REGISTRY.get_or_init(|| Mutex::new(MeshRegistry::new()))
}

impl Debug for GpuMesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuMesh")
            .field("vertices", &self.mesh.vertices)
            .field("primitive_type", &self.primitive_type)
            .finish()
    }
}

impl Clone for GpuMesh {
    fn clone(&self) -> Self {
        Self {
            mesh: Mesh {
                indices: self.mesh.indices.clone(),
                vertices: self.mesh.vertices.clone(),
                texture: self.mesh.texture.clone(),
            },
            primitive_type: self.primitive_type.clone(),
        }
    }
}
