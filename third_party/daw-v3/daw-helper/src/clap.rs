//! A minimal CLAP host (DESIGN 10.6): loads plugin libraries, creates and drives plugin
//! instances. Runs only in the helper process (DESIGN 10.1).
//!
//! Threading follows CLAP: everything except processing happens on the helper's main
//! thread; each instance's processing runs on its own thread ([`Processor`]).

#![allow(unsafe_code)]

mod gui;

use clap_sys::audio_buffer::clap_audio_buffer;
use clap_sys::entry::clap_plugin_entry;
use clap_sys::events::{
    CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_DONT_RECORD, CLAP_EVENT_NOTE_OFF, CLAP_EVENT_NOTE_ON,
    CLAP_EVENT_PARAM_GESTURE_BEGIN, CLAP_EVENT_PARAM_GESTURE_END, CLAP_EVENT_PARAM_MOD,
    CLAP_EVENT_PARAM_VALUE, clap_event_header, clap_event_note, clap_event_param_gesture,
    clap_event_param_mod, clap_event_param_value, clap_input_events, clap_output_events,
};
use clap_sys::ext::audio_ports::{
    CLAP_AUDIO_PORT_IS_MAIN, CLAP_EXT_AUDIO_PORTS, clap_audio_port_info, clap_plugin_audio_ports,
};
use clap_sys::ext::latency::{CLAP_EXT_LATENCY, clap_plugin_latency};
use clap_sys::ext::log::{CLAP_EXT_LOG, clap_host_log, clap_log_severity};
use clap_sys::ext::note_name::{
    CLAP_EXT_NOTE_NAME, clap_host_note_name, clap_note_name, clap_plugin_note_name,
};
use clap_sys::ext::note_ports::{CLAP_EXT_NOTE_PORTS, clap_plugin_note_ports};
use clap_sys::ext::params::{
    CLAP_EXT_PARAMS, CLAP_PARAM_IS_AUTOMATABLE, CLAP_PARAM_IS_BYPASS, CLAP_PARAM_IS_ENUM,
    CLAP_PARAM_IS_HIDDEN, CLAP_PARAM_IS_MODULATABLE, CLAP_PARAM_IS_READONLY, CLAP_PARAM_IS_STEPPED,
    CLAP_PARAM_RESCAN_ALL, CLAP_PARAM_RESCAN_INFO, CLAP_PARAM_RESCAN_TEXT,
    CLAP_PARAM_RESCAN_VALUES, clap_host_params, clap_param_clear_flags, clap_param_info,
    clap_param_rescan_flags, clap_plugin_params,
};
use clap_sys::ext::state::{CLAP_EXT_STATE, clap_host_state, clap_plugin_state};
use clap_sys::ext::tail::{CLAP_EXT_TAIL, clap_plugin_tail};
use clap_sys::ext::thread_check::{CLAP_EXT_THREAD_CHECK, clap_host_thread_check};
use clap_sys::factory::plugin_factory::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory};
use clap_sys::host::clap_host;
use clap_sys::id::clap_id;
use clap_sys::plugin::{clap_plugin, clap_plugin_descriptor};
use clap_sys::process::{CLAP_PROCESS_ERROR, clap_process};
use clap_sys::stream::{clap_istream, clap_ostream};
use clap_sys::version::{CLAP_VERSION, clap_version_is_compatible};
use daw_ipc::layout::{EV_NOTE_OFF, EV_NOTE_ON, EV_PARAM, EV_PARAM_MOD, STATUS_ERROR, STATUS_OK};
use daw_ipc::protocol::{EditKind, ParamEdit, ParamInfo, PluginDesc};
use daw_ipc::{Event, MAX_BLOCK, MAX_EVENTS};
use std::cell::Cell;
use std::ffi::{CStr, CString, c_char, c_void};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::ThreadId;

thread_local! {
    static AUDIO_THREAD: Cell<bool> = const { Cell::new(false) };
}

fn cstr_lossy(p: *const c_char) -> String {
    if p.is_null() {
        return String::new();
    }
    // SAFETY: plugins pass NUL-terminated strings.
    unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
}

fn array_str(a: &[c_char]) -> String {
    let bytes: Vec<u8> = a
        .iter()
        .take_while(|c| **c != 0)
        .map(|c| *c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// A loaded plugin library (a `.clap` file).
pub struct Library {
    entry: *const clap_plugin_entry,
    path: PathBuf,
    // Dropped last: the entry points into it.
    _lib: libloading::Library,
}

impl std::fmt::Debug for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Library").field("path", &self.path).finish()
    }
}

// SAFETY: the CLAP entry and factory are thread-safe by specification.
unsafe impl Send for Library {}
// SAFETY: as above.
unsafe impl Sync for Library {}

/// The file to load for a plugin path: macOS bundles keep the binary inside.
fn binary_path(path: &Path) -> PathBuf {
    if path.is_dir() {
        let macos = path.join("Contents").join("MacOS");
        if let Ok(rd) = std::fs::read_dir(&macos) {
            let mut files: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
            files.sort();
            if let Some(f) = files.into_iter().next() {
                return f;
            }
        }
    }
    path.to_path_buf()
}

impl Library {
    pub fn load(path: &Path) -> Result<Library, String> {
        let bin = binary_path(path);
        // SAFETY: loading a plugin library runs its initialisers; that is the point of the
        // helper process, which isolates whatever they do from the DAW.
        let lib = unsafe { libloading::Library::new(&bin) }
            .map_err(|e| format!("cannot load {}: {e}", bin.display()))?;
        // SAFETY: `clap_entry` is a data symbol of type `clap_plugin_entry`.
        let entry: *const clap_plugin_entry = unsafe {
            *lib.get::<*const clap_plugin_entry>(b"clap_entry\0")
                .map_err(|e| format!("{} is not a CLAP plugin: {e}", bin.display()))?
        };
        if entry.is_null() {
            return Err(format!("{}: null clap_entry", bin.display()));
        }
        // SAFETY: `entry` points to the library's static entry structure.
        let e = unsafe { &*entry };
        if !clap_version_is_compatible(e.clap_version) {
            return Err(format!(
                "{}: incompatible CLAP version {}.{}",
                bin.display(),
                e.clap_version.major,
                e.clap_version.minor
            ));
        }
        let cpath = CString::new(path.to_string_lossy().as_bytes()).map_err(|e| e.to_string())?;
        // SAFETY: calling the entry's own `init` once, with a valid path.
        let ok = e.init.is_some_and(|f| unsafe { f(cpath.as_ptr()) });
        if !ok {
            return Err(format!("{}: clap_entry.init failed", bin.display()));
        }
        Ok(Library {
            entry,
            path: path.to_path_buf(),
            _lib: lib,
        })
    }

