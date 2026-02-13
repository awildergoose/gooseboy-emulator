use wasmtime::Caller;

use crate::wasm::{WASMHostState, WASMPointer, WASMRuntime};

pub fn link_console(runtime: &WASMRuntime) -> anyhow::Result<()> {
    let memory = runtime.memory.clone();
    runtime.linker.with(|linker| {
        linker
            .func_wrap(
                "console",
                "log",
                move |mut caller: Caller<'_, WASMHostState>, ptr: WASMPointer, len: u32| {
                    let mem = memory.with(|m| m.unwrap().data(&mut caller));
                    let slice = &mem[ptr as usize..(ptr + len) as usize];
                    let string = std::str::from_utf8(slice).unwrap_or("<invalid utf8>");
                    println!("{string}");
                },
            )
            .cloned()
    })?;

    Ok(())
}
