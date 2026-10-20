//! A plugin of either format the helper hosts (DESIGN 10.6): CLAP, or VST3 (M6 plan W3).
//! A file is tried as CLAP first, then as a VST3 module; the rest of the helper sees one
//! library, instance and processor type.

use crate::{clap, vst3};
use daw_ipc::Event;
use daw_ipc::protocol::{ParamInfo, PluginDesc};
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;

/// A loaded plugin file.
#[derive(Debug, Clone)]
pub enum Library {
    Clap(Arc<clap::Library>),
    Vst3(Arc<vst3::Module>),
}

impl Library {
    /// Loads a file as CLAP, else as VST3; the error names both attempts.
    pub fn load(path: &Path) -> Result<Library, String> {
        match clap::Library::load(path) {
            Ok(l) => Ok(Library::Clap(Arc::new(l))),
            Err(clap_err) => match vst3::Module::load(path) {
                Ok(m) => Ok(Library::Vst3(Arc::new(m))),
                Err(vst3_err) => Err(format!("{clap_err}; {vst3_err}")),
            },
        }
    }

    /// The plugins in the file, each with its format.
    pub fn descriptors(&self) -> Vec<PluginDesc> {
        let (mut d, format) = match self {
            Library::Clap(l) => (l.descriptors(), "clap"),
            Library::Vst3(m) => (m.descriptors(), "vst3"),
        };
        for x in &mut d {
            x.format = format.to_owned();
        }
        d
    }

    pub fn create(&self, id: &str) -> Result<Instance, String> {
        match self {
            Library::Clap(l) => l.create(id).map(Instance::Clap),
            Library::Vst3(m) => m.instantiate(id).map(|i| Instance::Vst3(Box::new(i))),
        }
    }
}

/// A plugin instance.
#[derive(Debug)]
pub enum Instance {
    Clap(clap::Instance),
    Vst3(Box<vst3::Instance>),
}

impl Instance {
    pub fn descriptor(&self) -> PluginDesc {
        match self {
            Instance::Clap(i) => PluginDesc {
                format: "clap".into(),
                ..i.descriptor()
            },
            Instance::Vst3(i) => PluginDesc {
                format: "vst3".into(),
                ..i.descriptor()
            },
        }
    }

    pub fn params(&self) -> Vec<ParamInfo> {
        match self {
            Instance::Clap(i) => i.params(),
            Instance::Vst3(i) => i.params(),
        }
    }

    pub fn main_ports(&self) -> (u32, u32) {
        match self {
            Instance::Clap(i) => i.main_ports(),
            Instance::Vst3(i) => i.main_ports(),
        }
    }

    pub fn takes_notes(&self) -> bool {
        match self {
            Instance::Clap(i) => i.takes_notes(),
            Instance::Vst3(i) => i.takes_notes(),
        }
    }

    /// The plugin's names for its keys (M18 W9): CLAP's note-name extension, VST3's
    /// program pitch names for its current program (D211).
    pub fn note_names(&self) -> Vec<daw_ipc::protocol::NoteName> {
        match self {
            Instance::Clap(i) => i.note_names(),
            Instance::Vst3(i) => i.note_names(),
        }
    }

    pub fn save_state(&self) -> Result<Vec<u8>, String> {
        match self {
            Instance::Clap(i) => i.save_state(),
            Instance::Vst3(i) => i.save_state(),
        }
    }

    pub fn load_state(&self, bytes: &[u8]) -> Result<(), String> {
        match self {
            Instance::Clap(i) => i.load_state(bytes),
            Instance::Vst3(i) => i.load_state(bytes),
        }
    }

    pub fn flush_params(&self, params: &[(u32, f64)]) {
        match self {
            Instance::Clap(i) => i.flush_params(params),
            Instance::Vst3(i) => i.flush_params(params),
        }
    }

    pub fn activate(&mut self, sample_rate: f64, max_block: u32) -> Result<(), String> {
        match self {
            Instance::Clap(i) => i.activate(sample_rate, max_block),
            Instance::Vst3(i) => i.activate(sample_rate, max_block),
        }
    }

    pub fn latency(&self) -> u32 {
        match self {
            Instance::Clap(i) => i.latency(),
            Instance::Vst3(i) => i.latency(),
        }
    }

    pub fn tail(&self) -> u32 {
        match self {
            Instance::Clap(i) => i.tail(),
            Instance::Vst3(i) => i.tail(),
        }
    }

    /// Main-thread duties between requests: callbacks the plugin asked for (CLAP), the
    /// values its processor was sent handed to its controller (VST3, D216).
    pub fn service(&self) {
        match self {
            Instance::Clap(i) => i.service(),
            Instance::Vst3(i) => i.service(),
        }
    }

    /// The plugin's own parameter edits since the last call (D222: its editor's), in its
    /// units.
    pub fn take_edits(&self) -> Vec<daw_ipc::protocol::ParamEdit> {
        match self {
            Instance::Clap(i) => i.take_edits(),
            Instance::Vst3(i) => i.take_edits(),
        }
    }

    /// What a CLAP plugin asked the host to clear since the last call (D222): parameter
    /// and flags.
    pub fn take_clears(&self) -> Vec<(u32, u32)> {
        match self {
            Instance::Clap(i) => i.take_clears(),
            Instance::Vst3(_) => Vec::new(),
        }
    }

