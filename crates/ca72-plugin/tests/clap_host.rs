//! MIDI Learn as a CLAP host meets it, through the plug-in's own CLAP entry point in this
//! process (decisions.md R34; `third_party/nih-plug/PATCHES.md`, change 10): a learned
//! controller's MIDI message sets its parameter at its time, and the plug-in tells the host as a
//! parameter value flagged not to be recorded (the MIDI is what a host records), asking for no
//! rescan of the parameters; at one sample
//! the host's automation is set before the controller, whatever the host's order; the
//! assignments go out with the plug-in's state and come back with it, and a state from before
//! them leaves an instance none. No editor is ever opened: all of it plays with the editor
//! closed.

#![allow(unsafe_code, clippy::unwrap_used)]

// (Linked for its `clap_entry`, which nothing else here names.)
extern crate ca72_plugin;

use std::ffi::{CStr, c_char, c_void};
use std::ptr::{null, null_mut};

use clap_sys::audio_buffer::clap_audio_buffer;
use clap_sys::entry::clap_plugin_entry;
use clap_sys::events::{
    CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_DONT_RECORD, CLAP_EVENT_IS_LIVE, CLAP_EVENT_MIDI,
    CLAP_EVENT_PARAM_VALUE, clap_event_header, clap_event_midi, clap_event_param_value,
    clap_input_events, clap_output_events,
};
use clap_sys::ext::params::{
    CLAP_EXT_PARAMS, clap_host_params, clap_param_clear_flags, clap_param_rescan_flags,
    clap_plugin_params,
};
use clap_sys::ext::state::{CLAP_EXT_STATE, clap_plugin_state};
use clap_sys::factory::plugin_factory::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory};
use clap_sys::host::clap_host;
use clap_sys::id::clap_id;
use clap_sys::plugin::clap_plugin;
use clap_sys::process::clap_process;
use clap_sys::stream::{clap_istream, clap_ostream};
use clap_sys::version::CLAP_VERSION;
use std::sync::atomic::{AtomicUsize, Ordering};

unsafe extern "C" {
    /// The plug-in's CLAP entry point (`nih_export_clap!`), linked from this crate.
    static clap_entry: clap_plugin_entry;
}

const RATE: f64 = 48_000.0;
const BLOCK: u32 = 256;

/// A parameter's CLAP id: nih-plug's hash of its string id (`wrapper::util::hash_param_id`).
fn id(param: &str) -> clap_id {
    let mut h: u32 = 0;
    for b in param.bytes() {
        h = h.wrapping_mul(31).wrapping_add(u32::from(b));
    }
    h & !(1 << 31)
}

/// The host's count of rescans asked of it, of any kind (`host_data`).
unsafe extern "C" fn rescan(host: *const clap_host, _: clap_param_rescan_flags) {
    // SAFETY: `host_data` is the instance's counter, alive as long as its host.
    unsafe { (*(*host).host_data.cast::<AtomicUsize>()).fetch_add(1, Ordering::SeqCst) };
}
unsafe extern "C" fn clear(_: *const clap_host, _: clap_id, _: clap_param_clear_flags) {}
unsafe extern "C" fn request(_: *const clap_host) {}

static HOST_PARAMS: clap_host_params = clap_host_params {
    rescan: Some(rescan),
    clear: Some(clear),
    request_flush: Some(request),
};

unsafe extern "C" fn host_extension(_: *const clap_host, ext: *const c_char) -> *const c_void {
    // SAFETY: the plug-in passes an extension's name.
    if unsafe { CStr::from_ptr(ext) } == CLAP_EXT_PARAMS {
        (&raw const HOST_PARAMS).cast()
    } else {
        null()
    }
}

/// An event for the plug-in.
#[derive(Clone, Copy, Debug)]
enum In {
    /// A control change: time, channel (0 to 15), controller, value (0 to 127).
    Cc(u32, u8, u8, u8),
    /// The host's automation: time, parameter, value (CLAP's: normalized for a knob, the
    /// position for a selector).
    Param(u32, &'static str, f64),
}

/// A parameter value the plug-in told the host: time, flags, parameter, value.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Out {
    time: u32,
    flags: u32,
    param: clap_id,
    value: f64,
}

