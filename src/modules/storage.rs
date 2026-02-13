use wasmtime::Caller;

use crate::{
    storage::{Storage, get_storage},
    wasm::{WASMHostState, WASMPointer, WASMPointerMut, WASMRuntime},
};

pub fn link_storage(runtime: &WASMRuntime) -> anyhow::Result<()> {
    runtime.linker.with(|linker| {
        let memory = runtime.memory.clone();
        linker.func_wrap(
            "storage",
            "storage_read",
            #[allow(clippy::cast_sign_loss)]
            move |mut caller: Caller<'_, WASMHostState>,
                  offset: i32,
                  ptr: WASMPointerMut,
                  len: i32| {
                let mem_slice = memory.with(|m| m.unwrap().data_mut(&mut caller));
                get_storage()
                    .lock()
                    .read(mem_slice, offset as usize, ptr, len as usize)
            },
        )?;
        let memory = runtime.memory.clone();
        linker.func_wrap(
            "storage",
            "storage_write",
            #[allow(clippy::cast_sign_loss)]
            move |mut caller: Caller<'_, WASMHostState>,
                  offset: i32,
                  ptr: WASMPointer,
                  len: i32| {
                let mem_slice = memory.with(|m| m.unwrap().data_mut(&mut caller));
                get_storage()
                    .lock()
                    .write(mem_slice, offset as usize, ptr, len as usize)
            },
        )?;
        linker.func_wrap("storage", "storage_size", |_: Caller<'_, WASMHostState>| {
            Storage::size()
        })?;
        linker
            .func_wrap(
                "storage",
                "storage_clear",
                |_: Caller<'_, WASMHostState>| get_storage().lock().clear(),
            )
            .cloned()
    })?;

    Ok(())
}
