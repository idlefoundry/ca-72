//! The host side of VST3 (DESIGN 10.6; M6 plan W1, D78), shaped like the CLAP backend
//! (`clap`): a loaded [`Module`] lists its plugins and makes [`Instance`]s, and an
//! instance hands a [`Processor`] to the audio thread.
//!
//! - **Modules.** A `.vst3` bundle (the platform's binary inside `Contents/`) or a plain
//!   library file, with its entry point (`ModuleEntry`, `InitDll` or `BundleEntry`) and
//!   factory (`GetPluginFactory`). A plugin is named by its processor's class ID, as 32 hex
//!   digits.
//! - **Instances.** The processor (`IComponent`, `IAudioProcessor`) and its controller
//!   (`IEditController`, its own class or the same object), initialized with the host
//!   context and connected to each other, the controller told the processor's state.
//! - **Parameters** are the controller's, as normalized values (0 to 1): what VST3 sends
//!   and stores. Changes reach the processor as parameter changes in the block, at their
//!   frame, and the controller as `setParamNormalized`.
//! - **Modulation** is destructive (DESIGN 11.4): VST3 has no modulation of its own, so a
//!   modulated parameter's processor is sent its base value plus the modulation amount as
//!   its value. Its controller (its editor) is told the base alone (the CA-72's PATCHES.md
//!   change 6: the window neither chases the modulation nor fights a drag there); the
//!   processor's own state may hold a modulated instant, so the host gives the base back
//!   after a state is loaded.
//! - **State** is the processor's (`IComponent::getState`) and the controller's, in one
//!   blob: `VST3` then each part's length and bytes.
//!
//! - **Notes** reach an instrument's event input as note-on and note-off events at their
//!   frames, with key, channel, velocity and note ID (W2).
//!
//! - **The editor** (W5): the controller's view (`createView("editor")`) embedded in a
//!   window the helper creates and owns (`crate::window`), as a CLAP GUI is; resize
//!   requests go through the host's `IPlugFrame`.
//!
//! - **Names for keys** (D211): the program pitch names (`IUnitInfo`) of the program the
//!   note input's unit plays, keys 0 to 127 ([`Instance::note_names`]). While it runs
//!   (D216), the values the processor is sent reach the controller too, on the main thread
//!   ([`Instance::service`]), and the names are asked again when a program change moves to
//!   another program, the plugin edits a parameter itself (`performEdit`, its editor) or
//!   says its programs or parameters changed (`IUnitHandler`, `restartComponent`).
//!
//! Not yet (M6 plan): note expressions, buses beyond the main stereo pair and the event
//! bus, a plugin's own parameter edits sent back to the host (to its processor and the
//! project), and Linux's `IRunLoop` (a plugin view that needs the host's event loop on
//! X11).
//!
//! Unsafe code: this module is the host side of the VST3 ABI (see `docs/unsafe.md`).

#![allow(unsafe_code)]
#![allow(non_snake_case)]

use crate::window::{self, Window, WindowEvent};
use daw_ipc::layout::{EV_NOTE_OFF, EV_NOTE_ON, EV_PARAM, EV_PARAM_MOD, STATUS_ERROR, STATUS_OK};
use daw_ipc::protocol::{EditKind, ParamEdit};
use daw_ipc::protocol::{NoteName, ParamInfo, PluginDesc};
use daw_ipc::{Event, MAX_EVENTS};
use serde_json::{Value, json};
use std::ffi::{CStr, c_char, c_void};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use vst3::Steinberg::Vst::*;
use vst3::Steinberg::*;
use vst3::{Class, ComPtr, ComWrapper, Interface};

/// The platform type this helper's windows offer a plugin view (VST3's `kPlatformType…`).
#[cfg(all(unix, not(target_os = "macos")))]
const PLATFORM: &CStr = c"X11EmbedWindowID";
#[cfg(windows)]
const PLATFORM: &CStr = c"HWND";
#[cfg(target_os = "macos")]
const PLATFORM: &CStr = c"NSView";

/// Points per parameter in one block, at most (more: the last point takes the latest).
const MAX_POINTS: usize = 64;

/// Values a plugin's editor sets (`performEdit`) that one block takes, at most; the rest
/// wait for the next (D222).
const EDITS_PER_BLOCK: usize = 256;

/// Parameters a plugin's processor can report values of in one block (D234,
/// `outputParameterChanges`); more find no queue (`addParameterData` gives none).
const OUTPUTS_PER_BLOCK: usize = 256;

/// A class ID as VST3 prints one: its four 32-bit words in hex, the same on every
/// platform (Windows keeps the bytes in COM's order, others in order), so a project names a
/// plugin the same way everywhere.
pub fn cid_hex(cid: &TUID) -> String {
    let b: Vec<u8> = cid.iter().map(|x| *x as u8).collect();
    let be = |i: usize| u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
    let (w0, w1) = if cfg!(windows) {
        (
            u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            (u32::from(b[5]) << 24)
                | (u32::from(b[4]) << 16)
                | (u32::from(b[7]) << 8)
                | u32::from(b[6]),
        )
    } else {
        (be(0), be(4))
    };
    format!("{w0:08X}{w1:08X}{:08X}{:08X}", be(8), be(12))
}

/// The class ID [`cid_hex`] printed.
pub fn cid_parse(s: &str) -> Option<TUID> {
    if s.len() != 32 || !s.is_ascii() {
        return None;
    }
    let w = |i: usize| u32::from_str_radix(&s[8 * i..8 * i + 8], 16).ok();
    Some(vst3::uid(w(0)?, w(1)?, w(2)?, w(3)?))
}

