use wasmtime::Caller;

use crate::{
    gpu::renderer::get_gpu_renderer,
    wasm::{WASMHostState, WASMPointer, WASMRuntime},
};

pub fn link_gpu(runtime: &WASMRuntime) -> anyhow::Result<()> {
    let memory = runtime.memory.clone();

    runtime.linker.with(|linker| {
        linker.func_wrap(
            "gpu",
            "get_camera_transform",
            move |mut caller: Caller<'_, WASMHostState>, ptr: WASMPointer| {
                let mem = memory.with(|m| m.unwrap().data_mut(&mut caller));
                let transform = &get_gpu_renderer().lock().camera.transform;
                transform.write(mem, ptr);
            },
        )?;
        linker.func_wrap(
            "gpu",
            "set_camera_transform",
            |_: Caller<'_, WASMHostState>, x: f32, y: f32, z: f32, yaw: f32, pitch: f32| {
                let transform = &mut get_gpu_renderer().lock().camera.transform;
                transform.x = x;
                transform.y = y;
                transform.z = z;
                transform.yaw = yaw;
                transform.pitch = pitch;
            },
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
                |_: Caller<'_, WASMHostState>, _offset: u32, _ptr: WASMPointer, _len: u32| 0i32,
            )
            .cloned()
    })?;

    Ok(())
}
