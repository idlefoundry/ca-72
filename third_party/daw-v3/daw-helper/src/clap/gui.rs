//! Plugin GUIs (DESIGN 10.5) and what they need from the host on the main thread: the
//! host side of `clap.gui`, `clap.timer-support` and `clap.posix-fd-support`.
//!
//! A plugin that can embed its view in this platform's windows gets a window the helper
//! creates and owns ([`crate::window`]); one that can only float opens its own window.
//! Either way the window lives and dies with the helper process.

use super::{Instance, host_data};
use crate::window::{self, Window, WindowEvent};
use clap_sys::ext::gui::{CLAP_EXT_GUI, clap_host_gui, clap_plugin_gui, clap_window};
use clap_sys::ext::timer_support::{
    CLAP_EXT_TIMER_SUPPORT, clap_host_timer_support, clap_plugin_timer_support,
};
use clap_sys::host::clap_host;
use clap_sys::id::clap_id;
use serde_json::{Value, json};
use std::ffi::{CString, c_char};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// What a plugin asked of the host for its GUI and main-thread work, in `HostData`.
#[derive(Debug, Default)]
pub(super) struct GuiHost {
    /// A size the plugin asked for (`width << 32 | height`); 0 for none.
    resize: AtomicU64,
    show: AtomicBool,
    hide: AtomicBool,
    /// The plugin's own floating window was closed: 1, or 2 if it destroyed its GUI too.
    closed: AtomicU8,
    timers: Mutex<Vec<Timer>>,
    next_timer: AtomicU32,
    #[cfg(unix)]
    fds: Mutex<Vec<(i32, u32)>>,
}

#[derive(Debug)]
struct Timer {
    id: clap_id,
    period: Duration,
    due: Instant,
}

/// An open GUI.
#[derive(Debug)]
pub(super) struct View {
    /// The helper's window the view is embedded in; `None` for a floating view.
    window: Option<Window>,
    title: String,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

unsafe extern "C" fn gui_resize_hints_changed(_: *const clap_host) {}

unsafe extern "C" fn gui_request_resize(h: *const clap_host, width: u32, height: u32) -> bool {
    // SAFETY: the plugin passes its host pointer.
    let Some(d) = (unsafe { host_data(h) }) else {
        return false;
    };
    d.gui.resize.store(
        (u64::from(width) << 32) | u64::from(height),
        Ordering::Release,
    );
    true
}

unsafe extern "C" fn gui_request_show(h: *const clap_host) -> bool {
    // SAFETY: the plugin passes its host pointer.
    unsafe { host_data(h) }.is_some_and(|d| {
        d.gui.show.store(true, Ordering::Release);
        true
    })
}

unsafe extern "C" fn gui_request_hide(h: *const clap_host) -> bool {
    // SAFETY: the plugin passes its host pointer.
    unsafe { host_data(h) }.is_some_and(|d| {
        d.gui.hide.store(true, Ordering::Release);
        true
    })
}

unsafe extern "C" fn gui_closed(h: *const clap_host, was_destroyed: bool) {
    // SAFETY: the plugin passes its host pointer.
    if let Some(d) = unsafe { host_data(h) } {
        d.gui
            .closed
            .store(if was_destroyed { 2 } else { 1 }, Ordering::Release);
    }
}

pub(super) static HOST_GUI: clap_host_gui = clap_host_gui {
    resize_hints_changed: Some(gui_resize_hints_changed),
    request_resize: Some(gui_request_resize),
    request_show: Some(gui_request_show),
    request_hide: Some(gui_request_hide),
    closed: Some(gui_closed),
};

unsafe extern "C" fn register_timer(h: *const clap_host, period_ms: u32, id: *mut clap_id) -> bool {
    // SAFETY: the plugin passes its host pointer.
    let Some(d) = (unsafe { host_data(h) }) else {
        return false;
    };
    if id.is_null() {
        return false;
    }
    let n = d.gui.next_timer.fetch_add(1, Ordering::Relaxed);
    let period = Duration::from_millis(u64::from(period_ms.max(1)));
    lock(&d.gui.timers).push(Timer {
        id: n,
        period,
        due: Instant::now() + period,
    });
    // SAFETY: `id` is the plugin's out-parameter.
    unsafe { *id = n };
    true
}

unsafe extern "C" fn unregister_timer(h: *const clap_host, id: clap_id) -> bool {
    // SAFETY: the plugin passes its host pointer.
    let Some(d) = (unsafe { host_data(h) }) else {
        return false;
    };
    let mut t = lock(&d.gui.timers);
    let before = t.len();
    t.retain(|x| x.id != id);
    t.len() != before
}

pub(super) static HOST_TIMER: clap_host_timer_support = clap_host_timer_support {
    register_timer: Some(register_timer),
    unregister_timer: Some(unregister_timer),
};

#[cfg(unix)]
mod fd {
    use super::{clap_host, host_data, lock};
    use clap_sys::ext::posix_fd_support::clap_host_posix_fd_support;

