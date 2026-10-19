//! Cross-process wake-ups on a 32-bit word in shared memory (DESIGN 10.2): a futex on
//! Linux, `os_sync_wait_on_address` on macOS (14.4 and later), a named auto-reset event
//! paired with the word on Windows. Waiters always re-check the word, so spurious wake-ups
//! and wake-ups that arrive before the wait are harmless.

#![allow(unsafe_code)]

use std::io;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// One direction of signalling: a word in shared memory plus, on Windows, an event.
#[derive(Debug)]
pub struct Signal {
    word: *const AtomicU32,
    #[cfg(windows)]
    event: windows_sys::Win32::Foundation::HANDLE,
}

// SAFETY: the word is an atomic in shared memory that outlives the signal (the owner of
// the mapping keeps it alive); the Windows event handle is usable from any thread.
unsafe impl Send for Signal {}
// SAFETY: as above.
unsafe impl Sync for Signal {}

/// Why a wait ended without the condition holding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitError {
    Timeout,
}

impl Signal {
    /// Creates the signal for `word` (the host side). `name` names the Windows event and
    /// is ignored elsewhere.
    ///
    /// # Safety
    /// `word` must point into shared memory that stays mapped while the signal is used.
    pub unsafe fn create(word: *const AtomicU32, name: &str) -> io::Result<Signal> {
        let _ = name;
        Ok(Signal {
            word,
            #[cfg(windows)]
            event: sys::create_event(name)?,
        })
    }

    /// Opens the signal the host created (the helper side).
    ///
    /// # Safety
    /// As for [`Signal::create`].
    pub unsafe fn open(word: *const AtomicU32, name: &str) -> io::Result<Signal> {
        let _ = name;
        Ok(Signal {
            word,
            #[cfg(windows)]
            event: sys::open_event(name)?,
        })
    }

    pub fn word(&self) -> &AtomicU32 {
        // SAFETY: the constructor's contract keeps the word mapped.
        unsafe { &*self.word }
    }

    /// Stores `value` (release) and wakes the waiter.
    pub fn store(&self, value: u32) {
        self.word().store(value, Ordering::Release);
        self.wake();
    }

    /// Sets bits (release) and wakes the waiter.
    pub fn set_bits(&self, bits: u32) {
        self.word().fetch_or(bits, Ordering::Release);
        self.wake();
    }

    /// Wakes the waiter, if any.
    pub fn wake(&self) {
        #[cfg(windows)]
        sys::set_event(self.event);
        #[cfg(not(windows))]
        sys::wake(self.word());
    }

    /// Waits until `done(word)` holds or the deadline passes, and returns the word's
    /// value (acquire).
    pub fn wait_until(
        &self,
        done: impl Fn(u32) -> bool,
        deadline: Option<Instant>,
    ) -> Result<u32, WaitError> {
        self.wait_spinning(done, deadline, Duration::ZERO)
    }

    /// Like [`Signal::wait_until`], but first spins for up to `spin` before sleeping: for
    /// waits that usually end within microseconds (a plugin's block), spinning avoids
    /// the cost and the scheduling delay of a sleep and a wake-up.
    pub fn wait_spinning(
        &self,
        done: impl Fn(u32) -> bool,
        deadline: Option<Instant>,
        spin: Duration,
    ) -> Result<u32, WaitError> {
        if !spin.is_zero() {
            let start = Instant::now();
            let mut n = 0u32;
            loop {
                let v = self.word().load(Ordering::Acquire);
                if done(v) {
                    return Ok(v);
                }
                n += 1;
                // Check the clock only now and then.
                if n.is_multiple_of(64) && start.elapsed() >= spin {
                    break;
                }
                std::hint::spin_loop();
            }
        }
        loop {
            let v = self.word().load(Ordering::Acquire);
            if done(v) {
                return Ok(v);
            }
            let timeout = match deadline {
                None => None,
                Some(d) => {
                    let now = Instant::now();
                    if now >= d {
                        return Err(WaitError::Timeout);
                    }
                    Some(d - now)
                }
            };
            #[cfg(windows)]
            sys::wait_event(self.event, timeout);
            #[cfg(not(windows))]
            sys::wait(self.word(), v, timeout);
        }
    }
}

impl Drop for Signal {
    fn drop(&mut self) {
        #[cfg(windows)]
        sys::close(self.event);
    }
}

/// Clamps a timeout for the platform calls.
#[allow(dead_code)]
fn clamp(t: Duration) -> Duration {
    t.min(Duration::from_secs(3600))
}

#[cfg(target_os = "linux")]
mod sys {
    use std::sync::atomic::AtomicU32;
    use std::time::Duration;

    pub fn wait(word: &AtomicU32, current: u32, timeout: Option<Duration>) {
        let ts = timeout.map(|t| {
            let t = super::clamp(t);
            libc::timespec {
                tv_sec: t.as_secs() as libc::time_t,
                tv_nsec: t.subsec_nanos() as libc::c_long,
            }
        });
        let tsp = ts
            .as_ref()
            .map_or(std::ptr::null(), |t| t as *const libc::timespec);
        // SAFETY: a shared (not private) futex wait on a valid, aligned word; the kernel
        // returns at once if the word no longer holds `current`.
        unsafe {
            libc::syscall(
                libc::SYS_futex,
                word.as_ptr(),
                libc::FUTEX_WAIT,
                current,
                tsp,
                std::ptr::null::<u32>(),
                0u32,
            );
        }
    }