    fn factory(&self) -> Option<&clap_plugin_factory> {
        // SAFETY: `entry` is valid while the library is loaded.
        let e = unsafe { &*self.entry };
        let get = e.get_factory?;
        // SAFETY: asking the entry for the standard plugin factory.
        let f = unsafe { get(CLAP_PLUGIN_FACTORY_ID.as_ptr()) } as *const clap_plugin_factory;
        // SAFETY: a non-null factory pointer stays valid while the library is loaded.
        (!f.is_null()).then(|| unsafe { &*f })
    }

    /// The plugins in the library.
    pub fn descriptors(&self) -> Vec<PluginDesc> {
        let Some(f) = self.factory() else {
            return Vec::new();
        };
        // SAFETY: factory calls on a valid factory.
        let n = f.get_plugin_count.map_or(0, |c| unsafe { c(f) });
        (0..n)
            .filter_map(|i| {
                // SAFETY: `i` is below the plugin count.
                let d = f.get_plugin_descriptor.map(|g| unsafe { g(f, i) })?;
                // SAFETY: a non-null descriptor is valid while the library is loaded.
                (!d.is_null()).then(|| desc(unsafe { &*d }))
            })
            .collect()
    }

    /// Creates and initialises a plugin instance.
    pub fn create(self: &Arc<Library>, id: &str) -> Result<Instance, String> {
        let f = self.factory().ok_or("the library has no plugin factory")?;
        let create = f.create_plugin.ok_or("the factory cannot create plugins")?;
        let cid = CString::new(id).map_err(|e| e.to_string())?;
        let mut host = Box::new(HostData {
            clap: clap_host {
                clap_version: CLAP_VERSION,
                host_data: std::ptr::null_mut(),
                name: c"DAW-v3".as_ptr(),
                vendor: c"DAW-v3".as_ptr(),
                url: c"".as_ptr(),
                version: c"0.0.1".as_ptr(),
                get_extension: Some(host_get_extension),
                request_restart: Some(host_request_restart),
                request_process: Some(host_request_process),
                request_callback: Some(host_request_callback),
            },
            main_thread: std::thread::current().id(),
            callback: AtomicBool::new(false),
            restart: AtomicBool::new(false),
            dirty: AtomicBool::new(false),
            values: AtomicBool::new(false),
            infos: AtomicBool::new(false),
            names: Arc::new(AtomicBool::new(false)),
            edits: Arc::new(std::sync::Mutex::new(Vec::with_capacity(EDITS_SHARED))),
            flush: Arc::new(AtomicBool::new(false)),
            clears: std::sync::Mutex::new(Vec::new()),
            gui: gui::GuiHost::default(),
        });
        host.clap.host_data = &mut *host as *mut HostData as *mut c_void;
        // SAFETY: the host structure is boxed, so its address is stable for the plugin's
        // lifetime (the instance owns the box and destroys the plugin first).
        let plugin = unsafe { create(f, &host.clap, cid.as_ptr()) };
        if plugin.is_null() {
            return Err(format!("the library has no plugin `{id}`"));
        }
        let inst = Instance {
            plugin,
            host,
            lib: self.clone(),
            active: false,
            view: None,
            given: std::sync::OnceLock::new(),
        };
        // SAFETY: `plugin` is a fresh instance; `init` is called once, on the main thread.
        let ok = unsafe { (*plugin).init.is_some_and(|i| i(plugin)) };
        if !ok {
            return Err(format!("`{id}` failed to initialise"));
        }
        Ok(inst)
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        // SAFETY: `deinit` once, after every instance is gone (instances hold an `Arc`).
        unsafe {
            if let Some(d) = (*self.entry).deinit {
                d();
            }
        }
    }
}

fn desc(d: &clap_plugin_descriptor) -> PluginDesc {
    let mut features = Vec::new();
    if !d.features.is_null() {
        for i in 0.. {
            // SAFETY: `features` is a NULL-terminated array of C strings.
            let p = unsafe { *d.features.add(i) };
            if p.is_null() || i > 64 {
                break;
            }
            features.push(cstr_lossy(p));
        }
    }
    PluginDesc {
        id: cstr_lossy(d.id),
        name: cstr_lossy(d.name),
        vendor: cstr_lossy(d.vendor),
        version: cstr_lossy(d.version),
        description: cstr_lossy(d.description),
        features,
        format: "clap".into(),
    }
}

/// What the helper gives a plugin as `clap_host`.
struct HostData {
    clap: clap_host,
    main_thread: ThreadId,
    callback: AtomicBool,
    restart: AtomicBool,
    dirty: AtomicBool,
    /// Its parameters' values may have changed by its own doing (D228: a preset loaded in
    /// its editor): it rescanned their values (`clap_host_params.rescan`) or marked its
    /// state dirty (`clap_host_state.mark_dirty`). Read again for the project.
    values: AtomicBool,
    /// Its parameters' descriptions or texts changed (`clap_host_params.rescan` of info, text
    /// or all): listed again by the host (PATCHES.md change 5).
    infos: AtomicBool,
    /// Its names for its keys may have changed (D216): the plugin said so
    /// (`clap_host_note_name.changed`, a parameter rescan) or edited a parameter itself
    /// (an output parameter event, as its editor sends). Shared with its processor.
    names: Arc<AtomicBool>,
    /// The plugin's own parameter edits (D222: its editor's), as its output events
    /// brought them, until taken; shared with its processor, which never waits for it.
    edits: Arc<std::sync::Mutex<Vec<ParamEdit>>>,
    /// The plugin asked for a parameter flush (`clap_host_params.request_flush`).
    flush: Arc<AtomicBool>,
    /// What refers to a parameter, to be cleared (`clap_host_params.clear`): ID and flags.
    clears: std::sync::Mutex<Vec<(u32, u32)>>,
    gui: gui::GuiHost,
}

/// Edits a processor hands the main thread at once, at most (D222); more wait a block.
const EDITS_SHARED: usize = 4096;

/// Edits one call of a plugin's (a block, a flush) brings, at most; more are dropped.
const EDITS_PER_CALL: usize = 1024;

/// # Safety
/// `h` must be the `clap_host` of a live [`HostData`].
unsafe fn host_data<'a>(h: *const clap_host) -> Option<&'a HostData> {
    if h.is_null() {
        return None;
    }
    // SAFETY: `host_data` points to the boxed `HostData` that contains `*h`.
    unsafe { ((*h).host_data as *const HostData).as_ref() }
}

unsafe extern "C" fn host_get_extension(_: *const clap_host, id: *const c_char) -> *const c_void {
    if id.is_null() {
        return std::ptr::null();
    }
    // SAFETY: the plugin passes a NUL-terminated extension ID.
    let id = unsafe { CStr::from_ptr(id) };
    if id == CLAP_EXT_THREAD_CHECK {
        &THREAD_CHECK as *const _ as *const c_void
    } else if id == CLAP_EXT_LOG {
        &LOG as *const _ as *const c_void
    } else if id == CLAP_EXT_STATE {
        &HOST_STATE as *const _ as *const c_void
    } else if id == CLAP_EXT_NOTE_NAME {
        &HOST_NOTE_NAME as *const _ as *const c_void
    } else if id == CLAP_EXT_PARAMS {
        &HOST_PARAMS as *const _ as *const c_void
    } else {
        gui::host_extension(id).unwrap_or(std::ptr::null())
    }
}