    unsafe extern "C" fn register_fd(h: *const clap_host, fd: i32, flags: u32) -> bool {
        // SAFETY: the plugin passes its host pointer.
        let Some(d) = (unsafe { host_data(h) }) else {
            return false;
        };
        let mut fds = lock(&d.gui.fds);
        fds.retain(|x| x.0 != fd);
        fds.push((fd, flags));
        true
    }

    unsafe extern "C" fn unregister_fd(h: *const clap_host, fd: i32) -> bool {
        // SAFETY: the plugin passes its host pointer.
        let Some(d) = (unsafe { host_data(h) }) else {
            return false;
        };
        let mut fds = lock(&d.gui.fds);
        let before = fds.len();
        fds.retain(|x| x.0 != fd);
        fds.len() != before
    }

    pub(in super::super) static HOST_FD: clap_host_posix_fd_support = clap_host_posix_fd_support {
        register_fd: Some(register_fd),
        modify_fd: Some(register_fd),
        unregister_fd: Some(unregister_fd),
    };
}
#[cfg(unix)]
pub(super) use fd::HOST_FD;

/// The host extensions of this module, by ID.
pub(super) fn host_extension(id: &std::ffi::CStr) -> Option<*const std::ffi::c_void> {
    if id == CLAP_EXT_GUI {
        return Some(&HOST_GUI as *const _ as *const _);
    }
    if id == CLAP_EXT_TIMER_SUPPORT {
        return Some(&HOST_TIMER as *const _ as *const _);
    }
    #[cfg(unix)]
    if id == clap_sys::ext::posix_fd_support::CLAP_EXT_POSIX_FD_SUPPORT {
        return Some(&HOST_FD as *const _ as *const _);
    }
    None
}

impl Instance {
    fn gui_ext(&self) -> Option<&clap_plugin_gui> {
        self.ext::<clap_plugin_gui>(CLAP_EXT_GUI)
    }

