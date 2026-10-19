//! The shipped VST3 test plugin (M6 plan W1): the CLAP test plugin's effect as a VST3
//! plugin, so the sandbox's tests hold VST3 hosting to the same expectations.
//!
//! `y[n] = gain * (x[n - latency] + mix * x[n - latency - delay])`
//!
//! - It is split in two classes, as most VST3 plugins are: a processor (`IComponent`,
//!   `IAudioProcessor`) and a controller (`IEditController`), connected by the host.
//! - Parameters have the CLAP plugin's IDs, ranges and defaults, normalized as VST3 wants
//!   (`plain = min + value * (max - min)`, stepped ones rounded).
//! - `latency` is reported (`getLatencySamples`) and applied when the processor is made
//!   active; the delay line is audio state a crash loses.
//! - The processor's state is the CLAP plugin's format: a magic, then (ID, value) pairs;
//!   the controller reads it back (`setComponentState`).
//! - The hidden `crash` and `hang` parameters abort the process or stop it responding when
//!   they change during processing, and are never saved.
//! - An editor (`createView("editor")`): a view of one colour embedded in the host's window
//!   (`gui`).
//!
//! A second class, the instrument `DAW-v3 Test Sine`, takes notes instead of audio: each
//! note is a sine at its key's equal-tempered pitch (A4 = 440 Hz) and its velocity, from
//! its note-on to its note-off, the sum going through the same gain and delay.
//!
//! The instrument's controller (a class of its own) also describes units and programs
//! (`IUnitInfo`, D211): the unit "Drums", which plays the note input (`getUnitByBus`), has
//! the program list "Kits", whose programs name keys (program pitch names): Kit A 36 "VST
//! Kick", 38 "VST Snare" and 42 "VST Hat"; Kit B 36 "VST Deep Kick", 39 "VST Clap" and 46
//! "VST Open Hat"; Kit C 35 "VST Low Kick", 40 "VST Rim" and 49 "VST Crash". Its
//! program-change parameter `Kit` (not automatable) chooses the program. The root unit and
//! a unit "Pads" each have a list of their own, listed before and after the kits, whose
//! programs name 37 "Not Ours", and an automatable program-change parameter, listed before
//! and after `Kit` and set to their second program: what a host must not take for the note
//! input's. Programs change only names; the sound is the same. Two hidden parameters stand
//! in for its editor (D216): `Edit kit` chooses a kit as the editor would (`performEdit`),
//! and `Rename` renames the keys with or without a word to the host (`IUnitHandler`,
//! `restartComponent`).
//!
//! The instrument has more than eight hundred parameters (D222): after its effect's six, 799
//! hidden spares and `Level` (its output's level, twice its value), then its program
//! changes, past the 800th (more than the host's processor has queues for in a block, 768).
//! More stand-ins for its editor, hidden, edit its gain as an
//! editor does: in a gesture of two values (`Edit gain`), with a value and no gesture
//! (`Nudge gain`), or in a gesture held open (`Hold gain`) until `Release gain`. At 0 the
//! gain's stand-ins do nothing. `Load preset` (D228) stands in for a preset loaded in its
//! editor: the controller sets the gain and `Level` itself and says only that its values
//! changed (`restartComponent(kParamValuesChanged)`); while `Program values` is set, a kit
//! the host gives it carries a `Level` of its own, said the same way.
//!
//! Its state beyond its parameters (D234): a "sample", which its processor plays at a level
//! of its own (1 unless one is loaded: its output times it), kept in the processor's state
//! (only when not 1, so that states without one stay as they were). Stand-ins load one: in
//! a preset that also sets the gain and `Level` (`Load sample preset`, said with
//! `kParamValuesChanged`), alone (`Load sample`, said with `IComponentHandler2::setDirty`),
//! or without a word (`Load sample quietly`); `Change quietly` sets the gain and `Level`
//! without a word. Its processor reports values itself (`outputParameterChanges`): its
//! output's peak on the read-only `Meter`, every block, and its gain when `Processor gain`
//! sets it.
//!
//! Output is plain arithmetic without feedback: bit-exact for the same input and events,
//! whatever the block size.
//!
//! Unsafe code: this crate implements the VST3 ABI; every entry point dereferences
//! pointers the host passes. See `docs/unsafe.md`.

#![allow(unsafe_code)]
#![allow(non_snake_case)]

mod gui;

use std::ffi::{c_char, c_void};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use vst3::Steinberg::Vst::*;
use vst3::Steinberg::*;
use vst3::{Class, ComRef, ComWrapper, uid};

/// Stable parameter IDs, as the CLAP test plugin's.
pub const P_GAIN: u32 = 1;
pub const P_MIX: u32 = 2;
pub const P_DELAY: u32 = 3;
pub const P_LATENCY: u32 = 4;
pub const P_CRASH: u32 = 100;
pub const P_HANG: u32 = 101;
/// The instrument's program-change parameters (D211): the drum unit's (its kit), which
/// plays the notes, and the root's and the pad unit's.
pub const P_KIT: u32 = 200;
pub const P_LAYER: u32 = 201;
pub const P_PAD: u32 = 202;
/// Stand-ins for the instrument's editor (D216), hidden: `Edit kit` set to a value makes
/// the controller choose that kit itself and tell the host as an editor does
/// (`beginEdit`, `performEdit`, `endEdit` of `Kit`); `Rename` set to step 1 to 6 renames
/// every key (a suffix " 1" to " 6") and does: 1 `notifyProgramListChange`, 2
/// `notifyUnitSelection`, 3 `restartComponent(kParamValuesChanged)`, 4
/// `restartComponent(kParamTitlesChanged)`, 5 `notifyUnitByBusChange`, 6
/// `restartComponent(kLatencyChanged)` (a restart, not a word about names); step 7 renames
/// them (" 7") without a word, and 0 takes the suffix away, also without one.
pub const P_EDIT: u32 = 203;
pub const P_SIGNAL: u32 = 204;
/// More stand-ins for its editor (D222), hidden: `Edit gain` set to a value makes the
/// controller set its gain to it in one gesture of two values (half of it, then it);
/// `Nudge gain` sets it with the same two values and no gesture. The host takes them to the
/// processor and the project. At 0 (their rest, as when taken away) they do nothing.
pub const P_EDIT_GAIN: u32 = 205;
pub const P_NUDGE: u32 = 206;
/// A gesture held open (D222): `Hold gain` begins one on the gain and sets it, and
/// `Release gain` set to anything but 0 ends it.
pub const P_HOLD_GAIN: u32 = 207;
pub const P_RELEASE: u32 = 208;
/// While set (not 0), the controller answers every value the host gives its gain with
/// `performEdit` of it, as some controllers do (D222): not an edit for the project.
pub const P_ANSWER: u32 = 209;
/// A preset loaded in its editor (D228), hidden: `Load preset` set to a value makes the
/// controller set its gain and `Level` to it itself and say so with
/// `restartComponent(kParamValuesChanged)` alone (no edits); its processor is not told, so
/// the new values reach it only if the host reads them and sends them. At 0 (its rest) it
/// does nothing.
pub const P_PRESET: u32 = 210;
/// While set (not 0, D228), a kit the host gives `Kit` carries a value: the controller sets
/// `Level` to a quarter itself and says `restartComponent(kParamValuesChanged)`, as a
/// controller whose programs carry values does after a host's program change.
pub const P_PROGRAM_VALUES: u32 = 211;
/// Its state beyond its parameters (D234), hidden stand-ins at rest at 0 that act when set
/// to another value (as buttons do: the same value given again, as after a state the host
/// gives it, does nothing): `Load sample` set to a value makes its processor load a sample of
/// that level (its output times it)
/// and its controller say `IComponentHandler2::setDirty` alone; `Load sample preset` loads
/// one and sets the gain and `Level` to the value too, said with `kParamValuesChanged`;
/// `Change quietly` sets the gain and `Level` in its controller without a word, and `Load
/// sample quietly` loads a sample without one, a second and a half of audio after its unchanged
/// baseline is saved (apart from the host's value that asked for it: host values change
/// a state too).
pub const P_LOAD_SAMPLE: u32 = 212;
pub const P_SAMPLE_PRESET: u32 = 213;
pub const P_CHANGE_QUIETLY: u32 = 214;
pub const P_SAMPLE_QUIETLY: u32 = 215;
/// Its processor's own values (D234, `outputParameterChanges`): `Processor gain` set to a
/// value makes the processor set its gain to it, and while it is set the processor reports
/// its gain every block (as some processors report their values: the same value again, or
/// the host's); `Meter` (read-only) is its output's peak, reported every block.
pub const P_PROCESSOR_GAIN: u32 = 216;
pub const P_METER: u32 = 217;
/// The instrument's spare parameters (D222), hidden, listed between its effect's and its
/// program changes, so that the program changes come past the 800th: `Spare 1` to `Spare
/// 799`, and last `Level` (shown), its output's level, twice its value (1 at 0.5).
pub const P_SPARE: u32 = 1001;
pub const SPARES: usize = 800;
pub const P_LEVEL: u32 = P_SPARE + SPARES as u32 - 1;

/// Largest latency and delay, in frames.
pub const MAX_LATENCY: u32 = 4096;
pub const MAX_DELAY: u32 = 4800;

