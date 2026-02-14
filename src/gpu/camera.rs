#![allow(clippy::unused_self)]
use macroquad::{
    camera::Camera3D,
    math::{Mat4, Quat, Vec3, vec3},
};
use std::mem::size_of;

use crate::wasm::WASMPointer;

pub struct GpuCamera {
    pub cam: Camera3D,
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

        let (yaw, pitch) = Self::get_yaw_pitch(&self.cam);

        for value in [x, y, z, yaw, pitch] {
            mem[offset..offset + size_of::<f32>()].copy_from_slice(&value.to_le_bytes());
            offset += size_of::<f32>();
        }
    }

    pub fn read(&mut self, x: f32, y: f32, z: f32, yaw: f32, pitch: f32) {
        self.cam.position = vec3(x, y, z);
        self.set_yaw_pitch_roll(yaw, pitch, 0.0);
    }

    fn get_yaw_pitch(cam: &Camera3D) -> (f32, f32) {
        let forward = (cam.target - cam.position).normalize();
        let yaw = forward.z.atan2(forward.x);
        let pitch = forward.y.asin();
        (yaw, pitch)
    }

    fn set_yaw_pitch_roll(&mut self, yaw: f32, pitch: f32, roll: f32) {
        let forward = vec3(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        );
        let rotated_forward = Quat::from_axis_angle(forward.normalize(), roll) * forward;
        self.cam.target = self.cam.position + rotated_forward;
    }

    pub fn t_push(&self) {
        macroquad::camera::push_camera_state();
    }

    pub fn t_pop(&self) {
        macroquad::camera::pop_camera_state();
    }

    pub fn t_translate(&mut self, x: f32, y: f32, z: f32) {
        let delta = vec3(x, y, z);
        self.cam.position += delta;
        self.cam.target += delta;
    }

    pub fn t_rotate_axis(&mut self, x: f32, y: f32, z: f32, angle: f32) {
        let forward = (self.cam.target - self.cam.position).normalize();
        let rotation = Quat::from_axis_angle(Vec3::new(x, y, z).normalize(), angle);
        let new_forward = rotation * forward;
        self.cam.target = self.cam.position + new_forward;
    }

    pub fn t_rotate_euler(&mut self, yaw: f32, pitch: f32, roll: f32) {
        let forward = (self.cam.target - self.cam.position).normalize();

        let q_yaw = Quat::from_rotation_y(yaw);
        let q_pitch = Quat::from_rotation_x(pitch);
        let q_roll = Quat::from_axis_angle(forward, roll);

        let delta_rot = q_yaw * q_pitch * q_roll;
        let new_forward = delta_rot * forward;
        self.cam.target = self.cam.position + new_forward;
    }

    pub fn t_scale(&mut self, sx: f32, sy: f32, sz: f32) {
        let forward = self.cam.target - self.cam.position;
        self.cam.target =
            self.cam.position + Vec3::new(forward.x * sx, forward.y * sy, forward.z * sz);
    }

    pub fn t_load_matrix(&mut self, m: [f32; 16]) {
        let mat = Mat4::from_cols_array(&m);
        self.cam.position = mat.transform_point3(Vec3::ZERO);
        let forward = mat.transform_vector3(Vec3::Z).normalize();
        self.cam.target = self.cam.position + forward;
    }

    pub fn t_mul_matrix(&mut self, m: [f32; 16]) {
        let mat = Mat4::from_cols_array(&m);
        let pos = self.cam.position;
        let forward = self.cam.target - pos;
        self.cam.position = mat.transform_point3(pos);
        self.cam.target = self.cam.position + mat.transform_vector3(forward);
    }

    pub fn t_identity(&mut self) {
        self.cam.target = self.cam.position + Vec3::Z;
    }
}