fn cstr(a: &[c_char]) -> String {
    let bytes: Vec<u8> = a
        .iter()
        .take_while(|c| **c != 0)
        .map(|c| *c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

fn wstr(a: &[TChar]) -> String {
    let units: Vec<u16> = a.iter().take_while(|c| **c != 0).copied().collect();
    String::from_utf16_lossy(&units)
}

/// The binary of a plugin path: inside a bundle's `Contents/`, else the file itself.
fn binary_path(path: &Path) -> PathBuf {
    if !path.is_dir() {
        return path.to_path_buf();
    }
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let contents = path.join("Contents");
    let arch = std::env::consts::ARCH;
    let candidates = if cfg!(windows) {
        vec![
            contents
                .join(format!("{arch}-win"))
                .join(format!("{stem}.vst3")),
        ]
    } else if cfg!(target_os = "macos") {
        vec![contents.join("MacOS").join(&stem)]
    } else {
        vec![
            contents
                .join(format!("{arch}-linux"))
                .join(format!("{stem}.so")),
        ]
    };
    candidates
        .into_iter()
        .find(|c| c.exists())
        .unwrap_or_else(|| path.to_path_buf())
}

// ---------------------------------------------------------------------------------------
// The host's own objects

/// A stream over bytes in memory (`IBStream`), for state.
struct Stream {
    data: Mutex<(Vec<u8>, usize)>,
}

impl Class for Stream {
    type Interfaces = (IBStream,);
}

impl Stream {
    fn new(bytes: Vec<u8>) -> ComWrapper<Stream> {
        ComWrapper::new(Stream {
            data: Mutex::new((bytes, 0)),
        })
    }

    fn bytes(&self) -> Vec<u8> {
        self.data.lock().map(|d| d.0.clone()).unwrap_or_default()
    }
}

impl IBStreamTrait for Stream {
    unsafe fn read(&self, buffer: *mut c_void, n: int32, read: *mut int32) -> tresult {
        let Ok(mut d) = self.data.lock() else {
            return kResultFalse;
        };
        let (data, pos) = &mut *d;
        let take = (n.max(0) as usize).min(data.len().saturating_sub(*pos));
        if take > 0 && !buffer.is_null() {
            // SAFETY: the plugin passes a buffer of at least `n` bytes; `take` <= `n`.
            unsafe { std::ptr::copy_nonoverlapping(data[*pos..].as_ptr(), buffer.cast(), take) };
        }
        *pos += take;
        if !read.is_null() {
            // SAFETY: a non-null count pointer is the plugin's to receive.
            unsafe { *read = take as int32 };
        }
        kResultOk
    }

    unsafe fn write(&self, buffer: *mut c_void, n: int32, written: *mut int32) -> tresult {
        let Ok(mut d) = self.data.lock() else {
            return kResultFalse;
        };
        let n = n.max(0) as usize;
        if n > 0 && buffer.is_null() {
            return kInvalidArgument;
        }
        let (data, pos) = &mut *d;
        if *pos + n > data.len() {
            data.resize(*pos + n, 0);
        }
        if n > 0 {
            // SAFETY: the plugin passes `n` readable bytes.
            let src = unsafe { std::slice::from_raw_parts(buffer.cast::<u8>(), n) };
            data[*pos..*pos + n].copy_from_slice(src);
        }
        *pos += n;
        if !written.is_null() {
            // SAFETY: a non-null count pointer is the plugin's to receive.
            unsafe { *written = n as int32 };
        }
        kResultOk
    }

    unsafe fn seek(&self, to: int64, mode: int32, result: *mut int64) -> tresult {
        let Ok(mut d) = self.data.lock() else {
            return kResultFalse;
        };
        let (data, pos) = &mut *d;
        // The seek modes are C enum constants: `i32` on Windows, `u32` elsewhere.
        use IBStream_::IStreamSeekMode_::{kIBSeekCur, kIBSeekEnd, kIBSeekSet};
        let mode = i64::from(mode);
        let base = if mode == kIBSeekSet as i64 {
            0
        } else if mode == kIBSeekCur as i64 {
            *pos as i64
        } else if mode == kIBSeekEnd as i64 {
            data.len() as i64
        } else {
            return kInvalidArgument;
        };
        let new = base + to;
        if new < 0 {
            return kResultFalse;
        }
        *pos = new as usize;
        if !result.is_null() {
            // SAFETY: a non-null result pointer is the plugin's to receive.
            unsafe { *result = new };
        }
        kResultOk
    }

    unsafe fn tell(&self, pos: *mut int64) -> tresult {
        let Ok(d) = self.data.lock() else {
            return kResultFalse;
        };
        if pos.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: checked for null above.
        unsafe { *pos = d.1 as int64 };
        kResultOk
    }
}

/// One parameter's changes in a block (`IParamValueQueue`), reused from block to block.
struct Queue {
    inner: Mutex<(ParamID, Vec<(int32, ParamValue)>)>,
}

impl Class for Queue {
    type Interfaces = (IParamValueQueue,);
}

impl IParamValueQueueTrait for Queue {
    unsafe fn getParameterId(&self) -> ParamID {
        self.inner.lock().map_or(0, |q| q.0)
    }

    unsafe fn getPointCount(&self) -> int32 {
        self.inner.lock().map_or(0, |q| q.1.len() as int32)
    }

    unsafe fn getPoint(&self, index: int32, offset: *mut int32, value: *mut ParamValue) -> tresult {
        let Ok(q) = self.inner.lock() else {
            return kResultFalse;
        };
        let Some((o, v)) = q.1.get(index.max(0) as usize) else {
            return kInvalidArgument;
        };
        if offset.is_null() || value.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: both checked for null; the plugin passes places to receive.
        unsafe {
            *offset = *o;
            *value = *v;
        }
        kResultOk
    }

    unsafe fn addPoint(&self, offset: int32, value: ParamValue, index: *mut int32) -> tresult {
        // (A plugin's own output queue: it is told when there is no room.)
        let Ok(mut q) = self.inner.lock() else {
            return kResultFalse;
        };
        if q.1.len() >= MAX_POINTS {
            return kResultFalse;
        }
        q.1.push((offset, value));
        if !index.is_null() {
            // SAFETY: a non-null index pointer is the plugin's to receive.
            unsafe { *index = q.1.len() as int32 - 1 };
        }
        kResultOk
    }
}

/// A block's parameter changes (`IParameterChanges`): a pool of queues made off the audio
/// thread, so it never allocates. A parameter finds its queue by its place in the
/// processor's table of every parameter (D222: no cap on how many a plugin has).
struct Changes {
    queues: Vec<ComWrapper<Queue>>,
    slots: Mutex<Slots>,
    /// Every parameter's ID by its place (sorted), so that a queue the plugin adds (its
    /// output changes, D234) is found by a binary search, not by asking each queue.
    ids: Vec<ParamID>,
}

/// Which queues hold which parameters this block.
struct Slots {
    used: usize,
    /// By parameter place: its queue + 1 this block, else 0.
    queue_of: Vec<u32>,
    /// By queue: the parameter place it holds (`usize::MAX`: one a plugin added).
    place_of: Vec<usize>,
}

impl Class for Changes {
    type Interfaces = (IParameterChanges,);
}

impl Changes {
    /// For the parameters `ids` (sorted: their places), `pool` of them changed in one block
    /// at most.
    fn new(ids: &[ParamID], pool: usize) -> ComWrapper<Changes> {
        let places = ids.len();
        let pool = pool.max(1);
        ComWrapper::new(Changes {
            ids: ids.to_vec(),
            queues: (0..pool)
                .map(|_| {
                    ComWrapper::new(Queue {
                        inner: Mutex::new((0, Vec::with_capacity(MAX_POINTS))),
                    })
                })
                .collect(),
            slots: Mutex::new(Slots {
                used: 0,
                queue_of: vec![0; places],
                place_of: vec![usize::MAX; pool],
            }),
        })
    }

    fn clear(&self) {
        if let Ok(mut s) = self.slots.lock() {
            let used = s.used;
            for q in 0..used {
                if let Ok(mut x) = self.queues[q].inner.lock() {
                    x.1.clear();
                }
                let place = s.place_of[q];
                if let Some(slot) = s.queue_of.get_mut(place) {
                    *slot = 0;
                }
                s.place_of[q] = usize::MAX;
            }
            s.used = 0;
        }
    }

    /// Queues in all.
    fn pool(&self) -> usize {
        self.queues.len()
    }

    /// Queues left for parameters not yet changed this block.
    fn free(&self) -> usize {
        self.slots
            .lock()
            .map_or(0, |s| self.queues.len().saturating_sub(s.used))
    }

    /// Whether the parameter at `place` has a queue this block.
    fn has(&self, place: usize) -> bool {
        self.slots
            .lock()
            .is_ok_and(|s| s.queue_of.get(place).is_some_and(|q| *q != 0))
    }

    /// Adds a point for parameter `id` (at `place` in the table) at `offset`: false when
    /// no queue is left for it. A queue full of points takes the latest in its last.
    fn add(&self, place: usize, id: ParamID, offset: int32, value: f64) -> bool {
        let Ok(mut s) = self.slots.lock() else {
            return false;
        };
        let q = match s.queue_of.get(place).copied() {
            Some(0) if s.used < self.queues.len() => {
                let q = s.used;
                s.used += 1;
                s.queue_of[place] = q as u32 + 1;
                s.place_of[q] = place;
                if let Ok(mut x) = self.queues[q].inner.lock() {
                    x.0 = id;
                }
                q
            }
            Some(n) if n != 0 => n as usize - 1,
            _ => return false,
        };
        drop(s);
        if let Ok(mut x) = self.queues[q].inner.lock() {
            if x.1.len() < MAX_POINTS {
                x.1.push((offset, value));
            } else if let Some(last) = x.1.last_mut() {
                *last = (offset.max(last.0), value);
            }
        }
        true
    }
}

impl Changes {
    /// Each queue's latest point (by offset; of points at one offset, the last added), as
    /// (parameter, value): a plugin's output changes (D234). Never allocates.
    fn latest(&self, mut f: impl FnMut(ParamID, f64)) {
        let used = self.slots.lock().map_or(0, |s| s.used);
        for q in &self.queues[..used.min(self.queues.len())] {
            if let Ok(x) = q.inner.lock()
                && let Some(p) = x.1.iter().max_by_key(|p| p.0)
            {
                f(x.0, p.1);
            }
        }
    }
}

impl IParameterChangesTrait for Changes {
    unsafe fn getParameterCount(&self) -> int32 {
        self.slots.lock().map_or(0, |s| s.used as int32)
    }

    unsafe fn getParameterData(&self, index: int32) -> *mut IParamValueQueue {
        let used = self.slots.lock().map_or(0, |s| s.used);
        match self
            .queues
            .get(index.max(0) as usize)
            .filter(|_| (index as usize) < used)
        {
            Some(q) => q
                .as_com_ref::<IParamValueQueue>()
                .map_or(std::ptr::null_mut(), |r| r.as_ptr()),
            None => std::ptr::null_mut(),
        }
    }

    unsafe fn addParameterData(
        &self,
        id: *const ParamID,
        index: *mut int32,
    ) -> *mut IParamValueQueue {
        if id.is_null() {
            return std::ptr::null_mut();
        }
        // SAFETY: checked for null; the plugin passes the parameter's ID.
        let id = unsafe { *id };
        let Ok(mut s) = self.slots.lock() else {
            return std::ptr::null_mut();
        };
        let used = s.used;
        // Its queue by its place (the review's finding: asking each queue in use made a
        // block of many output changes quadratic); a parameter the table lacks, by asking.
        let place = self.ids.binary_search(&id).ok();
        let found = match place {
            Some(p) => s
                .queue_of
                .get(p)
                .and_then(|q| q.checked_sub(1))
                .map(|q| q as usize),
            None => self.queues[..used]
                .iter()
                .position(|q| q.inner.lock().is_ok_and(|q| q.0 == id)),
        };
        let i = match found {
            Some(i) => i,
            None if used < self.queues.len() => {
                s.used += 1;
                if let Some(p) = place {
                    s.queue_of[p] = used as u32 + 1;
                    s.place_of[used] = p;
                }
                if let Ok(mut q) = self.queues[used].inner.lock() {
                    q.0 = id;
                    q.1.clear();
                }
                used
            }
            None => return std::ptr::null_mut(),
        };
        if !index.is_null() {
            // SAFETY: a non-null index pointer is the plugin's to receive.
            unsafe { *index = i as int32 };
        }
        self.queues[i]
            .as_com_ref::<IParamValueQueue>()
            .map_or(std::ptr::null_mut(), |r| r.as_ptr())
    }
}

/// A block's note events (`IEventList`): a preallocated list the audio thread refills.
struct Events {
    list: Mutex<Vec<vst3::Steinberg::Vst::Event>>,
}

impl Class for Events {
    type Interfaces = (IEventList,);
}

impl Events {
    fn new() -> ComWrapper<Events> {
        ComWrapper::new(Events {
            list: Mutex::new(Vec::with_capacity(daw_ipc::MAX_EVENTS)),
        })
    }
}

impl IEventListTrait for Events {
    unsafe fn getEventCount(&self) -> int32 {
        self.list.lock().map_or(0, |l| l.len() as int32)
    }

    unsafe fn getEvent(&self, index: int32, e: *mut vst3::Steinberg::Vst::Event) -> tresult {
        let Ok(l) = self.list.lock() else {
            return kResultFalse;
        };
        match (l.get(index.max(0) as usize), e.is_null()) {
            (Some(x), false) => {
                // SAFETY: checked for null; the plugin passes an event to fill.
                unsafe { *e = *x };
                kResultOk
            }
            _ => kInvalidArgument,
        }
    }

    unsafe fn addEvent(&self, e: *mut vst3::Steinberg::Vst::Event) -> tresult {
        let Ok(mut l) = self.list.lock() else {
            return kResultFalse;
        };
        if e.is_null() || l.len() == l.capacity() {
            return kResultFalse;
        }
        // SAFETY: checked for null; the plugin passes an event to copy.
        l.push(unsafe { *e });
        kResultOk
    }
}

/// What the host gives a plugin's view (`IPlugFrame`): its resize requests, applied on the
/// main thread; on Linux also the host's run loop (`IRunLoop`), which most plug-ins' editors
/// there need (their file descriptors and timers, served on the main thread).
struct Frame {
    resize: Mutex<Option<(u32, u32)>>,
    #[cfg(all(unix, not(target_os = "macos")))]
    run_loop: run_loop::RunLoop,
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
impl Class for Frame {
    type Interfaces = (IPlugFrame,);
}

#[cfg(all(unix, not(target_os = "macos")))]
impl Class for Frame {
    type Interfaces = (IPlugFrame, Linux::IRunLoop);
}

/// The host's run loop on Linux (CA-72 patch, third_party/daw-v3/PATCHES.md): the editor's
/// file descriptors, polled without waiting, and its timers, both served from
/// `gui_service` on the helper's main thread, every 10 ms. A handler is called with no lock
/// held: it may register or unregister handlers as it runs.
#[cfg(all(unix, not(target_os = "macos")))]
mod run_loop {
    use std::sync::Mutex;
    use std::time::{Duration, Instant};
    use vst3::ComPtr;
    use vst3::Steinberg::Linux::*;
    use vst3::Steinberg::*;

    #[derive(Default)]
    pub(super) struct RunLoop {
        fds: Mutex<Vec<(ComPtr<IEventHandler>, FileDescriptor)>>,
        timers: Mutex<Vec<(ComPtr<ITimerHandler>, Duration, Instant)>>,
    }

    fn same<T: vst3::Interface>(a: &ComPtr<T>, b: *mut T) -> bool {
        a.as_ptr() == b
    }

    impl RunLoop {
        /// Serves the descriptors that are ready and the timers that are due.
        pub(super) fn service(&self) {
            let fds: Vec<_> = self.fds.lock().map(|f| f.clone()).unwrap_or_default();
            if !fds.is_empty() {
                let mut polled: Vec<libc::pollfd> = fds
                    .iter()
                    .map(|(_, fd)| libc::pollfd {
                        fd: *fd,
                        events: libc::POLLIN,
                        revents: 0,
                    })
                    .collect();
                #[allow(unsafe_code)]
                // SAFETY: `polled` is a live array of its length; a zero timeout never waits.
                let n = unsafe { libc::poll(polled.as_mut_ptr(), polled.len() as libc::nfds_t, 0) };
                if n > 0 {
                    for ((h, fd), p) in fds.iter().zip(&polled) {
                        if p.revents != 0 {
                            #[allow(unsafe_code)]
                            // SAFETY: the handler is alive (we hold a reference); the main thread.
                            unsafe {
                                h.onFDIsSet(*fd)
                            };
                        }
                    }
                }
            }
            let now = Instant::now();
            let due: Vec<ComPtr<ITimerHandler>> = match self.timers.lock() {
                Ok(mut t) => t
                    .iter_mut()
                    .filter(|(_, _, next)| *next <= now)
                    .map(|(h, every, next)| {
                        *next = now + *every;
                        h.clone()
                    })
                    .collect(),
                Err(_) => Vec::new(),
            };
            for h in due {
                #[allow(unsafe_code)]
                // SAFETY: as above.
                unsafe {
                    h.onTimer()
                };
            }
        }

        /// Drops every handler (the editor closed).
        pub(super) fn clear(&self) {
            let fds = self.fds.lock().map(|mut f| std::mem::take(&mut *f));
            let timers = self.timers.lock().map(|mut t| std::mem::take(&mut *t));
            drop((fds, timers));
        }
    }

    #[allow(unsafe_code)]
    impl IRunLoopTrait for super::Frame {
        unsafe fn registerEventHandler(
            &self,
            handler: *mut IEventHandler,
            fd: FileDescriptor,
        ) -> tresult {
            // SAFETY: the plugin passes a live handler, or null; we take a reference.
            let Some(h) = (unsafe { vst3::ComRef::from_raw(handler) }).map(|r| r.to_com_ptr())
            else {
                return kInvalidArgument;
            };
            if let Ok(mut f) = self.run_loop.fds.lock() {
                f.push((h, fd));
            }
            kResultOk
        }

        unsafe fn unregisterEventHandler(&self, handler: *mut IEventHandler) -> tresult {
            let gone = self
                .run_loop
                .fds
                .lock()
                .map(|mut f| {
                    let mut gone = Vec::new();
                    f.retain(|(h, fd)| {
                        let keep = !same(h, handler);
                        if !keep {
                            gone.push((h.clone(), *fd));
                        }
                        keep
                    });
                    gone
                })
                .unwrap_or_default();
            // Released with no lock held.
            drop(gone);
            kResultOk
        }

        unsafe fn registerTimer(
            &self,
            handler: *mut ITimerHandler,
            milliseconds: TimerInterval,
        ) -> tresult {
            // SAFETY: as for `registerEventHandler`.
            let Some(h) = (unsafe { vst3::ComRef::from_raw(handler) }).map(|r| r.to_com_ptr())
            else {
                return kInvalidArgument;
            };
            let every = Duration::from_millis(milliseconds.max(1));
            if let Ok(mut t) = self.run_loop.timers.lock() {
                t.push((h, every, Instant::now() + every));
            }
            kResultOk
        }

        unsafe fn unregisterTimer(&self, handler: *mut ITimerHandler) -> tresult {
            let gone = self
                .run_loop
                .timers
                .lock()
                .map(|mut t| {
                    let mut gone = Vec::new();
                    t.retain(|(h, _, _)| {
                        let keep = !same(h, handler);
                        if !keep {
                            gone.push(h.clone());
                        }
                        keep
                    });
                    gone
                })
                .unwrap_or_default();
            drop(gone);
            kResultOk
        }
    }
}

impl IPlugFrameTrait for Frame {
    unsafe fn resizeView(&self, _view: *mut IPlugView, size: *mut ViewRect) -> tresult {
        // SAFETY: the plugin passes its new size, or null.
        let Some(r) = (unsafe { size.as_ref() }) else {
            return kInvalidArgument;
        };
        let (w, h) = (
            (r.right - r.left).max(1) as u32,
            (r.bottom - r.top).max(1) as u32,
        );
        if let Ok(mut x) = self.resize.lock() {
            *x = Some((w, h));
        }
        kResultOk
    }
}

/// An open editor: the plugin's view in a window the helper owns.
struct Editor {
    view: ComPtr<IPlugView>,
    frame: ComWrapper<Frame>,
    window: Window,
    title: String,
}

/// The host application (`IHostApplication`), the context plugins are initialized with.
struct Host;

impl Class for Host {
    type Interfaces = (IHostApplication,);
}

impl IHostApplicationTrait for Host {
    unsafe fn getName(&self, name: *mut String128) -> tresult {
        // SAFETY: the plugin passes a `String128` to fill, or null.
        let Some(name) = (unsafe { name.as_mut() }) else {
            return kInvalidArgument;
        };
        for (d, s) in name
            .iter_mut()
            .zip("DAW-v3".encode_utf16().chain(std::iter::once(0)))
        {
            *d = s as TChar;
        }
        kResultOk
    }

    unsafe fn createInstance(
        &self,
        _cid: *mut TUID,
        _iid: *mut TUID,
        obj: *mut *mut c_void,
    ) -> tresult {
        if !obj.is_null() {
            // SAFETY: a non-null out pointer is the plugin's to receive.
            unsafe { *obj = std::ptr::null_mut() };
        }
        kNotImplemented
    }
}

/// Whether `text` is a number alone (no units of its own): digits, a sign, a point.
fn bare(text: &str) -> bool {
    text.chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | ',' | ' '))
}

/// What a plugin's controller tells the host (`IComponentHandler`, `IUnitHandler`):
/// edits (not yet sent back), restart requests, and changes that may rename its keys.
struct Handler {
    restart: Arc<AtomicBool>,
    /// Its names for its keys may have changed (D216).
    names: Arc<AtomicBool>,
    /// Its parameters' values changed by its own doing (D228: a preset loaded in its
    /// editor, `restartComponent(kParamValuesChanged)`): read again for the project.
    values: Arc<AtomicBool>,
    /// Something besides its parameters changed (D234: `setDirty`).
    dirty: AtomicBool,
    /// Its parameters' names or list changed (`restartComponent(kParamTitlesChanged)`):
    /// listed again by the host (PATCHES.md change 6).
    titles: AtomicBool,
    /// The controller's program changes, whose edits may rename its keys (D216); set once
    /// the controller is ready.
    programs: std::sync::OnceLock<Vec<u32>>,
    /// The editor's edits, for the project (D222), until taken.
    edits: Mutex<Vec<ParamEdit>>,
    /// The editor's values, for the processor's next block (D222): the latest of each.
    to_processor: Arc<Mutex<Vec<(u32, f64)>>>,
}

impl Handler {
    fn edit(&self, param: ParamID, kind: EditKind, value: Option<f64>) {
        if let Ok(mut e) = self.edits.lock() {
            e.push(ParamEdit { param, kind, value });
        }
    }
}

impl Class for Handler {
    type Interfaces = (
        IComponentHandler,
        IComponentHandler2,
        IUnitHandler,
        IUnitHandler2,
    );
}

/// What a controller says besides its edits (D234): `setDirty`, its word that something
/// besides its parameters changed (a sample loaded, a preset): read again for the project,
/// its state with its values.
impl IComponentHandler2Trait for Handler {
    unsafe fn setDirty(&self, state: TBool) -> tresult {
        if state != 0 {
            self.dirty.store(true, Ordering::Release);
            self.values.store(true, Ordering::Release);
        }
        kResultOk
    }

    /// Not done: the host opens a plugin's editor when a person asks for it.
    unsafe fn requestOpenEditor(&self, _name: FIDString) -> tresult {
        kResultFalse
    }

    /// A group of edits shares a time (not kept: each gesture is its own step, D222).
    unsafe fn startGroupEdit(&self) -> tresult {
        kResultOk
    }

    unsafe fn finishGroupEdit(&self) -> tresult {
        kResultOk
    }
}

/// The plugin's editor's edits (D222): each value goes to the processor in its next block,
/// as `IComponentHandler` is for ("transfer of parameter editing to component via host"),
/// and with the gesture's beginning and end to the project.
impl IComponentHandlerTrait for Handler {
    unsafe fn beginEdit(&self, id: ParamID) -> tresult {
        self.edit(id, EditKind::Begin, None);
        kResultOk
    }

    /// An edit in the plugin's editor; of a program change, its names are asked again
    /// (D216).
    unsafe fn performEdit(&self, id: ParamID, value: ParamValue) -> tresult {
        if self.programs.get().is_none_or(|p| p.contains(&id)) {
            self.names.store(true, Ordering::Release);
        }
        let value = value.clamp(0.0, 1.0);
        {
            let mut q = crate::lock_soon(&self.to_processor);
            match q.iter_mut().find(|x| x.0 == id) {
                Some(x) => x.1 = value,
                None => q.push((id, value)),
            }
        }
        self.edit(id, EditKind::Value, Some(value));
        kResultOk
    }

    unsafe fn endEdit(&self, id: ParamID) -> tresult {
        self.edit(id, EditKind::End, None);
        kResultOk
    }

    unsafe fn restartComponent(&self, flags: int32) -> tresult {
        use RestartFlags_::{
            kIoChanged, kLatencyChanged, kParamTitlesChanged, kParamValuesChanged, kReloadComponent,
        };
        if flags & (kReloadComponent | kIoChanged | kLatencyChanged) != 0 {
            self.restart.store(true, Ordering::Relaxed);
        }
        // Values or titles changed (a program loaded): the names may have too (D216).
        if flags & (kReloadComponent | kParamValuesChanged | kParamTitlesChanged) != 0 {
            self.names.store(true, Ordering::Release);
        }
        // Values changed (a preset loaded): read again into the project (D228).
        if flags & (kReloadComponent | kParamValuesChanged) != 0 {
            self.values.store(true, Ordering::Release);
        }
        // Names or the list changed: listed again by the host (PATCHES.md change 6).
        if flags & (kReloadComponent | kParamTitlesChanged) != 0 {
            self.titles.store(true, Ordering::Release);
        }
        kResultOk
    }
}

impl IUnitHandler2Trait for Handler {
    /// The units the buses' channels play changed (D216): the note input's may be another,
    /// so the names are asked again.
    unsafe fn notifyUnitByBusChange(&self) -> tresult {
        self.names.store(true, Ordering::Release);
        kResultOk
    }
}

impl IUnitHandlerTrait for Handler {
    /// Another unit chosen in the plugin (D216): the names are asked again.
    unsafe fn notifyUnitSelection(&self, _unit: UnitID) -> tresult {
        self.names.store(true, Ordering::Release);
        kResultOk
    }

    /// A program list changed, its pitch names with it (the SDK's word for "pitch names
    /// changed", D216): the names are asked again.
    unsafe fn notifyProgramListChange(&self, _list: ProgramListID, _program: int32) -> tresult {
        self.names.store(true, Ordering::Release);
        kResultOk
    }
}

/// The values a processor was sent, for the controller (D216): the audio thread records
/// each parameter's latest by ID without allocating or waiting, and the main thread hands
/// them to the controller (`setParamNormalized`), as VST3 asks of a host.
struct Mirror {
    /// Parameter IDs, sorted.
    ids: Vec<u32>,
    /// By index in `ids`: the latest value's bits, and whether it is new.
    values: Vec<std::sync::atomic::AtomicU64>,
    fresh: Vec<AtomicBool>,
    /// Some value is new.
    any: AtomicBool,
    /// The program changes (`kIsProgramChange`): ID and step count, to ask for names
    /// only when one moves to another program.
    programs: Vec<(u32, i32)>,
    /// By index in `ids`: shown, not taken (`kIsReadOnly`: a meter), for the processor's
    /// own output values (D234); and the value last handed to the controller (main thread;
    /// `u64::MAX`: none).
    read_only: Vec<bool>,
    seen: Vec<std::sync::atomic::AtomicU64>,
}

impl Mirror {
    /// Records `value` for parameter `id` (audio thread).
    fn set(&self, id: u32, value: f64) {
        if let Ok(i) = self.ids.binary_search(&id) {
            self.values[i].store(value.to_bits(), Ordering::Relaxed);
            self.fresh[i].store(true, Ordering::Release);
            self.any.store(true, Ordering::Release);
        }
    }

    /// The program a program change's value stands for (VST3's discrete value), or the
    /// value itself for one without steps.
    fn program(steps: i32, value: f64) -> i64 {
        let v = value.clamp(0.0, 1.0);
        if steps > 0 {
            ((v * (f64::from(steps) + 1.0)) as i64).min(i64::from(steps))
        } else {
            (v * 1e9) as i64
        }
    }
}

// ---------------------------------------------------------------------------------------
// Modules

/// A loaded VST3 module.
pub struct Module {
    factory: ComPtr<IPluginFactory>,
    path: PathBuf,
    // Dropped last: the factory lives in it.
    lib: libloading::Library,
}

impl std::fmt::Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module").field("path", &self.path).finish()
    }
}

