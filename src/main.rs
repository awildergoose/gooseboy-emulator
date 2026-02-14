use std::fs;

use macroquad::{miniquad::window::order_quit, prelude::*};

use crate::{
    audio_manager::get_raw_audio_manager,
    gpu::renderer::get_gpu_renderer,
    profiler::{begin_profiler, end_profiler, get_profile_averages, rebegin_profiler},
    storage::get_storage,
    wasm::init_wasm,
};

mod audio_manager;
mod gpu;
mod modules;
mod profiler;
mod storage;
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
        .filter_level(log::LevelFilter::Info)
        .filter(Some("gooseboy_emulator"), log::LevelFilter::Trace)
        .init();

    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "tests/generic.wasm".to_string());
    let data = fs::read(path).expect("failed to open wasm file");
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
    texture.set_filter(FilterMode::Nearest);

    if wasm.gpu_main().is_ok() {
        log::info!("gpu main function called!");
    }

    prevent_quit();

    loop {
        if is_quit_requested() {
            break;
        }

        begin_profiler("audio update");
        {
            get_raw_audio_manager().lock().update();
        } // release lock

        rebegin_profiler("WASM update");
        wasm.update().expect("wasm update failed");

        rebegin_profiler("copy framebuffer");
        wasm.get_framebuffer_into(&mut fb_buf)
            .expect("failed to fill framebuffer");

        rebegin_profiler("upload texture");
        texture.update_from_bytes(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, &fb_buf);

        rebegin_profiler("clear");
        clear_background(BLACK);

        rebegin_profiler("draw 3d");
        {
            let mut gpu = get_gpu_renderer().lock();
            set_camera(&gpu.camera.cam);
            gpu.execute_commands();
        }

        rebegin_profiler("reset camera");
        set_default_camera();

        rebegin_profiler("draw texture");
        draw_texture(&texture, 0.0, 0.0, WHITE);

        rebegin_profiler("profiler");
        let font_size = 24.0f32;
        let color = Color::new(0.0, 1.0, 0.0, 0.5);
        draw_text(
            &format!("FPS: {}", get_fps()),
            0.0,
            font_size,
            font_size,
            color,
        );
        let mut avgs = get_profile_averages();
        avgs.sort_by(|a, b| b.1.cmp(&a.1));

        #[allow(clippy::cast_precision_loss)]
        for (i, (label, avg)) in avgs.iter().enumerate() {
            let avg_ms = avg.as_secs_f64() * 1000.0;
            draw_text(
                &format!("{label}: {avg_ms:6.3} ms"),
                0.0,
                font_size + font_size.mul_add(i as f32, font_size),
                font_size,
                color,
            );
        }
        end_profiler();

        next_frame().await;
    }

    get_storage().lock().write_to_disk();
    order_quit();
}