unsafe extern "C" fn host_request_restart(h: *const clap_host) {
    // SAFETY: the plugin passes its host pointer.
    if let Some(d) = unsafe { host_data(h) } {
        d.restart.store(true, Ordering::Relaxed);
    }
}

unsafe extern "C" fn host_request_process(_: *const clap_host) {}

unsafe extern "C" fn host_request_callback(h: *const clap_host) {
    // SAFETY: the plugin passes its host pointer.
    if let Some(d) = unsafe { host_data(h) } {
        d.callback.store(true, Ordering::Release);
    }
}

unsafe extern "C" fn is_main_thread(h: *const clap_host) -> bool {
    // SAFETY: the plugin passes its host pointer.
    unsafe { host_data(h) }.is_some_and(|d| d.main_thread == std::thread::current().id())
}

unsafe extern "C" fn is_audio_thread(_: *const clap_host) -> bool {
    AUDIO_THREAD.with(Cell::get)
}

static THREAD_CHECK: clap_host_thread_check = clap_host_thread_check {
    is_main_thread: Some(is_main_thread),
    is_audio_thread: Some(is_audio_thread),
};

unsafe extern "C" fn host_log(
    _: *const clap_host,
    severity: clap_log_severity,
    msg: *const c_char,
) {
    eprintln!("plugin log [{severity}]: {}", cstr_lossy(msg));
}

static LOG: clap_host_log = clap_host_log {
    log: Some(host_log),
};

/// Its state changed (D228): a preset it loaded, or an edit of its own; its values are read
/// again once it is quiet, and those that differ go to the project.
unsafe extern "C" fn host_mark_dirty(h: *const clap_host) {
    // SAFETY: the plugin passes its host pointer.
    if let Some(d) = unsafe { host_data(h) } {
        d.dirty.store(true, Ordering::Relaxed);
        d.values.store(true, Ordering::Release);
    }
}

static HOST_STATE: clap_host_state = clap_host_state {
    mark_dirty: Some(host_mark_dirty),
};

/// The plugin's word that its names for its keys changed (D216): they are asked again.
unsafe extern "C" fn host_note_names_changed(h: *const clap_host) {
    // SAFETY: the plugin passes its host pointer.
    if let Some(d) = unsafe { host_data(h) } {
        d.names.store(true, Ordering::Release);
    }
}

static HOST_NOTE_NAME: clap_host_note_name = clap_host_note_name {
    changed: Some(host_note_names_changed),
};

/// Parameters' values or descriptions changed (a program loaded, D216): its names for its
/// keys may have too, and are asked again; its values (`CLAP_PARAM_RESCAN_VALUES`, or all)
/// are read again for the project (D228). New or removed parameters and new ranges
/// (`CLAP_PARAM_RESCAN_ALL`, `_INFO`) are taken when the device is made again.
unsafe extern "C" fn host_params_rescan(h: *const clap_host, flags: clap_param_rescan_flags) {
    let names = CLAP_PARAM_RESCAN_VALUES | CLAP_PARAM_RESCAN_INFO | CLAP_PARAM_RESCAN_ALL;
    // SAFETY: the plugin passes its host pointer.
    let Some(d) = (unsafe { host_data(h) }) else {
        return;
    };
    if flags & names != 0 {
        d.names.store(true, Ordering::Release);
    }
    if flags & (CLAP_PARAM_RESCAN_VALUES | CLAP_PARAM_RESCAN_ALL) != 0 {
        d.values.store(true, Ordering::Release);
    }
    if flags & (CLAP_PARAM_RESCAN_INFO | CLAP_PARAM_RESCAN_TEXT | CLAP_PARAM_RESCAN_ALL) != 0 {
        d.infos.store(true, Ordering::Release);
    }
}

/// The plugin asks the host to clear what refers to a parameter (D222): sent to the DAW,
/// which takes the automation, modulation and MIDI mappings of it out of the project as
/// the flags say. [main-thread]
unsafe extern "C" fn host_params_clear(
    h: *const clap_host,
    id: clap_id,
    flags: clap_param_clear_flags,
) {
    // SAFETY: the plugin passes its host pointer.
    if let Some(d) = unsafe { host_data(h) }
        && let Ok(mut c) = d.clears.lock()
    {
        c.push((id, flags));
    }
}

/// The plugin asks for a parameter flush (D222): the next `process` is one; an active
/// instance with no block for a while is flushed on its audio thread, one not active on the
/// main thread. [thread-safe, !audio-thread]
unsafe extern "C" fn host_params_request_flush(h: *const clap_host) {
    // SAFETY: the plugin passes its host pointer.
    if let Some(d) = unsafe { host_data(h) } {
        d.flush.store(true, Ordering::Release);
    }
}

static HOST_PARAMS: clap_host_params = clap_host_params {
    rescan: Some(host_params_rescan),
    clear: Some(host_params_clear),
    request_flush: Some(host_params_request_flush),
};

/// One plugin instance, owned by the helper's main thread.
pub struct Instance {
    plugin: *const clap_plugin,
    host: Box<HostData>,
    lib: Arc<Library>,
    active: bool,
    /// The plugin's GUI while open.
    view: Option<gui::View>,
    /// The values the host gave it (D234), made with its parameters' list when first
    /// needed: at activation's flush, then by its processor.
    given: std::sync::OnceLock<Arc<crate::watch::Given>>,
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instance")
            .field("library", &self.lib.path)
            .finish()
    }
}

// SAFETY: CLAP instances may be used from the main thread and one audio thread; the
// helper follows those rules (see the module comment).
unsafe impl Send for Instance {}
// SAFETY: as above.
unsafe impl Sync for Instance {}

struct VecOut {
    data: Vec<u8>,
}

unsafe extern "C" fn ostream_write(s: *const clap_ostream, buf: *const c_void, size: u64) -> i64 {
    // SAFETY: `ctx` is the `VecOut` we passed; `buf` holds `size` bytes.
    unsafe {
        let out = &mut *((*s).ctx as *mut VecOut);
        let bytes = std::slice::from_raw_parts(buf as *const u8, size as usize);
        out.data.extend_from_slice(bytes);
    }
    size as i64
}

struct SliceIn<'a> {
    data: &'a [u8],
    pos: usize,
}