// SAFETY: a VST3 factory may be used from any thread (the SDK's hosts share it).
unsafe impl Send for Module {}
// SAFETY: as above.
unsafe impl Sync for Module {}

type EntryFn = unsafe extern "system" fn(*mut c_void) -> bool;
type InitFn = unsafe extern "system" fn() -> bool;
type FactoryFn = unsafe extern "system" fn() -> *mut IPluginFactory;

impl Module {
    pub fn load(path: &Path) -> Result<Module, String> {
        let bin = binary_path(path);
        // SAFETY: loading a plugin library runs its initialisers; that is the point of the
        // helper process, which isolates whatever they do from the DAW.
        let lib = unsafe { libloading::Library::new(&bin) }
            .map_err(|e| format!("cannot load {}: {e}", bin.display()))?;
        // The module's own entry point, where the platform has one.
        let entry = if cfg!(windows) {
            // SAFETY: `InitDll` takes nothing and returns a bool, per the VST3 module ABI.
            unsafe { lib.get::<InitFn>(b"InitDll\0") }
                .ok()
                // SAFETY: calling the module's initialiser once.
                .map(|f| unsafe { f() })
        } else {
            let name: &[u8] = if cfg!(target_os = "macos") {
                b"BundleEntry\0"
            } else {
                b"ModuleEntry\0"
            };
            // SAFETY: the entry takes a handle (the library's, or the bundle's; a null one
            // for plugins that do not need it) and returns a bool, per the VST3 module ABI.
            unsafe { lib.get::<EntryFn>(name) }
                .ok()
                // SAFETY: calling the module's entry once.
                .map(|f| unsafe { f(std::ptr::null_mut()) })
        };
        if entry == Some(false) {
            return Err(format!(
                "{}: the module's entry point failed",
                bin.display()
            ));
        }
        // SAFETY: `GetPluginFactory` takes nothing and returns the factory, per the ABI.
        let get = unsafe { lib.get::<FactoryFn>(b"GetPluginFactory\0") }
            .map_err(|e| format!("{} is not a VST3 module: {e}", bin.display()))?;
        // SAFETY: calling the factory function; it returns an owned reference or null.
        let raw = unsafe { get() };
        // SAFETY: a non-null factory is a live COM object whose reference we now own.
        let factory = unsafe { ComPtr::from_raw(raw) }
            .ok_or_else(|| format!("{}: no plugin factory", bin.display()))?;
        Ok(Module {
            factory,
            path: path.to_path_buf(),
            lib,
        })
    }

