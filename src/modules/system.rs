use std::time::{SystemTime, UNIX_EPOCH};

use wasmtime::Caller;

use crate::wasm::{WASMHostState, WASMRuntime};

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
                |_: Caller<'_, WASMHostState>| {
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos() as i64
                },
            )
            .cloned()
    })?;

    Ok(())
}
