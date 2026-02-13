use std::fs;

use macroquad::prelude::*;

use crate::wasm::init_wasm;

mod modules;
mod utils;
pub mod wasm;

pub const SCREEN_WIDTH: i32 = 640;
pub const SCREEN_HEIGHT: i32 = 480;

fn window_conf() -> Conf {
    Conf {
        window_title: "Gooseboy Emulator".to_owned(),
        window_width: SCREEN_WIDTH,
        window_height: SCREEN_HEIGHT,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    env_logger::builder()
        .filter(Some("gooseboy_emulator"), log::LevelFilter::Info)
        .init();

    let data = fs::read("tests/badapple.wasm").expect("failed to open wasm file");
    let mut wasm = init_wasm(data).expect("failed to init wasm");
    log::info!("initialized!");
    wasm.main().expect("failed to call main function");
    log::info!("main function called!");

    let fb_width = SCREEN_WIDTH as usize;
    let fb_height = SCREEN_HEIGHT as usize;
    let fb_size = fb_width * fb_height * 4;
    let mut fb_buf = vec![0u8; fb_size];

    #[allow(clippy::cast_possible_truncation)]
    let texture = Texture2D::from_rgba8(fb_width as u16, fb_height as u16, &fb_buf);

    loop {
        wasm.update().expect("wasm update failed");
        wasm.get_framebuffer_into(&mut fb_buf)
            .expect("failed to fill framebuffer");
        texture.update_from_bytes(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, &fb_buf);

        clear_background(BLACK);
        draw_texture(&texture, 0.0, 0.0, WHITE);

        next_frame().await;
    }
}
