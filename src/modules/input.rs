use wasmtime::Caller;

use crate::{
    SCREEN_HEIGHT, SCREEN_WIDTH,
    utils::{map_button, map_key},
    wasm::{WASMHostState, WASMRuntime},
};

pub fn link_input(runtime: &WASMRuntime) -> anyhow::Result<()> {
    runtime.linker.with(|linker| {
        linker.func_wrap("input", "get_key_code", |_: Caller<'_, WASMHostState>| {
            if let Some(ch) = macroquad::prelude::get_char_pressed() {
                return ch as i32;
            }
            -1i32
        })?;
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
        #[allow(clippy::cast_possible_truncation)]
        linker.func_wrap("input", "get_mouse_x", |_: Caller<'_, WASMHostState>| {
            macroquad::input::mouse_position().0 as i32
        })?;
        #[allow(clippy::cast_possible_truncation)]
        linker.func_wrap("input", "get_mouse_y", |_: Caller<'_, WASMHostState>| {
            macroquad::input::mouse_position().1 as i32
        })?;
        linker.func_wrap(
            "input",
            "get_mouse_accumulated_dx",
            |_: Caller<'_, WASMHostState>| {
                f64::from(-macroquad::input::mouse_delta_position().x) * f64::from(SCREEN_WIDTH)
            },
        )?;
        linker.func_wrap(
            "input",
            "get_mouse_accumulated_dy",
            |_: Caller<'_, WASMHostState>| {
                f64::from(-macroquad::input::mouse_delta_position().y) * f64::from(SCREEN_HEIGHT)
            },
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
                macroquad::input::show_mouse(false);
                macroquad::input::set_cursor_grab(true);
                caller.data_mut().cursor_grabbed = true;
            },
        )?;
        linker
            .func_wrap(
                "input",
                "release_mouse",
                |mut caller: Caller<'_, WASMHostState>| {
                    macroquad::input::show_mouse(true);
                    macroquad::input::set_cursor_grab(false);
                    caller.data_mut().cursor_grabbed = false;
                },
            )
            .cloned()
    })?;

    Ok(())
}
