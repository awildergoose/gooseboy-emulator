use parking_lot::Mutex;
use std::{
    collections::HashMap,
    sync::OnceLock,
    time::{Duration, Instant},
};

#[derive(Default)]
pub struct Profiler {
    stack: Vec<(String, Instant)>,
    history: Vec<(String, Instant)>,
    totals: HashMap<String, Duration>,
    counts: HashMap<String, usize>,
}

impl Profiler {
    pub fn push(&mut self, label: String) {
        let now = Instant::now();
        self.stack.push((label.clone(), now));
        self.history.push((label, now));
    }

    pub fn pop(&mut self) -> Option<(String, Duration)> {
        let now = Instant::now();
        match self.stack.pop() {
            Some((label, start)) => {
                let elapsed = now.duration_since(start);
                let entry = self.totals.entry(label.clone()).or_default();
                *entry += elapsed;
                *self.counts.entry(label.clone()).or_default() += 1;
                Some((label, elapsed))
            }
            None => None,
        }
    }
}

fn get_profiler() -> &'static Mutex<Profiler> {
    static PROFILER: OnceLock<Mutex<Profiler>> = OnceLock::new();
    PROFILER.get_or_init(|| Mutex::new(Profiler::default()))
}

pub fn begin_profiler(label: &str) {
    let mut profiler = get_profiler().lock();
    profiler.push(label.to_string());
}

pub fn end_profiler() -> Option<(String, Duration)> {
    let mut profiler = get_profiler().lock();
    profiler.pop()
}

pub fn rebegin_profiler(label: &str) -> Option<(String, Duration)> {
    let ret = end_profiler();
    begin_profiler(label);
    ret
}

pub fn get_profile_averages() -> Vec<(String, Duration)> {
    let profiler = get_profiler().lock();
    profiler
        .totals
        .iter()
        .map(|(label, &total)| {
            let count = *profiler.counts.get(label).unwrap_or(&1);
            #[allow(clippy::cast_possible_truncation)]
            (label.clone(), total / count as u32)
        })
        .collect()
}
