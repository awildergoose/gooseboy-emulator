use crate::wasm::WASMPointer;

#[derive(Default)]
pub struct CameraTransform {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl CameraTransform {
    pub fn write(&self, mem: &mut [u8], ptr: WASMPointer) {
        let mut offset = ptr as usize;

        for value in [self.x, self.y, self.z, self.yaw, self.pitch] {
            mem[offset..offset + size_of::<f32>()].copy_from_slice(&value.to_le_bytes());
            offset += size_of::<f32>();
        }
    }
}

pub struct GpuCamera {
    pub transform: CameraTransform,
}

impl GpuCamera {
    pub fn new() -> Self {
        Self {
            transform: CameraTransform::default(),
        }
    }
}