/// The events of one process call, in the host's order (each boxed: `headers` points into it).
#[allow(clippy::vec_box)]
struct Events {
    headers: Vec<*const clap_event_header>,
    _midi: Vec<Box<clap_event_midi>>,
    _params: Vec<Box<clap_event_param_value>>,
}

unsafe extern "C" fn in_size(list: *const clap_input_events) -> u32 {
    // SAFETY: `ctx` is the call's `Events`.
    unsafe {
        let events = &*(*list).ctx.cast::<Events>();
        events.headers.len() as u32
    }
}

unsafe extern "C" fn in_get(list: *const clap_input_events, i: u32) -> *const clap_event_header {
    // SAFETY: as above.
    unsafe {
        let events = &*(*list).ctx.cast::<Events>();
        events.headers.get(i as usize).copied().unwrap_or(null())
    }
}

unsafe extern "C" fn out_push(
    list: *const clap_output_events,
    e: *const clap_event_header,
) -> bool {
    // SAFETY: `ctx` is the call's `Vec<Out>`; the plug-in's event is valid for the call.
    unsafe {
        let out = &mut *(*list).ctx.cast::<Vec<Out>>();
        if (*e).space_id == CLAP_CORE_EVENT_SPACE_ID && (*e).type_ == CLAP_EVENT_PARAM_VALUE {
            let p = &*e.cast::<clap_event_param_value>();
            out.push(Out {
                time: p.header.time,
                flags: p.header.flags,
                param: p.param_id,
                value: p.value,
            });
        }
    }
    true
}

unsafe extern "C" fn write(stream: *const clap_ostream, buffer: *const c_void, size: u64) -> i64 {
    // SAFETY: `ctx` is a `Vec<u8>`; the plug-in's buffer holds `size` bytes.
    unsafe {
        let v = &mut *(*stream).ctx.cast::<Vec<u8>>();
        v.extend_from_slice(std::slice::from_raw_parts(
            buffer.cast::<u8>(),
            size as usize,
        ));
    }
    size as i64
}

unsafe extern "C" fn read(stream: *const clap_istream, buffer: *mut c_void, size: u64) -> i64 {
    // SAFETY: `ctx` is a `(Vec<u8>, usize)` (the bytes and where reading is); the plug-in's
    // buffer has room for `size`.
    unsafe {
        let (v, at) = &mut *(*stream).ctx.cast::<(Vec<u8>, usize)>();
        let n = (v.len() - *at).min(size as usize);
        std::ptr::copy_nonoverlapping(v[*at..].as_ptr(), buffer.cast::<u8>(), n);
        *at += n;
        n as i64
    }
}

/// An instance of the plug-in, activated and processing, as a host has it.
struct Instance {
    plugin: *const clap_plugin,
    _host: Box<clap_host>,
    /// How often the plug-in asked its host to rescan its parameters.
    rescans: Box<AtomicUsize>,
}

impl Instance {
    fn new() -> Instance {
        let rescans = Box::new(AtomicUsize::new(0));
        let host = Box::new(clap_host {
            clap_version: CLAP_VERSION,
            host_data: (&raw const *rescans).cast_mut().cast(),
            name: c"ca72 tests".as_ptr(),
            vendor: c"".as_ptr(),
            url: c"".as_ptr(),
            version: c"0".as_ptr(),
            get_extension: Some(host_extension),
            request_restart: Some(request),
            request_process: Some(request),
            request_callback: Some(request),
        });
        // SAFETY: the plug-in's own entry point and factory, used as CLAP has them.
        unsafe {
            let entry = &clap_entry;
            assert!((entry.init.unwrap())(c"".as_ptr()));
            let factory = (entry.get_factory.unwrap())(CLAP_PLUGIN_FACTORY_ID.as_ptr())
                .cast::<clap_plugin_factory>();
            let plugin = ((*factory).create_plugin.unwrap())(
                factory,
                &*host,
                c"com.idlefoundry.ca-72".as_ptr(),
            );
            assert!(!plugin.is_null());
            assert!(((*plugin).init.unwrap())(plugin));
            assert!(((*plugin).activate.unwrap())(plugin, RATE, 1, BLOCK));
            assert!(((*plugin).start_processing.unwrap())(plugin));
            Instance {
                plugin,
                _host: host,
                rescans,
            }
        }
    }