    /// Opens the plugin's GUI titled `title`, at `at` on the screen if given (a window
    /// already open is shown and moved there).
    pub fn gui_open(&mut self, title: &str, at: Option<(i32, i32)>) -> Result<Value, String> {
        if let Some(v) = &mut self.view {
            if let Some(w) = &mut v.window {
                if let Some((x, y)) = at {
                    w.move_to(x, y);
                }
                w.show();
            }
            let plugin = self.plugin;
            if let Some(show) = self.gui_ext().and_then(|g| g.show) {
                // SAFETY: showing a created GUI on the main thread.
                unsafe { show(plugin) };
            }
            return Ok(self.gui_info());
        }
        let plugin = self.plugin;
        let gui = *self.gui_ext().ok_or("the plugin has no GUI")?;
        let api = window::api();
        // SAFETY: GUI calls on the main thread with a valid plugin, in the order CLAP
        // prescribes (create, size, parent, show).
        unsafe {
            let supported = |floating: bool| {
                gui.is_api_supported
                    .is_some_and(|f| f(plugin, api.as_ptr(), floating))
            };
            let embed = supported(false);
            if !embed && !supported(true) {
                return Err(format!(
                    "the plugin's GUI does not support this platform's windows ({})",
                    api.to_string_lossy()
                ));
            }
            if !gui.create.is_some_and(|f| f(plugin, api.as_ptr(), !embed)) {
                return Err("the plugin could not create its GUI".into());
            }
            let destroy = || {
                if let Some(d) = gui.destroy {
                    d(plugin);
                }
            };
            let window = if embed {
                if let Some(s) = gui.set_scale {
                    s(plugin, 1.0);
                }
                let (mut w, mut h) = (0u32, 0u32);
                if !gui.get_size.is_some_and(|f| f(plugin, &mut w, &mut h)) || w == 0 || h == 0 {
                    (w, h) = (400, 300);
                }
                let resizable = gui.can_resize.is_some_and(|f| f(plugin));
                let mut win = match Window::create(title, w, h, at, resizable) {
                    Ok(win) => win,
                    Err(e) => {
                        destroy();
                        return Err(e);
                    }
                };
                let cw = clap_window {
                    api: api.as_ptr(),
                    specific: win.handle(),
                };
                if !gui.set_parent.is_some_and(|f| f(plugin, &cw)) {
                    destroy();
                    return Err("the plugin could not embed its GUI in the window".into());
                }
                win.show();
                // The window manager maps a new window when it gets to it: wait a moment
                // so that the reply says whether it is on screen.
                let began = Instant::now();
                while !win.check().visible && began.elapsed() < Duration::from_secs(1) {
                    window::pump();
                    std::thread::sleep(Duration::from_millis(10));
                }
                Some(win)
            } else {
                let t = CString::new(title.replace('\0', " ")).unwrap_or_default();
                if let Some(s) = gui.suggest_title {
                    s(plugin, t.as_ptr() as *const c_char);
                }
                None
            };
            if let Some(show) = gui.show {
                show(plugin);
            }
            self.host.gui.closed.store(0, Ordering::Relaxed);
            self.view = Some(View {
                window,
                title: title.to_owned(),
            });
        }
        Ok(self.gui_info())
    }

    /// The open GUI: whether it floats, its size, where it is, and whether the window is
    /// on screen with the plugin's view in it.
    pub fn gui_info(&self) -> Value {
        let Some(v) = &self.view else {
            return json!({"open": false});
        };
        match &v.window {
            Some(w) => {
                let check = w.check();
                let (width, height) = w.size();
                json!({
                    "open": true,
                    "helper": std::process::id(),
                    "floating": false,
                    "api": window::api().to_string_lossy(),
                    "title": v.title,
                    "window": w.id(),
                    "width": width,
                    "height": height,
                    "position": w.position(),
                    "visible": check.visible,
                    "children": check.children,
                })
            }
            None => json!({
                "open": true,
                "helper": std::process::id(),
                "floating": true,
                "api": window::api().to_string_lossy(),
                "title": v.title,
            }),
        }
    }

    /// Moves the GUI's window (a floating plugin window places itself).
    pub fn gui_move(&mut self, x: i32, y: i32) -> Result<Value, String> {
        match self.view.as_ref().and_then(|v| v.window.as_ref()) {
            Some(w) => {
                w.move_to(x, y);
                Ok(self.gui_info())
            }
            None if self.view.is_some() => Err("a floating plugin window places itself".into()),
            None => Err("the plugin's GUI is not open".into()),
        }
    }

    /// Closes the GUI: hidden and destroyed, the window with it.
    pub fn gui_close(&mut self) {
        let Some(v) = self.view.take() else {
            return;
        };
        let plugin = self.plugin;
        let destroyed = self.host.gui.closed.swap(0, Ordering::Relaxed) == 2;
        if let Some(gui) = self.gui_ext() {
            // SAFETY: hiding and destroying a created GUI on the main thread, before its
            // window goes.
            unsafe {
                if !destroyed {
                    if let Some(h) = gui.hide {
                        h(plugin);
                    }
                    if let Some(d) = gui.destroy {
                        d(plugin);
                    }
                }
            }
        }
        drop(v);
    }