unsafe extern "C" fn istream_read(s: *const clap_istream, buf: *mut c_void, size: u64) -> i64 {
    // SAFETY: `ctx` is the `SliceIn` we passed; `buf` has room for `size` bytes.
    unsafe {
        let src = &mut *((*s).ctx as *mut SliceIn<'_>);
        let n = (size as usize).min(src.data.len() - src.pos);
        std::ptr::copy_nonoverlapping(src.data[src.pos..].as_ptr(), buf as *mut u8, n);
        src.pos += n;
        n as i64
    }
}

impl Instance {
    fn ext<T>(&self, id: &CStr) -> Option<&T> {
        // SAFETY: a valid plugin; `get_extension` is callable from any thread.
        let p = unsafe {
            (*self.plugin)
                .get_extension
                .map(|g| g(self.plugin, id.as_ptr()))
        }?;
        // SAFETY: a non-null extension pointer has the type its ID names and lives as long
        // as the plugin.
        (!p.is_null()).then(|| unsafe { &*(p as *const T) })
    }

    pub fn descriptor(&self) -> PluginDesc {
        // SAFETY: a valid plugin's descriptor.
        let d = unsafe { (*self.plugin).desc };
        if d.is_null() {
            PluginDesc::default()
        } else {
            // SAFETY: non-null descriptor owned by the library.
            desc(unsafe { &*d })
        }
    }

    pub fn params(&self) -> Vec<ParamInfo> {
        let Some(ext) = self.ext::<clap_plugin_params>(CLAP_EXT_PARAMS) else {
            return Vec::new();
        };
        // SAFETY: parameter queries on the main thread.
        let n = ext.count.map_or(0, |c| unsafe { c(self.plugin) });
        // A plugin that names its keys: its enumerations may choose the names (D216).
        let names = self
            .ext::<clap_plugin_note_name>(CLAP_EXT_NOTE_NAME)
            .is_some();
        let mut out = Vec::new();
        for i in 0..n {
            // SAFETY: an all-zero `clap_param_info` is a valid value to be overwritten.
            let mut info: clap_param_info = unsafe { std::mem::zeroed() };
            // SAFETY: `info` is writable; `i` is below the count.
            let ok = ext
                .get_info
                .is_some_and(|g| unsafe { g(self.plugin, i, &mut info) });
            if !ok {
                continue;
            }
            let stepped = info.flags & CLAP_PARAM_IS_STEPPED != 0;
            out.push(ParamInfo {
                id: info.id,
                name: array_str(&info.name),
                short: String::new(),
                units: String::new(),
                steps: if stepped {
                    (info.max_value - info.min_value).round().max(0.0) as u32
                } else {
                    0
                },
                read_only: info.flags & CLAP_PARAM_IS_READONLY != 0,
                bypass: info.flags & CLAP_PARAM_IS_BYPASS != 0,
                module: array_str(&info.module),
                min: info.min_value,
                max: info.max_value,
                default: info.default_value,
                stepped: info.flags & CLAP_PARAM_IS_STEPPED != 0,
                program: names && info.flags & CLAP_PARAM_IS_ENUM != 0,
                hidden: info.flags & CLAP_PARAM_IS_HIDDEN != 0,
                automatable: info.flags & CLAP_PARAM_IS_AUTOMATABLE != 0,
                modulatable: info.flags & CLAP_PARAM_IS_MODULATABLE != 0,
                value: {
                    let mut v = 0.0f64;
                    // SAFETY: a parameter query on the main thread; `v` is writable.
                    ext.get_value
                        .is_some_and(|g| unsafe { g(self.plugin, info.id, &mut v) })
                        .then_some(v)
                },
                plain: Vec::new(),
            });
        }
        out
    }

    /// Channel counts of the main input and output ports.
    pub fn main_ports(&self) -> (u32, u32) {
        let count = |is_input: bool| -> u32 {
            self.ports(is_input)
                .iter()
                .find(|p| p.flags & CLAP_AUDIO_PORT_IS_MAIN != 0)
                .or(self.ports(is_input).first())
                .map_or(0, |p| p.channel_count)
        };
        (count(true), count(false))
    }

    fn ports(&self, is_input: bool) -> Vec<clap_audio_port_info> {
        let Some(ext) = self.ext::<clap_plugin_audio_ports>(CLAP_EXT_AUDIO_PORTS) else {
            return Vec::new();
        };
        // SAFETY: port queries on the main thread.
        let n = ext.count.map_or(0, |c| unsafe { c(self.plugin, is_input) });
        (0..n)
            .filter_map(|i| {
                // SAFETY: an all-zero port info is a valid value to be overwritten.
                let mut info: clap_audio_port_info = unsafe { std::mem::zeroed() };
                // SAFETY: `info` is writable; `i` is below the count.
                let ok = ext
                    .get
                    .is_some_and(|g| unsafe { g(self.plugin, i, is_input, &mut info) });
                ok.then_some(info)
            })
            .collect()
    }

    pub fn takes_notes(&self) -> bool {
        self.ext::<clap_plugin_note_ports>(CLAP_EXT_NOTE_PORTS)
            // SAFETY: note-port query on the main thread.
            .is_some_and(|e| e.count.is_some_and(|c| unsafe { c(self.plugin, true) } > 0))
    }

    /// The plugin's names for its keys (CLAP `note-name`, M18 W9), keys 0 to 127 only.
    pub fn note_names(&self) -> Vec<daw_ipc::protocol::NoteName> {
        let Some(ext) = self.ext::<clap_plugin_note_name>(CLAP_EXT_NOTE_NAME) else {
            return Vec::new();
        };
        // SAFETY: note-name queries on the main thread.
        let n = ext.count.map_or(0, |c| unsafe { c(self.plugin) }).min(4096);
        let mut out = Vec::new();
        for i in 0..n {
            // SAFETY: an all-zero `clap_note_name` is a valid value to be overwritten.
            let mut name: clap_note_name = unsafe { std::mem::zeroed() };
            // SAFETY: `name` is writable; `i` is below the count.
            let ok = ext
                .get
                .is_some_and(|g| unsafe { g(self.plugin, i, &mut name) });
            // Every port's or the main note port's: the host plays the first.
            if ok && (0..128).contains(&name.key) && (name.port == -1 || name.port == 0) {
                out.push(daw_ipc::protocol::NoteName {
                    key: name.key,
                    channel: name.channel,
                    name: array_str(&name.name),
                });
            }
        }
        out
    }

    pub fn save_state(&self) -> Result<Vec<u8>, String> {
        let ext = self
            .ext::<clap_plugin_state>(CLAP_EXT_STATE)
            .ok_or("the plugin has no state extension")?;
        let save = ext.save.ok_or("the plugin cannot save state")?;
        let mut out = VecOut { data: Vec::new() };
        let stream = clap_ostream {
            ctx: &mut out as *mut VecOut as *mut c_void,
            write: Some(ostream_write),
        };
        // SAFETY: saving on the main thread into our stream, which outlives the call.
        if unsafe { save(self.plugin, &stream) } {
            Ok(out.data)
        } else {
            Err("the plugin failed to save its state".into())
        }
    }

    pub fn load_state(&self, bytes: &[u8]) -> Result<(), String> {
        let ext = self
            .ext::<clap_plugin_state>(CLAP_EXT_STATE)
            .ok_or("the plugin has no state extension")?;
        let load = ext.load.ok_or("the plugin cannot load state")?;
        let mut src = SliceIn {
            data: bytes,
            pos: 0,
        };
        let stream = clap_istream {
            ctx: &mut src as *mut SliceIn<'_> as *mut c_void,
            read: Some(istream_read),
        };
        // SAFETY: loading on the main thread from our stream, which outlives the call.
        if unsafe { load(self.plugin, &stream) } {
            Ok(())
        } else {
            Err("the plugin rejected the state".into())
        }
    }

    /// The values the host gave it (D234).
    fn given(&self) -> &Arc<crate::watch::Given> {
        self.given.get_or_init(|| {
            Arc::new(crate::watch::Given::new(
                self.params().iter().map(|p| p.id).collect(),
            ))
        })
    }

    /// Whether it marked its state dirty since the last call (D234).
    pub fn take_dirty(&self) -> bool {
        self.host.dirty.swap(false, Ordering::AcqRel)
    }

    /// The parameters the host gave a value since the last call (D234).
    pub fn take_host_given(&self) -> Vec<u32> {
        self.given().take()
    }

    /// The value the host last gave parameter `id` (D234).
    pub fn host_value(&self, id: u32) -> Option<f64> {
        self.given().value(id)
    }

    /// Whether its GUI is open (D234: its values are polled meanwhile).
    pub fn editor_open(&self) -> bool {
        self.view.is_some()
    }

    /// Sets parameters while not processing (`params.flush` on the main thread).
    pub fn flush_params(&self, params: &[(u32, f64)]) {
        for (id, v) in params {
            self.given().set(*id, *v);
        }
        let Some(flush) = self
            .ext::<clap_plugin_params>(CLAP_EXT_PARAMS)
            .and_then(|e| e.flush)
        else {
            return;
        };
        let mut list = EventList::with_capacity(params.len());
        for (id, v) in params {
            list.push(&Event {
                kind: EV_PARAM,
                time: 0,
                param: *id,
                value: *v,
                ..Event::default()
            });
        }
        let mut sink = OutSink::new(self.host.names.clone());
        let input = list.input();
        let output = sink.output();
        // SAFETY: flushing on the main thread with valid event lists.
        unsafe { flush(self.plugin, &input, &output) };
        let mut open = Vec::new();
        sink.drop_answers(0, &list, &mut open);
        sink.hand(&self.host.edits, true);
    }

    /// Flushes on the main thread if the plugin asked and it is not active (D222).
    fn flush_requested(&self) {
        if self.active || !self.host.flush.swap(false, Ordering::AcqRel) {
            return;
        }
        let Some(flush) = self
            .ext::<clap_plugin_params>(CLAP_EXT_PARAMS)
            .and_then(|e| e.flush)
        else {
            return;
        };
        let list = EventList::with_capacity(1);
        let mut sink = OutSink::new(self.host.names.clone());
        let input = list.input();
        let output = sink.output();
        // SAFETY: flushing on the main thread, not active, with valid event lists.
        unsafe { flush(self.plugin, &input, &output) };
        sink.hand(&self.host.edits, true);
    }

    /// The plugin's own parameter edits since the last call (D222), a gesture's beginning
    /// and end with the value it has then.
    pub fn take_edits(&self) -> Vec<ParamEdit> {
        let mut edits: Vec<ParamEdit> = crate::lock_soon(&self.host.edits).drain(..).collect();
        let ext = self.ext::<clap_plugin_params>(CLAP_EXT_PARAMS);
        for e in edits.iter_mut().filter(|e| e.value.is_none()) {
            let mut v = 0.0f64;
            // SAFETY: a parameter query on the main thread; `v` is writable.
            let ok = ext
                .and_then(|x| x.get_value)
                .is_some_and(|g| unsafe { g(self.plugin, e.param, &mut v) });
            e.value = ok.then_some(v);
        }
        edits
    }

    /// What the plugin asked the host to clear since the last call (D222): parameter and
    /// `CLAP_PARAM_CLEAR_*` flags.
    pub fn take_clears(&self) -> Vec<(u32, u32)> {
        self.host
            .clears
            .lock()
            .map(|mut c| std::mem::take(&mut *c))
            .unwrap_or_default()
    }

    /// Whether its names for its keys may have changed since the last call (D216).
    pub fn take_names_changed(&self) -> bool {
        self.host.names.swap(false, Ordering::AcqRel)
    }

    /// Whether it said since the last call that its parameters' values may have changed by
    /// its own doing (D228: a rescan of values, its state marked dirty).
    pub fn take_values_changed(&self) -> bool {
        self.host.values.swap(false, Ordering::AcqRel)
    }

    /// Whether it rescanned its parameters' descriptions or texts since the last call
    /// (PATCHES.md change 5).
    pub fn take_params_changed(&self) -> bool {
        self.host.infos.swap(false, Ordering::AcqRel)
    }

    /// Its text for each value (plain; PATCHES.md change 5): `value_to_text`.
    pub fn param_text(&self, values: &[(u32, f64)]) -> Vec<Option<String>> {
        let Some(f) = self
            .ext::<clap_plugin_params>(CLAP_EXT_PARAMS)
            .and_then(|e| e.value_to_text)
        else {
            return vec![None; values.len()];
        };
        values
            .iter()
            .map(|(id, v)| {
                let mut buf = [0 as c_char; 256];
                // SAFETY: a parameter query on the main thread; `buf` holds its capacity.
                let ok = unsafe { f(self.plugin, *id, *v, buf.as_mut_ptr(), buf.len() as u32) };
                ok.then(|| array_str(&buf).trim().to_owned())
            })
            .collect()
    }

    /// Every parameter's value as it has it now (D228), plain (its units); not those it
    /// shows but does not take (`CLAP_PARAM_IS_READONLY`: a meter, a display).
    pub fn param_values(&self) -> Vec<(u32, f64)> {
        let Some(ext) = self.ext::<clap_plugin_params>(CLAP_EXT_PARAMS) else {
            return Vec::new();
        };
        // SAFETY: parameter queries on the main thread.
        let n = ext.count.map_or(0, |c| unsafe { c(self.plugin) });
        let mut out = Vec::with_capacity(n as usize);
        for i in 0..n {
            // SAFETY: an all-zero `clap_param_info` is a valid value to be overwritten.
            let mut info: clap_param_info = unsafe { std::mem::zeroed() };
            let listed = ext.get_info.is_some_and(|g| {
                // SAFETY: `info` is writable; `i` is below the count.
                unsafe { g(self.plugin, i, &mut info) }
            });
            if !listed || info.flags & CLAP_PARAM_IS_READONLY != 0 {
                continue;
            }
            let mut v = 0.0f64;
            let read = ext.get_value.is_some_and(|g| {
                // SAFETY: a parameter query on the main thread; `v` is writable.
                unsafe { g(self.plugin, info.id, &mut v) }
            });
            if read {
                out.push((info.id, v));
            }
        }
        out
    }

    pub fn activate(&mut self, sample_rate: f64, max_block: u32) -> Result<(), String> {
        // SAFETY: activation on the main thread, once until deactivated.
        let ok = unsafe {
            (*self.plugin)
                .activate
                .is_some_and(|a| a(self.plugin, sample_rate, 1, max_block))
        };
        if !ok {
            return Err("the plugin failed to activate".into());
        }
        self.active = true;
        Ok(())
    }

    pub fn deactivate(&mut self) {
        if self.active {
            // SAFETY: deactivating an active plugin on the main thread, after its
            // processing thread has stopped.
            unsafe {
                if let Some(d) = (*self.plugin).deactivate {
                    d(self.plugin);
                }
            }
            self.active = false;
        }
    }

    pub fn latency(&self) -> u32 {
        self.ext::<clap_plugin_latency>(CLAP_EXT_LATENCY)
            // SAFETY: latency query on the main thread while active.
            .and_then(|e| e.get.map(|g| unsafe { g(self.plugin) }))
            .unwrap_or(0)
    }

    pub fn tail(&self) -> u32 {
        self.ext::<clap_plugin_tail>(CLAP_EXT_TAIL)
            // SAFETY: tail query on the main thread.
            .and_then(|e| e.get.map(|g| unsafe { g(self.plugin) }))
            .unwrap_or(0)
    }

    /// Runs `on_main_thread` if the plugin asked for a callback.
    pub fn service(&self) {
        if self.host.callback.swap(false, Ordering::Acquire) {
            // SAFETY: on the main thread, as requested.
            unsafe {
                if let Some(f) = (*self.plugin).on_main_thread {
                    f(self.plugin);
                }
            }
        }
        self.flush_requested();
    }

    /// Whether the plugin asked for a restart (for example after a latency change).
    pub fn restart_requested(&self) -> bool {
        self.host.restart.swap(false, Ordering::Relaxed)
    }

    /// A processor for the audio thread, with buffers for `max_block` frames.
    pub fn processor(&self, max_block: usize) -> Processor {
        let make = |is_input: bool| -> Vec<Port> {
            self.ports(is_input)
                .iter()
                .map(|p| Port::new(p.channel_count as usize, max_block))
                .collect()
        };
        let inputs = make(true);
        let outputs = make(false);
        let main_in = self
            .ports(true)
            .iter()
            .position(|p| p.flags & CLAP_AUDIO_PORT_IS_MAIN != 0)
            .or((!inputs.is_empty()).then_some(0));
        let main_out = self
            .ports(false)
            .iter()
            .position(|p| p.flags & CLAP_AUDIO_PORT_IS_MAIN != 0)
            .or((!outputs.is_empty()).then_some(0));
        Processor {
            plugin: self.plugin,
            inputs,
            outputs,
            main_in,
            main_out,
            events: EventList::with_capacity(MAX_EVENTS),
            out: Some(OutSink::new(self.host.names.clone())),
            open: Vec::with_capacity(64),
            edits: self.host.edits.clone(),
            flush: self.host.flush.clone(),
            given: self.given().clone(),
            max_block,
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        self.gui_close();
        self.deactivate();
        // SAFETY: destroying the plugin once, on the main thread; the host box outlives
        // the call and is dropped afterwards.
        unsafe {
            if let Some(d) = (*self.plugin).destroy {
                d(self.plugin);
            }
        }
    }
}

/// One audio port's buffers.
struct Port {
    storage: Vec<Vec<f32>>,
    ptrs: Vec<*mut f32>,
    buffer: clap_audio_buffer,
}

impl Port {
    fn new(channels: usize, frames: usize) -> Port {
        let mut storage: Vec<Vec<f32>> = (0..channels).map(|_| vec![0.0; frames]).collect();
        let ptrs: Vec<*mut f32> = storage.iter_mut().map(|c| c.as_mut_ptr()).collect();
        Port {
            storage,
            ptrs,
            buffer: clap_audio_buffer {
                data32: std::ptr::null_mut(),
                data64: std::ptr::null_mut(),
                channel_count: channels as u32,
                latency: 0,
                constant_mask: 0,
            },
        }
    }

    fn reset_ptrs(&mut self) {
        for (p, c) in self.ptrs.iter_mut().zip(self.storage.iter_mut()) {
            *p = c.as_mut_ptr();
        }
        self.buffer.data32 = self.ptrs.as_mut_ptr();
    }
}

/// An event in CLAP form: every variant starts with a `clap_event_header`.
#[repr(C)]
#[derive(Clone, Copy)]
union ClapEvent {
    note: clap_event_note,
    value: clap_event_param_value,
    modulation: clap_event_param_mod,
}

/// A preallocated CLAP input event list.
struct EventList {
    events: Vec<ClapEvent>,
}

impl EventList {
    fn with_capacity(n: usize) -> EventList {
        EventList {
            events: Vec::with_capacity(n.max(1)),
        }
    }

    fn clear(&mut self) {
        self.events.clear();
    }

    /// Adds an event; drops it if the list is full (never allocates after creation).
    fn push(&mut self, e: &Event) {
        if self.events.len() == self.events.capacity() {
            return;
        }
        let header = |size: usize, ty: u16| clap_event_header {
            size: size as u32,
            time: e.time,
            space_id: CLAP_CORE_EVENT_SPACE_ID,
            type_: ty,
            flags: 0,
        };
        let ev = match e.kind {
            EV_NOTE_ON | EV_NOTE_OFF => ClapEvent {
                note: clap_event_note {
                    header: header(
                        std::mem::size_of::<clap_event_note>(),
                        if e.kind == EV_NOTE_ON {
                            CLAP_EVENT_NOTE_ON
                        } else {
                            CLAP_EVENT_NOTE_OFF
                        },
                    ),
                    note_id: e.note_id,
                    port_index: 0,
                    channel: e.channel,
                    key: e.key,
                    velocity: e.value,
                },
            },
            EV_PARAM => ClapEvent {
                value: clap_event_param_value {
                    header: header(
                        std::mem::size_of::<clap_event_param_value>(),
                        CLAP_EVENT_PARAM_VALUE,
                    ),
                    param_id: e.param,
                    cookie: std::ptr::null_mut(),
                    note_id: -1,
                    port_index: -1,
                    channel: -1,
                    key: -1,
                    value: e.value,
                },
            },
            EV_PARAM_MOD => ClapEvent {
                modulation: clap_event_param_mod {
                    header: header(
                        std::mem::size_of::<clap_event_param_mod>(),
                        CLAP_EVENT_PARAM_MOD,
                    ),
                    param_id: e.param,
                    cookie: std::ptr::null_mut(),
                    note_id: -1,
                    port_index: -1,
                    channel: -1,
                    key: -1,
                    amount: e.value,
                },
            },
            _ => return,
        };
        self.events.push(ev);
    }

    /// Whether the list holds a value `value` for parameter `param` (to a hair).
    fn has_value(&self, param: u32, value: f64) -> bool {
        self.events.iter().any(|e| {
            // SAFETY: every variant begins with its header.
            let h = unsafe { e.note.header };
            h.type_ == CLAP_EVENT_PARAM_VALUE && {
                // SAFETY: the type says this is a parameter value.
                let v = unsafe { e.value };
                v.param_id == param && (v.value - value).abs() <= 1e-9 * value.abs().max(1.0)
            }
        })
    }

    fn input(&self) -> clap_input_events {
        clap_input_events {
            ctx: self as *const EventList as *mut c_void,
            size: Some(list_size),
            get: Some(list_get),
        }
    }
}

unsafe extern "C" fn list_size(l: *const clap_input_events) -> u32 {
    // SAFETY: `ctx` is the `EventList` that built this list and outlives the call.
    unsafe { (*((*l).ctx as *const EventList)).events.len() as u32 }
}

unsafe extern "C" fn list_get(l: *const clap_input_events, i: u32) -> *const clap_event_header {
    // SAFETY: as above.
    let list = unsafe { &*((*l).ctx as *const EventList) };
    list.events.get(i as usize).map_or(std::ptr::null(), |e| {
        e as *const ClapEvent as *const clap_event_header
    })
}

/// Where a plugin's output events go (D216, D222): a parameter value it sends (an edit
/// in its editor) marks its names for its keys as maybe changed, and with gestures it is
/// kept as an edit for the project. Made off the audio thread; never allocates after.
struct OutSink {
    names: Arc<AtomicBool>,
    /// This call's edits, at most [`EDITS_PER_CALL`].
    edits: Vec<ParamEdit>,
}

impl OutSink {
    fn new(names: Arc<AtomicBool>) -> OutSink {
        OutSink {
            names,
            edits: Vec::with_capacity(EDITS_PER_CALL),
        }
    }

    /// An output event list over this sink, which must outlive its use and not move.
    fn output(&mut self) -> clap_output_events {
        clap_output_events {
            ctx: self as *mut OutSink as *mut c_void,
            try_push: Some(edits_push),
        }
    }

    /// Drops, from its edits since `from`, the values outside a gesture that answer the
    /// values of `input` (the same parameter and value: a plugin sending back what the host
    /// sent it; the review's finding), keeping the gestures open in `open`. Never
    /// allocates beyond `open`'s room.
    fn drop_answers(&mut self, from: usize, input: &EventList, open: &mut Vec<u32>) {
        let mut kept = from;
        for k in from..self.edits.len() {
            let e = self.edits[k];
            let keep = match e.kind {
                EditKind::Begin => {
                    if open.len() < open.capacity() || open.capacity() == 0 {
                        open.push(e.param);
                    }
                    true
                }
                EditKind::End => {
                    open.retain(|p| *p != e.param);
                    true
                }
                EditKind::Value => {
                    open.contains(&e.param) || !e.value.is_some_and(|v| input.has_value(e.param, v))
                }
            };
            if keep {
                self.edits[kept] = e;
                kept += 1;
            }
        }
        self.edits.truncate(kept);
    }

    /// Hands its edits to the main thread's list: waiting for it (`wait`, off the audio
    /// thread) or only if it is free, as many as it has room for (the rest wait).
    fn hand(&mut self, to: &std::sync::Mutex<Vec<ParamEdit>>, wait: bool) {
        if self.edits.is_empty() {
            return;
        }
        let list = if wait {
            Some(crate::lock_soon(to))
        } else {
            to.try_lock().ok()
        };
        if let Some(mut list) = list {
            let room = if wait {
                self.edits.len()
            } else {
                list.capacity() - list.len()
            };
            let n = room.min(self.edits.len());
            list.extend(self.edits.drain(..n));
        }
    }
}

/// Takes a plugin's output events into its [`OutSink`].
unsafe extern "C" fn edits_push(l: *const clap_output_events, e: *const clap_event_header) -> bool {
    // SAFETY: the plugin passes our list and a valid event header, or null; the list's
    // context is the sink that made it, alive and not otherwise used during the call.
    let (sink, h) = unsafe { (((*l).ctx as *mut OutSink).as_mut(), e.as_ref()) };
    let (Some(sink), Some(h)) = (sink, h) else {
        return false;
    };
    if h.space_id != CLAP_CORE_EVENT_SPACE_ID {
        return true;
    }
    // Not to be recorded (an automation's playback, the plugin says): no edit, though the
    // names may have changed (the review's finding).
    if h.flags & CLAP_EVENT_DONT_RECORD != 0 {
        if h.type_ == CLAP_EVENT_PARAM_VALUE {
            sink.names.store(true, Ordering::Release);
        }
        return true;
    }
    let edit = match h.type_ {
        CLAP_EVENT_PARAM_VALUE => {
            sink.names.store(true, Ordering::Release);
            // SAFETY: the type says this is a parameter value event.
            let v = unsafe { &*(e as *const clap_event_param_value) };
            ParamEdit {
                param: v.param_id,
                kind: EditKind::Value,
                value: Some(v.value),
            }
        }
        CLAP_EVENT_PARAM_GESTURE_BEGIN | CLAP_EVENT_PARAM_GESTURE_END => {
            // SAFETY: the type says this is a gesture event.
            let g = unsafe { &*(e as *const clap_event_param_gesture) };
            ParamEdit {
                param: g.param_id,
                kind: if h.type_ == CLAP_EVENT_PARAM_GESTURE_BEGIN {
                    EditKind::Begin
                } else {
                    EditKind::End
                },
                value: None,
            }
        }
        _ => return true,
    };
    if sink.edits.len() < sink.edits.capacity() {
        sink.edits.push(edit);
    }
    true
}

/// Drives one instance's processing; lives on the instance's audio thread.
pub struct Processor {
    plugin: *const clap_plugin,
    inputs: Vec<Port>,
    outputs: Vec<Port>,
    main_in: Option<usize>,
    main_out: Option<usize>,
    events: EventList,
    /// Its output events: the names flag and its edits (D216, D222); taken out of it for
    /// the call that fills it.
    out: Option<OutSink>,
    /// The plugin's gestures open, to tell its edits from its answers (D222).
    open: Vec<u32>,
    /// The instance's list of edits for the main thread, and its flush request (D222).
    edits: Arc<std::sync::Mutex<Vec<ParamEdit>>>,
    flush: Arc<AtomicBool>,
    /// The values the host gives it, for its main thread's watch (D234).
    given: Arc<crate::watch::Given>,
    max_block: usize,
}

impl std::fmt::Debug for Processor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Processor").finish_non_exhaustive()
    }
}

