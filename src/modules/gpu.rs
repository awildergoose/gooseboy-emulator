use wasmtime::Caller;

use crate::wasm::{WASMHostState, WASMPointer, WASMRuntime};

pub fn link_gpu(runtime: &WASMRuntime) -> anyhow::Result<()> {
    runtime.linker.with(|linker| {
        linker.func_wrap(
            "gpu",
            "get_camera_transform",
            |_: Caller<'_, WASMHostState>, _ptr: WASMPointer| {},
        )?;
        linker.func_wrap(
            "gpu",
            "set_camera_transform",
            |_: Caller<'_, WASMHostState>, _x: f32, _y: f32, _z: f32, _yaw: f32, _pitch: f32| {},
        )?;
        linker.func_wrap(
            "gpu",
            "submit_gpu_commands",
            |_: Caller<'_, WASMHostState>, _ptr: WASMPointer, _count: u32| {},
        )?;
        linker
            .func_wrap(
                "gpu",
                "gpu_read",
                |_: Caller<'_, WASMHostState>, _offset: u32, _ptr: WASMPointer, _len: u32| {},
            )
            .cloned()
    })?;

    Ok(())
}
