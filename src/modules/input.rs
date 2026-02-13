use wasmtime::Caller;

use crate::{
    utils::{map_button, map_key},
    wasm::{WASMHostState, WASMRuntime},
};

pub fn link_input(runtime: &WASMRuntime) -> anyhow::Result<()> {
    runtime.linker.with(|linker| {
        linker.func_wrap("input", "get_key_code", |_: Caller<'_, WASMHostState>| 0i32)?;
        linker.func_wrap(
            "input",
            "get_key",
            |_: Caller<'_, WASMHostState>, key: i32| {
                i32::from(macroquad::prelude::is_key_down(map_key(key)))
            },
        )?;
        linker.func_wrap(
            "input",
            "get_mouse_button",
            |_: Caller<'_, WASMHostState>, button: i32| {
                i32::from(macroquad::prelude::is_mouse_button_down(map_button(button)))
            },
        )?;
        linker.func_wrap("input", "get_mouse_x", |_: Caller<'_, WASMHostState>| {
            macroquad::input::mouse_position().0
        })?;
        linker.func_wrap("input", "get_mouse_y", |_: Caller<'_, WASMHostState>| {
            macroquad::input::mouse_position().1
        })?;
        // TODO: impl
        linker.func_wrap(
            "input",
            "get_mouse_accumulated_dx",
            |_: Caller<'_, WASMHostState>| 0f64,
        )?;
        // TODO: impl
        linker.func_wrap(
            "input",
            "get_mouse_accumulated_dy",
            |_: Caller<'_, WASMHostState>| 0f64,
        )?;
        linker.func_wrap(
            "input",
            "is_mouse_grabbed",
            |caller: Caller<'_, WASMHostState>| i32::from(caller.data().cursor_grabbed),
        )?;
        linker.func_wrap(
            "input",
            "grab_mouse",
            |mut caller: Caller<'_, WASMHostState>| {
                macroquad::input::set_cursor_grab(true);
                caller.data_mut().cursor_grabbed = true;
            },
        )?;
        linker
            .func_wrap(
                "input",
                "release_mouse",
                |mut caller: Caller<'_, WASMHostState>| {
                    macroquad::input::set_cursor_grab(false);
                    caller.data_mut().cursor_grabbed = false;
                },
            )
            .cloned()
    })?;

    Ok(())
}
