//! Win32 windows.

use super::{Check, WindowEvent};
use clap_sys::ext::gui::clap_window_handle;
use std::cell::RefCell;
use std::collections::HashMap;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{ClientToScreen, DKGRAY_BRUSH, GetStockObject};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AdjustWindowRectEx, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow,
    DispatchMessageW, GW_CHILD, GetWindow, IDC_ARROW, IsWindowVisible, LoadCursorW, MSG, PM_REMOVE,
    PeekMessageW, RegisterClassExW, SW_HIDE, SW_SHOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOZORDER, SetWindowPos, ShowWindow, TranslateMessage, WM_CLOSE, WM_SIZE, WNDCLASSEXW,
    WS_CAPTION, WS_CLIPCHILDREN, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU,
    WS_THICKFRAME,
};

thread_local! {
    /// Events the window procedure saw, by window, until each window takes its own.
    static QUEUED: RefCell<HashMap<usize, Vec<WindowEvent>>> = RefCell::new(HashMap::new());
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

const CLASS: &str = "DawPluginWindow";

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_CLOSE => {
            // Closing hides it; the helper destroys it when the DAW closes the GUI.
            QUEUED.with(|q| {
                q.borrow_mut()
                    .entry(hwnd as usize)
                    .or_default()
                    .push(WindowEvent::Closed)
            });
            // SAFETY: our own window, on its thread.
            unsafe { ShowWindow(hwnd, SW_HIDE) };
            0
        }
        WM_SIZE => {
            let (w, h) = ((lp & 0xFFFF) as u32, ((lp >> 16) & 0xFFFF) as u32);
            if w > 0 && h > 0 {
                QUEUED.with(|q| {
                    q.borrow_mut()
                        .entry(hwnd as usize)
                        .or_default()
                        .push(WindowEvent::Resized(w, h))
                });
            }
            // SAFETY: default handling of the message.
            unsafe { DefWindowProcW(hwnd, msg, wp, lp) }
        }
        // SAFETY: default handling of every other message.
        _ => unsafe { DefWindowProcW(hwnd, msg, wp, lp) },
    }
}

fn register() -> Result<(), String> {
    thread_local! {
        static DONE: RefCell<bool> = const { RefCell::new(false) };
    }
    if DONE.with(|d| *d.borrow()) {
        return Ok(());
    }
    let class = wide(CLASS);
    // SAFETY: registering a window class with static strings and our procedure.
    let atom = unsafe {
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: GetModuleHandleW(std::ptr::null()),
            hIcon: std::ptr::null_mut(),
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: GetStockObject(DKGRAY_BRUSH),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };
        RegisterClassExW(&wc)
    };
    if atom == 0 {
        return Err("Win32: cannot register the plugin window class".into());
    }
    DONE.with(|d| *d.borrow_mut() = true);
    Ok(())
}

/// Runs the thread's messages: close requests and new sizes are queued for the windows.
pub fn pump() {
    // SAFETY: the standard message loop, without blocking, on the helper's main thread.
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// A top-level Win32 window.
#[derive(Debug)]
pub struct Window {
    hwnd: HWND,
    style: u32,
    size: (u32, u32),
}

impl Window {
    /// A window whose content is `width` by `height`, titled `title`, at `at` on the
    /// screen if given; fixed in size unless `resizable`. Not yet shown.
    pub fn create(
        title: &str,
        width: u32,
        height: u32,
        at: Option<(i32, i32)>,
        resizable: bool,
    ) -> Result<Window, String> {
        register()?;
        let mut style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_CLIPCHILDREN;
        if resizable {
            style |= WS_THICKFRAME | WS_MAXIMIZEBOX;
        }
        let mut r = RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        let (class, title) = (wide(CLASS), wide(title));
        let (x, y) = at.unwrap_or((CW_USEDEFAULT, CW_USEDEFAULT));
        // SAFETY: creating a top-level window of our registered class.
        let hwnd = unsafe {
            AdjustWindowRectEx(&mut r, style, 0, 0);
            CreateWindowExW(
                0,
                class.as_ptr(),
                title.as_ptr(),
                style,
                x,
                y,
                r.right - r.left,
                r.bottom - r.top,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                GetModuleHandleW(std::ptr::null()),
                std::ptr::null(),
            )
        };
        if hwnd.is_null() {
            return Err("Win32: cannot create a window".into());
        }
        Ok(Window {
            hwnd,
            style,
            size: (width, height),
        })
    }

    pub fn handle(&self) -> clap_window_handle {
        clap_window_handle { win32: self.hwnd }
    }

    pub fn id(&self) -> u64 {
        self.hwnd as usize as u64
    }

    pub fn show(&mut self) {
        // SAFETY: our own window.
        unsafe { ShowWindow(self.hwnd, SW_SHOW) };
    }

    pub fn hide(&mut self) {
        // SAFETY: our own window.
        unsafe { ShowWindow(self.hwnd, SW_HIDE) };
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.size = (width, height);
        let mut r = RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        // SAFETY: our own window.
        unsafe {
            AdjustWindowRectEx(&mut r, self.style, 0, 0);
            SetWindowPos(
                self.hwnd,
                std::ptr::null_mut(),
                0,
                0,
                r.right - r.left,
                r.bottom - r.top,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }

    pub fn move_to(&self, x: i32, y: i32) {
        // SAFETY: our own window.
        unsafe {
            SetWindowPos(
                self.hwnd,
                std::ptr::null_mut(),
                x,
                y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }

    /// Where the window's content is on the screen.
    pub fn position(&self) -> Option<(i32, i32)> {
        let mut p = POINT { x: 0, y: 0 };
        // SAFETY: our own window.
        (unsafe { ClientToScreen(self.hwnd, &mut p) } != 0).then_some((p.x, p.y))
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn check(&self) -> Check {
        // SAFETY: our own window.
        unsafe {
            Check {
                visible: IsWindowVisible(self.hwnd) != 0,
                children: usize::from(!GetWindow(self.hwnd, GW_CHILD).is_null()),
            }
        }
    }

    pub fn events(&mut self) -> Vec<WindowEvent> {
        let events = QUEUED.with(|q| {
            q.borrow_mut()
                .remove(&(self.hwnd as usize))
                .unwrap_or_default()
        });
        for e in &events {
            if let WindowEvent::Resized(w, h) = e {
                self.size = (*w, *h);
            }
        }
        events
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        // SAFETY: destroying our own window on its thread.
        unsafe { DestroyWindow(self.hwnd) };
        QUEUED.with(|q| q.borrow_mut().remove(&(self.hwnd as usize)));
    }
}
