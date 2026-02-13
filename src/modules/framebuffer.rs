use wasmtime::Caller;

use crate::{
    SCREEN_HEIGHT, SCREEN_WIDTH,
    wasm::{WASMHostState, WASMPointer, WASMRuntime},
};

pub fn link_framebuffer(runtime: &WASMRuntime) -> anyhow::Result<()> {
    let memory = runtime.memory.clone();

    runtime.linker.with(|linker| {
        linker.func_wrap(
            "framebuffer",
            "get_framebuffer_width",
            |_: Caller<'_, WASMHostState>| SCREEN_WIDTH as u32,
        )?;

        linker.func_wrap(
            "framebuffer",
            "get_framebuffer_height",
            |_: Caller<'_, WASMHostState>| SCREEN_HEIGHT as u32,
        )?;

        linker
            .func_wrap(
                "framebuffer",
                "clear_surface",
                move |mut caller: Caller<'_, WASMHostState>,
                      ptr: WASMPointer,
                      size: u32,
                      color: u32| {
                    let mem = memory.with(|m| m.unwrap().data_mut(&mut caller));
                    let slice = &mut mem[ptr as usize..(ptr + size) as usize];
                    for (i, c) in slice.iter_mut().enumerate().take(size as usize) {
                        *c = ((color >> ((i % 4) * 8)) & 0xFF) as u8;
                    }
                },
            )
            .cloned()
    })?;

    Ok(())
}
