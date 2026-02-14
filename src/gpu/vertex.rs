#[repr(u8)]
#[derive(Debug)]
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