    /// Main-thread GUI duties: timers, file descriptors, the window's events and the
    /// plugin's requests. Returns true when the user closed the GUI.
    pub fn gui_service(&mut self) -> bool {
        self.run_timers();
        #[cfg(unix)]
        self.run_fds();
        let g = &self.host.gui;
        let resize = g.resize.swap(0, Ordering::Acquire);
        let show = g.show.swap(false, Ordering::Acquire);
        let hide = g.hide.swap(false, Ordering::Acquire);
        let floating_closed = g.closed.load(Ordering::Acquire) != 0;
        let plugin = self.plugin;
        let gui = self.gui_ext().copied();
        if self.view.is_none() {
            return false;
        }
        if floating_closed {
            self.gui_close();
            return true;
        }
        let Some(w) = self.view.as_mut().and_then(|v| v.window.as_mut()) else {
            return false;
        };
        if resize != 0 {
            w.set_size((resize >> 32) as u32, resize as u32);
        }
        if show {
            w.show();
        }
        if hide {
            w.hide();
        }
        let mut closed = false;
        for e in w.events() {
            match e {
                WindowEvent::Closed => closed = true,
                WindowEvent::Resized(mut width, mut height) => {
                    let Some(gui) = gui else { continue };
                    // SAFETY: resizing a created, resizable GUI on the main thread.
                    unsafe {
                        if gui.can_resize.is_some_and(|f| f(plugin)) {
                            if let Some(a) = gui.adjust_size {
                                a(plugin, &mut width, &mut height);
                            }
                            if let Some(s) = gui.set_size {
                                s(plugin, width, height);
                            }
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

    /// Calls the plugin's timers that are due.
    fn run_timers(&self) {
        let now = Instant::now();
        let due: Vec<clap_id> = {
            let mut timers = lock(&self.host.gui.timers);
            timers
                .iter_mut()
                .filter(|t| t.due <= now)
                .map(|t| {
                    t.due = now + t.period;
                    t.id
                })
                .collect()
        };
        if due.is_empty() {
            return;
        }
        let Some(on_timer) = self
            .ext::<clap_plugin_timer_support>(CLAP_EXT_TIMER_SUPPORT)
            .and_then(|t| t.on_timer)
        else {
            return;
        };
        for id in due {
            // SAFETY: the plugin's timer callback, on the main thread.
            unsafe { on_timer(self.plugin, id) };
        }
    }

    /// Calls the plugin for its file descriptors that are ready.
    #[cfg(unix)]
    fn run_fds(&self) {
        use clap_sys::ext::posix_fd_support::{
            CLAP_EXT_POSIX_FD_SUPPORT, CLAP_POSIX_FD_ERROR, CLAP_POSIX_FD_READ,
            CLAP_POSIX_FD_WRITE, clap_plugin_posix_fd_support,
        };
        let fds: Vec<(i32, u32)> = lock(&self.host.gui.fds).clone();
        if fds.is_empty() {
            return;
        }
        let mut polls: Vec<libc::pollfd> = fds
            .iter()
            .map(|&(fd, flags)| libc::pollfd {
                fd,
                events: if flags & CLAP_POSIX_FD_READ != 0 {
                    libc::POLLIN
                } else {
                    0
                } | if flags & CLAP_POSIX_FD_WRITE != 0 {
                    libc::POLLOUT
                } else {
                    0
                },
                revents: 0,
            })
            .collect();
        // SAFETY: polling the plugin's descriptors without waiting.
        let n = unsafe { libc::poll(polls.as_mut_ptr(), polls.len() as libc::nfds_t, 0) };
        if n <= 0 {
            return;
        }
        let Some(on_fd) = self
            .ext::<clap_plugin_posix_fd_support>(CLAP_EXT_POSIX_FD_SUPPORT)
            .and_then(|f| f.on_fd)
        else {
            return;
        };
        for p in polls.iter().filter(|p| p.revents != 0) {
            let mut flags = 0;
            if p.revents & libc::POLLIN != 0 {
                flags |= CLAP_POSIX_FD_READ;
            }
            if p.revents & libc::POLLOUT != 0 {
                flags |= CLAP_POSIX_FD_WRITE;
            }
            if p.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
                flags |= CLAP_POSIX_FD_ERROR;
            }
            // SAFETY: the plugin's descriptor callback, on the main thread.
            unsafe { on_fd(self.plugin, p.fd, flags) };
        }
    }
}