    fn extension<T>(&self, name: &CStr) -> &T {
        // SAFETY: the plug-in's extension of that name is a `T`.
        unsafe {
            let e = ((*self.plugin).get_extension.unwrap())(self.plugin, name.as_ptr());
            assert!(!e.is_null(), "{name:?}");
            &*e.cast::<T>()
        }
    }

    /// One block with `events` in this order; what the plug-in told the host of parameters.
    fn process(&self, events: &[In]) -> Vec<Out> {
        let mut ev = Events {
            headers: Vec::new(),
            _midi: Vec::new(),
            _params: Vec::new(),
        };
        for &e in events {
            match e {
                In::Cc(time, channel, cc, value) => {
                    let m = Box::new(clap_event_midi {
                        header: clap_event_header {
                            size: size_of::<clap_event_midi>() as u32,
                            time,
                            space_id: CLAP_CORE_EVENT_SPACE_ID,
                            type_: CLAP_EVENT_MIDI,
                            flags: CLAP_EVENT_IS_LIVE,
                        },
                        port_index: 0,
                        data: [0xb0 | channel, cc, value],
                    });
                    ev.headers.push(&raw const m.header);
                    ev._midi.push(m);
                }
                In::Param(time, param, value) => {
                    let p = Box::new(clap_event_param_value {
                        header: clap_event_header {
                            size: size_of::<clap_event_param_value>() as u32,
                            time,
                            space_id: CLAP_CORE_EVENT_SPACE_ID,
                            type_: CLAP_EVENT_PARAM_VALUE,
                            flags: 0,
                        },
                        param_id: id(param),
                        cookie: null_mut(),
                        note_id: -1,
                        port_index: -1,
                        channel: -1,
                        key: -1,
                        value,
                    });
                    ev.headers.push(&raw const p.header);
                    ev._params.push(p);
                }
            }
        }
        let n = BLOCK as usize;
        let (mut out_l, mut out_r, mut in_l, mut in_r) = (
            vec![0.0f32; n],
            vec![0.0f32; n],
            vec![0.0f32; n],
            vec![0.0f32; n],
        );
        let mut outs = [out_l.as_mut_ptr(), out_r.as_mut_ptr()];
        let mut ins = [in_l.as_mut_ptr(), in_r.as_mut_ptr()];
        let mut output = clap_audio_buffer {
            data32: outs.as_mut_ptr(),
            data64: null_mut(),
            channel_count: 2,
            latency: 0,
            constant_mask: 0,
        };
        let input = clap_audio_buffer {
            data32: ins.as_mut_ptr(),
            data64: null_mut(),
            channel_count: 2,
            latency: 0,
            constant_mask: 0,
        };
        let in_events = clap_input_events {
            ctx: (&raw mut ev).cast(),
            size: Some(in_size),
            get: Some(in_get),
        };
        let mut told: Vec<Out> = Vec::new();
        let out_events = clap_output_events {
            ctx: (&raw mut told).cast(),
            try_push: Some(out_push),
        };
        let process = clap_process {
            steady_time: -1,
            frames_count: BLOCK,
            transport: null(),
            audio_inputs: &input,
            audio_outputs: &mut output,
            audio_inputs_count: 1,
            audio_outputs_count: 1,
            in_events: &in_events,
            out_events: &out_events,
        };
        // SAFETY: a process call as CLAP has it, everything it points to alive through it.
        unsafe {
            ((*self.plugin).process.unwrap())(self.plugin, &process);
        }
        told
    }

