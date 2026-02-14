use dashmap::DashMap;
use fast_cell::FastCell;
use macroquad::texture::Texture2D;
use parking_lot::Mutex;
use std::sync::OnceLock;

pub type TextureId = u64;

pub struct TextureRegistry {
    textures: DashMap<TextureId, FastCell<Texture2D>>,
    last_id: TextureId,
    missing_texture: FastCell<Texture2D>,
}

impl TextureRegistry {
    pub fn new() -> Self {
        Self {
            last_id: 0,
            textures: DashMap::new(),
            missing_texture: FastCell::new(Texture2D::from_file_with_format(
                include_bytes!("../../missing.png"),
                None,
            )),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn create_texture(&mut self, width: u32, height: u32, rgba: &[u8]) -> FastCell<Texture2D> {
        let texture = Texture2D::from_rgba8(width as u16, height as u16, rgba);
        let texture = FastCell::new(texture);
        self.textures.insert(self.last_id, texture.clone());
        self.last_id += 1;
        texture
    }

    pub fn find_texture(&self, id: TextureId) -> Option<FastCell<Texture2D>> {
        self.textures.get(&id).map(|f| f.value().clone())
    }

    pub fn get_default_texture(&self) -> FastCell<Texture2D> {
        self.missing_texture.clone()
    }
}

pub fn get_texture_registry() -> &'static Mutex<TextureRegistry> {
    static TEXTURE_REGISTRY: OnceLock<Mutex<TextureRegistry>> = OnceLock::new();
    TEXTURE_REGISTRY.get_or_init(|| Mutex::new(TextureRegistry::new()))
}
