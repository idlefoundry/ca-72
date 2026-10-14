//! Save the comparison presets through the actual VST3 state interface, without audio.
#![allow(unsafe_code, clippy::unwrap_used)]

use std::cell::Cell;
use std::collections::BTreeMap;
use std::ffi::c_void;
use std::path::Path;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};

use ca72_plugin::Ca72;
use ca72_plugin::library::parse_factory;
use ca72_plugin::params::{Ca72Params, QualityMode};
use nih_plug::prelude::*;
use nih_plug::wrapper::state::ParamValue;
use nih_plug::wrapper::vst3::vst3_sys;
use serde_json::json;
use vst3_sys as vst3_com;
use vst3_sys::base::{IBStream, IPluginFactory, kResultFalse, kResultOk, tresult};
use vst3_sys::utils::SharedVstPtr;
use vst3_sys::vst::{IComponent, IEditController};
use vst3_sys::{ComInterface, IID, VST3, VstPtr};

unsafe extern "system" {
    #[link_name = "GetPluginFactory"]
    fn plugin_factory() -> *mut c_void;
}

#[VST3(implements(IBStream))]
struct Stream {
    bytes: Arc<Mutex<Vec<u8>>>,
    at: Cell<usize>,
}

impl IBStream for Stream {
    unsafe fn read(&self, buffer: *mut c_void, n: i32, read: *mut i32) -> tresult {
        if n < 0 || (n > 0 && buffer.is_null()) {
            return kResultFalse;
        }
        let bytes = self.bytes.lock().unwrap();
        let at = self.at.get();
        let count = (n as usize).min(bytes.len().saturating_sub(at));
        if count > 0 {
            // SAFETY: the host gives a buffer of n bytes; count is within both buffers.
            unsafe { std::ptr::copy_nonoverlapping(bytes[at..].as_ptr(), buffer.cast(), count) };
        }
        self.at.set(at + count);
        if !read.is_null() {
            // SAFETY: a non-null IBStream result pointer is writable for this synchronous call.
            unsafe { *read = count as i32 };
        }
        kResultOk
    }

    unsafe fn write(&self, buffer: *const c_void, n: i32, written: *mut i32) -> tresult {
        if n < 0 || (n > 0 && buffer.is_null()) {
            return kResultFalse;
        }
        let mut bytes = self.bytes.lock().unwrap();
        let at = self.at.get();
        let Some(end) = at.checked_add(n as usize) else {
            return kResultFalse;
        };
        let length = end.max(bytes.len());
        bytes.resize(length, 0);
        if n > 0 {
            // SAFETY: the caller supplies n readable bytes, and the destination was resized.
            unsafe {
                std::ptr::copy_nonoverlapping(buffer.cast(), bytes[at..].as_mut_ptr(), n as usize)
            };
        }
        self.at.set(end);
        if !written.is_null() {
            // SAFETY: a non-null IBStream result pointer is writable for this synchronous call.
            unsafe { *written = n };
        }
        kResultOk
    }

    unsafe fn seek(&self, offset: i64, mode: i32, result: *mut i64) -> tresult {
        let base = match mode {
            0 => 0,
            1 => self.at.get() as i64,
            2 => self.bytes.lock().unwrap().len() as i64,
            _ => return kResultFalse,
        };
        let Some(at) = base
            .checked_add(offset)
            .and_then(|v| usize::try_from(v).ok())
        else {
            return kResultFalse;
        };
        self.at.set(at);
        if !result.is_null() {
            // SAFETY: the host supplies a writable result pointer, checked above.
            unsafe { *result = at as i64 };
        }
        kResultOk
    }

    unsafe fn tell(&self, at: *mut i64) -> tresult {
        if at.is_null() {
            return kResultFalse;
        }
        // SAFETY: the host supplies a writable position pointer, checked above.
        unsafe { *at = self.at.get() as i64 };
        kResultOk
    }
}

fn stream(bytes: Arc<Mutex<Vec<u8>>>) -> VstPtr<dyn IBStream> {
    // SAFETY: this COM allocation begins with the IBStream vtable and one owned reference.
    unsafe { VstPtr::owned(Box::into_raw(Stream::allocate(bytes, Cell::new(0))).cast()) }.unwrap()
}

fn shared(stream: &VstPtr<dyn IBStream>) -> SharedVstPtr<dyn IBStream> {
    // SAFETY: SharedVstPtr is the interface pointer, borrowed for the duration of this call.
    unsafe { std::mem::transmute_copy(&stream.as_ptr()) }
}

