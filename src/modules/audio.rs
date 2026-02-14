use wasmtime::Caller;

use crate::{
    audio_manager::{SoundId, get_raw_audio_manager},
    wasm::{WASMHostState, WASMPointer, WASMRuntime},
};

type WASMSoundId = SoundId;

pub fn link_audio(runtime: &WASMRuntime) -> anyhow::Result<()> {
    runtime.linker.with(|linker| {
        let memory = runtime.memory.clone();

        linker.func_wrap(
            "audio",
            "play_audio",
            move |mut caller: Caller<'_, WASMHostState>, ptr: WASMPointer, len: u32| {
                let mem = memory.with(|m| m.unwrap().data(&mut caller));
                let pcm_raw = &mem[ptr as usize..(ptr + len) as usize];
                let pcm = pcm_raw
                    .chunks_exact(2)
                    .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
                    .collect::<Vec<i16>>();

                get_raw_audio_manager().lock().play(&pcm, 44100)
            },
        )?;
        linker.func_wrap(
            "audio",
            "stop_audio",
            |_: Caller<'_, WASMHostState>, id: WASMSoundId| {
                get_raw_audio_manager().lock().stop(id);
            },
        )?;
        linker.func_wrap("audio", "stop_all_audio", |_: Caller<'_, WASMHostState>| {
            get_raw_audio_manager().lock().stop_all_sounds();
        })?;
        linker.func_wrap(
            "audio",
            "set_audio_volume",
            |_: Caller<'_, WASMHostState>, id: WASMSoundId, volume: f32| {
                get_raw_audio_manager()
                    .lock()
                    .set_volume(id, f64::from(volume));
            },
        )?;
        linker.func_wrap(
            "audio",
            "set_audio_pitch",
            |_: Caller<'_, WASMHostState>, id: WASMSoundId, pitch: f32| {
                get_raw_audio_manager()
                    .lock()
                    .set_pitch(id, f64::from(pitch));
            },
        )?;
        linker
            .func_wrap(
                "audio",
                "is_audio_playing",
                |_: Caller<'_, WASMHostState>, id: WASMSoundId| {
                    i32::from(get_raw_audio_manager().lock().is_playing(id))
                },
            )
            .cloned()
    })?;

    Ok(())
}