/// The plugin's name, as its class lists it.
pub const NAME: &str = "DAW-v3 Test Gain";
/// The processor's class ID, as the host names the plugin (`device.plugin`).
pub const PROCESSOR_CID: TUID = uid(0x44415733, 0x54657374, 0x4761696E, 0x50726F63);
/// The instrument's name and processor class ID (M6 plan W2): sine voices, one per note,
/// through the same gain and delay.
pub const SINE_NAME: &str = "DAW-v3 Test Sine";
pub const SINE_CID: TUID = uid(0x44415733, 0x54657374, 0x53696E65, 0x50726F63);
/// The controller's class ID.
pub const CONTROLLER_CID: TUID = uid(0x44415733, 0x54657374, 0x4761696E, 0x43746C72);
/// The instrument's controller's class ID: the same parameters, and its units and programs.
pub const SINE_CONTROLLER_CID: TUID = uid(0x44415733, 0x54657374, 0x53696E65, 0x43746C72);

const RING: usize = 16384;
const MASK: usize = RING - 1;
const _: () = assert!(RING > (MAX_LATENCY + MAX_DELAY) as usize);

/// State format: this magic, then (parameter ID as u32, value as f64) pairs, little endian.
const STATE_MAGIC: &[u8; 8] = b"DAWTP\0\0\x01";
/// The pair's ID for its sample's level (D234), a state beyond its parameters: present
/// only when not 1.
const SAMPLE_ID: u32 = 0xFFFF_FF01;

struct ParamDef {
    id: u32,
    name: &'static str,
    min: f64,
    max: f64,
    default: f64,
    stepped: bool,
    hidden: bool,
    saved: bool,
}

const PARAMS: [ParamDef; 6] = [
    ParamDef {
        id: P_GAIN,
        name: "Gain",
        min: 0.0,
        max: 2.0,
        default: 1.0,
        stepped: false,
        hidden: false,
        saved: true,
    },
    ParamDef {
        id: P_MIX,
        name: "Delay mix",
        min: 0.0,
        max: 1.0,
        default: 0.0,
        stepped: false,
        hidden: false,
        saved: true,
    },
    ParamDef {
        id: P_DELAY,
        name: "Delay",
        min: 1.0,
        max: MAX_DELAY as f64,
        default: 480.0,
        stepped: true,
        hidden: false,
        saved: true,
    },
    ParamDef {
        id: P_LATENCY,
        name: "Latency",
        min: 0.0,
        max: MAX_LATENCY as f64,
        default: 0.0,
        stepped: true,
        hidden: false,
        saved: true,
    },
    ParamDef {
        id: P_CRASH,
        name: "Crash",
        min: 0.0,
        max: 1.0,
        default: 0.0,
        stepped: true,
        hidden: true,
        saved: false,
    },
    ParamDef {
        id: P_HANG,
        name: "Hang",
        min: 0.0,
        max: 1.0,
        default: 0.0,
        stepped: true,
        hidden: true,
        saved: false,
    },
];

/// The instrument's program lists (D211), in the order it lists them: ID, name, and each
/// program's name and pitch names. The kits are between two lists that are not the note
/// input's.
type Programs = &'static [(&'static str, &'static [(i16, &'static str)])];
const KITS: ProgramListID = 1;
const LAYERS: ProgramListID = 2;
const PADS: ProgramListID = 3;
const NOT_OURS: Programs = &[
    ("Other A", &[(37, "Not Ours")]),
    ("Other B", &[(37, "Not Ours")]),
];
const LISTS: [(ProgramListID, &str, Programs); 3] = [
    (LAYERS, "Layers", NOT_OURS),
    (
        KITS,
        "Kits",
        &[
            (
                "Kit A",
                &[(36, "VST Kick"), (38, "VST Snare"), (42, "VST Hat")],
            ),
            (
                "Kit B",
                &[
                    (36, "VST Deep Kick"),
                    (39, "VST Clap"),
                    (46, "VST Open Hat"),
                ],
            ),
            (
                "Kit C",
                &[(35, "VST Low Kick"), (40, "VST Rim"), (49, "VST Crash")],
            ),
        ],
    ),
    (PADS, "Pads", NOT_OURS),
];

/// The instrument's units: ID, parent, name and program list. The drums, a unit of their
/// own, play the note input (`getUnitByBus`); the root and the pads do not.
const DRUM_UNIT: UnitID = 1;
const PAD_UNIT: UnitID = 2;
const UNITS: [(UnitID, UnitID, &str, ProgramListID); 3] = [
    (kRootUnitId, kNoParentUnitId, "Root", LAYERS),
    (DRUM_UNIT, kRootUnitId, "Drums", KITS),
    (PAD_UNIT, kRootUnitId, "Pads", PADS),
];

/// The instrument's program-change parameters: ID, title, unit, list, default (normalized)
/// and whether it is automatable. The kit's is between the others, which are automatable
/// and on their second program. It is not automatable, as many plugins' are (the SDK
/// forbids automating a program change that sets automatable parameters), so not shown.
const PROGRAM_PARAMS: [(u32, &str, UnitID, ProgramListID, f64, bool); 3] = [
    (P_LAYER, "Layer", kRootUnitId, LAYERS, 1.0, true),
    (P_KIT, "Kit", DRUM_UNIT, KITS, 0.0, false),
    (P_PAD, "Pad", PAD_UNIT, PADS, 1.0, true),
];

/// The instrument's editor stand-ins (D216): ID, title and steps.
const EDIT_PARAMS: [(u32, &str, i32); 15] = [
    (P_EDIT, "Edit kit", 2),
    (P_SIGNAL, "Rename", 7),
    (P_EDIT_GAIN, "Edit gain", 0),
    (P_NUDGE, "Nudge gain", 0),
    (P_HOLD_GAIN, "Hold gain", 0),
    (P_RELEASE, "Release gain", 0),
    (P_ANSWER, "Answer gain", 0),
    (P_PRESET, "Load preset", 0),
    (P_PROGRAM_VALUES, "Program values", 0),
    (P_LOAD_SAMPLE, "Load sample", 0),
    (P_SAMPLE_PRESET, "Load sample preset", 0),
    (P_CHANGE_QUIETLY, "Change quietly", 0),
    (P_SAMPLE_QUIETLY, "Load sample quietly", 0),
    (P_PROCESSOR_GAIN, "Processor gain", 0),
    (P_METER, "Meter", 0),
];

fn programs(list: ProgramListID) -> Option<Programs> {
    LISTS.iter().find(|l| l.0 == list).map(|l| l.2)
}

/// A program parameter's program for a normalized value (VST3's discrete value).
fn program_index(list: ProgramListID, normalized: f64) -> usize {
    let steps = programs(list).map_or(0, |p| p.len().saturating_sub(1));
    ((normalized.clamp(0.0, 1.0) * (steps + 1) as f64) as usize).min(steps)
}

fn def(id: u32) -> Option<&'static ParamDef> {
    PARAMS.iter().find(|p| p.id == id)
}

fn to_plain(p: &ParamDef, normalized: f64) -> f64 {
    let v = p.min + normalized.clamp(0.0, 1.0) * (p.max - p.min);
    if p.stepped { v.round() } else { v }
}

