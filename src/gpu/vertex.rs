#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub u: f32,
    pub v: f32,
}

impl Vertex {
    #[must_use]
    #[allow(clippy::many_single_char_names)]
    pub const fn new(x: f32, y: f32, z: f32, u: f32, v: f32) -> Self {
        Self { x, y, z, u, v }
    }
}

#[repr(u8)]
pub enum PrimitiveType {
    Triangles,
    Quads,
}

impl PrimitiveType {
    #[must_use]
    pub fn repr(v: u8) -> Self {
        match v {
            0 => Self::Triangles,
            1 => Self::Quads,
            _ => panic!("unknown primitive type: {v}"),
        }
    }
}
