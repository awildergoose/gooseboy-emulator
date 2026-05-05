use wasmtime::Caller;

use crate::{
    utils::get_time_nanos,
    wasm::{WASMHostState, WASMRuntime},
};

pub fn link_system(runtime: &WASMRuntime) -> anyhow::Result<()> {
    let memory = runtime.memory.clone();
    runtime.linker.with(|linker| {
        let memory2 = memory.clone();
        linker.func_wrap(
            "system",
            "has_permission",
            |_: Caller<'_, WASMHostState>, _permission: i32| 1i32,
        )?;
        linker.func_wrap(
            "system",
            "get_time_nanos",
            |_: Caller<'_, WASMHostState>| get_time_nanos(),
        )?;
        linker
            .func_wrap(
                "system",
                "get_platform_name",
                move |mut caller: Caller<'_, WASMHostState>, ptr: i32| {
                    const PLATFORM: &str = "gooseboy-emulator";
                    let mem = memory2.with(|c| c.unwrap().data_mut(&mut caller));
                    let slice = &mut mem[ptr as usize..][..PLATFORM.len()];

                    slice.copy_from_slice(PLATFORM.as_bytes());

                    PLATFORM.len() as i32
                },
            )
            .cloned()
    })?;

    Ok(())
}
