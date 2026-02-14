use kira::{
    AudioManager, AudioManagerSettings, Decibels, Frame, Tween,
    sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
};
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

pub type SoundId = u64;

pub struct PlayingSound {
    handle: StaticSoundHandle,
    #[allow(unused)]
    id: SoundId,
}

pub struct RawAudioManager {
    manager: Arc<Mutex<AudioManager>>,
    active: Arc<Mutex<HashMap<SoundId, PlayingSound>>>,
    next_id: Arc<Mutex<SoundId>>,
    max_concurrent_sounds: usize,
}

impl RawAudioManager {
    pub fn new(max_concurrent_sounds: usize) -> Self {
        let manager = AudioManager::new(AudioManagerSettings::default()).unwrap();

        Self {
            manager: Arc::new(Mutex::new(manager)),
            active: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
            max_concurrent_sounds,
        }
    }

    pub fn play(&self, pcm: &[i16], sample_rate: u32) -> anyhow::Result<SoundId> {
        let mut active = self.active.lock();
        if active.len() >= self.max_concurrent_sounds {
            return Err(anyhow::anyhow!("too many sounds"));
        }

        let frames = pcm
            .chunks_exact(2)
            .map(|c| {
                let l = f32::from(c[0]) / 32768.0;
                let r = f32::from(c[1]) / 32768.0;
                Frame::new(l, r)
            })
            .collect();
        let sound_data = StaticSoundData {
            sample_rate,
            settings: StaticSoundSettings::default(),
            slice: None,
            frames,
        };

        let mut manager = self.manager.lock();
        let handle = manager.play(sound_data).unwrap();
        drop(manager);
        let mut next_id = self.next_id.lock();
        let id = *next_id;
        *next_id += 1;
        drop(next_id);

        active.insert(id, PlayingSound { handle, id });
        drop(active);
        Ok(id)
    }

    pub fn stop(&self, id: SoundId) {
        let mut active = self.active.lock();

        if let Some(mut ps) = active.remove(&id) {
            ps.handle.stop(Tween::default());
        }
    }

    pub fn stop_all_sounds(&self) {
        let mut active = self.active.lock();

        for ps in active.values_mut() {
            ps.handle.stop(Tween::default());
        }

        active.clear();
    }

    pub fn set_volume(&self, id: SoundId, volume: f64) {
        let mut active = self.active.lock();

        if let Some(ps) = active.get_mut(&id) {
            let volume = volume.clamp(0.0, 10.0);

            #[allow(clippy::cast_possible_truncation)]
            ps.handle
                .set_volume(Decibels::from(volume as f32), Tween::default());
        }
    }

    pub fn set_pitch(&self, id: SoundId, pitch: f64) {
        let mut active = self.active.lock();

        if let Some(ps) = active.get_mut(&id) {
            let pitch = pitch.clamp(0.1, 10.0);
            ps.handle.set_playback_rate(pitch, Tween::default());
        }
    }

    pub fn is_playing(&self, id: SoundId) -> bool {
        let active = self.active.lock();
        active.contains_key(&id)
    }

    pub fn update(&self) {
        let mut active = self.active.lock();
        let finished: Vec<SoundId> = active
            .iter()
            .filter_map(|(&id, ps)| {
                if ps.handle.state().is_advancing() {
                    None
                } else {
                    Some(id)
                }
            })
            .collect();

        for id in finished {
            active.remove(&id);
        }
    }
}

pub fn get_raw_audio_manager() -> &'static Mutex<RawAudioManager> {
    static RAW_AUDIO_MANAGER: OnceLock<Mutex<RawAudioManager>> = OnceLock::new();
    RAW_AUDIO_MANAGER.get_or_init(|| Mutex::new(RawAudioManager::new(1_000)))
}
