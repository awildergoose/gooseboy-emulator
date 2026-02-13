use parking_lot::Mutex;
use std::{fs, sync::OnceLock};

use crate::wasm::{WASMPointer, WASMPointerMut};

pub const STORAGE_SIZE: usize = 8 * 1024 * 1024;

// TODO: support the original file format
pub struct Storage {
    pub data: Vec<u8>,
    pub dirty: bool,
}

impl Storage {
    pub fn new() -> Self {
        let mut this = Self {
            data: vec![0; STORAGE_SIZE],
            dirty: false,
        };

        this.read_from_disk();
        this
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn read(
        &self,
        mem_slice: &mut [u8],
        offset: usize,
        wasm_ptr: WASMPointerMut,
        len: usize,
    ) -> u32 {
        if offset >= STORAGE_SIZE {
            return 0;
        }
        let to_read = len.min(STORAGE_SIZE - offset);

        if wasm_ptr as usize + to_read > mem_slice.len() {
            return 0;
        }

        mem_slice[wasm_ptr as usize..wasm_ptr as usize + to_read]
            .copy_from_slice(&self.data[offset..offset + to_read]);

        to_read as u32
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn write(
        &mut self,
        mem_slice: &[u8],
        offset: usize,
        wasm_ptr: WASMPointer,
        len: usize,
    ) -> u32 {
        if offset >= STORAGE_SIZE {
            return 0;
        }
        let to_write = len.min(STORAGE_SIZE - offset);

        if wasm_ptr as usize + to_write > mem_slice.len() {
            return 0;
        }

        self.data[offset..offset + to_write]
            .copy_from_slice(&mem_slice[wasm_ptr as usize..wasm_ptr as usize + to_write]);

        self.dirty = true;
        to_write as u32
    }

    pub fn clear(&mut self) {
        self.data.fill(0);
        self.dirty = true;
    }

    #[allow(clippy::cast_possible_truncation)]
    pub const fn size() -> u32 {
        STORAGE_SIZE as u32
    }

    pub fn write_to_disk(&self) {
        fs::write(
            std::env::current_dir().unwrap().join("storage.bin"),
            self.data.clone(),
        )
        .expect("failed to write storage to disk");
    }

    pub fn read_from_disk(&mut self) {
        let data = fs::read(std::env::current_dir().unwrap().join("storage.bin"));
        if let Ok(data) = data {
            self.data = data;
        }
    }
}

pub fn get_storage() -> &'static Mutex<Storage> {
    static STORAGE: OnceLock<Mutex<Storage>> = OnceLock::new();
    STORAGE.get_or_init(|| Mutex::new(Storage::new()))
}
