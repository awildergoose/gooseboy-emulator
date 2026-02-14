#![allow(clippy::unused_self)]
use macroquad::{camera::Camera3D, math::vec3};

use crate::wasm::WASMPointer;

pub struct GpuCamera {
    pub cam: Camera3D,
}

fn get_camera_yaw_pitch(camera: &Camera3D) -> (f32, f32) {
    let forward = camera.target - camera.position;
    let forward_norm = forward.normalize();

    let yaw = forward_norm.z.atan2(forward_norm.x);
    let pitch = forward_norm.y.asin();

    (yaw, pitch)
}

fn set_camera_yaw_pitch(camera: &mut Camera3D, yaw: f32, pitch: f32) {
    let forward = vec3(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    );

    camera.target = camera.position + forward;
}

impl GpuCamera {
    pub fn new() -> Self {
        Self {
            cam: Camera3D::default(),
        }
    }

    pub fn write(&self, mem: &mut [u8], ptr: WASMPointer) {
        let mut offset = ptr as usize;

        let x = self.cam.position.x;
        let y = self.cam.position.y;
        let z = self.cam.position.z;
        let (yaw, pitch) = get_camera_yaw_pitch(&self.cam);

        for value in [x, y, z, yaw, pitch] {
            mem[offset..offset + size_of::<f32>()].copy_from_slice(&value.to_le_bytes());
            offset += size_of::<f32>();
        }
    }

    pub fn read(&mut self, x: f32, y: f32, z: f32, yaw: f32, pitch: f32) {
        self.cam.position = vec3(x, y, z);
        set_camera_yaw_pitch(&mut self.cam, yaw, pitch);
    }

    pub fn t_push(&self) {
        macroquad::camera::push_camera_state();
    }

    pub fn t_pop(&self) {
        macroquad::camera::pop_camera_state();
    }

    pub fn t_translate(&mut self, x: f32, y: f32, z: f32) {
        self.cam.position += vec3(x, y, z);
    }
}