    /// The plugins in the module: its audio processor classes.
    pub fn descriptors(&self) -> Vec<PluginDesc> {
        // SAFETY: an all-zero factory info is a valid value to be overwritten.
        let mut finfo: PFactoryInfo = unsafe { std::mem::zeroed() };
        // SAFETY: `finfo` is writable.
        let vendor = if unsafe { self.factory.getFactoryInfo(&mut finfo) } == kResultOk {
            cstr(&finfo.vendor)
        } else {
            String::new()
        };
        let factory2 = self.factory.cast::<IPluginFactory2>();
        // SAFETY: counting the factory's classes.
        let n = unsafe { self.factory.countClasses() };
        let mut out = Vec::new();
        for i in 0..n {
            // SAFETY: an all-zero class info is a valid value to be overwritten.
            let mut info: PClassInfo = unsafe { std::mem::zeroed() };
            // SAFETY: `info` is writable; `i` is below the count.
            if unsafe { self.factory.getClassInfo(i, &mut info) } != kResultOk
                || cstr(&info.category) != "Audio Module Class"
            {
                continue;
            }
            let mut desc = PluginDesc {
                id: cid_hex(&info.cid),
                name: cstr(&info.name),
                vendor: vendor.clone(),
                ..PluginDesc::default()
            };
            if let Some(f2) = &factory2 {
                // SAFETY: an all-zero class info is a valid value to be overwritten.
                let mut info2: PClassInfo2 = unsafe { std::mem::zeroed() };
                // SAFETY: `info2` is writable; `i` is below the count.
                if unsafe { f2.getClassInfo2(i, &mut info2) } == kResultOk {
                    let v = cstr(&info2.vendor);
                    if !v.is_empty() {
                        desc.vendor = v;
                    }
                    desc.version = cstr(&info2.version);
                    desc.features = cstr(&info2.subCategories)
                        .split('|')
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned)
                        .collect();
                }
            }
            out.push(desc);
        }
        out
    }

    fn create<I: Interface>(&self, cid: &TUID) -> Option<ComPtr<I>> {
        let mut obj: *mut c_void = std::ptr::null_mut();
        // SAFETY: asking the factory for a new object of class `cid` with interface `I`;
        // `obj` receives an owned reference.
        let r = unsafe {
            self.factory
                .createInstance(cid.as_ptr(), I::IID.as_ptr() as FIDString, &mut obj)
        };
        if r != kResultOk {
            return None;
        }
        // SAFETY: on success `obj` is a live object with interface `I`, and we own it.
        unsafe { ComPtr::from_raw(obj.cast::<I>()) }
    }

    /// Makes an instance of the plugin `id` (its processor's class ID in hex).
    pub fn instantiate(self: &Arc<Module>, id: &str) -> Result<Instance, String> {
        let cid = cid_parse(id).ok_or_else(|| format!("`{id}` is not a VST3 class ID"))?;
        let component = self
            .create::<IComponent>(&cid)
            .ok_or_else(|| format!("{}: cannot create {id}", self.path.display()))?;
        let host = ComWrapper::new(Host);
        let context = host
            .as_com_ref::<IHostApplication>()
            .and_then(|h| h.cast::<FUnknown>())
            .ok_or("no host context")?;
        // SAFETY: initializing the new component once with the host context.
        if unsafe { component.initialize(context.as_ptr()) } != kResultOk {
            return Err(format!("{id}: the component failed to initialize"));
        }
        let processor = component
            .cast::<IAudioProcessor>()
            .ok_or_else(|| format!("{id} is not an audio processor"))?;
        // The controller: its own class, or the component itself.
        let mut ccid: TUID = [0; 16];
        // SAFETY: `ccid` is writable.
        let separate = unsafe { component.getControllerClassId(&mut ccid) } == kResultOk
            && ccid != [0; 16]
            && ccid != cid;
        let controller = if separate {
            let c = self.create::<IEditController>(&ccid);
            if let Some(c) = &c {
                // SAFETY: initializing the new controller once with the host context.
                if unsafe { c.initialize(context.as_ptr()) } != kResultOk {
                    return Err(format!("{id}: the controller failed to initialize"));
                }
            }
            c
        } else {
            component.cast::<IEditController>()
        };
        let restart = Arc::new(AtomicBool::new(false));
        let names = Arc::new(AtomicBool::new(false));
        let values = Arc::new(AtomicBool::new(false));
        let edits_in = Arc::new(Mutex::new(Vec::with_capacity(EDITS_PER_BLOCK)));
        let handler = ComWrapper::new(Handler {
            restart: restart.clone(),
            names: names.clone(),
            values: values.clone(),
            dirty: AtomicBool::new(false),
            titles: AtomicBool::new(false),
            programs: std::sync::OnceLock::new(),
            edits: Mutex::new(Vec::new()),
            to_processor: edits_in.clone(),
        });
        let mut inst = Instance {
            module: self.clone(),
            id: id.to_owned(),
            component,
            processor,
            controller,
            separate,
            host,
            handler,
            restart,
            names,
            values,
            mirror: Mutex::new(None),
            outputs: Mutex::new(None),
            given: Mutex::new(Vec::new()),
            edits_in,
            host_values: Mutex::new(std::collections::BTreeMap::new()),
            open_gestures: Mutex::new(Vec::new()),
            active: false,
            pending: Mutex::new(Vec::new()),
            editor: None,
        };
        inst.connect();
        if let Some(c) = &inst.controller
            && let Some(h) = inst.handler.as_com_ref::<IComponentHandler>()
        {
            // SAFETY: handing the controller our handler, which outlives it (the instance
            // holds both).
            unsafe { c.setComponentHandler(h.as_ptr()) };
        }
        // The controller learns the processor's state.
        if let Ok(state) = inst.component_state() {
            inst.controller_component_state(&state);
        }
        let programs = inst.program_changes().into_iter().map(|p| p.0).collect();
        let _ = inst.handler.programs.set(programs);
        Ok(inst)
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        let name: &[u8] = if cfg!(windows) {
            b"ExitDll\0"
        } else if cfg!(target_os = "macos") {
            b"BundleExit\0"
        } else {
            b"ModuleExit\0"
        };
        // SAFETY: the module's exit takes nothing and returns a bool; called once, after
        // every object from it is gone (instances hold the module).
        if let Ok(f) = unsafe { self.lib.get::<InitFn>(name) } {
            // SAFETY: as above.
            unsafe { f() };
        }
    }
}

// ---------------------------------------------------------------------------------------
// Instances

/// A plugin instance: its processor and controller.
pub struct Instance {
    id: String,
    component: ComPtr<IComponent>,
    processor: ComPtr<IAudioProcessor>,
    controller: Option<ComPtr<IEditController>>,
    separate: bool,
    host: ComWrapper<Host>,
    handler: ComWrapper<Handler>,
    restart: Arc<AtomicBool>,
    /// Its names for its keys may have changed (D216): the handler's, and the host's own
    /// program changes.
    names: Arc<AtomicBool>,
    /// Its parameters' values changed by its own doing (D228): the handler's.
    values: Arc<AtomicBool>,
    /// The values its processor is sent, for the controller (D216); made with it.
    mirror: Mutex<Option<Arc<Mirror>>>,
    /// The values its processor reports itself (`outputParameterChanges`, D234), for the
    /// controller; made with it.
    outputs: Mutex<Option<Arc<Mirror>>>,
    /// The parameters the host gave the controller a value since the last call of
    /// [`Instance::take_host_given`] (D234). Main thread.
    given: Mutex<Vec<u32>>,
    /// The editor's values for the processor (D222), shared with the handler.
    edits_in: Arc<Mutex<Vec<(u32, f64)>>>,
    /// The values the host last gave the controller, and the editor's gestures open, to
    /// tell its edits from its answers (D222). Main thread.
    host_values: Mutex<std::collections::BTreeMap<u32, f64>>,
    open_gestures: Mutex<Vec<u32>>,
    active: bool,
    /// Parameter values set while not processing, for the next block.
    pending: Mutex<Vec<(u32, f64)>>,
    editor: Option<Editor>,
    // Dropped last.
    module: Arc<Module>,
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instance").field("id", &self.id).finish()
    }
}

// SAFETY: an instance is used from the helper's main thread only; the processor it hands
// out goes to the audio thread (VST3's threading model).
unsafe impl Send for Instance {}

impl Instance {
    fn connect(&mut self) {
        if !self.separate {
            return;
        }
        let (Some(a), Some(b)) = (
            self.component.cast::<IConnectionPoint>(),
            self.controller
                .as_ref()
                .and_then(|c| c.cast::<IConnectionPoint>()),
        ) else {
            return;
        };
        // SAFETY: connecting the two halves of one plugin, which both live as long as the
        // instance; disconnected on drop.
        unsafe {
            a.connect(b.as_ptr());
            b.connect(a.as_ptr());
        }
    }

    pub fn descriptor(&self) -> PluginDesc {
        self.module
            .descriptors()
            .into_iter()
            .find(|d| d.id == self.id)
            .unwrap_or_default()
    }

