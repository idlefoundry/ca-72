use std::ffi::c_void;
use std::ptr::null_mut;

use nih_plug::wrapper::vst3::vst3_sys;
use vst3_sys as vst3_com;
use vst3_sys::base::{kResultFalse, kResultOk, tresult};
use vst3_sys::utils::StaticVstPtr;
use vst3_sys::vst::{IParamValueQueue, IParameterChanges};
use vst3_sys::{ComInterface, VST3, VstPtr};

/// A parameter's VST3 id: nih-plug's hash of its string id (`wrapper::util::hash_param_id`).
pub fn id(param: &str) -> u32 {
    let mut h: u32 = 0;
    for b in param.bytes() {
        h = h.wrapping_mul(31).wrapping_add(u32::from(b));
    }
    h & !(1 << 31)
}

/// One parameter's changes in a process call.
#[VST3(implements(IParamValueQueue))]
struct Queue {
    id: u32,
    points: Vec<(i32, f64)>,
}

impl IParamValueQueue for Queue {
    unsafe fn get_parameter_id(&self) -> u32 {
        self.id
    }

    unsafe fn get_point_count(&self) -> i32 {
        self.points.len() as i32
    }

    unsafe fn get_point(&self, index: i32, sample_offset: *mut i32, value: *mut f64) -> tresult {
        let Some(&(offset, normalized)) = self.points.get(index as usize) else {
            return kResultFalse;
        };
        // SAFETY: the plug-in's pointers to its own locals, for this call.
        unsafe {
            *sample_offset = offset;
            *value = normalized;
        }
        kResultOk
    }

    unsafe fn add_point(&self, _offset: i32, _value: f64, _index: *mut i32) -> tresult {
        kResultFalse
    }
}

/// A process call's parameter changes.
#[VST3(implements(IParameterChanges))]
struct Changes {
    queues: Vec<VstPtr<dyn IParamValueQueue>>,
}

impl IParameterChanges for Changes {
    unsafe fn get_parameter_count(&self) -> i32 {
        self.queues.len() as i32
    }

    unsafe fn get_parameter_data(&self, index: i32) -> StaticVstPtr<dyn IParamValueQueue> {
        let queue = usize::try_from(index).ok().and_then(|i| self.queues.get(i));
        // SAFETY: a pointer `StaticVstPtr` wraps unowned, alive with this object.
        unsafe { pointer(queue.map_or(null_mut(), |q| q.as_ptr().cast())) }
    }

    unsafe fn add_parameter_data(
        &self,
        _id: *const u32,
        _index: *mut i32,
    ) -> StaticVstPtr<dyn IParamValueQueue> {
        // SAFETY: a null pointer.
        unsafe { pointer(null_mut()) }
    }
}

/// An interface pointer as `SharedVstPtr` or `StaticVstPtr`, each a transparent raw pointer.
///
/// # Safety
///
/// `P` is one of those two, for the interface `p` points to (or null).
pub unsafe fn pointer<P>(p: *mut c_void) -> P {
    assert_eq!(size_of::<P>(), size_of::<*mut c_void>());
    // SAFETY: the same representation (above, and the caller's promise of the type).
    unsafe { std::mem::transmute_copy(&p) }
}

/// One of the host's COM objects as its first (only) interface, owned by the pointer returned.
pub fn com<I: ComInterface + ?Sized, T>(object: Box<T>) -> VstPtr<I> {
    // SAFETY: a `VST3` object's first field is its first interface's vtable pointer, and it
    // begins with one reference, which the pointer takes.
    unsafe { VstPtr::owned(Box::into_raw(object).cast()) }.unwrap()
}
pub fn parameter_queue(id: u32, points: Vec<(i32, f64)>) -> VstPtr<dyn IParamValueQueue> {
    com(Queue::allocate(id, points))
}

pub fn parameter_changes(
    queues: Vec<VstPtr<dyn IParamValueQueue>>,
) -> VstPtr<dyn IParameterChanges> {
    com(Changes::allocate(queues))
}
