#![allow(clippy::unused_self)]
use macroquad::{
    camera::Camera3D,
    math::{Quat, vec3},
};
use std::mem::size_of;

use crate::wasm::WASMPointer;

pub struct GpuCamera {
    pub cam: Camera3D,
    yaw: f32,
    pitch: f32,
    roll: f32,
}

impl GpuCamera {
    pub fn new() -> Self {
        Self {
            cam: Camera3D::default(),
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
        }
    }

    pub fn write(&self, mem: &mut [u8], ptr: WASMPointer) {
        let mut offset = ptr as usize;

        let x = self.cam.position.x;
        let y = self.cam.position.y;
        let z = self.cam.position.z;

        for value in [x, y, z, self.yaw, self.pitch] {
            mem[offset..offset + size_of::<f32>()].copy_from_slice(&value.to_le_bytes());
            offset += size_of::<f32>();
        }
    }

    pub fn read(&mut self, x: f32, y: f32, z: f32, yaw: f32, pitch: f32) {
        self.cam.position = vec3(x, y, z);
        self.set_yaw_pitch_roll(yaw, pitch, 0.0);
    }

    fn set_yaw_pitch_roll(&mut self, yaw: f32, pitch: f32, roll: f32) {
        const MAX_PITCH: f32 = (89.0_f32).to_radians();
        let pitch = pitch.clamp(-MAX_PITCH, MAX_PITCH);

        self.yaw = yaw;
        self.pitch = pitch;
        self.roll = roll;

        let forward = vec3(
            -yaw.sin() * pitch.cos(),
            pitch.sin(),
            -yaw.cos() * pitch.cos(),
        );

        let world_up = vec3(0.0, 1.0, 0.0);

        let mut right = forward.cross(world_up);
        if right.length_squared() < 1e-6 {
            let fallback_up = vec3(0.0, 0.0, 1.0);
            right = forward.cross(fallback_up);
        }
        right = right.normalize_or_zero();

        let mut up = right.cross(forward).normalize_or_zero();

        if self.roll.abs() > 1e-6 {
            let q_roll = Quat::from_axis_angle(forward.normalize(), self.roll);
            up = (q_roll * up).normalize_or_zero();
        }

        self.cam.target = self.cam.position + forward;
        self.cam.up = up;
    }
}