    /// The controller's parameters, as normalized values.
    pub fn params(&self) -> Vec<ParamInfo> {
        let Some(c) = &self.controller else {
            return Vec::new();
        };
        // The unit whose program names the keys: its program change chooses them (D216).
        let names_unit = self
            .unit_info()
            .and_then(|u| Self::names_unit(&u))
            .map(|(unit, _)| unit);
        // SAFETY: parameter queries on the main thread.
        let n = unsafe { c.getParameterCount() };
        let mut out = Vec::new();
        for i in 0..n {
            // SAFETY: an all-zero parameter info is a valid value to be overwritten.
            let mut info: ParameterInfo = unsafe { std::mem::zeroed() };
            // SAFETY: `info` is writable; `i` is below the count.
            if unsafe { c.getParameterInfo(i, &mut info) } != kResultOk {
                continue;
            }
            let flags = info.flags as u32;
            let flag = |f: u32| flags & f != 0;
            out.push(ParamInfo {
                id: info.id,
                name: wstr(&info.title),
                short: wstr(&info.shortTitle),
                units: wstr(&info.units),
                steps: info.stepCount.max(0) as u32,
                read_only: flag(ParameterInfo_::ParameterFlags_::kIsReadOnly as u32),
                bypass: flag(ParameterInfo_::ParameterFlags_::kIsBypass as u32),
                module: String::new(),
                min: 0.0,
                max: 1.0,
                default: info.defaultNormalizedValue,
                stepped: false,
                hidden: flags & ParameterInfo_::ParameterFlags_::kIsHidden as u32 != 0,
                program: flags & ParameterInfo_::ParameterFlags_::kIsProgramChange as u32 != 0
                    && names_unit == Some(info.unitId),
                automatable: flags & ParameterInfo_::ParameterFlags_::kCanAutomate as u32 != 0,
                // VST3 has no modulation of its own: modulating overwrites (DESIGN 11.4).
                modulatable: false,
                // SAFETY: parameter queries on the main thread.
                value: Some(unsafe { c.getParamNormalized(info.id) }),
                // For the parameters a person or automation moves (a plugin may list
                // thousands that stand for MIDI controllers).
                plain: (0..=daw_ipc::protocol::PLAIN_STEPS)
                    .filter(|_| flags & ParameterInfo_::ParameterFlags_::kCanAutomate as u32 != 0)
                    .map(|k| {
                        let x = k as f64 / daw_ipc::protocol::PLAIN_STEPS as f64;
                        // SAFETY: as above.
                        unsafe { c.normalizedParamToPlain(info.id, x) }
                    })
                    .collect(),
            });
        }
        out
    }

    /// Channel counts of the first audio input and output buses.
    pub fn main_ports(&self) -> (u32, u32) {
        let count = |dir: BusDirections| -> u32 {
            // SAFETY: bus queries on the main thread.
            let n = unsafe {
                self.component
                    .getBusCount(MediaTypes_::kAudio as MediaType, dir as BusDirection)
            };
            if n < 1 {
                return 0;
            }
            // SAFETY: an all-zero bus info is a valid value to be overwritten.
            let mut info: BusInfo = unsafe { std::mem::zeroed() };
            // SAFETY: `info` is writable; bus 0 exists.
            let ok = unsafe {
                self.component.getBusInfo(
                    MediaTypes_::kAudio as MediaType,
                    dir as BusDirection,
                    0,
                    &mut info,
                )
            };
            if ok == kResultOk {
                info.channelCount.max(0) as u32
            } else {
                0
            }
        };
        (
            count(BusDirections_::kInput),
            count(BusDirections_::kOutput),
        )
    }

    /// Whether it takes notes (an event input bus).
    pub fn takes_notes(&self) -> bool {
        // SAFETY: bus query on the main thread.
        unsafe {
            self.component.getBusCount(
                MediaTypes_::kEvent as MediaType,
                BusDirections_::kInput as BusDirection,
            ) > 0
        }
    }

    /// The plugin's names for its keys (D211): VST3's program pitch names
    /// (`IUnitInfo::getProgramPitchName`), keys 0 to 127, of the program the note input
    /// plays. The unit is the one the plugin relates to the event input's first channel
    /// (`getUnitByBus`), else the root unit; the program list is that unit's, else the
    /// root unit's; the program is the value of that unit's program-change parameter
    /// (`kIsProgramChange`), else the list's first. A plugin without units, a program
    /// list or pitch names for that program gives none.
    pub fn note_names(&self) -> Vec<NoteName> {
        let Some(units) = self.unit_info() else {
            return Vec::new();
        };
        let Some((unit, list)) = Self::names_unit(&units) else {
            return Vec::new();
        };
        self.pitch_names(&units, unit, list)
    }

    /// The controller's `IUnitInfo` (it extends `IEditController`); a plugin of one class
    /// may give it on its component.
    fn unit_info(&self) -> Option<ComPtr<IUnitInfo>> {
        self.controller
            .as_ref()
            .and_then(|c| c.cast::<IUnitInfo>())
            .or_else(|| self.component.cast::<IUnitInfo>())
    }

    /// The unit whose program names the keys (D211), and its program list: the event
    /// input's first channel's unit, else the root unit.
    fn names_unit(units: &ComPtr<IUnitInfo>) -> Option<(UnitID, ProgramListID)> {
        let mut unit: UnitID = kRootUnitId;
        // SAFETY: a unit query on the main thread; `unit` is ours to fill.
        let related = unsafe {
            units.getUnitByBus(
                MediaTypes_::kEvent as MediaType,
                BusDirections_::kInput as BusDirection,
                0,
                0,
                &mut unit,
            )
        } == kResultOk;
        if !related {
            unit = kRootUnitId;
        }
        // SAFETY: a unit query on the main thread.
        let units_n = unsafe { units.getUnitCount() }.clamp(0, 4096);
        let list_of = |id: UnitID| -> Option<ProgramListID> {
            (0..units_n).find_map(|i| {
                // SAFETY: an all-zero unit info is a valid value to be overwritten.
                let mut info: UnitInfo = unsafe { std::mem::zeroed() };
                // SAFETY: `info` is writable; `i` is below the count.
                let ok = unsafe { units.getUnitInfo(i, &mut info) } == kResultOk;
                (ok && info.id == id && info.programListId != kNoProgramListId)
                    .then_some(info.programListId)
            })
        };
        match list_of(unit) {
            Some(l) => Some((unit, l)),
            None if unit != kRootUnitId => list_of(kRootUnitId).map(|l| (kRootUnitId, l)),
            None => None,
        }
    }

    /// The pitch names of the program `unit` plays from `list`.
    fn pitch_names(
        &self,
        units: &ComPtr<IUnitInfo>,
        unit: UnitID,
        list: ProgramListID,
    ) -> Vec<NoteName> {
        // SAFETY: a program list query on the main thread.
        let lists_n = unsafe { units.getProgramListCount() }.clamp(0, 4096);
        let count = (0..lists_n)
            .find_map(|i| {
                // SAFETY: an all-zero list info is a valid value to be overwritten.
                let mut info: ProgramListInfo = unsafe { std::mem::zeroed() };
                // SAFETY: `info` is writable; `i` is below the count.
                let ok = unsafe { units.getProgramListInfo(i, &mut info) } == kResultOk;
                (ok && info.id == list).then_some(info.programCount)
            })
            .unwrap_or(0);
        if count < 1 {
            return Vec::new();
        }
        let program = self.program(unit, count);
        // SAFETY: a program query on the main thread.
        if unsafe { units.hasProgramPitchNames(list, program) } != kResultTrue {
            return Vec::new();
        }
        // The root unit's names are every channel's; a unit of its own, the first's.
        let channel = if unit == kRootUnitId { -1 } else { 0 };
        (0..128i16)
            .filter_map(|key| {
                let mut name: String128 = [0; 128];
                // SAFETY: `name` is writable; the list, program and key are in range.
                let ok = unsafe { units.getProgramPitchName(list, program, key, &mut name) }
                    == kResultOk;
                let name = wstr(&name);
                (ok && !name.trim().is_empty()).then_some(NoteName { key, channel, name })
            })
            .collect()
    }

    /// The program `unit` plays now, of a list of `count`: the value of its
    /// program-change parameter as VST3 makes a discrete value of a normalized one
    /// (`min(stepCount, value * (stepCount + 1))`), else the first.
    fn program(&self, unit: UnitID, count: i32) -> i32 {
        let Some(c) = &self.controller else {
            return 0;
        };
        // SAFETY: parameter queries on the main thread.
        let n = unsafe { c.getParameterCount() };
        for i in 0..n {
            // SAFETY: an all-zero parameter info is a valid value to be overwritten.
            let mut info: ParameterInfo = unsafe { std::mem::zeroed() };
            // SAFETY: `info` is writable; `i` is below the count.
            if unsafe { c.getParameterInfo(i, &mut info) } != kResultOk
                || info.flags as u32 & ParameterInfo_::ParameterFlags_::kIsProgramChange as u32 == 0
                || info.unitId != unit
            {
                continue;
            }
            // SAFETY: as above.
            let v = unsafe { c.getParamNormalized(info.id) }.clamp(0.0, 1.0);
            let steps = if info.stepCount > 0 {
                info.stepCount
            } else {
                count - 1
            };
            let index = ((v * (f64::from(steps) + 1.0)) as i32).min(steps);
            return index.clamp(0, count - 1);
        }
        0
    }

    fn component_state(&self) -> Result<Vec<u8>, String> {
        let s = Stream::new(Vec::new());
        let stream = s.as_com_ref::<IBStream>().ok_or("no stream")?;
        // SAFETY: saving into our stream, which outlives the call.
        if unsafe { self.component.getState(stream.as_ptr()) } != kResultOk {
            return Err("the plugin failed to save its state".into());
        }
        Ok(s.bytes())
    }

    fn controller_component_state(&self, bytes: &[u8]) {
        if let Some(c) = &self.controller {
            let s = Stream::new(bytes.to_vec());
            if let Some(stream) = s.as_com_ref::<IBStream>() {
                // SAFETY: the controller reads our stream, which outlives the call.
                unsafe { c.setComponentState(stream.as_ptr()) };
            }
        }
    }

    /// The processor's and controller's state in one blob.
    pub fn save_state(&self) -> Result<Vec<u8>, String> {
        let component = self.component_state()?;
        let mut controller = Vec::new();
        if self.separate
            && let Some(c) = &self.controller
        {
            let s = Stream::new(Vec::new());
            if let Some(stream) = s.as_com_ref::<IBStream>() {
                // SAFETY: saving into our stream, which outlives the call.
                if unsafe { c.getState(stream.as_ptr()) } == kResultOk {
                    controller = s.bytes();
                }
            }
        }
        let mut out = b"VST3".to_vec();
        for part in [&component, &controller] {
            out.extend_from_slice(&(part.len() as u64).to_le_bytes());
            out.extend_from_slice(part);
        }
        Ok(out)
    }

    pub fn load_state(&self, bytes: &[u8]) -> Result<(), String> {
        let bad = || "not a VST3 state from this host".to_owned();
        let mut rest = bytes.strip_prefix(b"VST3").ok_or_else(bad)?;
        let mut parts = Vec::new();
        for _ in 0..2 {
            let len = rest.get(..8).ok_or_else(bad)?;
            let len = u64::from_le_bytes(len.try_into().map_err(|_| bad())?) as usize;
            let body = rest.get(8..8 + len).ok_or_else(bad)?;
            parts.push(body.to_vec());
            rest = &rest[8 + len..];
        }
        let s = Stream::new(parts[0].clone());
        let stream = s.as_com_ref::<IBStream>().ok_or("no stream")?;
        // SAFETY: the component reads our stream, which outlives the call.
        if unsafe { self.component.setState(stream.as_ptr()) } != kResultOk {
            return Err("the plugin rejected the state".into());
        }
        self.controller_component_state(&parts[0]);
        if self.separate
            && !parts[1].is_empty()
            && let Some(c) = &self.controller
        {
            let s = Stream::new(parts[1].clone());
            if let Some(stream) = s.as_com_ref::<IBStream>() {
                // SAFETY: the controller reads our stream, which outlives the call.
                unsafe { c.setState(stream.as_ptr()) };
            }
        }
        Ok(())
    }

    /// Sets parameters (normalized) while not processing: on the controller now, and on
    /// the processor at the start of the next block.
    pub fn flush_params(&self, params: &[(u32, f64)]) {
        if let Some(c) = &self.controller {
            for (id, v) in params {
                self.host_gave(*id, *v);
                // SAFETY: setting a controller value on the main thread.
                unsafe { c.setParamNormalized(*id, *v) };
            }
        }
        if let Ok(mut p) = self.pending.lock() {
            p.extend_from_slice(params);
        }
    }