// SAFETY: moved once to the audio thread, which then owns it; the plugin pointer is used
// there only for audio-thread calls.
unsafe impl Send for Processor {}

impl Processor {
    /// Marks the calling thread as the audio thread and starts processing.
    pub fn start(&mut self) -> bool {
        AUDIO_THREAD.with(|a| a.set(true));
        // SAFETY: `start_processing` on the audio thread of an active plugin.
        unsafe {
            (*self.plugin)
                .start_processing
                .is_none_or(|s| s(self.plugin))
        }
    }

    pub fn stop(&mut self) {
        // SAFETY: `stop_processing` on the audio thread.
        unsafe {
            if let Some(s) = (*self.plugin).stop_processing {
                s(self.plugin);
            }
        }
    }

    /// Processes one block in place: `input` and `output` are the transport slot's stereo
    /// channels. Returns a transport status.
    ///
    /// # Safety
    /// The four channel pointers must each be valid for `frames` samples, and `frames`
    /// must not exceed the processor's block size.
    pub unsafe fn process(
        &mut self,
        steady_time: i64,
        frames: usize,
        events: &[Event],
        input: [*mut f32; 2],
        output: [*mut f32; 2],
    ) -> i32 {
        let frames = frames.min(self.max_block).min(MAX_BLOCK);
        self.events.clear();
        for e in events {
            self.events.push(e);
            if e.kind == EV_PARAM {
                self.given.set(e.param, e.value);
            }
        }
        for p in self.inputs.iter_mut().chain(self.outputs.iter_mut()) {
            p.reset_ptrs();
        }
        // Point the main ports at the transport; extra channels keep their own buffers.
        if let Some(m) = self.main_in {
            let p = &mut self.inputs[m];
            if p.ptrs.len() == 1 {
                p.ptrs[0] = input[0];
            } else if p.ptrs.len() >= 2 {
                p.ptrs[0] = input[0];
                p.ptrs[1] = input[1];
            }
        }
        for (i, p) in self.inputs.iter_mut().enumerate() {
            if Some(i) != self.main_in {
                for c in &mut p.storage {
                    c[..frames].fill(0.0);
                }
            }
        }
        if let Some(m) = self.main_out {
            let p = &mut self.outputs[m];
            for (k, ptr) in p.ptrs.iter_mut().take(2).enumerate() {
                *ptr = output[k];
            }
        }
        let in_events = self.events.input();
        let Some(mut out) = self.out.take() else {
            return STATUS_ERROR;
        };
        let from = out.edits.len();
        let out_events = out.output();
        // A block is the flush a plugin may have asked for (D222).
        self.flush.store(false, Ordering::Release);
        let proc_ = clap_process {
            steady_time,
            frames_count: frames as u32,
            transport: std::ptr::null(),
            audio_inputs: std::ptr::null(),
            audio_outputs: std::ptr::null_mut(),
            audio_inputs_count: 0,
            audio_outputs_count: 0,
            in_events: &in_events,
            out_events: &out_events,
        };
        // SAFETY: the caller's contract covers the transport channels; the event lists
        // outlive the call.
        let status = unsafe { self.call_process(&proc_) };
        out.drop_answers(from, &self.events, &mut self.open);
        out.hand(&self.edits, false);
        self.out = Some(out);
        // A mono main output feeds both channels.
        if let Some(m) = self.main_out
            && self.outputs[m].ptrs.len() == 1
        {
            // SAFETY: both output channels are valid for `frames` samples.
            unsafe { std::ptr::copy_nonoverlapping(output[0], output[1], frames) };
        }
        if status == CLAP_PROCESS_ERROR {
            STATUS_ERROR
        } else {
            STATUS_OK
        }
    }

