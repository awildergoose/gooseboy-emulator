use wasmtime::Caller;

use crate::{
    SCREEN_HEIGHT, SCREEN_WIDTH,
    wasm::{WASMHostState, WASMPointer, WASMPointerMut, WASMRuntime},
};

#[allow(clippy::too_many_lines)]
pub fn link_framebuffer(runtime: &WASMRuntime) -> anyhow::Result<()> {
    let memory = runtime.memory.clone();

    runtime.linker.with(|linker| {
        let memory = memory.clone();

        linker.func_wrap(
            "framebuffer",
            "get_framebuffer_width",
            |_: Caller<'_, WASMHostState>| SCREEN_WIDTH as u32,
        )?;

        linker.func_wrap(
            "framebuffer",
            "get_framebuffer_height",
            |_: Caller<'_, WASMHostState>| SCREEN_HEIGHT as u32,
        )?;

        let memory2 = memory.clone();
        linker.func_wrap(
            "framebuffer",
            "blit_premultiplied_clipped",
            #[allow(clippy::cast_sign_loss)]
            move |mut caller: Caller<'_, WASMHostState>,
                  dest_ptr: WASMPointerMut,
                  dest_w: u32,
                  dest_h: u32,
                  dest_x: i32,
                  dest_y: i32,
                  src_w: u32,
                  src_h: u32,
                  src_ptr: WASMPointer,
                  blend: i32| {
                let mem = memory2.with(|m| m.unwrap().data_mut(&mut caller));
                let blend = match blend {
                    0 => false,
                    1 => true,
                    _ => panic!("blend is not a boolean!"),
                };

                let src = &mut mem[src_ptr as usize..(src_ptr + (src_w * src_h * 4)) as usize].to_vec();
                let dest = &mut mem[dest_ptr as usize..(dest_ptr + (dest_w * dest_h * 4)) as usize];
                let surf_w = dest_w.cast_signed();
                let surf_h = dest_h.cast_signed();

                if src_w == 0 || src_h == 0 {
                    return;
                }

                let src_left = dest_x;
                let src_top = dest_y;
                let src_right = dest_x + src_w.cast_signed();
                let src_bottom = dest_y + src_h.cast_signed();

                let vis_left = src_left.max(0);
                let vis_top = src_top.max(0);
                let vis_right = src_right.min(surf_w);
                let vis_bottom = src_bottom.min(surf_h);

                if vis_left >= vis_right || vis_top >= vis_bottom {
                    return;
                }

                let start_src_x = (vis_left - dest_x) as usize;
                let start_src_y = (vis_top - dest_y) as usize;

                let vis_w = (vis_right - vis_left) as usize;
                let vis_h = (vis_bottom - vis_top) as usize;

                for row in 0..vis_h {
                    let dst_y = (vis_top as usize) + row;
                    let src_row = (start_src_y + row) * (src_w as usize) * 4;
                    for col in 0..vis_w {
                        let dst_x = (vis_left as usize) + col;
                        let sidx = src_row + (start_src_x + col) * 4;
                        let didx = (dst_y * (dest_w as usize) + dst_x) * 4;

                        let sa = src[sidx + 3];
                        if sa == 0 {
                            continue;
                        }

                        if !blend || sa == 255 {
                            dest[didx..didx + 4].copy_from_slice(&src[sidx..sidx + 4]);
                            continue;
                        }

                        let sa_u32 = u32::from(sa);
                        let inv = 255u32 - sa_u32;

                        let sr = u32::from(src[sidx]);
                        let sg = u32::from(src[sidx + 1]);
                        let sb = u32::from(src[sidx + 2]);

                        let dr = u32::from(dest[didx]);
                        let dg = u32::from(dest[didx + 1]);
                        let db = u32::from(dest[didx + 2]);
                        let da = u32::from(dest[didx + 3]);

                        let out_r = sr + ((dr * inv + 127) / 255);
                        let out_g = sg + ((dg * inv + 127) / 255);
                        let out_b = sb + ((db * inv + 127) / 255);
                        let out_a = sa_u32 + ((da * inv + 127) / 255);

                        dest[didx] = out_r.min(255) as u8;
                        dest[didx + 1] = out_g.min(255) as u8;
                        dest[didx + 2] = out_b.min(255) as u8;
                        dest[didx + 3] = out_a.min(255) as u8;
                    }
                }
            },
        )?;

        linker
            .func_wrap(
                "framebuffer",
                "clear_surface",
                move |mut caller: Caller<'_, WASMHostState>,
                      ptr: WASMPointer,
                      size: u32,
                      color: u32| {
                    let mem = memory.with(|m| m.unwrap().data_mut(&mut caller));
                    let slice = &mut mem[ptr as usize..(ptr + size) as usize];
                    for (i, c) in slice.iter_mut().enumerate().take(size as usize) {
                        *c = ((color >> ((i % 4) * 8)) & 0xFF) as u8;
                    }
                },
            )
            .cloned()
    })?;

    Ok(())
}