    pub fn activate(&mut self, sample_rate: f64, max_block: u32) -> Result<(), String> {
        let (ins, outs) = self.main_ports();
        let mut in_arr = [SpeakerArr::kStereo];
        let mut out_arr = [SpeakerArr::kStereo];
        // SAFETY: arrangements for the main buses, on the main thread before activation.
        unsafe {
            self.processor.setBusArrangements(
                in_arr.as_mut_ptr(),
                i32::from(ins > 0),
                out_arr.as_mut_ptr(),
                i32::from(outs > 0),
            );
        }
        let mut setup = ProcessSetup {
            processMode: ProcessModes_::kRealtime as int32,
            symbolicSampleSize: SymbolicSampleSizes_::kSample32 as int32,
            maxSamplesPerBlock: max_block as int32,
            sampleRate: sample_rate,
        };
        // SAFETY: setting up processing before activation, on the main thread.
        if unsafe { self.processor.setupProcessing(&mut setup) } != kResultOk {
            return Err("the plugin refused the processing setup".into());
        }
        if self.takes_notes() {
            // SAFETY: activating the note input bus, on the main thread before activation.
            unsafe {
                self.component.activateBus(
                    MediaTypes_::kEvent as MediaType,
                    BusDirections_::kInput as BusDirection,
                    0,
                    1,
                )
            };
        }
        for (dir, n) in [
            (BusDirections_::kInput, ins),
            (BusDirections_::kOutput, outs),
        ] {
            if n > 0 {
                // SAFETY: activating the main audio bus.
                unsafe {
                    self.component.activateBus(
                        MediaTypes_::kAudio as MediaType,
                        dir as BusDirection,
                        0,
                        1,
                    )
                };
            }
        }
        // SAFETY: activating on the main thread.
        if unsafe { self.component.setActive(1) } != kResultOk {
            return Err("the plugin failed to activate".into());
        }
        self.active = true;
        // The editor's values from before (a controller answering the values it was set
        // with `performEdit`) are not sent, as its edits then are not (the review's finding).
        crate::lock_soon(&self.edits_in).clear();
        // Values set before activation reach the processor now, in a block of no samples
        // (VST3's parameter flush); a latency they change needs activating again.
        let pending = self
            .pending
            .lock()
            .map(|mut p| std::mem::take(&mut *p))
            .unwrap_or_default();
        if !pending.is_empty() {
            let before = self.latency();
            self.flush(&pending);
            if self.latency() != before {
                // SAFETY: reactivating on the main thread, as a latency change asks.
                unsafe {
                    self.component.setActive(0);
                    self.component.setActive(1);
                }
            }
        }
        Ok(())
    }

    /// Sends parameter values to the processor in a block of no samples: every one, however
    /// many (D222).
    fn flush(&self, values: &[(u32, f64)]) {
        let mut ids = self.param_ids();
        ids.sort_unstable();
        ids.dedup();
        let changes = Changes::new(&ids, ids.len().min(values.len()));
        for (id, v) in values {
            if let Ok(place) = ids.binary_search(id) {
                changes.add(place, *id, 0, *v);
            }
        }
        let Some(c) = changes.as_com_ref::<IParameterChanges>() else {
            return;
        };
        let mut data = ProcessData {
            processMode: ProcessModes_::kRealtime as int32,
            symbolicSampleSize: SymbolicSampleSizes_::kSample32 as int32,
            numSamples: 0,
            numInputs: 0,
            numOutputs: 0,
            inputs: std::ptr::null_mut(),
            outputs: std::ptr::null_mut(),
            inputParameterChanges: c.as_ptr(),
            outputParameterChanges: std::ptr::null_mut(),
            inputEvents: std::ptr::null_mut(),
            outputEvents: std::ptr::null_mut(),
            processContext: std::ptr::null_mut(),
        };
        // SAFETY: a flush on the main thread while no audio thread processes: processing
        // on, one call with no buses and our parameter changes, processing off.
        unsafe {
            self.processor.setProcessing(1);
            self.processor.process(&mut data);
            self.processor.setProcessing(0);
        }
    }

    pub fn deactivate(&mut self) {
        if self.active {
            // SAFETY: deactivating on the main thread after processing stopped.
            unsafe { self.component.setActive(0) };
            self.active = false;
        }
    }

    pub fn latency(&self) -> u32 {
        // SAFETY: latency query on the main thread.
        unsafe { self.processor.getLatencySamples() }
    }

    pub fn tail(&self) -> u32 {
        // SAFETY: tail query on the main thread.
        unsafe { self.processor.getTailSamples() }
    }

    /// Whether the plugin asked for a restart (for example after a latency change).
    pub fn restart_requested(&self) -> bool {
        self.restart.swap(false, Ordering::Relaxed)
    }

    /// The controller's parameter IDs, in its order.
    fn param_ids(&self) -> Vec<u32> {
        let Some(c) = &self.controller else {
            return Vec::new();
        };
        // SAFETY: parameter queries on the main thread.
        let n = unsafe { c.getParameterCount() };
        (0..n)
            .filter_map(|i| {
                // SAFETY: an all-zero parameter info is a valid value to be overwritten.
                let mut info: ParameterInfo = unsafe { std::mem::zeroed() };
                // SAFETY: `info` is writable; `i` is below the count.
                (unsafe { c.getParameterInfo(i, &mut info) } == kResultOk).then_some(info.id)
            })
            .collect()
    }

    /// The controller's program changes (`kIsProgramChange`): ID and step count.
    fn program_changes(&self) -> Vec<(u32, i32)> {
        let Some(c) = &self.controller else {
            return Vec::new();
        };
        // SAFETY: parameter queries on the main thread.
        let n = unsafe { c.getParameterCount() };
        (0..n)
            .filter_map(|i| {
                // SAFETY: an all-zero parameter info is a valid value to be overwritten.
                let mut info: ParameterInfo = unsafe { std::mem::zeroed() };
                // SAFETY: `info` is writable; `i` is below the count.
                let ok = unsafe { c.getParameterInfo(i, &mut info) } == kResultOk;
                (ok && info.flags as u32 & ParameterInfo_::ParameterFlags_::kIsProgramChange as u32
                    != 0)
                    .then_some((info.id, info.stepCount))
            })
            .collect()
    }

    /// The controller's parameters it shows but does not take (`kIsReadOnly`: meters).
    fn read_only_ids(&self) -> Vec<u32> {
        let Some(c) = &self.controller else {
            return Vec::new();
        };
        // SAFETY: parameter queries on the main thread.
        let n = unsafe { c.getParameterCount() };
        (0..n)
            .filter_map(|i| {
                // SAFETY: an all-zero parameter info is a valid value to be overwritten.
                let mut info: ParameterInfo = unsafe { std::mem::zeroed() };
                // SAFETY: `info` is writable; `i` is below the count.
                let ok = unsafe { c.getParameterInfo(i, &mut info) } == kResultOk;
                (ok && info.flags as u32 & ParameterInfo_::ParameterFlags_::kIsReadOnly as u32 != 0)
                    .then_some(info.id)
            })
            .collect()
    }

    /// Main-thread duties while it runs (D216): the values its processor was sent since
    /// the last call reach the controller (`setParamNormalized`), and a program change
    /// moved to another program marks the names as changed.
    pub fn service(&self) {
        // The host's values first, so that what the processor reports is told from them
        // (the review's finding).
        self.service_mirror();
        self.service_outputs();
    }

    /// The values the processor was sent, handed to the controller (D216).
    fn service_mirror(&self) {
        let (Some(c), Some(m)) = (
            &self.controller,
            self.mirror.lock().ok().and_then(|m| m.clone()),
        ) else {
            return;
        };
        if !m.any.swap(false, Ordering::AcqRel) {
            return;
        }
        for (i, id) in m.ids.iter().enumerate() {
            if !m.fresh[i].swap(false, Ordering::AcqRel) {
                continue;
            }
            let v = f64::from_bits(m.values[i].load(Ordering::Relaxed));
            let program = m.programs.iter().find(|p| p.0 == *id);
            // The program the controller has now (its editor may have chosen another since
            // the host last sent one), against the one it is given.
            let before = program.map(|p| {
                // SAFETY: reading a controller value on the main thread.
                Mirror::program(p.1, unsafe { c.getParamNormalized(*id) })
            });
            self.host_gave(*id, v);
            // SAFETY: setting a controller value on the main thread.
            unsafe { c.setParamNormalized(*id, v) };
            if let (Some(p), Some(before)) = (program, before)
                && Mirror::program(p.1, v) != before
            {
                self.names.store(true, Ordering::Release);
            }
        }
    }

    /// The values its processor reported itself since the last call (D234,
    /// `outputParameterChanges`), handed to the controller (`setParamNormalized`, as VST3
    /// asks of a host and as the SDK's own wrappers do, the latest of each). A meter's
    /// (`kIsReadOnly`) is only shown; another parameter's is a change of the plugin's own,
    /// read again for the project as its word that its values changed would be, unless it
    /// is what it reported last or the host's own value (a processor that reports its values
    /// every block, or echoes the host's, would otherwise keep its word coming: the
    /// review's finding; to an `f32`'s precision). A program change that moved asks for its
    /// names again.
    fn service_outputs(&self) {
        let (Some(c), Some(m)) = (
            &self.controller,
            self.outputs.lock().ok().and_then(|m| m.clone()),
        ) else {
            return;
        };
        if !m.any.swap(false, Ordering::AcqRel) {
            return;
        }
        for (i, id) in m.ids.iter().enumerate() {
            if !m.fresh[i].swap(false, Ordering::AcqRel) {
                continue;
            }
            let v = f64::from_bits(m.values[i].load(Ordering::Relaxed));
            let last = m.seen[i].swap(v.to_bits(), Ordering::Relaxed);
            // SAFETY: setting a controller value on the main thread.
            unsafe { c.setParamNormalized(*id, v) };
            let again = last != u64::MAX && crate::watch::near(f64::from_bits(last), v);
            let hosts = self
                .host_value(*id)
                .is_some_and(|h| crate::watch::near(h, v));
            if !m.read_only.get(i).copied().unwrap_or(false) && !again && !hosts {
                self.values.store(true, Ordering::Release);
                if m.programs.iter().any(|p| p.0 == *id) {
                    self.names.store(true, Ordering::Release);
                }
            }
        }
    }

    /// Whether it said since the last call that its parameters' names or list changed
    /// (`restartComponent(kParamTitlesChanged)`; PATCHES.md change 6).
    pub fn take_params_changed(&self) -> bool {
        self.handler.titles.swap(false, Ordering::AcqRel)
    }

    /// The controller's text for each value (normalized; PATCHES.md change 6):
    /// `getParamStringByValue`, its units after it where it gave a number alone.
    pub fn param_text(&self, values: &[(u32, f64)]) -> Vec<Option<String>> {
        let Some(c) = &self.controller else {
            return vec![None; values.len()];
        };
        let units: std::collections::BTreeMap<u32, String> = self
            .params()
            .into_iter()
            .filter(|p| !p.units.is_empty())
            .map(|p| (p.id, p.units))
            .collect();
        values
            .iter()
            .map(|(id, v)| {
                // SAFETY: an all-zero string is a valid value to be overwritten.
                let mut s: String128 = unsafe { std::mem::zeroed() };
                // SAFETY: a controller query on the main thread; `s` is writable.
                if unsafe { c.getParamStringByValue(*id, v.clamp(0.0, 1.0), &mut s) } != kResultOk {
                    return None;
                }
                let text = wstr(&s).trim().to_owned();
                Some(match units.get(id) {
                    Some(u) if !text.is_empty() && !text.ends_with(u.as_str()) && bare(&text) => {
                        format!("{text} {u}")
                    }
                    _ => text,
                })
            })
            .collect()
    }

    /// Whether its names for its keys may have changed since the last call (D216).
    pub fn take_names_changed(&self) -> bool {
        self.names.swap(false, Ordering::AcqRel)
    }

