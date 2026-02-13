use wasmtime::Caller;

use crate::wasm::{WASMHostState, WASMPointer, WASMPointerMut, WASMRuntime};

pub fn link_memory(runtime: &WASMRuntime) -> anyhow::Result<()> {
    let memory = runtime.memory.clone();
    runtime.linker.with(|linker| {
        let memory2 = memory.clone();
        linker.func_wrap(
            "memory",
            "mem_fill",
            move |mut caller: Caller<'_, WASMHostState>,
                  ptr: WASMPointerMut,
                  len: u32,
                  value: u32| {
                let mem = memory2.with(|m| m.unwrap().data_mut(&mut caller));
                let slice = &mut mem[ptr as usize..(ptr + len) as usize];

                #[allow(clippy::cast_sign_loss)]
                #[allow(clippy::cast_possible_truncation)]
                for iv in slice.iter_mut() {
                    *iv = value as u8;
                }
            },
        )?;

        let memory = memory.clone();
        linker
            .func_wrap(
                "memory",
                "mem_copy",
                move |mut caller: Caller<'_, WASMHostState>,
                      dst: WASMPointerMut,
                      src: WASMPointer,
                      len: u32| {
                    let mem = memory.with(|m| m.unwrap().data_mut(&mut caller));
                    let src_slice = mem[src as usize..(src as usize + len as usize)].to_vec();
                    let dst_slice = &mut mem[dst as usize..(dst as usize + len as usize)];
                    dst_slice.copy_from_slice(&src_slice);
                },
            )
            .cloned()
    })?;

    Ok(())
}