    /// Whether it said since the last call that its parameters' values changed by its own
    /// doing (D228: a preset loaded in its editor).
    pub fn take_values_changed(&self) -> bool {
        match self {
            Instance::Clap(i) => i.take_values_changed(),
            Instance::Vst3(i) => i.take_values_changed(),
        }
    }

    /// Every parameter's value now (D228), in its units.
    pub fn param_values(&self) -> Vec<(u32, f64)> {
        match self {
            Instance::Clap(i) => i.param_values(),
            Instance::Vst3(i) => i.param_values(),
        }
    }

    /// What it said of its state since the last call (D234): VST3's `setDirty` (something
    /// besides its parameters), CLAP's `mark_dirty` (its state).
    pub fn take_state_said(&self) -> Option<crate::watch::Said> {
        match self {
            Instance::Clap(i) => i.take_dirty().then_some(crate::watch::Said::State),
            Instance::Vst3(i) => i.take_dirty().then_some(crate::watch::Said::Besides),
        }
    }

    /// The parameters the host gave a value since the last call (D234).
    pub fn take_host_given(&self) -> Vec<u32> {
        match self {
            Instance::Clap(i) => i.take_host_given(),
            Instance::Vst3(i) => i.take_host_given(),
        }
    }

    /// The value the host last gave parameter `id`, in its units (D234).
    pub fn host_value(&self, id: u32) -> Option<f64> {
        match self {
            Instance::Clap(i) => i.host_value(id),
            Instance::Vst3(i) => i.host_value(id),
        }
    }

    /// Whether its editor is open (D234).
    pub fn editor_open(&self) -> bool {
        match self {
            Instance::Clap(i) => i.editor_open(),
            Instance::Vst3(i) => i.editor_open(),
        }
    }

    /// Whether its names for its keys may have changed since the last call (D216): the
    /// plugin said so, edited a parameter itself, or a program change moved.
    pub fn take_names_changed(&self) -> bool {
        match self {
            Instance::Clap(i) => i.take_names_changed(),
            Instance::Vst3(i) => i.take_names_changed(),
        }
    }

    pub fn restart_requested(&self) -> bool {
        match self {
            Instance::Clap(i) => i.restart_requested(),
            Instance::Vst3(i) => i.restart_requested(),
        }
    }

    pub fn processor(&self, max_block: usize) -> Processor {
        match self {
            Instance::Clap(i) => Processor::Clap(i.processor(max_block)),
            Instance::Vst3(i) => Processor::Vst3(i.processor(max_block)),
        }
    }

    pub fn gui_open(&mut self, title: &str, at: Option<(i32, i32)>) -> Result<Value, String> {
        match self {
            Instance::Clap(i) => i.gui_open(title, at),
            Instance::Vst3(i) => i.gui_open(title, at),
        }
    }

    pub fn gui_move(&mut self, x: i32, y: i32) -> Result<Value, String> {
        match self {
            Instance::Clap(i) => i.gui_move(x, y),
            Instance::Vst3(i) => i.gui_move(x, y),
        }
    }

    pub fn gui_close(&mut self) {
        match self {
            Instance::Clap(i) => i.gui_close(),
            Instance::Vst3(i) => i.gui_close(),
        }
    }

    pub fn gui_info(&self) -> Value {
        match self {
            Instance::Clap(i) => i.gui_info(),
            Instance::Vst3(i) => i.gui_info(),
        }
    }

    /// Whether the user closed the GUI since the last call.
    pub fn gui_service(&mut self) -> bool {
        match self {
            Instance::Clap(i) => i.gui_service(),
            Instance::Vst3(i) => i.gui_service(),
        }
    }
}

/// The audio thread's side of an instance.
#[derive(Debug)]
pub enum Processor {
    Clap(clap::Processor),
    Vst3(vst3::Processor),
}

impl Processor {
    pub fn start(&mut self) -> bool {
        match self {
            Processor::Clap(p) => p.start(),
            Processor::Vst3(p) => p.start(),
        }
    }

    pub fn stop(&mut self) {
        match self {
            Processor::Clap(p) => p.stop(),
            Processor::Vst3(p) => p.stop(),
        }
    }

    /// Between blocks, when none came for a while (D222): a CLAP plugin's flush it asked
    /// for, on the audio thread.
    pub fn idle(&mut self) {
        match self {
            Processor::Clap(p) => p.idle(),
            Processor::Vst3(p) => p.idle(),
        }
    }

    /// Processes one block in place.
    ///
    /// # Safety
    /// The four channel pointers must each be valid for `frames` samples, and `frames`
    /// must not exceed the processor's block size.
    #[allow(unsafe_code)]
    pub unsafe fn process(
        &mut self,
        steady_time: i64,
        frames: usize,
        events: &[Event],
        input: [*mut f32; 2],
        output: [*mut f32; 2],
    ) -> i32 {
        match self {
            // SAFETY: the caller's contract is the backends'.
            Processor::Clap(p) => unsafe { p.process(steady_time, frames, events, input, output) },
            // SAFETY: as above.
            Processor::Vst3(p) => unsafe { p.process(steady_time, frames, events, input, output) },
        }
    }
}
