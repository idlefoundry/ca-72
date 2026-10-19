//! The shipped CLAP test plugin (DESIGN 9.3, 10.4; M1 plan W5).
//!
//! A deterministic stereo effect for testing plugin hosting:
//!
//! `y[n] = gain * (x[n - latency] + mix * x[n - latency - delay])`
//!
//! - `latency` is declared to the host (for delay compensation tests) and applied when the
//!   plugin is activated;
//! - the delay line is internal audio state that a crash loses (for recovery tests);
//! - state save and load cover `gain`, `mix`, `delay` and `latency`;
//! - a small GUI of one colour, embedded in the host's window (`gui`);
//! - names for three keys (CLAP's note-name extension, M18 W9): 36 "Test Kick", 38 "Test
//!   Snare", 42 "Test Hat", on every channel; and two a host should not show for the roll's
//!   rows: 40 "Ch6 Rim" on channel 6 alone, 44 "Port2 Bell" on a second note port;
//! - names that change as it runs (D216): the hidden enumeration `names` (0 to 2) gives the
//!   three keys "Test", "Alt" or "Edit" names, and the plugin says so on the main thread
//!   (`clap_host_note_name.changed`); the hidden `edit names`, set to a value, stands in for
//!   its editor: the plugin sets `names` to it itself and sends the edit as an output event
//!   (a gesture and a parameter value), without a word through the note-name extension.
//!   Neither is saved;
//! - stand-ins for a preset loaded in its editor (D228): the hidden `load preset` and `load
//!   preset quietly` set its gain and delay mix itself and say so with a rescan of values or
//!   by marking its state dirty; and for its editor's knob held (D227): `hold gain` begins a
//!   gesture on its gain and sets it, `release gain` ends it; a hidden read-only `meter`;
//! - its state beyond its parameters (D234): a "sample" played at a level of its own (its
//!   output times it, 1 unless one is loaded), kept in its state only when not 1, and
//!   stand-ins that load one: alone, said by marking its state dirty (`load sample`), in a
//!   preset that also sets its gain and delay mix, said with a rescan of values (`load
//!   sample preset`), or without a word (`load sample quietly`); and `change quietly`, which
//!   sets its gain and delay mix without a word;
//!   it marks its state dirty after a host loads one (on its main thread, a moment later);
//! - the hidden `crash` and `hang` parameters abort the process or stop it responding
//!   when they are set during processing. They are never saved, so restoring a snapshot
//!   after an injected crash does not crash again.
//!
//! All arithmetic is plain `f32` with no feedback, so output is bit-exact for a given input
//! and the same events, whatever the block size.
//!
//! Unsafe code: this crate is an implementation of the CLAP C ABI; every entry point
//! dereferences pointers the host passes. See `docs/unsafe.md`.

#![allow(unsafe_code)]

mod gui;
use clap_sys::audio_buffer::clap_audio_buffer;
use clap_sys::entry::clap_plugin_entry;
use clap_sys::events::{
    CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_PARAM_GESTURE_BEGIN, CLAP_EVENT_PARAM_GESTURE_END,
    CLAP_EVENT_PARAM_MOD, CLAP_EVENT_PARAM_VALUE, clap_event_header, clap_event_param_gesture,
    clap_event_param_mod, clap_event_param_value, clap_input_events, clap_output_events,
};
use clap_sys::ext::audio_ports::{
    CLAP_AUDIO_PORT_IS_MAIN, CLAP_EXT_AUDIO_PORTS, CLAP_PORT_STEREO, clap_audio_port_info,
    clap_plugin_audio_ports,
};
use clap_sys::ext::latency::{CLAP_EXT_LATENCY, clap_plugin_latency};
use clap_sys::ext::params::{
    CLAP_EXT_PARAMS, CLAP_PARAM_IS_AUTOMATABLE, CLAP_PARAM_IS_ENUM, CLAP_PARAM_IS_HIDDEN,
    CLAP_PARAM_IS_MODULATABLE, CLAP_PARAM_IS_READONLY, CLAP_PARAM_IS_STEPPED, clap_param_info,
    clap_param_info_flags, clap_plugin_params,
};
use clap_sys::ext::state::{CLAP_EXT_STATE, clap_plugin_state};
use clap_sys::ext::tail::{CLAP_EXT_TAIL, clap_plugin_tail};
use clap_sys::factory::plugin_factory::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory};
use clap_sys::host::clap_host;
use clap_sys::id::clap_id;
use clap_sys::plugin::{clap_plugin, clap_plugin_descriptor};
use clap_sys::process::{
    CLAP_PROCESS_CONTINUE, CLAP_PROCESS_ERROR, clap_process, clap_process_status,
};
use clap_sys::stream::{clap_istream, clap_ostream};
use clap_sys::version::CLAP_VERSION;
use std::cell::UnsafeCell;
use std::ffi::{CStr, c_char, c_void};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

/// The plugin's CLAP identifier.
pub const PLUGIN_ID: &CStr = c"org.daw-v3.test-plugin";

/// Stable parameter IDs.
pub const P_GAIN: clap_id = 1;
pub const P_MIX: clap_id = 2;
pub const P_DELAY: clap_id = 3;
pub const P_LATENCY: clap_id = 4;
pub const P_CRASH: clap_id = 100;
pub const P_HANG: clap_id = 101;
/// Which names its keys have (D216), and the stand-in for its editor's choice of them.
pub const P_NAMES: clap_id = 102;
pub const P_EDIT: clap_id = 103;
/// Stand-ins for its editor and its own requests (D222), hidden and never saved: `nudge
/// gain` set to a value makes the plugin set its gain to it and send that as a value with
/// no gesture; `ask flush` makes it hold that gain as an editor's edit made between
/// blocks, ask the host for a flush (`clap_host_params.request_flush`, from the main
/// thread) and send it, in a gesture, from the next `process` or `flush`; `clear refs`
/// set to 1, 2 or 4 makes it ask the host to clear what refers to its gain with those
/// flags (`clap_host_params.clear`, all, automation or modulation). At 0 (their rest, as
/// when taken away) they do nothing.
pub const P_NUDGE: clap_id = 104;
pub const P_ASK_FLUSH: clap_id = 105;
pub const P_CLEAR: clap_id = 106;
/// How it answers the host's gain (D222), hidden and never saved: 1 sends the value back as
/// an output event, as some plugins do; 2 halves it and sends that marked
/// `CLAP_EVENT_DONT_RECORD`. Neither is an edit for the project.
pub const P_ANSWER: clap_id = 107;
/// A preset loaded in its editor (D228), hidden and never saved: `load preset` set to a
/// value makes the plugin set its gain to it and its delay mix to half of it itself, with no
/// output events, and say so on the main thread with `clap_host_params.rescan(VALUES)`;
/// `load preset quietly` does the same and says only `clap_host_state.mark_dirty`. At 0
/// (their rest) they do nothing.
pub const P_PRESET: clap_id = 108;
pub const P_PRESET_DIRTY: clap_id = 109;
/// A gesture held open (D227), hidden and never saved: `hold gain` set to a value makes the
/// plugin set its gain to it and send the beginning of a gesture and the value, as an
/// editor's knob held; `release gain` set to anything but 0 sends the gesture's end.
pub const P_HOLD: clap_id = 110;
pub const P_RELEASE: clap_id = 111;
/// A value it shows but does not take (D228), `CLAP_PARAM_IS_READONLY`: what a host reads
/// of it when its values change is not this.
pub const P_METER: clap_id = 112;
/// Its state beyond its parameters (D234), hidden stand-ins never saved, at rest at 0, that
/// act when set to another value (as buttons do: the same value given again does nothing):
/// `load sample` set to a value loads a sample of that level (its output times it) and marks its
/// state dirty; `load sample preset` sets its gain to the value, its delay mix and sample to
/// half of it, and rescans its values; `change quietly` sets its gain and delay mix as `load
/// preset` does, without a word; `load sample quietly` loads a sample without one, a second
/// and a half of audio after its unchanged baseline is saved (apart from the host's value
/// that asked for it: host values change a state too).
pub const P_LOAD_SAMPLE: clap_id = 113;
pub const P_SAMPLE_PRESET: clap_id = 114;
pub const P_CHANGE_QUIETLY: clap_id = 115;
pub const P_SAMPLE_QUIETLY: clap_id = 116;