    /// Between blocks, when none came for a while (D222): a flush the plugin asked for, on
    /// this, the audio thread, as CLAP asks of an active plugin.
    pub fn idle(&mut self) {
        if !self.flush.swap(false, Ordering::AcqRel) {
            return;
        }
        // SAFETY: a valid plugin; `get_extension` is callable from any thread.
        let ext = unsafe {
            (*self.plugin)
                .get_extension
                .map(|g| g(self.plugin, CLAP_EXT_PARAMS.as_ptr()))
        }
        .unwrap_or(std::ptr::null()) as *const clap_plugin_params;
        // SAFETY: a non-null extension pointer has the type its ID names.
        let Some(flush) = (unsafe { ext.as_ref() }).and_then(|e| e.flush) else {
            return;
        };
        let Some(mut out) = self.out.take() else {
            return;
        };
        self.events.clear();
        let input = self.events.input();
        let output = out.output();
        // SAFETY: flushing on the audio thread between blocks, with valid event lists.
        unsafe { flush(self.plugin, &input, &output) };
        out.hand(&self.edits, false);
        self.out = Some(out);
    }

    /// # Safety
    /// See [`Processor::process`].
    unsafe fn call_process(&mut self, base: &clap_process) -> i32 {
        // CLAP wants contiguous arrays of `clap_audio_buffer`; gather them on the stack
        // for the common port counts and fall back to the first port otherwise.
        const MAX_PORTS: usize = 8;
        let empty = clap_audio_buffer {
            data32: std::ptr::null_mut(),
            data64: std::ptr::null_mut(),
            channel_count: 0,
            latency: 0,
            constant_mask: 0,
        };
        let mut ins = [empty; MAX_PORTS];
        let mut outs = [empty; MAX_PORTS];
        let ni = self.inputs.len().min(MAX_PORTS);
        let no = self.outputs.len().min(MAX_PORTS);
        for (d, p) in ins.iter_mut().zip(self.inputs.iter_mut()) {
            p.buffer.data32 = p.ptrs.as_mut_ptr();
            *d = p.buffer;
        }
        for (d, p) in outs.iter_mut().zip(self.outputs.iter_mut()) {
            p.buffer.data32 = p.ptrs.as_mut_ptr();
            *d = p.buffer;
        }
        let proc_ = clap_process {
            audio_inputs: if ni == 0 {
                std::ptr::null()
            } else {
                ins.as_ptr()
            },
            audio_outputs: if no == 0 {
                std::ptr::null_mut()
            } else {
                outs.as_mut_ptr()
            },
            audio_inputs_count: ni as u32,
            audio_outputs_count: no as u32,
            ..*base
        };
        // SAFETY: every pointer in `proc_` is valid for the call (see the caller's
        // contract); `process` is called on the audio thread.
        unsafe {
            (*self.plugin)
                .process
                .map_or(CLAP_PROCESS_ERROR, |p| p(self.plugin, &proc_))
        }
    }
}