fn to_normalized(p: &ParamDef, plain: f64) -> f64 {
    if p.max > p.min {
        ((plain - p.min) / (p.max - p.min)).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn copy_cstring(src: &str, dst: &mut [c_char]) {
    let n = src.len().min(dst.len().saturating_sub(1));
    for (d, s) in dst.iter_mut().zip(src.bytes().take(n)) {
        *d = s as c_char;
    }
    if let Some(end) = dst.get_mut(n) {
        *end = 0;
    }
}

fn copy_wstring(src: &str, dst: &mut [TChar]) {
    let mut n = 0;
    for (d, s) in dst.iter_mut().zip(src.encode_utf16()) {
        *d = s as TChar;
        n += 1;
    }
    let end = n.min(dst.len().saturating_sub(1));
    if let Some(e) = dst.get_mut(end) {
        *e = 0;
    }
}

/// Plain values of every parameter, by index in `PARAMS`.
fn defaults() -> [f64; 6] {
    let mut v = [0.0; 6];
    for (i, p) in PARAMS.iter().enumerate() {
        v[i] = p.default;
    }
    v
}

fn save(values: &[f64; 6]) -> Vec<u8> {
    let mut out = STATE_MAGIC.to_vec();
    for (i, p) in PARAMS.iter().enumerate() {
        if p.saved {
            out.extend_from_slice(&p.id.to_le_bytes());
            out.extend_from_slice(&values[i].to_le_bytes());
        }
    }
    out
}

/// The processor's state: its values, and its sample's level when not 1 (D234).
fn save_with_sample(values: &[f64; 6], sample: f64) -> Vec<u8> {
    let mut out = save(values);
    if sample != 1.0 {
        out.extend_from_slice(&SAMPLE_ID.to_le_bytes());
        out.extend_from_slice(&sample.to_le_bytes());
    }
    out
}

/// The sample's level a state holds (1 when none; D234).
fn sample_of(bytes: &[u8]) -> f64 {
    let Some(rest) = bytes.strip_prefix(STATE_MAGIC.as_slice()) else {
        return 1.0;
    };
    rest.chunks_exact(12)
        .find(|c| c[..4] == SAMPLE_ID.to_le_bytes())
        .map(|c| {
            let mut v = [0u8; 8];
            v.copy_from_slice(&c[4..12]);
            f64::from_le_bytes(v)
        })
        .filter(|v| v.is_finite())
        .map_or(1.0, |v| v.clamp(0.0, 1.0))
}

fn load(bytes: &[u8]) -> Option<Vec<(usize, f64)>> {
    let rest = bytes.strip_prefix(STATE_MAGIC.as_slice())?;
    if rest.len() % 12 != 0 {
        return None;
    }
    let mut out = Vec::new();
    for c in rest.chunks_exact(12) {
        let id = u32::from_le_bytes([c[0], c[1], c[2], c[3]]);
        let mut v = [0u8; 8];
        v.copy_from_slice(&c[4..12]);
        if id == SAMPLE_ID {
            // Not a parameter: its sample (`sample_of`).
            continue;
        }
        let i = PARAMS.iter().position(|p| p.id == id && p.saved)?;
        let p = &PARAMS[i];
        let value = f64::from_le_bytes(v);
        if !value.is_finite() {
            return None;
        }
        out.push((i, value.clamp(p.min, p.max)));
    }
    Some(out)
}

/// Reads a whole stream.
unsafe fn read_all(stream: *mut IBStream) -> Option<Vec<u8>> {
    // SAFETY: the host passes a live stream or null; `from_raw` returns `None` for null.
    let s = unsafe { ComRef::from_raw(stream) }?;
    let mut out = Vec::new();
    let mut buf = [0u8; 1024];
    loop {
        let mut got = 0;
        // SAFETY: `buf` is writable for the length passed; `s` is the host's live stream.
        let r = unsafe { s.read(buf.as_mut_ptr().cast(), buf.len() as i32, &mut got) };
        if r != kResultOk || got <= 0 {
            break;
        }
        out.extend_from_slice(&buf[..got as usize]);
        if out.len() > 1 << 20 {
            return None;
        }
    }
    Some(out)
}

unsafe fn write_all(stream: *mut IBStream, bytes: &[u8]) -> tresult {
    // SAFETY: the host passes a live stream or null; `from_raw` returns `None` for null.
    let Some(s) = (unsafe { ComRef::from_raw(stream) }) else {
        return kInvalidArgument;
    };
    let mut written = 0;
    // SAFETY: `bytes` is readable for the length passed; `s` is the host's live stream.
    let r = unsafe {
        s.write(
            bytes.as_ptr() as *mut c_void,
            bytes.len() as i32,
            &mut written,
        )
    };
    if r == kResultOk && written as usize == bytes.len() {
        kResultOk
    } else {
        kResultFalse
    }
}

// ---------------------------------------------------------------------------------------
// The processor

struct Dsp {
    ring: Vec<[f32; 2]>,
    pos: usize,
    /// Latency and delay as applied (latency when made active).
    latency: usize,
}

/// A sounding note: key, note ID, velocity, phase and phase step (radians).
type Voice = (i16, i32, f64, f64, f64);

struct Processor {
    /// Plain values, as f64 bits.
    values: [AtomicU64; 6],
    /// Its sample's level (D234), as f64 bits: its state beyond its parameters.
    sample: AtomicU64,
    /// Whether it reports its gain every block (`Processor gain` set).
    report_gain: AtomicBool,
    /// A sample to load quietly (`u64::MAX`: none), and the frames of audio until then.
    /// A negative count waits for a state save to capture the unchanged baseline.
    quiet_sample: AtomicU64,
    quiet_in: std::sync::atomic::AtomicI64,
    /// The values its D234 stand-ins were last given (as `D_PS_STAND_INS`), as f64 bits:
    /// they act on a new value only.
    stand_ins: [AtomicU64; 4],
    /// The instrument's `Level`, normalized, as f64 bits (D222).
    level: AtomicU64,
    dsp: Mutex<Dsp>,
    active: AtomicBool,
    /// The instrument: notes in, no audio in.
    instrument: bool,
    sample_rate: AtomicU64,
    voices: Mutex<Vec<Voice>>,
}

impl Class for Processor {
    type Interfaces = (
        IComponent,
        IAudioProcessor,
        IProcessContextRequirements,
        IConnectionPoint,
    );
}

impl Processor {
    fn new(instrument: bool) -> Processor {
        let d = defaults();
        Processor {
            instrument,
            sample_rate: AtomicU64::new(48_000f64.to_bits()),
            voices: Mutex::new(Vec::with_capacity(64)),
            level: AtomicU64::new(0.5f64.to_bits()),
            sample: AtomicU64::new(1.0f64.to_bits()),
            report_gain: AtomicBool::new(false),
            quiet_sample: AtomicU64::new(u64::MAX),
            quiet_in: std::sync::atomic::AtomicI64::new(0),
            stand_ins: std::array::from_fn(|_| AtomicU64::new(0f64.to_bits())),
            values: [
                AtomicU64::new(d[0].to_bits()),
                AtomicU64::new(d[1].to_bits()),
                AtomicU64::new(d[2].to_bits()),
                AtomicU64::new(d[3].to_bits()),
                AtomicU64::new(d[4].to_bits()),
                AtomicU64::new(d[5].to_bits()),
            ],
            dsp: Mutex::new(Dsp {
                ring: vec![[0.0; 2]; RING],
                pos: 0,
                latency: 0,
            }),
            active: AtomicBool::new(false),
        }
    }

    fn value(&self, i: usize) -> f64 {
        f64::from_bits(self.values[i].load(Ordering::Relaxed))
    }

    fn snapshot(&self) -> [f64; 6] {
        let mut v = [0.0; 6];
        for (i, x) in v.iter_mut().enumerate() {
            *x = self.value(i);
        }
        v
    }

    fn set(&self, id: u32, normalized: f64) {
        // A D234 stand-in acts on a new value only.
        if let Some(k) = D_PS_STAND_INS.iter().position(|x| *x == id)
            && self.stand_ins[k].swap(normalized.to_bits(), Ordering::Relaxed)
                == normalized.to_bits()
        {
            return;
        }
        if id == P_LEVEL && self.instrument {
            self.level
                .store(normalized.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
        }
        // A sample loaded (D234's stand-ins; 0 is their rest).
        if matches!(id, P_LOAD_SAMPLE | P_SAMPLE_PRESET) && self.instrument && normalized > 0.0 {
            self.sample
                .store(normalized.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
        }
        if id == P_SAMPLE_QUIETLY && self.instrument && normalized > 0.0 {
            self.quiet_sample
                .store(normalized.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
            self.quiet_in.store(-1, Ordering::Relaxed);
        }
        // Its own gain, reported (D234).
        if id == P_PROCESSOR_GAIN && self.instrument && normalized > 0.0 {
            let v = normalized.clamp(0.0, 1.0);
            self.values[0].store(to_plain(&PARAMS[0], v).to_bits(), Ordering::Relaxed);
            self.report_gain.store(true, Ordering::Relaxed);
        }
        if let Some(i) = PARAMS.iter().position(|p| p.id == id) {
            let plain = to_plain(&PARAMS[i], normalized);
            self.values[i].store(plain.to_bits(), Ordering::Relaxed);
            if id == P_CRASH && plain >= 0.5 {
                std::process::abort();
            }
            if id == P_HANG && plain >= 0.5 {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                }
            }
        }
    }
}

/// Frames of audio after the quiet sample's baseline is saved until it is loaded (1.5 s
/// at 48 kHz). A delayed helper must capture the trigger before the state-only change.
const QUIET_FRAMES: i64 = 72_000;

/// The processor's D234 stand-ins, which act on a new value only.
const D_PS_STAND_INS: [u32; 4] = [
    P_LOAD_SAMPLE,
    P_SAMPLE_PRESET,
    P_SAMPLE_QUIETLY,
    P_PROCESSOR_GAIN,
];

impl Processor {
    /// `n` frames of audio went by: a sample waiting to be loaded quietly is, in time.
    fn tick_quiet(&self, n: usize) {
        if self.quiet_sample.load(Ordering::Relaxed) == u64::MAX
            || self.quiet_in.load(Ordering::Acquire) < 0
        {
            return;
        }
        if self.quiet_in.fetch_sub(n as i64, Ordering::Relaxed) <= n as i64 {
            let v = self.quiet_sample.swap(u64::MAX, Ordering::Relaxed);
            if v != u64::MAX {
                self.sample.store(v, Ordering::Relaxed);
            }
        }
    }

    /// Reports what it set itself (D234): its gain once `Processor gain` set it, and the
    /// instrument's output peak on `Meter` (read-only) each block, at the block's end.
    ///
    /// # Safety
    /// `out` must be the host's output parameter changes for this call, or null.
    unsafe fn report(&self, out: *mut IParameterChanges, peak: Option<f32>) {
        // SAFETY: the caller's output changes, or null.
        let Some(out) = (unsafe { ComRef::from_raw(out) }) else {
            return;
        };
        let add = |id: u32, value: f64| {
            let mut index = 0;
            // SAFETY: the host's live changes; `index` is ours to fill.
            let q = unsafe { out.addParameterData(&id, &mut index) };
            // SAFETY: the host's queue for `id`, or null.
            if let Some(q) = unsafe { ComRef::from_raw(q) } {
                let mut at = 0;
                // SAFETY: the host's live queue; `at` is ours to fill.
                unsafe { q.addPoint(0, value, &mut at) };
            }
        };
        if self.report_gain.load(Ordering::Relaxed) {
            add(P_GAIN, to_normalized(&PARAMS[0], self.value(0)));
        }
        if let Some(p) = peak.filter(|_| self.instrument) {
            add(P_METER, f64::from(p).clamp(0.0, 1.0));
        }
    }
}

impl IPluginBaseTrait for Processor {
    unsafe fn initialize(&self, _context: *mut FUnknown) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl IConnectionPointTrait for Processor {
    unsafe fn connect(&self, _other: *mut IConnectionPoint) -> tresult {
        kResultOk
    }

    unsafe fn disconnect(&self, _other: *mut IConnectionPoint) -> tresult {
        kResultOk
    }

    unsafe fn notify(&self, _message: *mut IMessage) -> tresult {
        kResultOk
    }
}

impl IComponentTrait for Processor {
    unsafe fn getControllerClassId(&self, class_id: *mut TUID) -> tresult {
        if class_id.is_null() {
            return kInvalidArgument;
        }
        let cid = if self.instrument {
            SINE_CONTROLLER_CID
        } else {
            CONTROLLER_CID
        };
        // SAFETY: the host passes a `TUID` to fill (VST3 `getControllerClassId`).
        unsafe { *class_id = cid };
        kResultOk
    }

    unsafe fn setIoMode(&self, _mode: IoMode) -> tresult {
        kResultOk
    }

    unsafe fn getBusCount(&self, media: MediaType, dir: BusDirection) -> i32 {
        let input = dir as BusDirections == BusDirections_::kInput;
        match (media as MediaTypes, input) {
            (MediaTypes_::kAudio, true) => i32::from(!self.instrument),
            (MediaTypes_::kAudio, false) => 1,
            (MediaTypes_::kEvent, true) => i32::from(self.instrument),
            _ => 0,
        }
    }

    unsafe fn getBusInfo(
        &self,
        media: MediaType,
        dir: BusDirection,
        index: i32,
        bus: *mut BusInfo,
    ) -> tresult {
        // SAFETY: the host passes a `BusInfo` to fill, or null.
        let Some(bus) = (unsafe { bus.as_mut() }) else {
            return kInvalidArgument;
        };
        // SAFETY: our own bus count, which reads nothing the host passes.
        if index != 0 || unsafe { self.getBusCount(media, dir) } < 1 {
            return kInvalidArgument;
        }
        if media as MediaTypes == MediaTypes_::kEvent {
            bus.mediaType = media;
            bus.direction = dir;
            bus.channelCount = 16;
            copy_wstring("Notes", &mut bus.name);
            bus.busType = BusTypes_::kMain as BusType;
            bus.flags = BusInfo_::BusFlags_::kDefaultActive as uint32;
            return kResultOk;
        }
        bus.mediaType = MediaTypes_::kAudio as MediaType;
        bus.direction = dir;
        bus.channelCount = 2;
        copy_wstring(
            if dir as BusDirections == BusDirections_::kInput {
                "Input"
            } else {
                "Output"
            },
            &mut bus.name,
        );
        bus.busType = BusTypes_::kMain as BusType;
        bus.flags = BusInfo_::BusFlags_::kDefaultActive as uint32;
        kResultOk
    }

    unsafe fn getRoutingInfo(
        &self,
        _in_info: *mut RoutingInfo,
        _out_info: *mut RoutingInfo,
    ) -> tresult {
        kNotImplemented
    }

    unsafe fn activateBus(
        &self,
        _media: MediaType,
        _dir: BusDirection,
        _index: i32,
        _state: TBool,
    ) -> tresult {
        kResultOk
    }

    unsafe fn setActive(&self, state: TBool) -> tresult {
        let on = state != 0;
        if on {
            let latency = self.value(3).clamp(0.0, MAX_LATENCY as f64) as usize;
            if let Ok(mut d) = self.dsp.lock() {
                d.ring.iter_mut().for_each(|f| *f = [0.0; 2]);
                d.pos = 0;
                d.latency = latency;
            }
        }
        self.active.store(on, Ordering::Relaxed);
        kResultOk
    }

    unsafe fn setState(&self, state: *mut IBStream) -> tresult {
        // SAFETY: `state` is the host's stream for this call.
        let Some(bytes) = (unsafe { read_all(state) }) else {
            return kResultFalse;
        };
        let Some(pairs) = load(&bytes) else {
            return kResultFalse;
        };
        for (i, v) in pairs {
            self.values[i].store(v.to_bits(), Ordering::Relaxed);
        }
        self.sample
            .store(sample_of(&bytes).to_bits(), Ordering::Relaxed);
        kResultOk
    }

    unsafe fn getState(&self, state: *mut IBStream) -> tresult {
        let pending = self.quiet_sample.load(Ordering::Relaxed);
        let sample = f64::from_bits(self.sample.load(Ordering::Relaxed));
        // SAFETY: `state` is the host's stream for this call.
        let result = unsafe { write_all(state, &save_with_sample(&self.snapshot(), sample)) };
        if result == kResultOk
            && pending != u64::MAX
            && self.quiet_sample.load(Ordering::Relaxed) == pending
        {
            // D234 cannot tell a silent state change from host values seen in the same
            // state poll. Start this fixture's delay only after the baseline is saved;
            // elapsed audio alone does not promise a turn of the helper's main thread.
            let _ = self.quiet_in.compare_exchange(
                -1,
                QUIET_FRAMES,
                Ordering::Release,
                Ordering::Relaxed,
            );
        }
        result
    }
}

impl IAudioProcessorTrait for Processor {
    unsafe fn setBusArrangements(
        &self,
        inputs: *mut SpeakerArrangement,
        num_ins: i32,
        outputs: *mut SpeakerArrangement,
        num_outs: i32,
    ) -> tresult {
        let want_ins = i32::from(!self.instrument);
        if num_ins != want_ins || num_outs != 1 || outputs.is_null() {
            return kResultFalse;
        }
        // SAFETY: checked for null above; the host passes one arrangement.
        if unsafe { *outputs != SpeakerArr::kStereo } {
            return kResultFalse;
        }
        if want_ins == 1 {
            // SAFETY: the host passes one input arrangement, or null.
            if inputs.is_null() || unsafe { *inputs != SpeakerArr::kStereo } {
                return kResultFalse;
            }
        }
        kResultTrue
    }

    unsafe fn getBusArrangement(
        &self,
        _dir: BusDirection,
        index: i32,
        arr: *mut SpeakerArrangement,
    ) -> tresult {
        if index != 0 || arr.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: checked for null above; the host passes an arrangement to fill.
        unsafe { *arr = SpeakerArr::kStereo };
        kResultOk
    }

    unsafe fn canProcessSampleSize(&self, size: i32) -> tresult {
        if size as SymbolicSampleSizes == SymbolicSampleSizes_::kSample32 {
            kResultTrue
        } else {
            kResultFalse
        }
    }

    unsafe fn getLatencySamples(&self) -> u32 {
        self.value(3).clamp(0.0, MAX_LATENCY as f64) as u32
    }

    unsafe fn setupProcessing(&self, setup: *mut ProcessSetup) -> tresult {
        // SAFETY: the host passes its setup for this call, or null.
        if let Some(s) = unsafe { setup.as_ref() }
            && s.sampleRate > 0.0
        {
            self.sample_rate
                .store(s.sampleRate.to_bits(), Ordering::Relaxed);
        }
        kResultOk
    }

    unsafe fn setProcessing(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn process(&self, data: *mut ProcessData) -> tresult {
        // SAFETY: the host passes its `ProcessData` for this call, or null.
        let Some(data) = (unsafe { data.as_ref() }) else {
            return kInvalidArgument;
        };
        let n = data.numSamples.max(0) as usize;
        // Parameter changes take effect from their sample offset, as CLAP events do: the
        // block is cut at each change.
        let mut changes: Vec<(usize, u32, f64)> = Vec::new();
        // SAFETY: the host's parameter changes for this block, or null.
        if let Some(pc) = unsafe { ComRef::from_raw(data.inputParameterChanges) } {
            // SAFETY: `pc` is the host's live parameter changes.
            for q in 0..unsafe { pc.getParameterCount() } {
                // SAFETY: the host's queue for index `q`, or null.
                let Some(queue) = (unsafe { ComRef::from_raw(pc.getParameterData(q)) }) else {
                    continue;
                };
                // SAFETY: `queue` is the host's live queue.
                let id = unsafe { queue.getParameterId() };
                // SAFETY: `queue` is the host's live queue.
                for k in 0..unsafe { queue.getPointCount() } {
                    let (mut offset, mut value) = (0, 0.0);
                    // SAFETY: `queue` is live; `offset` and `value` are ours to fill.
                    if unsafe { queue.getPoint(k, &mut offset, &mut value) } == kResultOk {
                        changes.push((offset.clamp(0, n as i32) as usize, id, value));
                    }
                }
            }
        }
        changes.sort_by_key(|c| c.0);
        // Notes: (offset, on, key, note ID, velocity), in time order.
        let mut notes: Vec<(usize, bool, i16, i32, f64)> = Vec::new();
        // SAFETY: the host's event list for this block, or null.
        if let Some(list) = unsafe { ComRef::from_raw(data.inputEvents) } {
            // SAFETY: `list` is the host's live event list.
            for k in 0..unsafe { list.getEventCount() } {
                // SAFETY: an all-zero event is a valid value to be overwritten.
                let mut e: Event = unsafe { std::mem::zeroed() };
                // SAFETY: `e` is ours to fill; `k` is below the count.
                if unsafe { list.getEvent(k, &mut e) } != kResultOk {
                    continue;
                }
                let at = e.sampleOffset.clamp(0, n as i32) as usize;
                // The event types are C enum constants: `i32` on Windows, `u32` elsewhere.
                let t = i64::from(e.r#type);
                if t == Event_::EventTypes_::kNoteOnEvent as i64 {
                    // SAFETY: a note-on event's union holds `noteOn`.
                    let on = unsafe { e.__field0.noteOn };
                    notes.push((at, true, on.pitch, on.noteId, f64::from(on.velocity)));
                } else if t == Event_::EventTypes_::kNoteOffEvent as i64 {
                    // SAFETY: a note-off event's union holds `noteOff`.
                    let off = unsafe { e.__field0.noteOff };
                    notes.push((at, false, off.pitch, off.noteId, 0.0));
                }
            }
        }
        notes.sort_by_key(|x| x.0);
        let has_in = !self.instrument;
        if (has_in && (data.numInputs < 1 || data.inputs.is_null()))
            || data.numOutputs < 1
            || data.outputs.is_null()
        {
            for (_, id, v) in changes {
                self.set(id, v);
            }
            // SAFETY: the host's output changes for this call, or null.
            unsafe { self.report(data.outputParameterChanges, None) };
            return kResultOk;
        }
        // SAFETY: checked for null above; the host passes at least one output bus.
        let outb = unsafe { &*data.outputs };
        if outb.numChannels < 2 {
            return kResultOk;
        }
        // SAFETY: the output bus has at least two channel pointers (checked above).
        let outs = unsafe { std::slice::from_raw_parts(outb.__field0.channelBuffers32, 2) };
        let ins: &[*mut f32] = if has_in {
            // SAFETY: checked for null above; the host passes at least one input bus.
            let inb = unsafe { &*data.inputs };
            if inb.numChannels < 2 {
                return kResultOk;
            }
            // SAFETY: the input bus has at least two channel pointers (checked above).
            unsafe { std::slice::from_raw_parts(inb.__field0.channelBuffers32, 2) }
        } else {
            &[]
        };
        if ins.iter().chain(outs.iter()).any(|p| p.is_null()) {
            return kInvalidArgument;
        }
        let Ok(mut voices) = self.voices.lock() else {
            return kResultFalse;
        };
        let sr = f64::from_bits(self.sample_rate.load(Ordering::Relaxed));
        let mut next_note = 0;
        let Ok(mut dsp) = self.dsp.lock() else {
            return kResultFalse;
        };
        self.tick_quiet(n);
        let mut next = 0;
        let mut peak = 0.0f32;
        for i in 0..n {
            while next < changes.len() && changes[next].0 <= i {
                let (_, id, v) = changes[next];
                self.set(id, v);
                next += 1;
            }
            while next_note < notes.len() && notes[next_note].0 <= i {
                let (_, on, key, id, velocity) = notes[next_note];
                voices.retain(|v| v.0 != key);
                if on && voices.len() < voices.capacity() {
                    let hz = 440.0 * ((f64::from(key) - 69.0) / 12.0).exp2();
                    voices.push((key, id, velocity, 0.0, std::f64::consts::TAU * hz / sr));
                }
                next_note += 1;
            }
            let level = if self.instrument {
                2.0 * f64::from_bits(self.level.load(Ordering::Relaxed))
            } else {
                1.0
            };
            let sample = f64::from_bits(self.sample.load(Ordering::Relaxed));
            let (gain, mix) = (
                (self.value(0) * level * sample) as f32,
                self.value(1) as f32,
            );
            let delay = self.value(2).clamp(1.0, MAX_DELAY as f64) as usize;
            let x = if has_in {
                // SAFETY: `i` < `numSamples`, the length of every channel buffer.
                unsafe { [*ins[0].add(i), *ins[1].add(i)] }
            } else {
                let mut y = 0.0f64;
                for v in voices.iter_mut() {
                    y += v.2 * v.3.sin();
                    v.3 += v.4;
                }
                [y as f32; 2]
            };
            let pos = dsp.pos;
            dsp.ring[pos & MASK] = x;
            let lat = dsp.latency;
            let a = dsp.ring[pos.wrapping_sub(lat) & MASK];
            let b = dsp.ring[pos.wrapping_sub(lat + delay) & MASK];
            // SAFETY: `i` < `numSamples`, the length of every channel buffer.
            unsafe {
                *outs[0].add(i) = gain * (a[0] + mix * b[0]);
                *outs[1].add(i) = gain * (a[1] + mix * b[1]);
                peak = peak.max((*outs[0].add(i)).abs());
            }
            dsp.pos = pos.wrapping_add(1);
        }
        for (_, id, v) in &changes[next..] {
            self.set(*id, *v);
        }
        // SAFETY: the host's output changes for this call, or null.
        unsafe { self.report(data.outputParameterChanges, Some(peak)) };
        kResultOk
    }

    unsafe fn getTailSamples(&self) -> u32 {
        MAX_DELAY + MAX_LATENCY
    }
}

impl IProcessContextRequirementsTrait for Processor {
    unsafe fn getProcessContextRequirements(&self) -> u32 {
        0
    }
}

// ---------------------------------------------------------------------------------------
// The controller

struct Controller {
    /// Normalized values.
    values: Mutex<[f64; 6]>,
    /// The instrument's: its program parameters' normalized values, as `PROGRAM_PARAMS`.
    programs: Option<Mutex<[f64; 3]>>,
    /// The instrument's editor stand-ins' values, as `EDIT_PARAMS`, and the suffix its
    /// names have (0: none).
    edits: Mutex<[f64; EDIT_PARAMS.len()]>,
    /// The instrument's spares' values (the last `Level`'s), normalized.
    spares: Mutex<Vec<f64>>,
    suffix: std::sync::atomic::AtomicU32,
    /// The host's handler, for the stand-ins' edits and words.
    handler: Mutex<Option<vst3::ComPtr<IComponentHandler>>>,
}

impl Class for Controller {
    type Interfaces = (IEditController, IConnectionPoint, IUnitInfo);
}

impl Controller {
    fn new(instrument: bool) -> Controller {
        let d = defaults();
        let mut v = [0.0; 6];
        for (i, p) in PARAMS.iter().enumerate() {
            v[i] = to_normalized(p, d[i]);
        }
        Controller {
            values: Mutex::new(v),
            programs: instrument.then(|| Mutex::new(PROGRAM_PARAMS.map(|p| p.4))),
            edits: Mutex::new([0.0; EDIT_PARAMS.len()]),
            spares: Mutex::new({
                let mut v = vec![0.0; if instrument { SPARES } else { 0 }];
                if let Some(l) = v.last_mut() {
                    *l = 0.5;
                }
                v
            }),
            suffix: std::sync::atomic::AtomicU32::new(0),
            handler: Mutex::new(None),
        }
    }

    /// The index of spare `id`, for the instrument.
    fn spare(&self, id: u32) -> Option<usize> {
        self.programs.as_ref()?;
        (P_SPARE..=P_LEVEL)
            .contains(&id)
            .then(|| (id - P_SPARE) as usize)
    }

    /// The spares listed: the instrument's.
    fn spares(&self) -> usize {
        if self.programs.is_some() { SPARES } else { 0 }
    }

    /// The index in `EDIT_PARAMS` of editor stand-in `id`, for the instrument.
    fn edit_param(&self, id: u32) -> Option<usize> {
        self.programs.as_ref()?;
        EDIT_PARAMS.iter().position(|p| p.0 == id)
    }

    /// What the editor stand-ins do when set (D216): see `P_EDIT` and `P_SIGNAL`.
    fn edited(&self, id: u32, value: f64) {
        let handler = self.handler.lock().ok().and_then(|h| h.clone());
        let Some(h) = handler else { return };
        if matches!(
            id,
            P_ANSWER | P_PROGRAM_VALUES | P_SAMPLE_QUIETLY | P_PROCESSOR_GAIN | P_METER
        ) {
            return;
        }
        // Its state beyond its parameters (D234): its processor loads the sample; the
        // controller says so with `setDirty` alone, or sets the preset's values and says
        // they changed, or sets them without a word. 0 is their rest.
        if matches!(id, P_LOAD_SAMPLE | P_SAMPLE_PRESET | P_CHANGE_QUIETLY) {
            if value <= 0.0 {
                return;
            }
            if id != P_LOAD_SAMPLE {
                if let Ok(mut v) = self.values.lock() {
                    v[0] = value;
                }
                if let Ok(mut s) = self.spares.lock()
                    && let Some(level) = s.last_mut()
                {
                    *level = value;
                }
            }
            // SAFETY: the host's handler, on the thread it called us on (its main thread);
            // its second interface, if it has one, is its own object.
            unsafe {
                if id == P_LOAD_SAMPLE
                    && let Some(h2) = h.cast::<IComponentHandler2>()
                {
                    h2.setDirty(1);
                } else if id == P_SAMPLE_PRESET {
                    h.restartComponent(RestartFlags_::kParamValuesChanged);
                }
            }
            return;
        }
        if id == P_PRESET {
            // 0 is its rest: taken away, it loads nothing.
            if value <= 0.0 {
                return;
            }
            if let Ok(mut v) = self.values.lock() {
                v[0] = value;
            }
            if let Ok(mut s) = self.spares.lock()
                && let Some(level) = s.last_mut()
            {
                *level = value;
            }
            use RestartFlags_::kParamValuesChanged;
            // SAFETY: the host's handler, on the thread it called us on (its main thread).
            unsafe { h.restartComponent(kParamValuesChanged) };
            return;
        }
        if matches!(id, P_EDIT_GAIN | P_NUDGE | P_HOLD_GAIN | P_RELEASE) {
            // 0 is the stand-ins' rest: taken away, they edit nothing.
            if value <= 0.0 {
                return;
            }
            if id == P_RELEASE {
                // SAFETY: the host's handler, on the thread it called us on.
                unsafe { h.endEdit(P_GAIN) };
                return;
            }
            if let Ok(mut v) = self.values.lock() {
                v[0] = value;
            }
            // SAFETY: the host's handler, on the thread it called us on (its main thread).
            unsafe {
                if id == P_HOLD_GAIN {
                    h.beginEdit(P_GAIN);
                    h.performEdit(P_GAIN, value);
                } else if id == P_EDIT_GAIN {
                    h.beginEdit(P_GAIN);
                    h.performEdit(P_GAIN, value / 2.0);
                    h.performEdit(P_GAIN, value);
                    h.endEdit(P_GAIN);
                } else {
                    h.performEdit(P_GAIN, value / 2.0);
                    h.performEdit(P_GAIN, value);
                }
            }
            return;
        }
        if id == P_EDIT {
            if let Some(Ok(mut v)) = self.programs.as_ref().map(Mutex::lock) {
                v[1] = value;
            }
            // SAFETY: the host's handler, on the thread it called us on (its main thread).
            unsafe {
                h.beginEdit(P_KIT);
                h.performEdit(P_KIT, value);
                h.endEdit(P_KIT);
            }
            return;
        }
        let step = (value.clamp(0.0, 1.0) * f64::from(EDIT_PARAMS[1].2)).round() as u32;
        self.suffix.store(step, Ordering::Relaxed);
        use RestartFlags_::{kLatencyChanged, kParamTitlesChanged, kParamValuesChanged};
        let kit = self
            .programs
            .as_ref()
            .and_then(|p| p.lock().ok().map(|v| program_index(KITS, v[1]) as i32))
            .unwrap_or(0);
        // SAFETY: as above; the unit handler is the host's own object, if it has one.
        unsafe {
            match step {
                1 => {
                    if let Some(u) = h.cast::<IUnitHandler>() {
                        u.notifyProgramListChange(KITS, kit);
                    }
                }
                2 => {
                    if let Some(u) = h.cast::<IUnitHandler>() {
                        u.notifyUnitSelection(DRUM_UNIT);
                    }
                }
                3 => {
                    h.restartComponent(kParamValuesChanged);
                }
                4 => {
                    h.restartComponent(kParamTitlesChanged);
                }
                5 => {
                    if let Some(u) = h.cast::<IUnitHandler2>() {
                        u.notifyUnitByBusChange();
                    }
                }
                6 => {
                    h.restartComponent(kLatencyChanged);
                }
                _ => {}
            }
        }
    }

    /// The index in `PROGRAM_PARAMS` of program parameter `id`, for the instrument.
    fn program_param(&self, id: u32) -> Option<usize> {
        self.programs.as_ref()?;
        PROGRAM_PARAMS.iter().position(|p| p.0 == id)
    }

    /// Its units: the instrument's, else the root alone (without a program list).
    fn units(&self) -> &'static [(UnitID, UnitID, &'static str, ProgramListID)] {
        if self.programs.is_some() {
            &UNITS
        } else {
            &UNITS[..1]
        }
    }

    /// Its program lists: the instrument's, else none.
    fn lists(&self) -> &'static [(ProgramListID, &'static str, Programs)] {
        if self.programs.is_some() { &LISTS } else { &[] }
    }
}

impl IPluginBaseTrait for Controller {
    unsafe fn initialize(&self, _context: *mut FUnknown) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl IConnectionPointTrait for Controller {
    unsafe fn connect(&self, _other: *mut IConnectionPoint) -> tresult {
        kResultOk
    }

    unsafe fn disconnect(&self, _other: *mut IConnectionPoint) -> tresult {
        kResultOk
    }

    unsafe fn notify(&self, _message: *mut IMessage) -> tresult {
        kResultOk
    }
}

impl IEditControllerTrait for Controller {
    unsafe fn setComponentState(&self, state: *mut IBStream) -> tresult {
        // SAFETY: `state` is the host's stream for this call.
        let Some(pairs) = (unsafe { read_all(state) }).and_then(|b| load(&b)) else {
            return kResultFalse;
        };
        if let Ok(mut v) = self.values.lock() {
            for (i, plain) in pairs {
                v[i] = to_normalized(&PARAMS[i], plain);
            }
        }
        kResultOk
    }

    unsafe fn setState(&self, _state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn getState(&self, _state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn getParameterCount(&self) -> i32 {
        let programs = if self.programs.is_some() {
            PROGRAM_PARAMS.len() + EDIT_PARAMS.len()
        } else {
            0
        };
        (PARAMS.len() + self.spares() + programs) as i32
    }

    unsafe fn getParameterInfo(&self, index: i32, info: *mut ParameterInfo) -> tresult {
        // SAFETY: the host passes a `ParameterInfo` to fill, or null.
        let Some(info) = (unsafe { info.as_mut() }) else {
            return kInvalidArgument;
        };
        // The instrument's spares come after the effect's, then its program parameters.
        if let Some(k) = (index.max(0) as usize)
            .checked_sub(PARAMS.len())
            .filter(|k| *k < self.spares())
        {
            let level = k == SPARES - 1;
            info.id = P_SPARE + k as u32;
            let title = if level {
                "Level".to_owned()
            } else {
                format!("Spare {}", k + 1)
            };
            copy_wstring(&title, &mut info.title);
            copy_wstring(&title, &mut info.shortTitle);
            copy_wstring("", &mut info.units);
            info.stepCount = 0;
            info.defaultNormalizedValue = if level { 0.5 } else { 0.0 };
            info.unitId = kRootUnitId;
            info.flags = if level {
                ParameterInfo_::ParameterFlags_::kCanAutomate
            } else {
                ParameterInfo_::ParameterFlags_::kIsHidden
            };
            return kResultOk;
        }
        let program = (index.max(0) as usize).checked_sub(PARAMS.len() + self.spares());
        if let Some(&(id, title, unit, list, default, automatable)) = program
            .filter(|_| self.programs.is_some())
            .and_then(|i| PROGRAM_PARAMS.get(i))
        {
            info.id = id;
            copy_wstring(title, &mut info.title);
            copy_wstring(title, &mut info.shortTitle);
            copy_wstring("", &mut info.units);
            info.stepCount = programs(list).map_or(0, |p| p.len() as i32 - 1);
            info.defaultNormalizedValue = default;
            info.unitId = unit;
            use ParameterInfo_::ParameterFlags_::{kCanAutomate, kIsList, kIsProgramChange};
            info.flags = kIsProgramChange | kIsList | if automatable { kCanAutomate } else { 0 };
            return kResultOk;
        }
        // Then the editor stand-ins, hidden.
        if let Some(&(id, title, steps)) = program
            .filter(|_| self.programs.is_some())
            .and_then(|i| i.checked_sub(PROGRAM_PARAMS.len()))
            .and_then(|i| EDIT_PARAMS.get(i))
        {
            info.id = id;
            copy_wstring(title, &mut info.title);
            copy_wstring(title, &mut info.shortTitle);
            copy_wstring("", &mut info.units);
            info.stepCount = steps;
            info.defaultNormalizedValue = 0.0;
            info.unitId = kRootUnitId;
            info.flags = ParameterInfo_::ParameterFlags_::kIsHidden;
            if id == P_METER {
                info.flags |= ParameterInfo_::ParameterFlags_::kIsReadOnly;
            }
            return kResultOk;
        }
        let Some(p) = PARAMS.get(index.max(0) as usize) else {
            return kInvalidArgument;
        };
        info.id = p.id;
        copy_wstring(p.name, &mut info.title);
        copy_wstring(p.name, &mut info.shortTitle);
        copy_wstring(
            if p.id == P_DELAY || p.id == P_LATENCY {
                "frames"
            } else {
                ""
            },
            &mut info.units,
        );
        info.stepCount = if p.stepped { (p.max - p.min) as i32 } else { 0 };
        info.defaultNormalizedValue = to_normalized(p, p.default);
        info.unitId = 0;
        let mut flags = ParameterInfo_::ParameterFlags_::kCanAutomate;
        if p.hidden {
            flags = ParameterInfo_::ParameterFlags_::kIsHidden;
        }
        info.flags = flags;
        kResultOk
    }

    unsafe fn getParamStringByValue(&self, id: u32, value: f64, string: *mut String128) -> tresult {
        // SAFETY: the host passes a `String128` to fill, or null.
        let Some(s) = (unsafe { string.as_mut() }) else {
            return kInvalidArgument;
        };
        if let Some(i) = self.program_param(id) {
            let list = PROGRAM_PARAMS[i].3;
            let name = programs(list).map_or("", |p| p[program_index(list, value)].0);
            copy_wstring(name, s);
            return kResultOk;
        }
        if let Some(i) = self.edit_param(id) {
            let step = (value.clamp(0.0, 1.0) * f64::from(EDIT_PARAMS[i].2)).round();
            copy_wstring(&format!("{step:.0}"), s);
            return kResultOk;
        }
        if self.spare(id).is_some() {
            copy_wstring(&format!("{:.3}", value.clamp(0.0, 1.0)), s);
            return kResultOk;
        }
        let Some(p) = def(id) else {
            return kInvalidArgument;
        };
        let plain = to_plain(p, value);
        let text = if p.stepped {
            format!("{plain:.0}")
        } else {
            format!("{plain:.3}")
        };
        copy_wstring(&text, s);
        kResultOk
    }

    unsafe fn getParamValueByString(
        &self,
        id: u32,
        string: *mut TChar,
        value: *mut f64,
    ) -> tresult {
        if self.program_param(id).is_some()
            || self.edit_param(id).is_some()
            || self.spare(id).is_some()
        {
            return kResultFalse;
        }
        let Some(p) = def(id) else {
            return kInvalidArgument;
        };
        if string.is_null() || value.is_null() {
            return kInvalidArgument;
        }
        let mut len = 0;
        // SAFETY: `string` is non-null (checked above) and zero-terminated within 128 units, as VST3 strings are.
        while len < 128 && unsafe { *string.add(len) } != 0 {
            len += 1;
        }
        // SAFETY: the first `len` units were read just above.
        let units: Vec<u16> = (0..len).map(|i| unsafe { *string.add(i) }).collect();
        match String::from_utf16_lossy(&units).trim().parse::<f64>() {
            Ok(plain) => {
                // SAFETY: checked for null above.
                unsafe { *value = to_normalized(p, plain) };
                kResultOk
            }
            Err(_) => kResultFalse,
        }
    }

    unsafe fn normalizedParamToPlain(&self, id: u32, value: f64) -> f64 {
        if let Some(i) = self.program_param(id) {
            return program_index(PROGRAM_PARAMS[i].3, value) as f64;
        }
        if let Some(i) = self.edit_param(id) {
            let steps = f64::from(EDIT_PARAMS[i].2);
            return if steps > 0.0 {
                (value.clamp(0.0, 1.0) * steps).round()
            } else {
                value.clamp(0.0, 1.0)
            };
        }
        if self.spare(id).is_some() {
            return value.clamp(0.0, 1.0);
        }
        def(id).map_or(0.0, |p| to_plain(p, value))
    }

    unsafe fn plainParamToNormalized(&self, id: u32, plain: f64) -> f64 {
        if let Some(i) = self.program_param(id) {
            let steps = programs(PROGRAM_PARAMS[i].3).map_or(0, |p| p.len().saturating_sub(1));
            return if steps > 0 {
                (plain / steps as f64).clamp(0.0, 1.0)
            } else {
                0.0
            };
        }
        if let Some(i) = self.edit_param(id) {
            let steps = f64::from(EDIT_PARAMS[i].2);
            return if steps > 0.0 {
                (plain / steps).clamp(0.0, 1.0)
            } else {
                plain.clamp(0.0, 1.0)
            };
        }
        if self.spare(id).is_some() {
            return plain.clamp(0.0, 1.0);
        }
        def(id).map_or(0.0, |p| to_normalized(p, plain))
    }

    unsafe fn getParamNormalized(&self, id: u32) -> f64 {
        if let (Some(i), Some(Ok(v))) = (
            self.program_param(id),
            self.programs.as_ref().map(Mutex::lock),
        ) {
            return v[i];
        }
        if let (Some(i), Ok(v)) = (self.edit_param(id), self.edits.lock()) {
            return v[i];
        }
        if let (Some(i), Ok(v)) = (self.spare(id), self.spares.lock()) {
            return v.get(i).copied().unwrap_or(0.0);
        }
        let i = PARAMS.iter().position(|p| p.id == id);
        match (i, self.values.lock()) {
            (Some(i), Ok(v)) => v[i],
            _ => 0.0,
        }
    }

    unsafe fn setParamNormalized(&self, id: u32, value: f64) -> tresult {
        if let (Some(i), Some(Ok(mut v))) = (
            self.program_param(id),
            self.programs.as_ref().map(Mutex::lock),
        ) {
            let moved = (v[i] - value.clamp(0.0, 1.0)).abs() > 1e-9;
            v[i] = value.clamp(0.0, 1.0);
            drop(v);
            // A kit that carries a value (D228's stand-in): `Level` set, and said.
            let carries = self
                .edits
                .lock()
                .ok()
                .and_then(|e| {
                    EDIT_PARAMS
                        .iter()
                        .position(|p| p.0 == P_PROGRAM_VALUES)
                        .map(|k| e[k] > 0.0)
                })
                .unwrap_or(false);
            if id == P_KIT && moved && carries {
                if let Ok(mut s) = self.spares.lock()
                    && let Some(level) = s.last_mut()
                {
                    *level = 0.25;
                }
                let handler = self.handler.lock().ok().and_then(|h| h.clone());
                if let Some(h) = handler {
                    // SAFETY: the host's handler, on the thread it called us on (its main
                    // thread).
                    unsafe { h.restartComponent(RestartFlags_::kParamValuesChanged) };
                }
            }
            return kResultOk;
        }
        if let Some(i) = self.edit_param(id) {
            let mut same = false;
            if let Ok(mut v) = self.edits.lock() {
                same = v[i] == value.clamp(0.0, 1.0);
                v[i] = value.clamp(0.0, 1.0);
            }
            // D234's stand-ins act on a new value only (D216's to D228's on every one).
            if same && matches!(id, P_LOAD_SAMPLE | P_SAMPLE_PRESET | P_CHANGE_QUIETLY) {
                return kResultOk;
            }
            self.edited(id, value.clamp(0.0, 1.0));
            return kResultOk;
        }
        if let Some(i) = self.spare(id) {
            if let Ok(mut v) = self.spares.lock()
                && let Some(x) = v.get_mut(i)
            {
                *x = value.clamp(0.0, 1.0);
            }
            return kResultOk;
        }
        let Some(i) = PARAMS.iter().position(|p| p.id == id) else {
            return kInvalidArgument;
        };
        if let Ok(mut v) = self.values.lock() {
            v[i] = value.clamp(0.0, 1.0);
        }
        // Answering the host's gain (D222), while `Answer gain` is set.
        let answer = self
            .edits
            .lock()
            .ok()
            .and_then(|v| {
                EDIT_PARAMS
                    .iter()
                    .position(|p| p.0 == P_ANSWER)
                    .map(|k| v[k])
            })
            .unwrap_or(0.0);
        let handler = self.handler.lock().ok().and_then(|h| h.clone());
        if id == P_GAIN
            && answer > 0.0
            && let Some(h) = handler
        {
            // SAFETY: the host's handler, on the thread it called us on.
            unsafe { h.performEdit(P_GAIN, value.clamp(0.0, 1.0)) };
        }
        kResultOk
    }

    unsafe fn setComponentHandler(&self, handler: *mut IComponentHandler) -> tresult {
        // SAFETY: the host passes its live handler, or null; we keep a reference of our own.
        let h = unsafe { ComRef::from_raw(handler) }.map(|r| r.to_com_ptr());
        if let Ok(mut x) = self.handler.lock() {
            *x = h;
        }
        kResultOk
    }

    unsafe fn createView(&self, name: *const c_char) -> *mut IPlugView {
        // SAFETY: the host passes a NUL-terminated view name, or null.
        if name.is_null() || unsafe { std::ffi::CStr::from_ptr(name) } != c"editor" {
            return std::ptr::null_mut();
        }
        ComWrapper::new(gui::Editor::default())
            .to_com_ptr::<IPlugView>()
            .map_or(std::ptr::null_mut(), |p| p.into_raw())
    }
}

impl IUnitInfoTrait for Controller {
    unsafe fn getUnitCount(&self) -> i32 {
        self.units().len() as i32
    }

    unsafe fn getUnitInfo(&self, index: i32, info: *mut UnitInfo) -> tresult {
        // SAFETY: the host passes a `UnitInfo` to fill, or null.
        let (Some(u), Some(info)) = (self.units().get(index.max(0) as usize), unsafe {
            info.as_mut()
        }) else {
            return kInvalidArgument;
        };
        info.id = u.0;
        info.parentUnitId = u.1;
        copy_wstring(u.2, &mut info.name);
        // The effect's root unit has no programs.
        info.programListId = if self.programs.is_some() {
            u.3
        } else {
            kNoProgramListId
        };
        kResultOk
    }

    unsafe fn getProgramListCount(&self) -> i32 {
        self.lists().len() as i32
    }

    unsafe fn getProgramListInfo(&self, index: i32, info: *mut ProgramListInfo) -> tresult {
        // SAFETY: the host passes a `ProgramListInfo` to fill, or null.
        let (Some(l), Some(info)) = (self.lists().get(index.max(0) as usize), unsafe {
            info.as_mut()
        }) else {
            return kInvalidArgument;
        };
        info.id = l.0;
        copy_wstring(l.1, &mut info.name);
        info.programCount = l.2.len() as i32;
        kResultOk
    }

    unsafe fn getProgramName(
        &self,
        list: ProgramListID,
        program: i32,
        name: *mut String128,
    ) -> tresult {
        let p = self
            .lists()
            .iter()
            .find(|l| l.0 == list)
            .and_then(|l| l.2.get(program.max(0) as usize));
        // SAFETY: the host passes a `String128` to fill, or null.
        match (p, unsafe { name.as_mut() }) {
            (Some(p), Some(name)) => {
                copy_wstring(p.0, name);
                kResultOk
            }
            _ => kInvalidArgument,
        }
    }

    unsafe fn getProgramInfo(
        &self,
        _list: ProgramListID,
        _program: i32,
        _attribute: vst3::Steinberg::Vst::CString,
        _value: *mut String128,
    ) -> tresult {
        kResultFalse
    }

    unsafe fn hasProgramPitchNames(&self, list: ProgramListID, program: i32) -> tresult {
        let has = self
            .lists()
            .iter()
            .find(|l| l.0 == list)
            .and_then(|l| l.2.get(program.max(0) as usize))
            .is_some_and(|p| !p.1.is_empty());
        if has { kResultTrue } else { kResultFalse }
    }

    unsafe fn getProgramPitchName(
        &self,
        list: ProgramListID,
        program: i32,
        pitch: i16,
        name: *mut String128,
    ) -> tresult {
        let n = self
            .lists()
            .iter()
            .find(|l| l.0 == list)
            .and_then(|l| l.2.get(program.max(0) as usize))
            .and_then(|p| p.1.iter().find(|k| k.0 == pitch));
        let suffix = self.suffix.load(Ordering::Relaxed);
        // SAFETY: the host passes a `String128` to fill, or null.
        match (n, unsafe { name.as_mut() }) {
            (Some(n), Some(name)) if suffix > 0 => {
                copy_wstring(&format!("{} {suffix}", n.1), name);
                kResultOk
            }
            (Some(n), Some(name)) => {
                copy_wstring(n.1, name);
                kResultOk
            }
            (None, Some(_)) => kResultFalse,
            _ => kInvalidArgument,
        }
    }

    unsafe fn getSelectedUnit(&self) -> UnitID {
        kRootUnitId
    }

    unsafe fn selectUnit(&self, _unit: UnitID) -> tresult {
        kResultOk
    }

    unsafe fn getUnitByBus(
        &self,
        media: MediaType,
        dir: BusDirection,
        bus: i32,
        channel: i32,
        unit: *mut UnitID,
    ) -> tresult {
        // The note input's channels are the drum unit's.
        let notes = media as MediaTypes == MediaTypes_::kEvent
            && dir as BusDirections == BusDirections_::kInput
            && bus == 0
            && (0..16).contains(&channel);
        if !notes || self.programs.is_none() || unit.is_null() {
            return kResultFalse;
        }
        // SAFETY: checked for null above; the host passes a unit ID to fill.
        unsafe { *unit = DRUM_UNIT };
        kResultOk
    }

    unsafe fn setUnitProgramData(
        &self,
        _list: i32,
        _program: i32,
        _data: *mut IBStream,
    ) -> tresult {
        kNotImplemented
    }
}

// ---------------------------------------------------------------------------------------
// The factory and module entry points

/// The factory's classes: ID, category, name and sub-categories.
const CLASSES: [(TUID, &str, &str, &str); 4] = [
    (PROCESSOR_CID, "Audio Module Class", NAME, "Fx"),
    (
        SINE_CID,
        "Audio Module Class",
        SINE_NAME,
        "Instrument|Synth",
    ),
    (CONTROLLER_CID, "Component Controller Class", NAME, ""),
    (
        SINE_CONTROLLER_CID,
        "Component Controller Class",
        SINE_NAME,
        "",
    ),
];

struct Factory;

impl Class for Factory {
    type Interfaces = (IPluginFactory, IPluginFactory2);
}

impl IPluginFactory2Trait for Factory {
    unsafe fn getClassInfo2(&self, index: i32, info: *mut PClassInfo2) -> tresult {
        // SAFETY: the host passes a `PClassInfo2` to fill, or null.
        let Some(info) = (unsafe { info.as_mut() }) else {
            return kInvalidArgument;
        };
        let Some(c) = CLASSES.get(index.max(0) as usize) else {
            return kInvalidArgument;
        };
        info.cid = c.0;
        info.cardinality = PClassInfo_::ClassCardinality_::kManyInstances as int32;
        copy_cstring(c.1, &mut info.category);
        copy_cstring(c.2, &mut info.name);
        info.classFlags = 0;
        copy_cstring(c.3, &mut info.subCategories);
        copy_cstring("DAW-v3", &mut info.vendor);
        copy_cstring(env!("CARGO_PKG_VERSION"), &mut info.version);
        copy_cstring("VST 3.8.0", &mut info.sdkVersion);
        kResultOk
    }
}

impl IPluginFactoryTrait for Factory {
    unsafe fn getFactoryInfo(&self, info: *mut PFactoryInfo) -> tresult {
        // SAFETY: the host passes a `PFactoryInfo` to fill, or null.
        let Some(info) = (unsafe { info.as_mut() }) else {
            return kInvalidArgument;
        };
        copy_cstring("DAW-v3", &mut info.vendor);
        copy_cstring("", &mut info.url);
        copy_cstring("", &mut info.email);
        info.flags = PFactoryInfo_::FactoryFlags_::kUnicode as int32;
        kResultOk
    }

    unsafe fn countClasses(&self) -> i32 {
        CLASSES.len() as i32
    }

    unsafe fn getClassInfo(&self, index: i32, info: *mut PClassInfo) -> tresult {
        // SAFETY: the host passes a `PClassInfo` to fill, or null.
        let Some(info) = (unsafe { info.as_mut() }) else {
            return kInvalidArgument;
        };
        let Some(c) = CLASSES.get(index.max(0) as usize) else {
            return kInvalidArgument;
        };
        info.cid = c.0;
        info.cardinality = PClassInfo_::ClassCardinality_::kManyInstances as int32;
        copy_cstring(c.1, &mut info.category);
        copy_cstring(c.2, &mut info.name);
        kResultOk
    }

    unsafe fn createInstance(
        &self,
        cid: FIDString,
        iid: FIDString,
        obj: *mut *mut c_void,
    ) -> tresult {
        if cid.is_null() || obj.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: `cid` is non-null (checked above) and points at a 16-byte class ID.
        let cid = unsafe { *(cid as *const TUID) };
        let instance = if cid == PROCESSOR_CID {
            ComWrapper::new(Processor::new(false)).to_com_ptr::<FUnknown>()
        } else if cid == SINE_CID {
            ComWrapper::new(Processor::new(true)).to_com_ptr::<FUnknown>()
        } else if cid == CONTROLLER_CID {
            ComWrapper::new(Controller::new(false)).to_com_ptr::<FUnknown>()
        } else if cid == SINE_CONTROLLER_CID {
            ComWrapper::new(Controller::new(true)).to_com_ptr::<FUnknown>()
        } else {
            None
        };
        match instance {
            Some(instance) => {
                let ptr = instance.as_ptr();
                // SAFETY: `ptr` is the live object just made; `iid` and `obj` are the host's.
                unsafe { ((*(*ptr).vtbl).queryInterface)(ptr, iid as *mut TUID, obj) }
            }
            None => kInvalidArgument,
        }
    }
}

#[cfg(target_os = "windows")]
#[unsafe(no_mangle)]
extern "system" fn InitDll() -> bool {
    true
}

#[cfg(target_os = "windows")]
#[unsafe(no_mangle)]
extern "system" fn ExitDll() -> bool {
    true
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
extern "system" fn BundleEntry(_bundle: *mut c_void) -> bool {
    true
}

#[cfg(target_os = "macos")]
#[unsafe(no_mangle)]
extern "system" fn BundleExit() -> bool {
    true
}

#[cfg(all(unix, not(target_os = "macos")))]
#[unsafe(no_mangle)]
extern "system" fn ModuleEntry(_library: *mut c_void) -> bool {
    true
}

#[cfg(all(unix, not(target_os = "macos")))]
#[unsafe(no_mangle)]
extern "system" fn ModuleExit() -> bool {
    true
}

/// The module's factory (VST3's `GetPluginFactory`).
#[unsafe(no_mangle)]
extern "system" fn GetPluginFactory() -> *mut IPluginFactory {
    ComWrapper::new(Factory)
        .to_com_ptr::<IPluginFactory>()
        .map_or(std::ptr::null_mut(), |p| p.into_raw())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_round_trips_and_refuses_what_is_not_its() {
        let mut v = defaults();
        v[0] = 0.5;
        v[2] = 960.0;
        let bytes = save(&v);
        let back = load(&bytes).unwrap_or_default();
        assert_eq!(back, [(0, 0.5), (1, 0.0), (2, 960.0), (3, 0.0)]);
        assert!(load(b"nope").is_none());
        // A crash or hang parameter in a state is refused: it would crash again.
        let mut bad = bytes.clone();
        bad[8..12].copy_from_slice(&P_CRASH.to_le_bytes());
        assert!(load(&bad).is_none());
    }

    /// D234: its sample's level is in its state when not 1, and a state without one has 1.
    #[test]
    fn a_states_sample_round_trips() {
        let v = defaults();
        assert_eq!(save_with_sample(&v, 1.0), save(&v), "none: as before");
        let bytes = save_with_sample(&v, 0.25);
        assert_eq!(sample_of(&bytes), 0.25);
        assert_eq!(
            load(&bytes).map(|p| p.len()),
            Some(4),
            "its values still load"
        );
        assert_eq!(sample_of(&save(&v)), 1.0);
    }

    #[test]
    fn values_normalize_and_step() {
        // A stepped value comes back from its normalized form exactly.
        let delay = def(P_DELAY).map(|p| to_plain(p, to_normalized(p, 960.0)));
        assert_eq!(delay, Some(960.0));
        let gain = def(P_GAIN).map(|p| to_normalized(p, 1.0));
        assert_eq!(gain, Some(0.5));
    }
}