/// Largest latency and delay, in frames.
pub const MAX_LATENCY: u32 = 4096;
pub const MAX_DELAY: u32 = 4800;

/// Ring buffer length: a power of two above the longest look-back.
const RING: usize = 16384;
const MASK: usize = RING - 1;
const _: () = assert!(RING > (MAX_LATENCY + MAX_DELAY) as usize);

/// No value (`TestPlugin::gui_gain`).
const NONE: u64 = u64::MAX;

/// Frames of audio after the quiet sample's baseline is saved until it is loaded (1.5 s
/// at 48 kHz). A delayed helper must capture the trigger before the state-only change.
const QUIET_FRAMES: i64 = 72_000;

/// State format: this magic, then (parameter ID as u32, value as f64) pairs, little endian.
const STATE_MAGIC: &[u8; 8] = b"DAWTP\0\0\x01";
/// The pair's ID for its sample's level (D234): its state beyond its parameters, present
/// only when not 1.
const SAMPLE_ID: u32 = 0xFFFF_FF01;

struct ParamDef {
    id: clap_id,
    name: &'static CStr,
    min: f64,
    max: f64,
    default: f64,
    flags: clap_param_info_flags,
    /// Part of the saved state.
    saved: bool,
}

const PARAMS: [ParamDef; 21] = [
    ParamDef {
        id: P_GAIN,
        name: c"Gain",
        min: 0.0,
        max: 2.0,
        default: 1.0,
        flags: CLAP_PARAM_IS_AUTOMATABLE | CLAP_PARAM_IS_MODULATABLE,
        saved: true,
    },
    ParamDef {
        id: P_MIX,
        name: c"Delay mix",
        min: 0.0,
        max: 1.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_AUTOMATABLE | CLAP_PARAM_IS_MODULATABLE,
        saved: true,
    },
    ParamDef {
        id: P_DELAY,
        name: c"Delay",
        min: 1.0,
        max: MAX_DELAY as f64,
        default: 480.0,
        flags: CLAP_PARAM_IS_STEPPED | CLAP_PARAM_IS_AUTOMATABLE,
        saved: true,
    },
    ParamDef {
        id: P_LATENCY,
        name: c"Latency",
        min: 0.0,
        max: MAX_LATENCY as f64,
        default: 0.0,
        flags: CLAP_PARAM_IS_STEPPED,
        saved: true,
    },
    ParamDef {
        id: P_CRASH,
        name: c"Crash",
        min: 0.0,
        max: 1.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_STEPPED | CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
    ParamDef {
        id: P_HANG,
        name: c"Hang",
        min: 0.0,
        max: 1.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_STEPPED | CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
    ParamDef {
        id: P_NAMES,
        name: c"Names",
        min: 0.0,
        max: 2.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_STEPPED | CLAP_PARAM_IS_ENUM | CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
    ParamDef {
        id: P_EDIT,
        name: c"Edit names",
        min: 0.0,
        max: 2.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_STEPPED | CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
    ParamDef {
        id: P_NUDGE,
        name: c"Nudge gain",
        min: 0.0,
        max: 2.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
    ParamDef {
        id: P_ASK_FLUSH,
        name: c"Ask flush",
        min: 0.0,
        max: 2.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
    ParamDef {
        id: P_CLEAR,
        name: c"Clear refs",
        min: 0.0,
        max: 7.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_STEPPED | CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
    ParamDef {
        id: P_ANSWER,
        name: c"Answer gain",
        min: 0.0,
        max: 2.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_STEPPED | CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
    ParamDef {
        id: P_PRESET,
        name: c"Load preset",
        min: 0.0,
        max: 2.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
    ParamDef {
        id: P_PRESET_DIRTY,
        name: c"Load preset quietly",
        min: 0.0,
        max: 2.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
    ParamDef {
        id: P_HOLD,
        name: c"Hold gain",
        min: 0.0,
        max: 2.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
    ParamDef {
        id: P_RELEASE,
        name: c"Release gain",
        min: 0.0,
        max: 1.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
    ParamDef {
        id: P_METER,
        name: c"Meter",
        min: 0.0,
        max: 1.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_READONLY | CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
    ParamDef {
        id: P_LOAD_SAMPLE,
        name: c"Load sample",
        min: 0.0,
        max: 1.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
    ParamDef {
        id: P_SAMPLE_PRESET,
        name: c"Load sample preset",
        min: 0.0,
        max: 2.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
    ParamDef {
        id: P_CHANGE_QUIETLY,
        name: c"Change quietly",
        min: 0.0,
        max: 2.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
    ParamDef {
        id: P_SAMPLE_QUIETLY,
        name: c"Load sample quietly",
        min: 0.0,
        max: 1.0,
        default: 0.0,
        flags: CLAP_PARAM_IS_HIDDEN,
        saved: false,
    },
];
const N: usize = PARAMS.len();

fn index_of(id: clap_id) -> Option<usize> {
    PARAMS.iter().position(|p| p.id == id)
}

fn clamp_param(i: usize, v: f64) -> f64 {
    let p = &PARAMS[i];
    let v = if v.is_finite() { v } else { p.default };
    let v = v.clamp(p.min, p.max);
    if p.flags & CLAP_PARAM_IS_STEPPED != 0 {
        v.round()
    } else {
        v
    }
}

/// State touched only by the audio thread, or by the main thread while the plugin is not
/// processing (CLAP's threading rules).
struct Audio {
    ring: Vec<[f32; 2]>,
    pos: usize,
    /// Non-destructive modulation amounts, by parameter index.
    mods: [f64; N],
}

struct TestPlugin {
    clap: clap_plugin,
    host: *const clap_host,
    /// Base values by parameter index, as `f64` bits: read by the main thread while the
    /// audio thread writes them.
    values: [AtomicU64; N],
    /// Latency fixed at activation.
    latency: AtomicU32,
    active: AtomicBool,
    /// Its names changed, to be said on the main thread (D216).
    names_changed: AtomicBool,
    /// A gain its editor stand-in set between blocks, as `f64` bits (`NONE`: none), sent
    /// from the next `process` or `flush`; a flush to ask for, and clear flags to ask with
    /// (0: none), from the main thread (D222).
    gui_gain: AtomicU64,
    ask_flush: AtomicBool,
    clear_flags: AtomicU32,
    /// A preset its stand-in loaded, to be said on the main thread (D228): 1 with a rescan
    /// of values, 2 with its state marked dirty (0: none).
    preset_word: AtomicU32,
    /// Its sample's level (D234), as `f64` bits: its state beyond its parameters.
    sample: AtomicU64,
    /// A sample to load quietly (`NONE`: none), and the frames of audio until then.
    /// A negative count waits for a state save to capture the unchanged baseline.
    quiet_sample: AtomicU64,
    quiet_in: std::sync::atomic::AtomicI64,
    audio: UnsafeCell<Audio>,
    /// The GUI (main thread only).
    gui: std::sync::Mutex<gui::Gui>,
}

// SAFETY: `audio` is only touched under CLAP's threading contract (the audio thread while
// processing, the main thread otherwise); everything else is atomic or immutable after
// creation. `host` is a pointer the host guarantees valid for the plugin's lifetime.
unsafe impl Sync for TestPlugin {}
// SAFETY: as above; the plugin moves between threads only as the host dictates.
unsafe impl Send for TestPlugin {}

impl TestPlugin {
    fn value(&self, id: clap_id) -> f64 {
        index_of(id).map_or(0.0, |i| {
            f64::from_bits(self.values[i].load(Ordering::Relaxed))
        })
    }

    fn set(&self, id: clap_id, v: f64) {
        if let Some(i) = index_of(id) {
            self.values[i].store(clamp_param(i, v).to_bits(), Ordering::Relaxed);
        }
    }

    /// Base value plus modulation, clamped.
    fn effective(&self, audio: &Audio, id: clap_id) -> f64 {
        match index_of(id) {
            Some(i) => clamp_param(i, self.value(id) + audio.mods[i]),
            None => 0.0,
        }
    }

    fn state_bytes(&self) -> Vec<u8> {
        let mut out = STATE_MAGIC.to_vec();
        for p in PARAMS.iter().filter(|p| p.saved) {
            out.extend_from_slice(&p.id.to_le_bytes());
            out.extend_from_slice(&self.value(p.id).to_le_bytes());
        }
        let sample = self.sample();
        if sample != 1.0 {
            out.extend_from_slice(&SAMPLE_ID.to_le_bytes());
            out.extend_from_slice(&sample.to_le_bytes());
        }
        out
    }

    /// `n` frames of audio went by: a sample waiting to be loaded quietly is, in time.
    fn tick_quiet(&self, n: usize) {
        if self.quiet_sample.load(Ordering::Relaxed) == NONE
            || self.quiet_in.load(Ordering::Acquire) < 0
        {
            return;
        }
        if self.quiet_in.fetch_sub(n as i64, Ordering::Relaxed) <= n as i64 {
            let v = self.quiet_sample.swap(NONE, Ordering::Relaxed);
            if v != NONE {
                self.set_sample(f64::from_bits(v));
            }
        }
    }

    /// Its sample's level (D234).
    fn sample(&self) -> f64 {
        f64::from_bits(self.sample.load(Ordering::Relaxed))
    }

    fn set_sample(&self, v: f64) {
        let v = if v.is_finite() {
            v.clamp(0.0, 1.0)
        } else {
            1.0
        };
        self.sample.store(v.to_bits(), Ordering::Relaxed);
    }

    fn load_state_bytes(&self, bytes: &[u8]) -> bool {
        let Some(rest) = bytes.strip_prefix(STATE_MAGIC) else {
            return false;
        };
        if !rest.len().is_multiple_of(12) {
            return false;
        }
        let mut parsed = Vec::new();
        let mut sample = 1.0;
        for chunk in rest.chunks_exact(12) {
            let mut id = [0u8; 4];
            let mut v = [0u8; 8];
            id.copy_from_slice(&chunk[..4]);
            v.copy_from_slice(&chunk[4..]);
            let id = u32::from_le_bytes(id);
            match index_of(id) {
                Some(i) if PARAMS[i].saved => parsed.push((id, f64::from_le_bytes(v))),
                _ if id == SAMPLE_ID => sample = f64::from_le_bytes(v),
                _ => return false,
            }
        }
        for (id, v) in parsed {
            self.set(id, v);
        }
        self.set_sample(sample);
        true
    }

    fn request_restart(&self) {
        if self.host.is_null() {
            return;
        }
        // SAFETY: the host pointer is valid for the plugin's lifetime and
        // `request_restart` is thread-safe per the CLAP spec.
        unsafe {
            if let Some(f) = (*self.host).request_restart {
                f(self.host);
            }
        }
    }

    /// Asks the host for a call on its main thread (`on_main_thread`).
    fn request_callback(&self) {
        if self.host.is_null() {
            return;
        }
        // SAFETY: the host pointer is valid for the plugin's lifetime and
        // `request_callback` is thread-safe per the CLAP spec.
        unsafe {
            if let Some(f) = (*self.host).request_callback {
                f(self.host);
            }
        }
    }

    /// The stand-in for its editor (D216): `names` set to `v` by the plugin itself, and the
    /// edit sent to the host as an editor's is (a gesture around a parameter value), with no
    /// word through the note-name extension.
    ///
    /// # Safety
    /// `out` must be the host's output event list for this call, or null.
    unsafe fn edit_names(&self, v: f64, time: u32, out: *const clap_output_events) {
        self.set(P_NAMES, v);
        // SAFETY: the caller's output list, or null.
        unsafe { self.send_edit(P_NAMES, time, true, out, 0) };
    }

    /// Sends parameter `id`'s value as an edit of its own (D216, D222): in a gesture
    /// (`gesture`) or as a value alone.
    ///
    /// # Safety
    /// `out` must be the host's output event list for this call, or null.
    unsafe fn send_edit(
        &self,
        id: clap_id,
        time: u32,
        gesture: bool,
        out: *const clap_output_events,
        flags: u32,
    ) {
        // SAFETY: the caller's output list, or null.
        unsafe { self.send_parts(id, time, (gesture, true, gesture), out, flags) };
    }

    /// Sends parts of an edit of parameter `id` (D227's held gesture): its gesture's
    /// beginning, its value, its end, as `parts` says.
    ///
    /// # Safety
    /// `out` must be the host's output event list for this call, or null.
    unsafe fn send_parts(
        &self,
        id: clap_id,
        time: u32,
        parts: (bool, bool, bool),
        out: *const clap_output_events,
        flags: u32,
    ) {
        // SAFETY: the caller's output list, or null.
        let Some(o) = (unsafe { out.as_ref() }) else {
            return;
        };
        let Some(push) = o.try_push else { return };
        let header = |size: usize, type_: u16| clap_event_header {
            size: size as u32,
            time,
            space_id: CLAP_CORE_EVENT_SPACE_ID,
            type_,
            flags,
        };
        let mark = |type_| clap_event_param_gesture {
            header: header(std::mem::size_of::<clap_event_param_gesture>(), type_),
            param_id: id,
        };
        let begin = mark(CLAP_EVENT_PARAM_GESTURE_BEGIN);
        let value = clap_event_param_value {
            header: header(
                std::mem::size_of::<clap_event_param_value>(),
                CLAP_EVENT_PARAM_VALUE,
            ),
            param_id: id,
            cookie: std::ptr::null_mut(),
            note_id: -1,
            port_index: -1,
            channel: -1,
            key: -1,
            value: self.value(id),
        };
        let end = mark(CLAP_EVENT_PARAM_GESTURE_END);
        // SAFETY: pushing our own events, each with its header first, through the host's
        // list for this call.
        unsafe {
            if parts.0 {
                push(out, &begin.header);
            }
            if parts.1 {
                push(out, &value.header);
            }
            if parts.2 {
                push(out, &end.header);
            }
        }
    }

    /// The gain its editor stand-in set between blocks (D222): set, and sent as an edit in
    /// a gesture, at the start of a `process` or `flush`.
    ///
    /// # Safety
    /// `out` must be the host's output event list for this call, or null.
    unsafe fn gui_edits(&self, out: *const clap_output_events) {
        let bits = self.gui_gain.swap(NONE, Ordering::AcqRel);
        if bits != NONE {
            self.set(P_GAIN, f64::from_bits(bits));
            // SAFETY: the caller's output list, or null.
            unsafe { self.send_edit(P_GAIN, 0, true, out, 0) };
        }
    }

    /// Applies a parameter event. Returns false for events it does not understand.
    ///
    /// # Safety
    /// `h` must point to a valid event whose size matches its type; `out` must be the
    /// host's output event list for this call, or null.
    unsafe fn event(
        &self,
        audio: &mut Audio,
        h: *const clap_event_header,
        out: *const clap_output_events,
    ) {
        // SAFETY: the caller guarantees `h` is a valid event header.
        let hdr = unsafe { &*h };
        if hdr.space_id != CLAP_CORE_EVENT_SPACE_ID {
            return;
        }
        match hdr.type_ {
            CLAP_EVENT_PARAM_VALUE => {
                // SAFETY: the type says this is a param-value event.
                let e = unsafe { &*(h as *const clap_event_param_value) };
                let before = self.value(e.param_id);
                if e.param_id == P_EDIT {
                    self.set(P_EDIT, e.value);
                    // SAFETY: the caller's output list.
                    unsafe { self.edit_names(self.value(P_EDIT), hdr.time, out) };
                    return;
                }
                // 0 is the stand-ins' rest: taken away, they do nothing.
                if (e.param_id == P_NUDGE || e.param_id == P_ASK_FLUSH) && e.value <= 0.0 {
                    self.set(e.param_id, 0.0);
                    return;
                }
                if e.param_id == P_NUDGE {
                    self.set(P_NUDGE, e.value);
                    self.set(P_GAIN, self.value(P_NUDGE));
                    // SAFETY: the caller's output list.
                    unsafe { self.send_edit(P_GAIN, hdr.time, false, out, 0) };
                    return;
                }
                if e.param_id == P_ASK_FLUSH {
                    self.set(P_ASK_FLUSH, e.value);
                    self.gui_gain
                        .store(self.value(P_ASK_FLUSH).to_bits(), Ordering::Release);
                    self.ask_flush.store(true, Ordering::Release);
                    self.request_callback();
                    return;
                }
                // A gesture held open (D227): its beginning and value, then its end.
                if e.param_id == P_HOLD || e.param_id == P_RELEASE {
                    self.set(e.param_id, e.value);
                    if e.value > 0.0 {
                        let hold = e.param_id == P_HOLD;
                        if hold {
                            self.set(P_GAIN, e.value);
                        }
                        // SAFETY: the caller's output list.
                        unsafe { self.send_parts(P_GAIN, hdr.time, (hold, hold, !hold), out, 0) };
                    }
                    return;
                }
                // Its state beyond its parameters (D234); 0 is their rest.
                if matches!(
                    e.param_id,
                    P_LOAD_SAMPLE | P_SAMPLE_PRESET | P_CHANGE_QUIETLY | P_SAMPLE_QUIETLY
                ) {
                    let before = self.value(e.param_id);
                    self.set(e.param_id, e.value);
                    if e.value > 0.0 && self.value(e.param_id) != before {
                        let v = self.value(e.param_id);
                        if matches!(e.param_id, P_SAMPLE_PRESET | P_CHANGE_QUIETLY) {
                            self.set(P_GAIN, v);
                            self.set(P_MIX, v / 2.0);
                        }
                        match e.param_id {
                            P_LOAD_SAMPLE => self.set_sample(v),
                            P_SAMPLE_PRESET => self.set_sample(v / 2.0),
                            P_SAMPLE_QUIETLY => {
                                self.quiet_sample.store(v.to_bits(), Ordering::Relaxed);
                                self.quiet_in.store(-1, Ordering::Relaxed);
                            }
                            _ => {}
                        }
                        let word = match e.param_id {
                            P_SAMPLE_PRESET => 1,
                            P_LOAD_SAMPLE => 2,
                            _ => 0,
                        };
                        if word != 0 {
                            self.preset_word.store(word, Ordering::Release);
                            self.request_callback();
                        }
                    }
                    return;
                }
                if e.param_id == P_PRESET || e.param_id == P_PRESET_DIRTY {
                    self.set(e.param_id, e.value);
                    // 0 is their rest: taken away, they load nothing.
                    if e.value > 0.0 {
                        self.set(P_GAIN, e.value);
                        self.set(P_MIX, e.value / 2.0);
                        let word = if e.param_id == P_PRESET { 1 } else { 2 };
                        self.preset_word.store(word, Ordering::Release);
                        self.request_callback();
                    }
                    return;
                }
                if e.param_id == P_CLEAR {
                    self.set(P_CLEAR, e.value);
                    self.clear_flags
                        .store(self.value(P_CLEAR) as u32, Ordering::Release);
                    self.request_callback();
                    return;
                }
                self.set(e.param_id, e.value);
                // Its answer to the host's gain: the value back, or half of it marked not
                // to be recorded (D222).
                let answer = self.value(P_ANSWER) as u32;
                if e.param_id == P_GAIN && answer != 0 {
                    let flags = if answer == 2 {
                        self.set(P_GAIN, self.value(P_GAIN) / 2.0);
                        clap_sys::events::CLAP_EVENT_DONT_RECORD
                    } else {
                        0
                    };
                    // SAFETY: the caller's output list.
                    unsafe { self.send_edit(P_GAIN, hdr.time, false, out, flags) };
                }
                if e.param_id == P_LATENCY
                    && self.active.load(Ordering::Relaxed)
                    && self.value(P_LATENCY) != before
                {
                    self.request_restart();
                }
                // Other names: said on the main thread (D216).
                if e.param_id == P_NAMES && self.value(P_NAMES) != before {
                    self.names_changed.store(true, Ordering::Release);
                    self.request_callback();
                }
            }
            CLAP_EVENT_PARAM_MOD => {
                // SAFETY: the type says this is a param-mod event.
                let e = unsafe { &*(h as *const clap_event_param_mod) };
                if let Some(i) = index_of(e.param_id) {
                    audio.mods[i] = if e.amount.is_finite() { e.amount } else { 0.0 };
                }
            }
            _ => {}
        }
    }

    /// Acts on the crash and hang parameters.
    fn inject_faults(&self) {
        if self.value(P_CRASH) >= 0.5 {
            std::process::abort();
        }
        if self.value(P_HANG) >= 0.5 {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}

/// # Safety
/// `p` must be a plugin created by this library and not yet destroyed.
unsafe fn plugin<'a>(p: *const clap_plugin) -> &'a TestPlugin {
    // SAFETY: `plugin_data` points to the `TestPlugin` that owns `*p` (see `create`).
    unsafe { &*((*p).plugin_data as *const TestPlugin) }
}

/// Walks an input event list in order.
///
/// # Safety
/// `list` must be a valid CLAP input event list.
unsafe fn events(list: *const clap_input_events) -> impl Iterator<Item = *const clap_event_header> {
    let (size, get) = if list.is_null() {
        (0, None)
    } else {
        // SAFETY: the caller guarantees `list` is valid.
        let l = unsafe { &*list };
        // SAFETY: calling the list's own accessor on itself.
        let n = l.size.map_or(0, |f| unsafe { f(list) });
        (n, l.get)
    };
    (0..size).filter_map(move |i| {
        // SAFETY: `i` is below the list's size.
        let h = get.map_or(std::ptr::null(), |g| unsafe { g(list, i) });
        (!h.is_null()).then_some(h)
    })
}

unsafe extern "C" fn init(_: *const clap_plugin) -> bool {
    true
}

unsafe extern "C" fn destroy(p: *const clap_plugin) {
    // SAFETY: `plugin_data` came from `Box::into_raw` in `create`; the host calls
    // `destroy` once.
    unsafe { drop(Box::from_raw((*p).plugin_data as *mut TestPlugin)) };
}

unsafe extern "C" fn activate(p: *const clap_plugin, _sr: f64, _min: u32, _max: u32) -> bool {
    // SAFETY: valid plugin pointer from the host.
    let pl = unsafe { plugin(p) };
    // SAFETY: activation happens on the main thread while not processing.
    let audio = unsafe { &mut *pl.audio.get() };
    audio.ring = vec![[0.0; 2]; RING];
    audio.pos = 0;
    audio.mods = [0.0; N];
    pl.latency
        .store(pl.value(P_LATENCY) as u32, Ordering::Relaxed);
    pl.active.store(true, Ordering::Relaxed);
    true
}

unsafe extern "C" fn deactivate(p: *const clap_plugin) {
    // SAFETY: valid plugin pointer from the host.
    unsafe { plugin(p) }.active.store(false, Ordering::Relaxed);
}

unsafe extern "C" fn start_processing(_: *const clap_plugin) -> bool {
    true
}

unsafe extern "C" fn stop_processing(_: *const clap_plugin) {}

unsafe extern "C" fn reset(p: *const clap_plugin) {
    // SAFETY: valid plugin pointer; `reset` is called on the audio thread.
    let pl = unsafe { plugin(p) };
    // SAFETY: audio-thread state, touched on the audio thread.
    let audio = unsafe { &mut *pl.audio.get() };
    audio.ring.iter_mut().for_each(|f| *f = [0.0; 2]);
    audio.pos = 0;
}

/// # Safety
/// `b` must be a valid audio buffer with at least two channels of `n` frames.
unsafe fn channels(b: *const clap_audio_buffer) -> Option<(*mut f32, *mut f32)> {
    if b.is_null() {
        return None;
    }
    // SAFETY: the caller guarantees `b` is valid.
    let b = unsafe { &*b };
    if b.channel_count < 2 || b.data32.is_null() {
        return None;
    }
    // SAFETY: `data32` has `channel_count` (at least two) channel pointers.
    let (l, r) = unsafe { (*b.data32, *b.data32.add(1)) };
    (!l.is_null() && !r.is_null()).then_some((l, r))
}

unsafe extern "C" fn process(
    p: *const clap_plugin,
    pr: *const clap_process,
) -> clap_process_status {
    // SAFETY: valid plugin and process pointers from the host, on the audio thread.
    let (pl, pr) = unsafe { (plugin(p), &*pr) };
    // SAFETY: audio-thread state, touched on the audio thread.
    let audio = unsafe { &mut *pl.audio.get() };
    if pr.audio_inputs_count < 1 || pr.audio_outputs_count < 1 || audio.ring.len() != RING {
        return CLAP_PROCESS_ERROR;
    }
    // SAFETY: the host provides the buffers it declared, each `frames_count` long.
    let (Some((il, ir)), Some((ol, or))) = (unsafe { channels(pr.audio_inputs) }, unsafe {
        channels(pr.audio_outputs)
    }) else {
        return CLAP_PROCESS_ERROR;
    };
    let n = pr.frames_count as usize;
    pl.tick_quiet(n);
    let latency = pl.latency.load(Ordering::Relaxed) as usize;
    pl.inject_faults();
    // SAFETY: the host's output list for this block.
    unsafe { pl.gui_edits(pr.out_events) };
    // SAFETY: the host's input event list for this block.
    let mut evs = unsafe { events(pr.in_events) }.peekable();
    let mut i = 0usize;
    while i < n {
        // Apply the events at or before frame `i`.
        while let Some(&h) = evs.peek() {
            // SAFETY: `h` is a valid event from the host.
            if unsafe { (*h).time } as usize > i {
                break;
            }
            // SAFETY: `h` is a valid event from the host; the output list is the host's.
            unsafe { pl.event(audio, h, pr.out_events) };
            evs.next();
        }
        pl.inject_faults();
        // SAFETY: `h` is a valid event from the host.
        let end = evs
            .peek()
            .map_or(n, |&h| (unsafe { (*h).time } as usize).clamp(i + 1, n));
        let gain = (pl.effective(audio, P_GAIN) * pl.sample()) as f32;
        let mix = pl.effective(audio, P_MIX) as f32;
        let delay = pl.effective(audio, P_DELAY) as usize;
        for j in i..end {
            // SAFETY: `j < n` and every channel holds `n` frames; inputs are read before
            // outputs are written, so in-place buffers are fine.
            let x = unsafe { [*il.add(j), *ir.add(j)] };
            audio.ring[audio.pos & MASK] = x;
            let d0 = audio.ring[audio.pos.wrapping_sub(latency) & MASK];
            let d1 = audio.ring[audio.pos.wrapping_sub(latency + delay) & MASK];
            // SAFETY: as above.
            unsafe {
                *ol.add(j) = gain * (d0[0] + mix * d1[0]);
                *or.add(j) = gain * (d0[1] + mix * d1[1]);
            }
            audio.pos = audio.pos.wrapping_add(1);
        }
        i = end;
    }
    // Events stamped at or after the block's end still apply.
    for h in evs {
        // SAFETY: `h` is a valid event from the host; the output list is the host's.
        unsafe { pl.event(audio, h, pr.out_events) };
    }
    CLAP_PROCESS_CONTINUE
}

unsafe extern "C" fn get_extension(_: *const clap_plugin, id: *const c_char) -> *const c_void {
    if id.is_null() {
        return std::ptr::null();
    }
    // SAFETY: the host passes a NUL-terminated extension ID.
    let id = unsafe { CStr::from_ptr(id) };
    if id == CLAP_EXT_PARAMS {
        &PARAMS_EXT as *const _ as *const c_void
    } else if id == CLAP_EXT_STATE {
        &STATE_EXT as *const _ as *const c_void
    } else if id == CLAP_EXT_LATENCY {
        &LATENCY_EXT as *const _ as *const c_void
    } else if id == CLAP_EXT_AUDIO_PORTS {
        &PORTS_EXT as *const _ as *const c_void
    } else if id == CLAP_EXT_TAIL {
        &TAIL_EXT as *const _ as *const c_void
    } else if id == clap_sys::ext::gui::CLAP_EXT_GUI {
        &gui::GUI_EXT as *const _ as *const c_void
    } else if id == clap_sys::ext::note_name::CLAP_EXT_NOTE_NAME {
        &NOTE_NAME_EXT as *const _ as *const c_void
    } else {
        std::ptr::null()
    }
}

/// Says its names changed (D216), asks for a flush and to clear what refers to its gain
/// (D222), on the main thread as the note-name and parameter extensions ask.
unsafe extern "C" fn on_main_thread(p: *const clap_plugin) {
    // SAFETY: valid plugin pointer from the host, on its main thread.
    let pl = unsafe { plugin(p) };
    if pl.host.is_null() {
        return;
    }
    // SAFETY: the host pointer is valid for the plugin's lifetime; its extension, if it
    // gives one, has the type its ID names.
    let params = unsafe {
        (*pl.host)
            .get_extension
            .map_or(std::ptr::null(), |g| g(pl.host, CLAP_EXT_PARAMS.as_ptr()))
            as *const clap_sys::ext::params::clap_host_params
    };
    let word = pl.preset_word.swap(0, Ordering::AcqRel);
    // SAFETY: as above.
    if let Some(params) = unsafe { params.as_ref() } {
        // A preset loaded (D228): its values rescanned.
        if word == 1
            && let Some(rescan) = params.rescan
        {
            // SAFETY: on the main thread, as `rescan` asks.
            unsafe { rescan(pl.host, clap_sys::ext::params::CLAP_PARAM_RESCAN_VALUES) };
        }
        let flags = pl.clear_flags.swap(0, Ordering::AcqRel);
        if flags != 0
            && let Some(clear) = params.clear
        {
            // SAFETY: on the main thread, as `clear` asks.
            unsafe { clear(pl.host, P_GAIN, flags) };
        }
        if pl.ask_flush.swap(false, Ordering::AcqRel)
            && let Some(request) = params.request_flush
        {
            // SAFETY: not on the audio thread, as `request_flush` asks.
            unsafe { request(pl.host) };
        }
    }
    // A preset loaded quietly (D228): its state marked dirty, nothing more.
    if word == 2 {
        // SAFETY: the host pointer is valid for the plugin's lifetime; its extension, if it
        // gives one, has the type its ID names.
        unsafe {
            let ext = (*pl.host)
                .get_extension
                .map_or(std::ptr::null(), |g| g(pl.host, CLAP_EXT_STATE.as_ptr()))
                as *const clap_sys::ext::state::clap_host_state;
            if let Some(mark) = ext.as_ref().and_then(|e| e.mark_dirty) {
                mark(pl.host);
            }
        }
    }
    if !pl.names_changed.swap(false, Ordering::AcqRel) {
        return;
    }
    // SAFETY: the host pointer is valid for the plugin's lifetime; its extension, if it
    // gives one, has the type its ID names.
    unsafe {
        let ext = (*pl.host).get_extension.map_or(std::ptr::null(), |g| {
            g(
                pl.host,
                clap_sys::ext::note_name::CLAP_EXT_NOTE_NAME.as_ptr(),
            )
        }) as *const clap_sys::ext::note_name::clap_host_note_name;
        if let Some(changed) = ext.as_ref().and_then(|e| e.changed) {
            changed(pl.host);
        }
    }
}

// ---- parameters -------------------------------------------------------------------

unsafe extern "C" fn params_count(_: *const clap_plugin) -> u32 {
    N as u32
}

fn copy_str(dst: &mut [c_char], s: &[u8]) {
    let n = s.len().min(dst.len().saturating_sub(1));
    for (d, b) in dst.iter_mut().zip(&s[..n]) {
        *d = *b as c_char;
    }
    if let Some(d) = dst.get_mut(n) {
        *d = 0;
    }
}

unsafe extern "C" fn params_get_info(
    _: *const clap_plugin,
    index: u32,
    info: *mut clap_param_info,
) -> bool {
    let Some(p) = PARAMS.get(index as usize) else {
        return false;
    };
    if info.is_null() {
        return false;
    }
    // SAFETY: the host passes a writable `clap_param_info`.
    let info = unsafe { &mut *info };
    info.id = p.id;
    info.flags = p.flags;
    info.cookie = std::ptr::null_mut();
    copy_str(&mut info.name, p.name.to_bytes());
    copy_str(&mut info.module, b"");
    info.min_value = p.min;
    info.max_value = p.max;
    info.default_value = p.default;
    true
}

unsafe extern "C" fn params_get_value(p: *const clap_plugin, id: clap_id, out: *mut f64) -> bool {
    if index_of(id).is_none() || out.is_null() {
        return false;
    }
    // SAFETY: valid plugin pointer and a writable `f64` from the host.
    unsafe { *out = plugin(p).value(id) };
    true
}

unsafe extern "C" fn params_value_to_text(
    _: *const clap_plugin,
    id: clap_id,
    value: f64,
    out: *mut c_char,
    cap: u32,
) -> bool {
    let Some(i) = index_of(id) else { return false };
    if out.is_null() || cap == 0 {
        return false;
    }
    let text = match id {
        P_DELAY | P_LATENCY => format!("{} samples", clamp_param(i, value)),
        P_CRASH | P_HANG => (if value >= 0.5 { "on" } else { "off" }).to_owned(),
        P_NAMES | P_EDIT => NAME_SETS[clamp_param(i, value) as usize].to_owned(),
        P_CLEAR => format!("{}", clamp_param(i, value)),
        _ => format!("{value:.3}"),
    };
    // SAFETY: the host passes a buffer of `cap` bytes.
    let dst = unsafe { std::slice::from_raw_parts_mut(out, cap as usize) };
    copy_str(dst, text.as_bytes());
    true
}

unsafe extern "C" fn params_text_to_value(
    _: *const clap_plugin,
    id: clap_id,
    text: *const c_char,
    out: *mut f64,
) -> bool {
    if index_of(id).is_none() || text.is_null() || out.is_null() {
        return false;
    }
    // SAFETY: the host passes a NUL-terminated string.
    let s = unsafe { CStr::from_ptr(text) }.to_string_lossy();
    let word = s.split_whitespace().next().unwrap_or("");
    let v = match word {
        "on" => 1.0,
        "off" => 0.0,
        w => match w.parse::<f64>() {
            Ok(v) => v,
            Err(_) => return false,
        },
    };
    // SAFETY: writable `f64` from the host.
    unsafe { *out = v };
    true
}

unsafe extern "C" fn params_flush(
    p: *const clap_plugin,
    in_: *const clap_input_events,
    out: *const clap_output_events,
) {
    // SAFETY: valid plugin pointer; `flush` runs on the main thread when not processing
    // or on the audio thread instead of `process`, so audio state is ours.
    let pl = unsafe { plugin(p) };
    // SAFETY: as above.
    let audio = unsafe { &mut *pl.audio.get() };
    // SAFETY: the host's output list for this flush.
    unsafe { pl.gui_edits(out) };
    // SAFETY: the host's event list.
    for h in unsafe { events(in_) } {
        // SAFETY: valid event from the host; the output list is the host's.
        unsafe { pl.event(audio, h, out) };
    }
}

static PARAMS_EXT: clap_plugin_params = clap_plugin_params {
    count: Some(params_count),
    get_info: Some(params_get_info),
    get_value: Some(params_get_value),
    value_to_text: Some(params_value_to_text),
    text_to_value: Some(params_text_to_value),
    flush: Some(params_flush),
};

// ---- state, latency, tail, audio ports --------------------------------------------

unsafe extern "C" fn state_save(p: *const clap_plugin, stream: *const clap_ostream) -> bool {
    if stream.is_null() {
        return false;
    }
    // SAFETY: valid plugin and stream pointers from the host.
    let (pl, s) = unsafe { (plugin(p), &*stream) };
    let Some(write) = s.write else { return false };
    let pending = pl.quiet_sample.load(Ordering::Relaxed);
    let bytes = pl.state_bytes();
    let mut done = 0;
    while done < bytes.len() {
        let rest = &bytes[done..];
        // SAFETY: writing from our own buffer through the host's stream.
        let n = unsafe { write(stream, rest.as_ptr() as *const c_void, rest.len() as u64) };
        if n <= 0 {
            return false;
        }
        done += n as usize;
    }
    if pending != NONE && pl.quiet_sample.load(Ordering::Relaxed) == pending {
        // The quiet fixture changes only after the helper can save its unchanged
        // baseline. Audio progress cannot assume a prior state poll on a busy machine.
        let _ =
            pl.quiet_in
                .compare_exchange(-1, QUIET_FRAMES, Ordering::Release, Ordering::Relaxed);
    }
    true
}

unsafe extern "C" fn state_load(p: *const clap_plugin, stream: *const clap_istream) -> bool {
    if stream.is_null() {
        return false;
    }
    // SAFETY: valid plugin and stream pointers from the host.
    let (pl, s) = unsafe { (plugin(p), &*stream) };
    let Some(read) = s.read else { return false };
    let mut bytes = Vec::new();
    let mut buf = [0u8; 256];
    loop {
        // SAFETY: reading into our own buffer through the host's stream.
        let n = unsafe { read(stream, buf.as_mut_ptr() as *mut c_void, buf.len() as u64) };
        if n < 0 {
            return false;
        }
        if n == 0 {
            break;
        }
        bytes.extend_from_slice(&buf[..n as usize]);
        if bytes.len() > 1 << 16 {
            return false;
        }
    }
    let loaded = pl.load_state_bytes(&bytes);
    // Then it marks its state dirty, a moment later on its main thread, as some plugins do
    // after a host loads one (D234: the host's doing, not a preset of its own).
    if loaded {
        pl.preset_word.store(2, Ordering::Release);
        pl.request_callback();
    }
    loaded
}

static STATE_EXT: clap_plugin_state = clap_plugin_state {
    save: Some(state_save),
    load: Some(state_load),
};

unsafe extern "C" fn latency_get(p: *const clap_plugin) -> u32 {
    // SAFETY: valid plugin pointer from the host.
    unsafe { plugin(p) }.latency.load(Ordering::Relaxed)
}

static LATENCY_EXT: clap_plugin_latency = clap_plugin_latency {
    get: Some(latency_get),
};

unsafe extern "C" fn tail_get(p: *const clap_plugin) -> u32 {
    // SAFETY: valid plugin pointer from the host.
    let pl = unsafe { plugin(p) };
    pl.latency.load(Ordering::Relaxed) + pl.value(P_DELAY) as u32
}

static TAIL_EXT: clap_plugin_tail = clap_plugin_tail {
    get: Some(tail_get),
};

/// The keys the plugin names (M18 W9): key, name, channel, port (-1: every one). The
/// first three's names begin with the set `names` chooses (D216).
const NOTE_NAMES: [(i16, &str, i16, i16); 5] = [
    (36, "Kick", -1, -1),
    (38, "Snare", -1, -1),
    (42, "Hat", -1, -1),
    (40, "Ch6 Rim", 5, -1),
    (44, "Port2 Bell", -1, 1),
];

/// The sets of names `names` chooses between (D216).
const NAME_SETS: [&str; 3] = ["Test", "Alt", "Edit"];

unsafe extern "C" fn note_name_count(_: *const clap_plugin) -> u32 {
    NOTE_NAMES.len() as u32
}

unsafe extern "C" fn note_name_get(
    p: *const clap_plugin,
    index: u32,
    out: *mut clap_sys::ext::note_name::clap_note_name,
) -> bool {
    let Some((key, name, channel, port)) = NOTE_NAMES.get(index as usize) else {
        return false;
    };
    if out.is_null() {
        return false;
    }
    let name = if index < 3 {
        // SAFETY: valid plugin pointer from the host.
        let set = unsafe { plugin(p) }.value(P_NAMES) as usize;
        format!("{} {name}", NAME_SETS[set.min(NAME_SETS.len() - 1)])
    } else {
        (*name).to_owned()
    };
    // SAFETY: the host passes a writable note name (checked for null above).
    let out = unsafe { &mut *out };
    out.name = [0; clap_sys::string_sizes::CLAP_NAME_SIZE];
    for (d, s) in out
        .name
        .iter_mut()
        .take(clap_sys::string_sizes::CLAP_NAME_SIZE - 1)
        .zip(name.as_bytes())
    {
        *d = *s as c_char;
    }
    out.port = *port;
    out.key = *key;
    out.channel = *channel;
    true
}

static NOTE_NAME_EXT: clap_sys::ext::note_name::clap_plugin_note_name =
    clap_sys::ext::note_name::clap_plugin_note_name {
        count: Some(note_name_count),
        get: Some(note_name_get),
    };

unsafe extern "C" fn ports_count(_: *const clap_plugin, _is_input: bool) -> u32 {
    1
}

unsafe extern "C" fn ports_get(
    _: *const clap_plugin,
    index: u32,
    _is_input: bool,
    info: *mut clap_audio_port_info,
) -> bool {
    if index != 0 || info.is_null() {
        return false;
    }
    // SAFETY: writable port info from the host.
    let info = unsafe { &mut *info };
    info.id = 0;
    copy_str(&mut info.name, b"Main");
    info.flags = CLAP_AUDIO_PORT_IS_MAIN;
    info.channel_count = 2;
    info.port_type = CLAP_PORT_STEREO.as_ptr();
    info.in_place_pair = 0;
    true
}

static PORTS_EXT: clap_plugin_audio_ports = clap_plugin_audio_ports {
    count: Some(ports_count),
    get: Some(ports_get),
};

// ---- descriptor, factory and entry ------------------------------------------------

struct Features([*const c_char; 3]);
// SAFETY: pointers to static, immutable C strings.
unsafe impl Sync for Features {}

static FEATURES: Features = Features([
    c"audio-effect".as_ptr(),
    c"stereo".as_ptr(),
    std::ptr::null(),
]);

static DESCRIPTOR: clap_plugin_descriptor = clap_plugin_descriptor {
    clap_version: CLAP_VERSION,
    id: PLUGIN_ID.as_ptr(),
    name: c"DAW-v3 Test Plugin".as_ptr(),
    vendor: c"DAW-v3".as_ptr(),
    url: c"".as_ptr(),
    manual_url: c"".as_ptr(),
    support_url: c"".as_ptr(),
    version: c"0.1.0".as_ptr(),
    description: c"Deterministic gain, latency and delay with crash and hang injection, for tests."
        .as_ptr(),
    features: FEATURES.0.as_ptr(),
};

unsafe extern "C" fn factory_count(_: *const clap_plugin_factory) -> u32 {
    1
}

unsafe extern "C" fn factory_descriptor(
    _: *const clap_plugin_factory,
    index: u32,
) -> *const clap_plugin_descriptor {
    if index == 0 {
        &DESCRIPTOR
    } else {
        std::ptr::null()
    }
}

unsafe extern "C" fn create(
    _: *const clap_plugin_factory,
    host: *const clap_host,
    id: *const c_char,
) -> *const clap_plugin {
    // SAFETY: the host passes a NUL-terminated plugin ID.
    if id.is_null() || unsafe { CStr::from_ptr(id) } != PLUGIN_ID {
        return std::ptr::null();
    }
    let pl = Box::new(TestPlugin {
        clap: clap_plugin {
            desc: &DESCRIPTOR,
            plugin_data: std::ptr::null_mut(),
            init: Some(init),
            destroy: Some(destroy),
            activate: Some(activate),
            deactivate: Some(deactivate),
            start_processing: Some(start_processing),
            stop_processing: Some(stop_processing),
            reset: Some(reset),
            process: Some(process),
            get_extension: Some(get_extension),
            on_main_thread: Some(on_main_thread),
        },
        host,
        values: std::array::from_fn(|i| AtomicU64::new(PARAMS[i].default.to_bits())),
        latency: AtomicU32::new(0),
        active: AtomicBool::new(false),
        names_changed: AtomicBool::new(false),
        gui_gain: AtomicU64::new(NONE),
        ask_flush: AtomicBool::new(false),
        clear_flags: AtomicU32::new(0),
        preset_word: AtomicU32::new(0),
        sample: AtomicU64::new(1.0f64.to_bits()),
        quiet_sample: AtomicU64::new(NONE),
        quiet_in: std::sync::atomic::AtomicI64::new(0),
        audio: UnsafeCell::new(Audio {
            ring: Vec::new(),
            pos: 0,
            mods: [0.0; N],
        }),
        gui: std::sync::Mutex::new(gui::Gui::default()),
    });
    let raw = Box::into_raw(pl);
    // SAFETY: `raw` is a fresh, valid allocation; the plugin points at itself so that
    // callbacks can find it, and `destroy` frees it.
    unsafe {
        (*raw).clap.plugin_data = raw as *mut c_void;
        &(*raw).clap
    }
}

static FACTORY: clap_plugin_factory = clap_plugin_factory {
    get_plugin_count: Some(factory_count),
    get_plugin_descriptor: Some(factory_descriptor),
    create_plugin: Some(create),
};

unsafe extern "C" fn entry_init(_path: *const c_char) -> bool {
    true
}

unsafe extern "C" fn entry_deinit() {}

unsafe extern "C" fn entry_get_factory(id: *const c_char) -> *const c_void {
    // SAFETY: the host passes a NUL-terminated factory ID.
    if !id.is_null() && unsafe { CStr::from_ptr(id) } == CLAP_PLUGIN_FACTORY_ID {
        &FACTORY as *const _ as *const c_void
    } else {
        std::ptr::null()
    }
}

/// The CLAP entry point.
#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static clap_entry: clap_plugin_entry = clap_plugin_entry {
    clap_version: CLAP_VERSION,
    init: Some(entry_init),
    deinit: Some(entry_deinit),
    get_factory: Some(entry_get_factory),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_round_trips_and_rejects_garbage() {
        let a = TestPlugin {
            clap: clap_plugin {
                desc: &DESCRIPTOR,
                plugin_data: std::ptr::null_mut(),
                init: None,
                destroy: None,
                activate: None,
                deactivate: None,
                start_processing: None,
                stop_processing: None,
                reset: None,
                process: None,
                get_extension: None,
                on_main_thread: None,
            },
            host: std::ptr::null(),
            values: std::array::from_fn(|i| AtomicU64::new(PARAMS[i].default.to_bits())),
            latency: AtomicU32::new(0),
            active: AtomicBool::new(false),
            names_changed: AtomicBool::new(false),
            gui_gain: AtomicU64::new(NONE),
            ask_flush: AtomicBool::new(false),
            clear_flags: AtomicU32::new(0),
            preset_word: AtomicU32::new(0),
            sample: AtomicU64::new(1.0f64.to_bits()),
            quiet_sample: AtomicU64::new(NONE),
            quiet_in: std::sync::atomic::AtomicI64::new(0),
            audio: UnsafeCell::new(Audio {
                ring: Vec::new(),
                pos: 0,
                mods: [0.0; N],
            }),
            gui: std::sync::Mutex::new(gui::Gui::default()),
        };
        a.set(P_GAIN, 0.5);
        a.set(P_DELAY, 100.4);
        a.set(P_CRASH, 1.0);
        let bytes = a.state_bytes();
        assert_eq!(bytes.len(), 8 + 4 * 12, "debug parameters are not saved");
        a.set(P_GAIN, 1.5);
        assert!(a.load_state_bytes(&bytes));
        assert_eq!(a.value(P_GAIN), 0.5);
        assert_eq!(a.value(P_DELAY), 100.0, "stepped parameters round");
        assert!(!a.load_state_bytes(b"nonsense"));
        let mut bad = bytes.clone();
        bad[8..12].copy_from_slice(&P_CRASH.to_le_bytes());
        assert!(
            !a.load_state_bytes(&bad),
            "debug parameters cannot be loaded"
        );
        // Its sample (D234): in its state when not 1, and back to 1 with a state without.
        a.set_sample(0.25);
        let with = a.state_bytes();
        assert_eq!(with.len(), bytes.len() + 12);
        assert!(a.load_state_bytes(&bytes));
        assert_eq!(a.sample(), 1.0);
        assert!(a.load_state_bytes(&with));
        assert_eq!(a.sample(), 0.25);
    }
}