    /// Whether it said since the last call that its parameters' values changed by its own
    /// doing (D228: `restartComponent(kParamValuesChanged)`, a preset loaded in its editor).
    pub fn take_values_changed(&self) -> bool {
        self.values.swap(false, Ordering::AcqRel)
    }

    /// Every parameter's value as its controller has it now (D228), normalized (its units);
    /// not those it shows but does not take (`kIsReadOnly`: a meter, a display).
    pub fn param_values(&self) -> Vec<(u32, f64)> {
        let Some(c) = &self.controller else {
            return Vec::new();
        };
        // SAFETY: parameter queries on the main thread.
        let n = unsafe { c.getParameterCount() };
        (0..n)
            .filter_map(|i| {
                // SAFETY: an all-zero parameter info is a valid value to be overwritten.
                let mut info: ParameterInfo = unsafe { std::mem::zeroed() };
                // SAFETY: `info` is writable; `i` is below the count.
                let ok = unsafe { c.getParameterInfo(i, &mut info) } == kResultOk;
                let read_only =
                    info.flags as u32 & ParameterInfo_::ParameterFlags_::kIsReadOnly as u32 != 0;
                // SAFETY: reading a controller value on the main thread.
                (ok && !read_only).then(|| (info.id, unsafe { c.getParamNormalized(info.id) }))
            })
            .collect()
    }

    /// The edits its editor made since the last call (D222), a gesture's beginning and end
    /// with the value the controller has then.
    pub fn take_edits(&self) -> Vec<ParamEdit> {
        let mut edits = self
            .handler
            .edits
            .lock()
            .map(|mut e| std::mem::take(&mut *e))
            .unwrap_or_default();
        if let Some(c) = &self.controller {
            for e in edits.iter_mut().filter(|e| e.value.is_none()) {
                // SAFETY: reading a controller value on the main thread.
                e.value = Some(unsafe { c.getParamNormalized(e.param) });
            }
        }
        // A value outside a gesture that is the one the host last gave the controller is
        // its answer to the host, not an edit (a controller answering `setParamNormalized`
        // with `performEdit`; the review's finding): dropped.
        let (Ok(mut open), Ok(gave)) = (self.open_gestures.lock(), self.host_values.lock()) else {
            return edits;
        };
        edits.retain(|e| match e.kind {
            EditKind::Begin => {
                open.push(e.param);
                true
            }
            EditKind::End => {
                open.retain(|p| *p != e.param);
                true
            }
            EditKind::Value => {
                open.contains(&e.param)
                    || !gave
                        .get(&e.param)
                        .zip(e.value)
                        .is_some_and(|(h, v)| (h - v).abs() <= 1e-9)
            }
        });
        edits
    }

    /// The host gave the controller parameter `id`'s value `v` (D222).
    fn host_gave(&self, id: u32, v: f64) {
        if let Ok(mut g) = self.host_values.lock() {
            g.insert(id, v);
        }
        if let Ok(mut g) = self.given.lock() {
            g.push(id);
        }
    }

    /// Whether its controller said something besides its parameters changed since the last
    /// call (D234: `setDirty`).
    pub fn take_dirty(&self) -> bool {
        self.handler.dirty.swap(false, Ordering::AcqRel)
    }

    /// The parameters the host gave a value since the last call (D234).
    pub fn take_host_given(&self) -> Vec<u32> {
        self.given
            .lock()
            .map(|mut g| std::mem::take(&mut *g))
            .unwrap_or_default()
    }

    /// The value the host last gave parameter `id` (D234).
    pub fn host_value(&self, id: u32) -> Option<f64> {
        self.host_values.lock().ok()?.get(&id).copied()
    }

    /// Whether its editor is open (D234: its values are polled meanwhile).
    pub fn editor_open(&self) -> bool {
        self.editor.is_some()
    }

    /// Opens the plugin's editor titled `title`, at `at` on the screen if given (an editor
    /// already open is shown and moved there).
    pub fn gui_open(&mut self, title: &str, at: Option<(i32, i32)>) -> Result<Value, String> {
        if let Some(e) = &mut self.editor {
            if let Some((x, y)) = at {
                e.window.move_to(x, y);
            }
            e.window.show();
            return Ok(self.gui_info());
        }
        let c = self
            .controller
            .as_ref()
            .ok_or("the plugin has no controller")?;
        // SAFETY: asking the controller for its editor on the main thread; the view comes
        // with a reference we own.
        let view = unsafe { ComPtr::from_raw(c.createView(c"editor".as_ptr())) }
            .ok_or("the plugin has no editor")?;
        // SAFETY: a platform type query on the main thread.
        if unsafe { view.isPlatformTypeSupported(PLATFORM.as_ptr()) } != kResultTrue {
            return Err(format!(
                "the plugin's editor does not support this platform's windows ({})",
                PLATFORM.to_string_lossy()
            ));
        }
        let mut rect = ViewRect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        // SAFETY: `rect` is ours to fill.
        let sized = unsafe { view.getSize(&mut rect) } == kResultOk;
        let (w, h) = (rect.right - rect.left, rect.bottom - rect.top);
        let (w, h) = if sized && w > 0 && h > 0 {
            (w as u32, h as u32)
        } else {
            (400, 300)
        };
        // SAFETY: a query on the main thread.
        let resizable = unsafe { view.canResize() } == kResultTrue;
        let mut win = Window::create(title, w, h, at, resizable)?;
        let frame = ComWrapper::new(Frame {
            resize: Mutex::new(None),
            #[cfg(all(unix, not(target_os = "macos")))]
            run_loop: run_loop::RunLoop::default(),
        });
        if let Some(f) = frame.as_com_ref::<IPlugFrame>() {
            // SAFETY: handing the view our frame, which outlives it (the editor holds both).
            unsafe { view.setFrame(f.as_ptr()) };
        }
        let handle = win.handle();
        // SAFETY: the union member read is the one this platform's windows fill.
        #[cfg(all(unix, not(target_os = "macos")))]
        let parent = unsafe { handle.x11 } as usize as *mut c_void;
        // SAFETY: as above.
        #[cfg(windows)]
        let parent = unsafe { handle.win32 };
        // SAFETY: as above.
        #[cfg(target_os = "macos")]
        let parent = unsafe { handle.cocoa };
        // SAFETY: attaching the view to our live window, of the type it supports.
        if unsafe { view.attached(parent, PLATFORM.as_ptr()) } != kResultOk {
            // SAFETY: undoing `setFrame` before the frame goes.
            unsafe { view.setFrame(std::ptr::null_mut()) };
            return Err("the plugin could not attach its editor to the window".into());
        }
        win.show();
        // The window manager maps a new window when it gets to it: wait a moment so that
        // the reply says whether it is on screen.
        let began = std::time::Instant::now();
        while !win.check().visible && began.elapsed() < std::time::Duration::from_secs(1) {
            window::pump();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        self.editor = Some(Editor {
            view,
            frame,
            window: win,
            title: title.to_owned(),
        });
        Ok(self.gui_info())
    }

    /// The open editor: its size, where it is, and whether the window is on screen with
    /// the plugin's view in it.
    pub fn gui_info(&self) -> Value {
        let Some(e) = &self.editor else {
            return json!({"open": false});
        };
        let check = e.window.check();
        let (width, height) = e.window.size();
        json!({
            "open": true,
            "helper": std::process::id(),
            "floating": false,
            "api": window::api().to_string_lossy(),
            "title": e.title,
            "window": e.window.id(),
            "width": width,
            "height": height,
            "position": e.window.position(),
            "visible": check.visible,
            "children": check.children,
        })
    }

    pub fn gui_move(&mut self, x: i32, y: i32) -> Result<Value, String> {
        match &self.editor {
            Some(e) => {
                e.window.move_to(x, y);
                Ok(self.gui_info())
            }
            None => Err("the plugin's editor is not open".into()),
        }
    }

    /// Closes the editor: the view removed, the window with it.
    pub fn gui_close(&mut self) {
        if let Some(e) = self.editor.take() {
            // SAFETY: detaching the view from its window and frame on the main thread,
            // before both go.
            unsafe {
                e.view.removed();
                e.view.setFrame(std::ptr::null_mut());
            }
            #[cfg(all(unix, not(target_os = "macos")))]
            e.frame.run_loop.clear();
            drop(e);
        }
    }

    /// Main-thread editor duties: the view's resize requests and the window's events.
    /// Returns true when the user closed the editor.
    pub fn gui_service(&mut self) -> bool {
        let Some(e) = &mut self.editor else {
            return false;
        };
        #[cfg(all(unix, not(target_os = "macos")))]
        e.frame.run_loop.service();
        if let Some((w, h)) = e.frame.resize.lock().ok().and_then(|mut r| r.take()) {
            e.window.set_size(w, h);
            let mut r = ViewRect {
                left: 0,
                top: 0,
                right: w as int32,
                bottom: h as int32,
            };
            // SAFETY: telling the view its new size on the main thread.
            unsafe { e.view.onSize(&mut r) };
        }
        let mut closed = false;
        for ev in e.window.events() {
            match ev {
                WindowEvent::Closed => closed = true,
                WindowEvent::Resized(w, h) => {
                    let mut r = ViewRect {
                        left: 0,
                        top: 0,
                        right: w as int32,
                        bottom: h as int32,
                    };
                    // SAFETY: size queries and changes on the main thread.
                    unsafe {
                        if e.view.canResize() == kResultTrue
                            && e.view.checkSizeConstraint(&mut r) == kResultOk
                        {
                            e.view.onSize(&mut r);
                        }
                    }
                }
            }
        }
        if closed {
            self.gui_close();
        }
        closed
    }

    /// A processor for the audio thread, with buffers for `max_block` frames, and a table of
    /// every parameter the plugin has, made here (D222: none is left out).
    pub fn processor(&self, max_block: usize) -> Processor {
        let (ins, outs) = self.main_ports();
        let pending = self
            .pending
            .lock()
            .map(|mut p| std::mem::take(&mut *p))
            .unwrap_or_default();
        // Every parameter by ID, with its value now: the base modulation adds to.
        let mut ids = self.param_ids();
        ids.sort_unstable();
        ids.dedup();
        let base: Vec<f64> = ids
            .iter()
            .map(|id| {
                let pending = pending.iter().rev().find(|p| p.0 == *id).map(|p| p.1);
                pending.unwrap_or_else(|| {
                    self.controller.as_ref().map_or(0.0, |c| {
                        // SAFETY: reading a controller value on the main thread.
                        unsafe { c.getParamNormalized(*id) }
                    })
                })
            })
            .collect();
        let programs = self.program_changes();
        let read_only = self.read_only_ids();
        let make = || {
            Arc::new(Mirror {
                values: ids
                    .iter()
                    .map(|_| std::sync::atomic::AtomicU64::new(0))
                    .collect(),
                fresh: ids.iter().map(|_| AtomicBool::new(false)).collect(),
                ids: ids.clone(),
                any: AtomicBool::new(false),
                programs: programs.clone(),
                read_only: ids.iter().map(|id| read_only.contains(id)).collect(),
                seen: ids
                    .iter()
                    .map(|_| std::sync::atomic::AtomicU64::new(u64::MAX))
                    .collect(),
            })
        };
        let mirror = make();
        if let Ok(mut m) = self.mirror.lock() {
            *m = Some(mirror.clone());
        }
        let outputs = make();
        if let Ok(mut m) = self.outputs.lock() {
            *m = Some(outputs.clone());
        }
        // Queues for as many parameters as one block can change: those its events carry,
        // and the values set before processing or by the editor that it takes.
        let pool = ids.len().min(MAX_EVENTS + EDITS_PER_BLOCK);
        let mut pending_queue = Vec::with_capacity(pending.len().max(EDITS_PER_BLOCK));
        pending_queue.extend(pending);
        Processor {
            mirror,
            outputs,
            out_changes: Changes::new(&ids, ids.len().min(OUTPUTS_PER_BLOCK)),
            modulation: vec![0.0; ids.len()],
            marked: vec![false; ids.len()],
            changes: Changes::new(&ids, pool),
            ids,
            base,
            edits: self.edits_in.clone(),
            notes: self.takes_notes().then(Events::new),
            processor: self.processor.clone(),
            pending: pending_queue,
            ins: ins.min(2) as usize,
            outs: outs.min(2) as usize,
            spare: vec![vec![0.0; max_block]; 2],
            max_block,
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        self.gui_close();
        self.deactivate();
        if self.separate
            && let (Some(a), Some(b)) = (
                self.component.cast::<IConnectionPoint>(),
                self.controller
                    .as_ref()
                    .and_then(|c| c.cast::<IConnectionPoint>()),
            )
        {
            // SAFETY: disconnecting what `connect` connected.
            unsafe {
                a.disconnect(b.as_ptr());
                b.disconnect(a.as_ptr());
            }
        }
        if let Some(c) = &self.controller
            && self.separate
        {
            // SAFETY: terminating the controller once, on the main thread.
            unsafe { c.terminate() };
        }
        // SAFETY: terminating the component once, on the main thread.
        unsafe { self.component.terminate() };
        let _ = &self.host;
    }
}

// ---------------------------------------------------------------------------------------
// Processing

/// The audio thread's side of an instance.
pub struct Processor {
    /// The values it sends, for the controller (D216).
    mirror: Arc<Mirror>,
    /// The values the plugin's processor reports itself (D234), for the controller: the
    /// block's output changes (`outputParameterChanges`), a pool made off the audio thread,
    /// the latest point of each recorded after the block.
    outputs: Arc<Mirror>,
    out_changes: ComWrapper<Changes>,
    /// Every parameter's ID, sorted (D222), and by its place there its base value and
    /// modulation amount (normalized).
    ids: Vec<u32>,
    base: Vec<f64>,
    modulation: Vec<f64>,
    /// By place: counted among a block's evented parameters (cleared after).
    marked: Vec<bool>,
    /// Values the plugin's editor set (`performEdit`), for the processor (D222).
    edits: Arc<Mutex<Vec<(u32, f64)>>>,
    /// Note events for the block, when the plugin takes notes.
    notes: Option<ComWrapper<Events>>,
    processor: ComPtr<IAudioProcessor>,
    changes: ComWrapper<Changes>,
    /// Values set before processing started, and the editor's that a block had no room
    /// for: sent with the next blocks.
    pending: Vec<(u32, f64)>,
    ins: usize,
    outs: usize,
    /// Channels for a mono bus's missing side, and silence for a missing input.
    spare: Vec<Vec<f32>>,
    max_block: usize,
}

impl std::fmt::Debug for Processor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Processor")
            .field("max_block", &self.max_block)
            .finish()
    }
}

