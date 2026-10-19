//! M6 AC6 on a machine with third-party VST3 plugins: with `DAW_VST3_PLUGIN` set to a
//! plugin file or bundle, every plugin in it is instantiated, activated, played a second
//! of noise (and a note, for an instrument) with finite output, and its state moved into a
//! second instance. Without the variable the test says so and passes: CI machines have no
//! third-party plugins of their own.

#![allow(unsafe_code, clippy::unwrap_used)]

use daw_helper::vst3::Module;
use daw_ipc::Event;
use daw_ipc::layout::{EV_NOTE_OFF, EV_NOTE_ON};
use std::path::Path;
use std::sync::Arc;

#[test]
fn a_third_party_plugin_loads_plays_and_keeps_its_state() {
    let Some(path) = std::env::var_os("DAW_VST3_PLUGIN") else {
        eprintln!("skipped: set DAW_VST3_PLUGIN to a VST3 plugin to run this check");
        return;
    };
    let m = Arc::new(Module::load(Path::new(&path)).unwrap());
    let plugins = m.descriptors();
    assert!(
        !plugins.is_empty(),
        "no plugins in {}",
        Path::new(&path).display()
    );
    for d in plugins {
        let mut a = m
            .instantiate(&d.id)
            .unwrap_or_else(|e| panic!("{}: {e}", d.name));
        let params = a.params();
        a.activate(48_000.0, 512).unwrap();
        let mut p = a.processor(512);
        assert!(p.start());
        let (mut peak, mut finite) = (0.0f32, true);
        let mut seed = 1u32;
        for block in 0..94 {
            let mut l: Vec<f32> = (0..512)
                .map(|_| {
                    seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    (seed >> 8) as f32 / (1u32 << 24) as f32 - 0.5
                })
                .collect();
            let mut r = l.clone();
            let (mut ol, mut or) = (vec![0.0f32; 512], vec![0.0f32; 512]);
            let notes: Vec<Event> = match block {
                0 if a.takes_notes() => vec![Event {
                    kind: EV_NOTE_ON,
                    key: 60,
                    value: 0.8,
                    note_id: 1,
                    ..Event::default()
                }],
                50 if a.takes_notes() => vec![Event {
                    kind: EV_NOTE_OFF,
                    key: 60,
                    note_id: 1,
                    ..Event::default()
                }],
                _ => Vec::new(),
            };
            // SAFETY: every buffer holds 512 samples.
            let status = unsafe {
                p.process(
                    block * 512,
                    512,
                    &notes,
                    [l.as_mut_ptr(), r.as_mut_ptr()],
                    [ol.as_mut_ptr(), or.as_mut_ptr()],
                )
            };
            assert_eq!(status, 0, "{}: block {block}", d.name);
            for v in ol.iter().chain(&or) {
                finite &= v.is_finite();
                peak = peak.max(v.abs());
            }
        }
        p.stop();
        assert!(finite, "{}: output not finite", d.name);
        let state = a.save_state().unwrap();
        let b = m.instantiate(&d.id).unwrap();
        b.load_state(&state)
            .unwrap_or_else(|e| panic!("{}: {e}", d.name));
        eprintln!(
            "{} ({}, {}): {} parameters, latency {}, peak {peak:.3}, state {} bytes, restored",
            d.name,
            d.vendor,
            d.features.join("|"),
            params.len(),
            a.latency(),
            state.len()
        );
    }
}