    pub fn wake(word: &AtomicU32) {
        // SAFETY: a shared futex wake on a valid, aligned word.
        unsafe {
            libc::syscall(
                libc::SYS_futex,
                word.as_ptr(),
                libc::FUTEX_WAKE,
                i32::MAX,
                std::ptr::null::<libc::timespec>(),
                std::ptr::null::<u32>(),
                0u32,
            );
        }
    }
}

#[cfg(target_os = "macos")]
mod sys {
    use std::ffi::c_void;
    use std::sync::atomic::AtomicU32;
    use std::time::Duration;

    const OS_SYNC_WAIT_ON_ADDRESS_SHARED: u32 = 1;
    const OS_SYNC_WAKE_BY_ADDRESS_SHARED: u32 = 1;
    const OS_CLOCK_MACH_ABSOLUTE_TIME: u32 = 32;

    // From <os/os_sync_wait_on_address.h>, macOS 14.4 and later.
    unsafe extern "C" {
        fn os_sync_wait_on_address(addr: *mut c_void, value: u64, size: usize, flags: u32) -> i32;
        fn os_sync_wait_on_address_with_timeout(
            addr: *mut c_void,
            value: u64,
            size: usize,
            flags: u32,
            clockid: u32,
            timeout_ns: u64,
        ) -> i32;
        fn os_sync_wake_by_address_all(addr: *mut c_void, size: usize, flags: u32) -> i32;
    }

    pub fn wait(word: &AtomicU32, current: u32, timeout: Option<Duration>) {
        let addr = word.as_ptr() as *mut c_void;
        // SAFETY: waiting on a valid, aligned 4-byte word in shared memory; the call
        // returns at once if the word no longer holds `current`.
        unsafe {
            match timeout {
                None => {
                    os_sync_wait_on_address(
                        addr,
                        u64::from(current),
                        4,
                        OS_SYNC_WAIT_ON_ADDRESS_SHARED,
                    );
                }
                Some(t) => {
                    let ns = super::clamp(t).as_nanos().max(1) as u64;
                    os_sync_wait_on_address_with_timeout(
                        addr,
                        u64::from(current),
                        4,
                        OS_SYNC_WAIT_ON_ADDRESS_SHARED,
                        OS_CLOCK_MACH_ABSOLUTE_TIME,
                        ns,
                    );
                }
            }
        }
    }

    pub fn wake(word: &AtomicU32) {
        // SAFETY: waking waiters on a valid, aligned word.
        unsafe {
            os_sync_wake_by_address_all(
                word.as_ptr() as *mut c_void,
                4,
                OS_SYNC_WAKE_BY_ADDRESS_SHARED,
            );
        }
    }
}

#[cfg(windows)]
mod sys {
    use std::io;
    use std::time::Duration;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::Threading::{
        CreateEventW, EVENT_MODIFY_STATE, OpenEventW, SetEvent, WaitForSingleObject,
    };

    const SYNCHRONIZE: u32 = 0x0010_0000;

    fn wide(name: &str) -> Vec<u16> {
        format!("Local\\{name}")
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect()
    }

    pub fn create_event(name: &str) -> io::Result<HANDLE> {
        let w = wide(name);
        // SAFETY: an auto-reset, initially unsignalled named event.
        let h = unsafe { CreateEventW(std::ptr::null(), 0, 0, w.as_ptr()) };
        if h.is_null() {
            return Err(io::Error::last_os_error());
        }
        Ok(h)
    }

    pub fn open_event(name: &str) -> io::Result<HANDLE> {
        let w = wide(name);
        // SAFETY: opening an event the host created, by name.
        let h = unsafe { OpenEventW(EVENT_MODIFY_STATE | SYNCHRONIZE, 0, w.as_ptr()) };
        if h.is_null() {
            return Err(io::Error::last_os_error());
        }
        Ok(h)
    }

    pub fn set_event(h: HANDLE) {
        // SAFETY: a valid event handle we own.
        unsafe { SetEvent(h) };
    }

    pub fn wait_event(h: HANDLE, timeout: Option<Duration>) {
        let ms = timeout.map_or(u32::MAX, |t| {
            // Round up so a short wait does not become a busy loop.
            (super::clamp(t).as_micros().div_ceil(1000)).max(1) as u32
        });
        // SAFETY: a valid event handle we own.
        unsafe { WaitForSingleObject(h, ms) };
    }

    pub fn close(h: HANDLE) {
        // SAFETY: closing our handle once.
        unsafe { CloseHandle(h) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shm::{Shm, unique_name};
    use std::sync::Arc;

    #[test]
    fn wakes_a_waiter_and_times_out() {
        let name = unique_name("w");
        let shm = Arc::new(Shm::create(&name, 4096).unwrap());
        let word = shm.as_ptr() as *const AtomicU32;
        let ev = unique_name("e");
        // SAFETY: the word is in `shm`, which outlives both signals.
        let (a, b) = unsafe {
            (
                Signal::create(word, &ev).unwrap(),
                Signal::open(word, &ev).unwrap(),
            )
        };
        let start = Instant::now();
        let r = b.wait_until(|v| v == 1, Some(Instant::now() + Duration::from_millis(30)));
        assert_eq!(r, Err(WaitError::Timeout));
        assert!(start.elapsed() >= Duration::from_millis(30));
        let t = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            a.store(1);
            a
        });
        assert_eq!(b.wait_until(|v| v == 1, None), Ok(1));
        drop(t.join().unwrap());
    }
}
