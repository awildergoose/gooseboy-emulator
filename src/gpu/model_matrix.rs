#![allow(dead_code)]

use macroquad::math::{Mat4, Quat, Vec3, vec3};

#[derive(Clone, Copy, Debug)]
pub struct ModelMatrix {
    pub mat: Mat4,
}

impl Default for ModelMatrix {
    fn default() -> Self {
        Self {
            mat: Mat4::IDENTITY,
        }
    }
}

impl ModelMatrix {
    pub const fn identity() -> Self {
        Self {
            mat: Mat4::IDENTITY,
        }
    }

    pub const fn from_cols_array(m: [f32; 16]) -> Self {
        Self {
            mat: Mat4::from_cols_array(&m),
        }
    }

    pub const fn as_cols_array(&self) -> [f32; 16] {
        self.mat.to_cols_array()
    }

    pub const fn load_matrix(&mut self, m: [f32; 16]) {
        self.mat = Mat4::from_cols_array(&m);
    }

    pub fn mul_matrix(&mut self, m: [f32; 16]) {
        let other = Mat4::from_cols_array(&m);
        self.mat *= other;
    }

    pub const fn set_identity(&mut self) {
        self.mat = Mat4::IDENTITY;
    }

    pub fn translate(&mut self, tx: f32, ty: f32, tz: f32) {
        let t = Mat4::from_translation(vec3(tx, ty, tz));
        self.mat *= t;
    }

    pub fn scale(&mut self, sx: f32, sy: f32, sz: f32) {
        let s = Mat4::from_scale_rotation_translation(
            vec3(sx, sy, sz),
            Quat::from_rotation_z(0.0),
            Vec3::ZERO,
        );
        self.mat *= s;
    }

    pub fn rotate_axis(&mut self, axis_x: f32, axis_y: f32, axis_z: f32, angle: f32) {
        let axis = Vec3::new(axis_x, axis_y, axis_z);
        if axis.length_squared() <= 1e-12 {
            return;
        }
        let q = Quat::from_axis_angle(axis.normalize(), angle);
        let r = Mat4::from_quat(q);
        self.mat *= r;
    }

    pub fn rotate_euler(&mut self, yaw: f32, pitch: f32, roll: f32) {
        let q_yaw = Quat::from_rotation_y(yaw);
        let q_pitch = Quat::from_rotation_x(pitch);
        let q_roll = Quat::from_rotation_z(roll);
        let q = q_yaw * q_pitch * q_roll;
        let r = Mat4::from_quat(q);

        self.mat *= r;
    }

    pub fn from_trs(position: Vec3, yaw: f32, pitch: f32, roll: f32, scale: Vec3) -> Self {
        let q =
            Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch) * Quat::from_rotation_z(roll);
        let mat = Mat4::from_scale_rotation_translation(scale, q, position);
        Self { mat }
    }

    pub fn from_trs_with_pivot(
        position: Vec3,
        pivot: Vec3,
        yaw: f32,
        pitch: f32,
        roll: f32,
        scale: Vec3,
    ) -> Self {
        let q =
            Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch) * Quat::from_rotation_z(roll);
        let rot_scale = Mat4::from_scale_rotation_translation(scale, q, Vec3::ZERO);
        let to_pivot = Mat4::from_translation(-pivot);
        let back = Mat4::from_translation(pivot + position);
        let mat = back * rot_scale * to_pivot;
        Self { mat }
    }
}

#[derive(Debug)]
pub struct ModelMatrixStack {
    stack: Vec<Mat4>,
    pub max_depth: usize,
}

impl ModelMatrixStack {
    pub fn new(max_depth: usize) -> Self {
        let mut stack = Vec::with_capacity(max_depth);
        stack.push(Mat4::IDENTITY);
        Self { stack, max_depth }
    }

    pub fn push(&mut self) {
        assert!(self.stack.len() < self.max_depth, "matrix stack overflow");
        let top = *self.stack.last().unwrap();
        self.stack.push(top);
    }

    pub fn pop(&mut self) {
        assert!(self.stack.len() > 1, "matrix stack underflow");
        self.stack.pop();
    }

    pub fn top_mut(&mut self) -> ModelMatrixRef<'_> {
        ModelMatrixRef {
            mat: self.stack.last_mut().unwrap(),
        }
    }

    pub fn top(&self) -> ModelMatrix {
        ModelMatrix {
            mat: *self.stack.last().unwrap(),
        }
    }

    pub fn set_top_from_cols(&mut self, cols: [f32; 16]) {
        let mat = Mat4::from_cols_array(&cols);
        *self.stack.last_mut().unwrap() = mat;
    }

    pub fn mul_top_by_cols(&mut self, cols: [f32; 16]) {
        let other = Mat4::from_cols_array(&cols);
        let top = self.stack.last_mut().unwrap();
        *top *= other;
    }
}

pub struct ModelMatrixRef<'a> {
    mat: &'a mut Mat4,
}

impl ModelMatrixRef<'_> {
    pub fn translate(&mut self, tx: f32, ty: f32, tz: f32) {
        let t = Mat4::from_translation(vec3(tx, ty, tz));
        *self.mat *= t;
    }

    pub fn scale(&mut self, sx: f32, sy: f32, sz: f32) {
        let s = Mat4::from_scale_rotation_translation(
            vec3(sx, sy, sz),
            Quat::from_rotation_z(0.0),
            Vec3::ZERO,
        );
        *self.mat *= s;
    }

    pub fn rotate_axis(&mut self, ax: f32, ay: f32, az: f32, angle: f32) {
        let axis = Vec3::new(ax, ay, az);
        if axis.length_squared() <= 1e-12 {
            return;
        }
        let q = Quat::from_axis_angle(axis.normalize(), angle);
        let r = Mat4::from_quat(q);
        *self.mat *= r;
    }

    pub fn rotate_euler(&mut self, yaw: f32, pitch: f32, roll: f32) {
        let q =
            Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch) * Quat::from_rotation_z(roll);
        let r = Mat4::from_quat(q);
        *self.mat *= r;
    }

    pub const fn load_cols(&mut self, cols: [f32; 16]) {
        *self.mat = Mat4::from_cols_array(&cols);
    }

    pub fn mul_cols(&mut self, cols: [f32; 16]) {
        let other = Mat4::from_cols_array(&cols);
        *self.mat *= other;
    }

    pub const fn as_cols(&self) -> [f32; 16] {
        self.mat.to_cols_array()
    }
}