    /// A parameter's value as the host reads it (CLAP's: normalized for a knob, the position
    /// for a selector).
    fn value(&self, param: &str) -> f64 {
        let params: &clap_plugin_params = self.extension(CLAP_EXT_PARAMS);
        let mut v = f64::NAN;
        // SAFETY: as CLAP has it.
        assert!(unsafe { (params.get_value.unwrap())(self.plugin, id(param), &mut v) });
        v
    }

    /// The plug-in's state as the host saves it.
    fn save(&self) -> Vec<u8> {
        let state: &clap_plugin_state = self.extension(CLAP_EXT_STATE);
        let mut bytes: Vec<u8> = Vec::new();
        let stream = clap_ostream {
            ctx: (&raw mut bytes).cast(),
            write: Some(write),
        };
        // SAFETY: as CLAP has it.
        assert!(unsafe { (state.save.unwrap())(self.plugin, &stream) });
        bytes
    }

    /// A state loaded as a host loads one: whether the plug-in took it.
    fn load(&self, bytes: &[u8]) -> bool {
        let state: &clap_plugin_state = self.extension(CLAP_EXT_STATE);
        let mut source = (bytes.to_vec(), 0usize);
        let stream = clap_istream {
            ctx: (&raw mut source).cast(),
            read: Some(read),
        };
        // SAFETY: as CLAP has it.
        unsafe { (state.load.unwrap())(self.plugin, &stream) }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        // SAFETY: as CLAP has it, once.
        unsafe {
            ((*self.plugin).stop_processing.unwrap())(self.plugin);
            ((*self.plugin).deactivate.unwrap())(self.plugin);
            ((*self.plugin).destroy.unwrap())(self.plugin);
        }
    }
}

/// nih-plug's CLAP state: its length (8 bytes, little-endian), then its JSON.
fn json_of(bytes: &[u8]) -> serde_json::Value {
    let n = u64::from_le_bytes(bytes[..8].try_into().unwrap()) as usize;
    serde_json::from_slice(&bytes[8..8 + n]).unwrap()
}

fn bytes_of(v: &serde_json::Value) -> Vec<u8> {
    let json = serde_json::to_vec(v).unwrap();
    let mut out = (json.len() as u64).to_le_bytes().to_vec();
    out.extend(json);
    out
}

/// `instance`'s state with its MIDI assignments' field set to `table` (removed: None).
fn with_table(instance: &Instance, table: Option<&str>) -> Vec<u8> {
    let mut state = json_of(&instance.save());
    let fields = state["fields"].as_object_mut().unwrap();
    match table {
        Some(t) => {
            fields.insert("midi_map".into(), serde_json::Value::String(t.into()));
        }
        None => {
            fields.remove("midi_map");
        }
    }
    bytes_of(&state)
}

/// OSCILLATOR-1 RANGE on CC 21 and CUTOFF on CC 74, channel 1; EMPHASIS on CC 71, channel 2 (in
/// the order the plug-in writes them: the controls').
const TABLE: &str = r#"{"version":1,"assignments":[{"param":"osc1_range","channel":1,"cc":21},{"param":"cutoff","channel":1,"cc":74},{"param":"emphasis","channel":2,"cc":71}]}"#;

