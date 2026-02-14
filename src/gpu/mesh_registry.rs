use dashmap::DashMap;
use fast_cell::FastCell;
use macroquad::models::Mesh;
use parking_lot::Mutex;
use std::{fmt::Debug, sync::OnceLock};

use crate::gpu::vertex::PrimitiveType;

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
        let mesh = GpuMesh {
            mesh: Mesh {
                indices: vec![],
                texture: None,
                vertices: vec![],
            },
            primitive_type,
        };
        let mesh = FastCell::new(mesh);
        self.meshes.insert(self.last_id, mesh.clone());
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