fn hash(id: &str) -> u32 {
    id.bytes()
        .fold(0_u32, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(b)))
        & !(1 << 31)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: comparison_states <bank.toml> <output.json>".into());
    }
    let sounds = parse_factory(&std::fs::read_to_string(&args[1])?)?;
    if sounds.len() != 24 {
        return Err("expected 24 presets".into());
    }
    let schema = Ca72Params::default();
    let map = schema.param_map();
    // SAFETY: the factory is exported by our linked CA-72 library and returns one owned ref.
    let factory = unsafe { VstPtr::<dyn IPluginFactory>::owned(plugin_factory().cast()) }.unwrap();
    let cid = IID {
        data: Ca72::PLATFORM_VST3_CLASS_ID,
    };
    let mut presets = Vec::new();
    for sound in sounds {
        let mut object = null_mut();
        // SAFETY: live factory, valid class/interface IDs and writable output pointer.
        unsafe {
            assert_eq!(
                factory.create_instance(&cid, &<dyn IComponent as ComInterface>::IID, &mut object),
                kResultOk
            );
        }
        // SAFETY: create_instance returned an owned IComponent reference of our own class.
        let component = unsafe { VstPtr::<dyn IComponent>::owned(object.cast()) }.unwrap();
        let controller: VstPtr<dyn IEditController> = component.cast().unwrap();
        let mut values = BTreeMap::new();
        for (id, ptr, _) in &map {
            // SAFETY: schema owns all of the parameter pointers throughout this loop.
            let plain = sound
                .values
                .iter()
                .find(|(key, _)| key == id)
                .map(|(_, v)| *v as f32)
                .unwrap_or_else(|| unsafe { ptr.default_plain_value() });
            let plain = if id == "quality" {
                QualityMode::Ultra.to_index() as f32
            } else {
                plain
            };
            let typed = match ptr {
                ParamPtr::FloatParam(_) => ParamValue::F32(plain),
                ParamPtr::IntParam(_) | ParamPtr::EnumParam(_) => {
                    ParamValue::I32(plain.round() as i32)
                }
                ParamPtr::BoolParam(_) => ParamValue::Bool(plain >= 0.5),
            };
            values.insert(id.clone(), typed);
        }
        for (id, _) in &sound.values {
            assert!(values.contains_key(id), "unknown sound parameter {id}");
        }
        let mut fields = schema.serialize_fields();
        fields.insert("preset".into(), serde_json::to_string(&sound.name)?);
        fields.insert("noise_seed".into(), "72".into());
        fields.insert("editor_panel".into(), "true".into());
        let state = PluginState {
            version: Ca72::VERSION.into(),
            params: values,
            fields,
        };
        let input = stream(Arc::new(Mutex::new(serde_json::to_vec(&state)?)));
        // SAFETY: our component and input stream remain owned throughout the synchronous call.
        unsafe { assert_eq!(component.set_state(shared(&input)), kResultOk) };
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let output = stream(bytes.clone());
        // SAFETY: our component and output stream remain owned throughout the synchronous call.
        unsafe { assert_eq!(component.get_state(shared(&output)), kResultOk) };
        let bytes = bytes.lock().unwrap().clone();
        let recalled: PluginState = serde_json::from_slice(&bytes)?;
        let mut parameters = Vec::new();
        for (id, ptr, _) in &map {
            // SAFETY: controller is a live interface and this ID comes from its parameter schema.
            let normalized = unsafe { controller.get_param_normalized(hash(id)) };
            // SAFETY: schema still owns the parameter addressed by ptr.
            let plain = unsafe { ptr.preview_plain(normalized as f32) };
            let wanted = match &state.params[id] {
                ParamValue::F32(v) => *v,
                ParamValue::I32(v) => *v as f32,
                ParamValue::Bool(v) => f32::from(u8::from(*v)),
                ParamValue::String(_) => unreachable!(),
            };
            assert!(
                (plain - wanted).abs() < 0.00002,
                "{} / {id}: {plain} != {wanted}",
                sound.name
            );
            // SAFETY: schema still owns the parameter addressed by ptr.
            let name = unsafe { ptr.name() };
            parameters.push(json!({"id": id, "vst3_id": hash(id), "name": name, "plain": plain, "normalized": normalized}));
        }
        assert_eq!(
            recalled.fields["preset"],
            serde_json::to_string(&sound.name)?
        );
        assert_eq!(recalled.fields["noise_seed"], "72");
        presets.push(json!({"name":sound.name,"state":recalled,"parameters":parameters}));
    }
    let result = json!({"format":1,"state_interface":"actual CA-72 VST3 IComponent set_state/get_state","audio_rendered":false,"presets":presets});
    std::fs::write(Path::new(&args[2]), serde_json::to_vec_pretty(&result)?)?;
    println!("24 complete presets recalled and saved through VST3; no audio rendered");
    Ok(())
}
