//! MIDI preset presses cross to the helper as bounded requests. The helper reads the library
//! and prepares normalised values; processing applies them through the host at the next block.
//! Neither file access, allocation nor a lock belongs to the audio thread (R-PRESET-KEYS).

use std::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use nih_plug::prelude::*;
use plugin_kit_queue::AtomicQueue;

use crate::{Ca72, library::Library, params::Ca72Params, presets};

#[derive(Debug)]
pub(crate) struct PresetActions {
    params: Arc<Ca72Params>,
    library: Library,
    requests: AtomicQueue,
    /// 0: helper may prepare; 1: audio may apply; 2: helper may publish the name.
    stage: AtomicU8,
    generation: AtomicU64,
    prepared_generation: AtomicU64,
    name: Mutex<Option<(String, String)>>,
    values: Vec<(String, ParamPtr, AtomicU32)>,
    /// The prepared preset's LFO assignments and EDGES (decisions.md R-LFO).
    lfo: crate::lfo::Patch,
}

impl PresetActions {
    pub(crate) fn new(params: Arc<Ca72Params>, library: Library) -> Self {
        let values = params
            .param_map()
            .into_iter()
            .map(|(id, ptr, _)| (id, ptr, AtomicU32::new(f32::NAN.to_bits())))
            .collect();
        Self {
            params,
            library,
            requests: AtomicQueue::new(64),
            stage: AtomicU8::new(0),
            generation: AtomicU64::new(0),
            prepared_generation: AtomicU64::new(0),
            name: Mutex::new(None),
            values,
            lfo: crate::lfo::Patch::default(),
        }
    }

    /// A full queue drops further presses, without blocking processing.
    pub(crate) fn request(&self, by: i32) {
        let generation = self.generation.load(Ordering::Acquire) & (u64::MAX >> 2);
        self.requests
            .try_push((generation << 2) | if by < 0 { 1 } else { 2 });
    }

    /// Host resets invalidate requests, including a patch already being prepared. They never
    /// wait for the helper or take its locks.
    pub(crate) fn cancel(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
    }

    #[cfg(test)]
    pub(crate) fn requested(&self) -> Option<u64> {
        self.requests.try_pop()
    }

    /// Only the helper calls this. A prepared patch stays intact until processing has applied
    /// every value. The next press then resolves against the last preset, in order.
    pub(crate) fn prepare(&self) {
        match self.stage.load(Ordering::Acquire) {
            1 => return,
            2 => {
                if let Ok(mut pending) = self.name.lock()
                    && let Some((before, after)) = pending.take()
                    && let Ok(mut name) = self.params.preset.write()
                    && self.prepared_generation.load(Ordering::Relaxed)
                        == self.generation.load(Ordering::Acquire) & (u64::MAX >> 2)
                    && *name == before
                {
                    *name = after;
                }
                self.stage.store(0, Ordering::Release);
            }
            _ => {}
        }
        let mut request = None;
        for _ in 0..64 {
            let Some(packet) = self.requests.try_pop() else {
                break;
            };
            let generation = packet >> 2;
            if generation == self.generation.load(Ordering::Acquire) & (u64::MAX >> 2) {
                request = Some((generation, packet & 3));
                break;
            }
        }
        let Some((generation, request)) = request else {
            return;
        };
        let mut entries = self.library.entries().0;
        if let Ok(filter) = self.params.midi_map.preset_list.lock()
            && let Some(names) = &*filter
        {
            entries.retain(|e| names.contains(&e.sound.name));
        }
        if entries.is_empty() {
            return;
        }
        let current = self
            .params
            .preset
            .read()
            .map(|s| s.clone())
            .unwrap_or_default();
        let at = entries
            .iter()
            .position(|e| e.sound.name.to_lowercase() == current.to_lowercase());
        let n = entries.len() as i32;
        let by = if request == 1 { -1 } else { 1 };
        let next = match at {
            Some(i) => (i as i32 + by).rem_euclid(n),
            None if by > 0 => 0,
            None => n - 1,
        } as usize;
        let entry = &entries[next];
        let targets = presets::targets(&self.params, entry);
        for (id, _, value) in &self.values {
            value.store(
                targets
                    .iter()
                    .find(|(target, _)| target == id)
                    .map_or(f32::NAN, |(_, v)| *v)
                    .to_bits(),
                Ordering::Relaxed,
            );
        }
        // (What its LFOs modulate, staged here and taken at `apply`: decisions.md R-LFO.)
        self.lfo.load_values(&entry.sound.values);
        if let Ok(mut pending) = self.name.lock() {
            *pending = Some((current, entry.sound.name.clone()));
        }
        self.prepared_generation
            .store(generation, Ordering::Relaxed);
        self.stage.store(1, Ordering::Release);
    }

    /// Only processing calls this: no allocation, file access or lock. The host learns every
    /// changed value by the same path as a learned parameter CC.
    pub(crate) fn apply(&self, context: &mut impl ProcessContext<Ca72>) -> bool {
        if self.stage.load(Ordering::Acquire) != 1 {
            return false;
        }
        if self.prepared_generation.load(Ordering::Relaxed)
            == self.generation.load(Ordering::Acquire) & (u64::MAX >> 2)
        {
            for (_, ptr, value) in &self.values {
                let v = f32::from_bits(value.load(Ordering::Relaxed));
                if v.is_finite() {
                    context.set_parameter_normalized(*ptr, v, 0);
                }
            }
            self.params.lfo_patch.copy_from(&self.lfo);
        }
        self.stage.store(2, Ordering::Release);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nih_plug::context::process::TestProcessContext;
    use std::time::{Duration, Instant};

    #[test]
    fn a_helper_delayed_across_reset_keeps_a_fresh_press() {
        let p = Ca72::default();
        let params = p.parameters().clone();
        let dir = tempfile::tempdir().unwrap();
        let library = Library::at(dir.path());
        let first = library.entries().0[0].sound.name.clone();
        let actions = Arc::new(PresetActions::new(params.clone(), library));
        let mut context =
            TestProcessContext::<Ca72>::new(params.clone(), 48_000.0, ProcessMode::Realtime);
        actions.request(1);
        actions.prepare();
        actions.apply(&mut context);
        context.reported.clear();
        let name = params.preset.write().unwrap();
        let helper = actions.clone();
        let thread = std::thread::spawn(move || helper.prepare());
        let start = Instant::now();
        while actions.name.try_lock().is_ok() {
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "the helper never reached the name lock"
            );
            std::thread::sleep(Duration::from_micros(100));
        }
        actions.cancel();
        actions.request(1);
        drop(name);
        thread.join().unwrap();
        assert!(
            params.preset.read().unwrap().is_empty(),
            "the old name is not published after reset"
        );
        assert!(
            actions.apply(&mut context),
            "the fresh press must survive the helper's old snapshot"
        );
        assert!(!context.reported.is_empty());
        actions.prepare();
        assert_eq!(*params.preset.read().unwrap(), first);
    }
}
