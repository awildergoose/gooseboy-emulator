use wasmtime::Caller;

use crate::{
    utils::get_time_nanos,
    wasm::{WASMHostState, WASMRuntime},
};

pub fn link_system(runtime: &WASMRuntime) -> anyhow::Result<()> {
    runtime.linker.with(|linker| {
        linker.func_wrap(
            "system",
            "has_permission",
            |_: Caller<'_, WASMHostState>, _permission: i32| 1i32,
        )?;
        linker
            .func_wrap(
                "system",
                "get_time_nanos",
                #[allow(clippy::cast_possible_truncation)]
                |_: Caller<'_, WASMHostState>| get_time_nanos(),
            )
            .cloned()
    })?;

    Ok(())
}
