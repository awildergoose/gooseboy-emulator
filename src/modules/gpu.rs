use wasmtime::Caller;

use crate::{
    gpu::{command::GpuCommand, renderer::get_gpu_renderer},
    wasm::{WASMHostState, WASMPointer, WASMRuntime},
};

pub fn link_gpu(runtime: &WASMRuntime) -> anyhow::Result<()> {
    let memory = runtime.memory.clone();

    runtime.linker.with(|linker| {
        let memory = runtime.memory.clone();
        linker.func_wrap(
            "gpu",
            "get_camera_transform",
            move |mut caller: Caller<'_, WASMHostState>, ptr: WASMPointer| {
                let mem = memory.with(|m| m.unwrap().data_mut(&mut caller));
                get_gpu_renderer().lock().camera.write(mem, ptr);
            },
        )?;
        linker.func_wrap(
            "gpu",
            "set_camera_transform",
            |_: Caller<'_, WASMHostState>, x: f32, y: f32, z: f32, yaw: f32, pitch: f32| {
                let camera = &mut get_gpu_renderer().lock().camera;
                camera.read(x, y, z, yaw, pitch);
            },
        )?;
        let memory = runtime.memory.clone();
        linker.func_wrap(
            "gpu",
            "submit_gpu_commands",
            move |mut caller: Caller<'_, WASMHostState>, ptr: WASMPointer, count: u32| {
                let mem = memory.with(|m| m.unwrap().data_mut(&mut caller));

                let mut offset = ptr as usize;
                let end = (ptr + count) as usize;

                while offset < end {
                    let size = GpuCommand::size_of(mem, offset);
                    let command = GpuCommand::deserialize(mem, offset);
                    get_gpu_renderer().lock().queue_command(command);

                    offset += 1 + (size as usize);
                }
            },
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
