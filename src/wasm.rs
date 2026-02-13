#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
use fast_cell::FastCell;
use std::time::{SystemTime, UNIX_EPOCH};
use wasmtime::{Engine, Instance, Linker, Memory, Module, Store};

use crate::{
    SCREEN_HEIGHT, SCREEN_WIDTH,
    modules::{console::link_console, framebuffer::link_framebuffer, input::link_input, memory::link_memory, system::link_system},
};

pub type WASMPointer = u32;
pub type WASMPointerMut = u32;
pub struct WASMHostState {
    pub cursor_grabbed: bool
}

pub struct WASMRuntime {
    pub engine: FastCell<Engine>,
    pub store: FastCell<Store<WASMHostState>>,
    pub memory: FastCell<Option<Memory>>,
    pub linker: FastCell<Linker<WASMHostState>>,
    pub instance: FastCell<Option<Instance>>,
}

impl WASMRuntime {
    pub fn main(&mut self) -> anyhow::Result<()> {
        let store = self.store.get_mut();
        let instance = self.instance.get_mut();
        instance
            .unwrap()
            .get_typed_func::<(), ()>(&mut *store, "main")?
            .call(store, ())
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn update(&mut self) -> anyhow::Result<()> {
        let store = self.store.get_mut();
        let instance = self.instance.get_mut();
        instance
            .unwrap()
            .get_typed_func::<i64, ()>(&mut *store, "update")?
            .call(
                store,
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos() as i64,
            )?;
        Ok(())
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn get_framebuffer(&mut self) -> anyhow::Result<Vec<u8>> {
        let store = self.store.get_mut();
        let instance = self.instance.get_mut();
        let memory = self.memory.get_mut().unwrap();

        let buffer_ptr = instance
            .unwrap()
            .get_typed_func::<(), i32>(&mut *store, "get_framebuffer_ptr")?
            .call(&mut *store, ())? as usize;

        let mut pixels = vec![0u8; (SCREEN_WIDTH * SCREEN_HEIGHT * 4) as usize];
        memory.read(&mut *store, buffer_ptr, &mut pixels)?;

        Ok(pixels)
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn get_framebuffer_into(&mut self, pixels: &mut [u8]) -> anyhow::Result<()> {
        let store = self.store.get_mut();
        let instance = self.instance.get_mut();
        let memory = self.memory.get_mut().unwrap();

        let buffer_ptr = instance
            .unwrap()
            .get_typed_func::<(), i32>(&mut *store, "get_framebuffer_ptr")?
            .call(&mut *store, ())? as usize;

        memory.read(&mut *store, buffer_ptr, pixels)?;

        Ok(())
    }
}

pub fn init_wasm(wasm: Vec<u8>) -> anyhow::Result<WASMRuntime> {
    let engine = Engine::default();
    log::info!("engine OK");
    let module = Module::new(&engine, wasm)?;
    log::info!("module OK");
    let store = Store::new(&engine, WASMHostState {
        cursor_grabbed: false
    });
    log::info!("store OK");
    let linker = <Linker<WASMHostState>>::new(&engine);
    log::info!("linker OK");
    let mut runtime = WASMRuntime {
        engine: FastCell::new(engine),
        store: FastCell::new(store),
        linker: FastCell::new(linker),
        memory: FastCell::new(None),
        instance: FastCell::new(None),
    };
    log::info!("runtime OK");

    link_framebuffer(&runtime)?;
    log::info!("framebuffer linked");
    link_console(&runtime)?;
    log::info!("console linked");
    link_memory(&runtime)?;
    log::info!("memory linked");
    link_input(&runtime)?;
    log::info!("input linked");
    link_system(&runtime)?;
    log::info!("system linked");

    let linker = runtime.linker.get_mut();
    let store = runtime.store.get_mut();
    let instance = linker.instantiate(&mut *store, &module)?;
    let memory = instance
        .get_export(&mut *store, "memory")
        .and_then(wasmtime::Extern::into_memory)
        .ok_or_else(|| anyhow::anyhow!("module did not export memory"))?;
    *runtime.memory.get_mut() = Some(memory);
    *runtime.instance.get_mut() = Some(instance);
    log::info!("linked module!");

    Ok(runtime)
}
