# Gooseboy Emulator

This is a simple, native emulator for the [Gooseboy](https://github.com/awildergoose/gooseboy/).

## Tests

This is the list of working crates from the gooseboy-rs library.

-   [ ] axolotl
-   [x] badapple
-   [ ] bsprenderer
-   [ ] chip8
-   [ ] doom
-   [x] generic
-   [ ] goosegpu
-   [ ] physics
-   [ ] riscv
-   [ ] tests

## Host Functions

### console

-   [x] fn log(ptr: Pointer, len: i32);

### framebuffer

-   [x] fn get_framebuffer_width() -> usize;
-   [x] fn get_framebuffer_height() -> usize;
-   [x] fn clear_surface(ptr: Pointer, size: i32, color: i32);
-   [x] fn blit_premultiplied_clipped(dest_ptr: Pointer, dest_w: usize, dest_h: usize, dest_x: i32, dest_y: i32, src_w: usize, src_h: usize, src_ptr: Pointer, blend: bool);

### memory

-   [x] fn mem_fill(addr: PointerMut, len: i32, value: i32);
-   [x] fn mem_copy(dst: PointerMut, src: Pointer, len: i32);

### input

-   [x] fn get_key_code() -> i32;
-   [x] fn get_key(key: i32) -> bool;
-   [x] fn get_mouse_button(btn: i32) -> bool;
-   [x] fn get_mouse_x() -> i32;
-   [x] fn get_mouse_y() -> i32;
-   [ ] fn get_mouse_accumulated_dx() -> f64;
-   [ ] fn get_mouse_accumulated_dy() -> f64;
-   [x] fn is_mouse_grabbed() -> bool;
-   [x] fn grab_mouse();
-   [x] fn release_mouse();

### audio

-   [ ] fn play_audio(ptr: Pointer, len: i32) -> i64;
-   [ ] fn stop_audio(id: i64);
-   [ ] fn stop_all_audio();
-   [ ] fn set_audio_volume(id: i64, volume: f32);
-   [ ] fn set_audio_pitch(id: i64, volume: f32);
-   [ ] fn is_audio_playing(id: i64) -> bool;

### storage

-   [ ] fn storage_read(offset: i32, ptr: PointerMut, len: i32) -> i32;
-   [ ] fn storage_write(offset: i32, ptr: Pointer, len: i32) -> i32;
-   [ ] fn storage_size() -> u32;
-   [ ] fn storage_clear();

### system

-   [x] fn get_time_nanos() -> i64;
-   [ ] fn has_permission(permission: i32) -> bool;

### gpu

-   [ ] fn get_camera_transform(ptr: PointerMut);
-   [ ] fn set_camera_transform(x: f32, y: f32, z: f32, yaw: f32, pitch: f32);
-   [ ] fn submit_gpu_commands(ptr: Pointer, count: i32);
-   [ ] fn gpu_read(offset: i32, ptr: Pointer, len: i32) -> i32;
