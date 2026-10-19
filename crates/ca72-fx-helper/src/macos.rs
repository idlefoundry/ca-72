//! The helper as an accessory application on macOS (decisions.md R-FX-DOCK).
//!
//! In the application's own bundle (`Contents/Helpers/ca72-fx-helper`, as first built) the helper
//! took the bundle's identifier and its default, an ordinary application, the first time a
//! plug-in it loaded spoke to AppKit: LaunchServices listed a second "CA-72", a Foreground
//! application that never finished launching, and the Dock showed it bouncing beside the synth's.
//! DAW-v3's helper made itself an accessory only as its first window opened, which was too late.
//! Now the application's shared instance is made and set to the accessory policy before
//! anything else: before a plug-in is loaded, and also when the helper runs bare (tests, a
//! development build), without a bundle to say `LSUIElement`.

#![allow(unsafe_code)]

use std::ffi::{CStr, c_char, c_void};

type Id = *mut c_void;
type Sel = *mut c_void;

#[link(name = "objc", kind = "dylib")]
unsafe extern "C" {
    fn objc_getClass(name: *const c_char) -> Id;
    fn sel_registerName(name: *const c_char) -> Sel;
    fn objc_msgSend();
}

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}

/// `NSApplicationActivationPolicyAccessory`.
const ACCESSORY: i64 = 1;

fn sel(name: &CStr) -> Sel {
    // SAFETY: registering a NUL-terminated selector name.
    unsafe { sel_registerName(name.as_ptr()) }
}

/// Makes this process an accessory application: no Dock icon or menu bar; its windows can still
/// be shown in front and take the keyboard when clicked. Called once, first, on the main thread
/// (DAW-v3's helper sets the same policy again as its first window opens, which changes nothing).
pub fn accessory() {
    // SAFETY: looking up AppKit's NSApplication class by its NUL-terminated name.
    let class = unsafe { objc_getClass(c"NSApplication".as_ptr()) };
    if class.is_null() {
        return;
    }
    // SAFETY: `objc_msgSend` called with the signatures of `+[NSApplication sharedApplication]`
    // (no arguments, returns the instance) and `-[NSApplication setActivationPolicy:]` (an
    // NSInteger, returns a BOOL), on the class and the instance it returns, on the main thread.
    unsafe {
        let shared: unsafe extern "C" fn(Id, Sel) -> Id =
            std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        let app = shared(class, sel(c"sharedApplication"));
        if app.is_null() {
            return;
        }
        let policy: unsafe extern "C" fn(Id, Sel, i64) -> i8 =
            std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        policy(app, sel(c"setActivationPolicy:"), ACCESSORY);
    }
}