/// A learned controller sets its parameter at its message's time, and the host is told: a
/// parameter value at that time, flagged not to be recorded (and not as a live gesture of the
/// user's), the value as set; a controller on another channel, the modulation wheel and one not
/// assigned tell nothing.
#[test]
fn a_learned_controller_is_set_at_its_time_and_told_not_to_be_recorded() {
    let p = Instance::new();
    assert!(p.load(&with_table(&p, Some(TABLE))));
    let before = p.rescans.load(Ordering::SeqCst);
    let told = p.process(&[
        In::Cc(10, 1, 74, 127),
        In::Cc(20, 0, 1, 127),
        In::Cc(30, 0, 75, 127),
        In::Cc(100, 0, 74, 127),
        In::Cc(150, 0, 21, 127),
    ]);
    assert_eq!(
        told,
        [
            Out {
                time: 100,
                flags: CLAP_EVENT_DONT_RECORD,
                param: id("cutoff"),
                value: 1.0
            },
            Out {
                time: 150,
                flags: CLAP_EVENT_DONT_RECORD,
                param: id("osc1_range"),
                value: 5.0
            },
        ]
    );
    assert_eq!(p.value("cutoff"), 1.0);
    assert_eq!(p.value("osc1_range"), 5.0);
    // And no rescan of the parameters is asked for (this host's process thread is its main
    // thread, so one would have come at once): the Audio Unit's wrapper rebuilds its whole
    // parameter list for one.
    assert_eq!(p.rescans.load(Ordering::SeqCst), before);
    // The same value again: nothing changes, nothing is told.
    assert!(p.process(&[In::Cc(5, 0, 74, 127)]).is_empty());
    // The host's own automation is not echoed back.
    assert!(p.process(&[In::Param(5, "cutoff", 0.25)]).is_empty());
    assert_eq!(p.value("cutoff"), 0.25);
}

/// At one sample, the host's automation of a parameter is set before a learned controller of
/// it, whichever the host lists first; later in time, whichever comes later holds.
#[test]
fn at_one_sample_automation_is_set_before_a_learned_controller() {
    let p = Instance::new();
    assert!(p.load(&with_table(&p, Some(TABLE))));
    p.process(&[In::Cc(64, 1, 71, 127), In::Param(64, "emphasis", 0.3)]);
    assert_eq!(p.value("emphasis"), 1.0, "the controller, listed first");
    p.process(&[In::Param(64, "emphasis", 0.3), In::Cc(64, 1, 71, 0)]);
    assert_eq!(p.value("emphasis"), 0.0, "the controller, listed second");
    p.process(&[In::Cc(64, 1, 71, 127), In::Param(96, "emphasis", 0.3)]);
    assert!(
        (p.value("emphasis") - 0.3).abs() < 1e-6,
        "the automation, later"
    );
    p.process(&[In::Param(32, "emphasis", 0.3), In::Cc(96, 1, 71, 0)]);
    assert_eq!(p.value("emphasis"), 0.0, "the controller, later");
}

/// The assignments go out with the state the host saves and come back with it (into another
/// instance); a state from before MIDI Learn leaves an instance with none; a table not
/// understood is none, and the state still loads.
#[test]
fn the_assignments_go_out_and_come_back_with_the_state() {
    let a = Instance::new();
    assert!(a.load(&with_table(&a, Some(TABLE))));
    let saved = a.save();
    let table = json_of(&saved)["fields"]["midi_map"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(table, TABLE);
    let b = Instance::new();
    assert!(
        b.process(&[In::Cc(0, 0, 74, 127)]).is_empty(),
        "a new instance: none"
    );
    assert!(b.load(&saved));
    assert_eq!(b.process(&[In::Cc(0, 0, 74, 127)]).len(), 1);
    // Older, with no table: none, though B had them.
    assert!(b.load(&with_table(&a, None)));
    assert!(b.process(&[In::Cc(0, 0, 74, 0)]).is_empty());
    assert_eq!(
        b.value("cutoff"),
        0.5,
        "the sound as the state has it (A's)"
    );
    assert!(b.load(&saved));
    assert!(b.load(&with_table(&a, Some("{\"version\":9,"))));
    assert!(
        b.process(&[In::Cc(0, 0, 74, 0), In::Cc(0, 1, 71, 0)])
            .is_empty()
    );
}

/// Two instances, one with assignments: the other's controllers do nothing.
#[test]
fn instances_keep_their_own_assignments() {
    let a = Instance::new();
    let b = Instance::new();
    assert!(a.load(&with_table(&a, Some(TABLE))));
    assert!(b.process(&[In::Cc(0, 0, 74, 127)]).is_empty());
    assert_eq!(b.value("cutoff"), 0.5);
    assert_eq!(a.process(&[In::Cc(0, 0, 74, 127)]).len(), 1);
}
