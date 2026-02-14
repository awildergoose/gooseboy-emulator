use crate::gpu::vertex::{PrimitiveType, Vertex};

pub struct CommandReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl CommandReader<'_> {
    fn read_u8(&mut self) -> u8 {
        let b = *self.buf.get(self.pos).unwrap();
        self.pos += 1;
        b
    }

    fn read_u32(&mut self) -> u32 {
        let bytes = self.buf.get(self.pos..self.pos + 4).unwrap();
        self.pos += 4;
        u32::from_le_bytes(bytes.try_into().unwrap())
    }

    fn read_f32(&mut self) -> f32 {
        let bytes = self.buf.get(self.pos..self.pos + 4).unwrap();
        self.pos += 4;
        f32::from_le_bytes(bytes.try_into().unwrap())
    }
}

#[repr(u8)]
pub enum GpuCommand {
    Push,
    Pop,
    PushRecord(PrimitiveType),
    PopRecord,
    DrawRecorded(u32),
    EmitVertex(Vertex),
    BindTexture(u32),
    RegisterTexture { w: u32, h: u32, rgba: Vec<u8> },
    Translate { x: f32, y: f32, z: f32 },
    RotateAxis { x: f32, y: f32, z: f32, angle: f32 },
    RotateEuler { yaw: f32, pitch: f32, roll: f32 },
    Scale { x: f32, y: f32, z: f32 },
    LoadMatrix([f32; 16]),
    MulMatrix([f32; 16]),
    Identity,
}

impl GpuCommand {
    #[allow(clippy::many_single_char_names)]
    pub fn deserialize(buf: &[u8]) -> Self {
        let mut reader = CommandReader { buf, pos: 0 };
        let opcode = reader.read_u8();

        match opcode {
            0x00 => {
                // Push(0),
                Self::Push
            }
            0x01 => {
                // Pop(0),
                Self::Pop
            }
            0x02 => {
                // PushRecord(1), // u8 primitiveType
                Self::PushRecord(PrimitiveType::repr(reader.read_u8()))
            }
            0x03 => {
                // PopRecord(0),
                Self::PopRecord
            }
            0x04 => {
                // DrawRecorded(4), // u32 id
                Self::DrawRecorded(reader.read_u32())
            }
            0x05 => {
                // EmitVertex(20), // f32 xyzuv[5]
                let x = reader.read_f32();
                let y = reader.read_f32();
                let z = reader.read_f32();
                let u = reader.read_f32();
                let v = reader.read_f32();

                Self::EmitVertex(Vertex { x, y, z, u, v })
            }
            0x06 => {
                // BindTexture(4), // u32 id
                Self::BindTexture(reader.read_u32())
            }
            0x07 => {
                // RegisterTexture(8), // u32 w, u32 h, byte[] rgba
                let w = reader.read_u32();
                let h = reader.read_u32();
                let mut rgba = vec![];

                for _ in 0..w * h * 4 {
                    rgba.push(reader.read_u8());
                }

                Self::RegisterTexture { w, h, rgba }
            }
            0x08 => {
                // Translate(12),
                let x = reader.read_f32();
                let y = reader.read_f32();
                let z = reader.read_f32();

                Self::Translate { x, y, z }
            }
            0x09 => {
                // RotateAxis(16),
                let x = reader.read_f32();
                let y = reader.read_f32();
                let z = reader.read_f32();
                let angle = reader.read_f32();

                Self::RotateAxis { x, y, z, angle }
            }
            0x0A => {
                // RotateEuler(12),
                let yaw = reader.read_f32();
                let pitch = reader.read_f32();
                let roll = reader.read_f32();

                Self::RotateEuler { yaw, pitch, roll }
            }
            0x0B => {
                // Scale(12),
                let x = reader.read_f32();
                let y = reader.read_f32();
                let z = reader.read_f32();

                Self::Scale { x, y, z }
            }
            0x0C => {
                // LoadMatrix(64),
                let m: [f32; 16] = std::array::from_fn(|_| reader.read_f32());
                Self::LoadMatrix(m)
            }
            0x0D => {
                // MulMatrix(64),
                let m: [f32; 16] = std::array::from_fn(|_| reader.read_f32());
                Self::MulMatrix(m)
            }
            0x0E => {
                // Identity(0);
                Self::Identity
            }
            _ => panic!("unknown gpu opcode: {opcode}"),
        }
    }
}