// SAFETY: moved once to the audio thread, which then owns it and makes only audio-thread
// calls through it.
unsafe impl Send for Processor {}

impl Processor {
    pub fn start(&mut self) -> bool {
        // SAFETY: starting processing on the audio thread of an active plugin.
        unsafe { self.processor.setProcessing(1) };
        true
    }

    /// Between blocks, when none came for a while: nothing (the editor's values wait for
    /// the next block).
    pub fn idle(&mut self) {}

    pub fn stop(&mut self) {
        // SAFETY: stopping processing on the audio thread.
        unsafe { self.processor.setProcessing(0) };
    }

    /// Processes one block: `input` and `output` are the transport slot's stereo channels.
    /// Parameter events carry normalized values. Returns a transport status.
    ///
    /// # Safety
    /// The four channel pointers must each be valid for `frames` samples, and `frames`
    /// must not exceed the processor's block size.
    pub unsafe fn process(
        &mut self,
        steady_time: i64,
        timing: daw_ipc::Timing,
        frames: usize,
        events: &[Event],
        input: [*mut f32; 2],
        output: [*mut f32; 2],
    ) -> i32 {
        let frames = frames.min(self.max_block);
        self.changes.clear();
        // The editor's values (D222), taken if nothing else holds them now: never waits.
        if let Ok(mut e) = self.edits.try_lock() {
            let room = self.pending.capacity() - self.pending.len();
            let n = e.len().min(room);
            self.pending.extend(e.drain(..n));
        }
        // Values set before processing and the editor's, at the block's start, with room
        // kept for the parameters this block's events change that have no queue yet (the
        // review's finding: counted once each, however many points they have); none to
        // keep when every parameter has a queue of its own.
        let mut room = if self.changes.pool() >= self.ids.len() {
            usize::MAX
        } else {
            let mut fresh = 0;
            let params = || {
                events
                    .iter()
                    .filter(|e| e.kind == EV_PARAM || e.kind == EV_PARAM_MOD)
            };
            for e in params() {
                if let Ok(place) = self.ids.binary_search(&e.param)
                    && !self.marked[place]
                {
                    self.marked[place] = true;
                    fresh += 1;
                }
            }
            for e in params() {
                if let Ok(place) = self.ids.binary_search(&e.param) {
                    self.marked[place] = false;
                }
            }
            self.changes.free().saturating_sub(fresh)
        };
        let mut kept = 0;
        for k in 0..self.pending.len() {
            let (id, v) = self.pending[k];
            let Ok(place) = self.ids.binary_search(&id) else {
                continue;
            };
            let fresh = !self.changes.has(place);
            if fresh && room == 0 {
                self.pending[kept] = (id, v);
                kept += 1;
                continue;
            }
            self.base[place] = v;
            let value = (v + self.modulation[place]).clamp(0.0, 1.0);
            if self.changes.add(place, id, 0, value) && fresh {
                room = room.saturating_sub(1);
            }
        }
        self.pending.truncate(kept);
        if let Some(list) = &self.notes
            && let Ok(mut l) = list.list.lock()
        {
            l.clear();
            for e in events
                .iter()
                .filter(|e| e.kind == EV_NOTE_ON || e.kind == EV_NOTE_OFF)
            {
                if l.len() == l.capacity() {
                    break;
                }
                // SAFETY: an all-zero event is a valid value to fill in.
                let mut v: vst3::Steinberg::Vst::Event = unsafe { std::mem::zeroed() };
                v.sampleOffset = (e.time as usize).min(frames.saturating_sub(1)) as int32;
                if e.kind == EV_NOTE_ON {
                    v.r#type = vst3::Steinberg::Vst::Event_::EventTypes_::kNoteOnEvent as u16;
                    v.__field0.noteOn = NoteOnEvent {
                        channel: e.channel,
                        pitch: e.key,
                        tuning: 0.0,
                        velocity: e.value as f32,
                        length: 0,
                        noteId: e.note_id,
                    };
                } else {
                    v.r#type = vst3::Steinberg::Vst::Event_::EventTypes_::kNoteOffEvent as u16;
                    v.__field0.noteOff = NoteOffEvent {
                        channel: e.channel,
                        pitch: e.key,
                        velocity: e.value as f32,
                        noteId: e.note_id,
                        tuning: 0.0,
                    };
                }
                l.push(v);
            }
        }
        for e in events {
            if e.kind != EV_PARAM && e.kind != EV_PARAM_MOD {
                continue;
            }
            // Every parameter the plugin has (D222); one it does not have, nothing.
            let Ok(place) = self.ids.binary_search(&e.param) else {
                continue;
            };
            if e.kind == EV_PARAM {
                self.base[place] = e.value;
            } else {
                self.modulation[place] = e.value;
            }
            let value = (self.base[place] + self.modulation[place]).clamp(0.0, 1.0);
            let at = (e.time as usize).min(frames.saturating_sub(1)) as int32;
            // No queue left (more different parameters than a block carries from the
            // sandbox): the value goes at the next block's start, if there is room.
            if !self.changes.add(place, e.param, at, value)
                && self.pending.len() < self.pending.capacity()
            {
                self.pending.push((e.param, self.base[place]));
            }
            // The controller (its editor) is told the base alone, never the modulation: a
            // modulated parameter's window does not chase it, and a drag there moves the
            // base (PATCHES.md change 6).
            if e.kind == EV_PARAM {
                self.mirror.set(e.param, self.base[place]);
            }
        }
        let mut in_ptrs = [input[0], input[1]];
        let mut out_ptrs = [output[0], output[1]];
        let mut in_bus = AudioBusBuffers {
            numChannels: self.ins as int32,
            silenceFlags: 0,
            __field0: AudioBusBuffers__type0 {
                channelBuffers32: in_ptrs.as_mut_ptr(),
            },
        };
        let mut out_bus = AudioBusBuffers {
            numChannels: self.outs as int32,
            silenceFlags: 0,
            __field0: AudioBusBuffers__type0 {
                channelBuffers32: out_ptrs.as_mut_ptr(),
            },
        };
        let Some(changes) = self.changes.as_com_ref::<IParameterChanges>() else {
            return STATUS_ERROR;
        };
        self.out_changes.clear();
        let Some(out_changes) = self.out_changes.as_com_ref::<IParameterChanges>() else {
            return STATUS_ERROR;
        };
        let mut data = ProcessData {
            processMode: ProcessModes_::kRealtime as int32,
            symbolicSampleSize: SymbolicSampleSizes_::kSample32 as int32,
            numSamples: frames as int32,
            numInputs: i32::from(self.ins > 0),
            numOutputs: i32::from(self.outs > 0),
            inputs: if self.ins > 0 {
                &mut in_bus
            } else {
                std::ptr::null_mut()
            },
            outputs: if self.outs > 0 {
                &mut out_bus
            } else {
                std::ptr::null_mut()
            },
            inputParameterChanges: changes.as_ptr(),
            outputParameterChanges: out_changes.as_ptr(),
            inputEvents: self
                .notes
                .as_ref()
                .and_then(|n| n.as_com_ref::<IEventList>())
                .map_or(std::ptr::null_mut(), |r| r.as_ptr()),
            outputEvents: std::ptr::null_mut(),
            processContext: std::ptr::null_mut(),
        };
        // The host's tempo and place, its transport stopped (CA-72's change 5).
        // SAFETY: a plain C struct, all zeros a valid value (nothing valid in its state).
        let mut context: ProcessContext = unsafe { std::mem::zeroed() };
        if timing.has_tempo() {
            use ProcessContext_::StatesAndFlags_::*;
            context.state =
                (kTempoValid | kProjectTimeMusicValid | kBarPositionValid | kTimeSigValid)
                    as uint32;
            context.sampleRate = timing.rate;
            context.projectTimeSamples = steady_time;
            context.continousTimeSamples = steady_time;
            context.projectTimeMusic = timing.beats;
            context.barPositionMusic = (timing.beats / 4.0).floor() * 4.0;
            context.tempo = timing.tempo;
            context.timeSigNumerator = 4;
            context.timeSigDenominator = 4;
            data.processContext = &mut context;
        }
        // SAFETY: every pointer in `data` is valid for the call (the caller's contract
        // covers the channels); `process` is called on the audio thread.
        let r = unsafe { self.processor.process(&mut data) };
        // What the processor reported itself (D234): the latest point of each, for the
        // controller.
        self.out_changes.latest(|id, v| self.outputs.set(id, v));
        // A mono output feeds both channels.
        if self.outs == 1 {
            // SAFETY: both output channels are valid for `frames` samples.
            unsafe { std::ptr::copy_nonoverlapping(output[0], output[1], frames) };
        }
        let _ = &self.spare;
        if r == kResultOk {
            STATUS_OK
        } else {
            STATUS_ERROR
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_ids_print_the_same_on_every_platform() {
        let cid = vst3::uid(0x44415733, 0x54657374, 0x4761696E, 0x50726F63);
        assert_eq!(cid_hex(&cid), "44415733546573744761696E50726F63");
        assert_eq!(cid_parse("44415733546573744761696E50726F63"), Some(cid));
        assert_eq!(cid_parse("not a class id"), None);
    }
}
