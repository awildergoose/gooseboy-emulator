use std::sync::OnceLock;

use crate::gpu::camera::GpuCamera;
use parking_lot::Mutex;

pub struct GpuRenderer {
    pub camera: GpuCamera,
}

impl GpuRenderer {
    pub fn new() -> Self {
        Self {
            camera: GpuCamera::new(),
        }
    }
}

pub fn get_gpu_renderer() -> &'static Mutex<GpuRenderer> {
    static GPU_RENDERER: OnceLock<Mutex<GpuRenderer>> = OnceLock::new();
    GPU_RENDERER.get_or_init(|| Mutex::new(GpuRenderer::new()))
}
