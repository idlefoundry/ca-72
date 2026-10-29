//! Named shared memory (DESIGN 10.2): POSIX `shm_open` on Linux and macOS, a
//! pagefile-backed named file mapping on Windows. The host creates a region; the helper
//! opens it by name. The memory starts zeroed.

#![allow(unsafe_code)]

use std::io;

/// A mapped shared memory region.
#[derive(Debug)]
pub struct Shm {
    ptr: *mut u8,
    len: usize,
    name: String,
    owner: bool,
    #[cfg(windows)]
    handle: windows_sys::Win32::Foundation::HANDLE,
}

// SAFETY: `Shm` only hands out the base pointer; synchronising access to the memory is the
// job of the protocol built on top (atomics in the region's header).
unsafe impl Send for Shm {}
// SAFETY: as above.
unsafe impl Sync for Shm {}

/// A process-unique name for a new region or event.
pub fn unique_name(tag: &str) -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    // macOS limits POSIX shared memory names to 31 bytes, including the slash.
    format!("daw{}{tag}{n}", std::process::id())
}

impl Shm {
    /// Creates a zeroed region of `len` bytes. The name must be new.
    pub fn create(name: &str, len: usize) -> io::Result<Shm> {
        let shm = sys::create(name, len)?;
        // Touch every page now rather than on the audio thread.
        for off in (0..len).step_by(4096) {
            // SAFETY: `off < len`, inside the mapping.
            unsafe { shm.ptr.add(off).write_volatile(0) };
        }
        Ok(shm)
    }

    /// Creates a zeroed region under a name from [`unique_name`]. Such a name holds this
    /// process's ID, so if it exists it was left by a process with the same ID that died
    /// without removing it (killed, or aborted); on Linux and macOS it stays until the
    /// next restart. It is removed and the region created again.
    pub fn create_unique(name: &str, len: usize) -> io::Result<Shm> {
        match Shm::create(name, len) {
            #[cfg(unix)]
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                sys::remove(name);
                Shm::create(name, len)
            }
            r => r,
        }
    }

    /// Opens a region the host created.
    pub fn open(name: &str, len: usize) -> io::Result<Shm> {
        sys::open(name, len)
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for Shm {
    fn drop(&mut self) {
        sys::close(self);
    }
}

#[cfg(unix)]
mod sys {
    use super::Shm;
    use std::ffi::CString;
    use std::io;

    fn cname(name: &str) -> io::Result<CString> {
        CString::new(format!("/{name}")).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
    }

    fn map(fd: libc::c_int, len: usize, name: &str, owner: bool) -> io::Result<Shm> {
        // SAFETY: mapping a shared memory object we hold open; the kernel validates the
        // arguments.
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };
        let err = io::Error::last_os_error();
        // SAFETY: closing the descriptor we opened; the mapping stays valid.
        unsafe { libc::close(fd) };
        if ptr == libc::MAP_FAILED {
            return Err(err);
        }
        Ok(Shm {
            ptr: ptr as *mut u8,
            len,
            name: name.to_owned(),
            owner,
        })
    }

    pub fn create(name: &str, len: usize) -> io::Result<Shm> {
        let c = cname(name)?;
        // SAFETY: `c` is a valid C string.
        let fd = unsafe {
            libc::shm_open(
                c.as_ptr(),
                libc::O_CREAT | libc::O_EXCL | libc::O_RDWR,
                // `shm_open` is variadic on macOS, where `mode_t` is 16 bits.
                0o600 as libc::c_uint,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `fd` is the object we just created.
        if unsafe { libc::ftruncate(fd, len as libc::off_t) } != 0 {
            let err = io::Error::last_os_error();
            // SAFETY: cleaning up the object we created.
            unsafe {
                libc::close(fd);
                libc::shm_unlink(c.as_ptr());
            }
            return Err(err);
        }
        map(fd, len, name, true).inspect_err(|_| {
            // SAFETY: removing the name we created.
            unsafe { libc::shm_unlink(c.as_ptr()) };
        })
    }

    pub fn open(name: &str, len: usize) -> io::Result<Shm> {
        let c = cname(name)?;
        // SAFETY: `c` is a valid C string.
        let fd = unsafe { libc::shm_open(c.as_ptr(), libc::O_RDWR, 0) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let mut st = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: `fd` is open and `st` is writable.
        if unsafe { libc::fstat(fd, st.as_mut_ptr()) } != 0 {
            let err = io::Error::last_os_error();
            // SAFETY: closing our descriptor.
            unsafe { libc::close(fd) };
            return Err(err);
        }
        // SAFETY: `fstat` succeeded, so `st` is initialised.
        let size = unsafe { st.assume_init() }.st_size as usize;
        if size < len {
            // SAFETY: closing our descriptor.
            unsafe { libc::close(fd) };
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("shared memory {name} is {size} bytes, expected {len}"),
            ));
        }
        map(fd, len, name, false)
    }

    /// Removes a name; mappings of it stay valid.
    pub fn remove(name: &str) {
        if let Ok(c) = cname(name) {
            // SAFETY: `c` is a valid C string.
            unsafe { libc::shm_unlink(c.as_ptr()) };
        }
    }

    pub fn close(shm: &mut Shm) {
        // SAFETY: unmapping our own mapping exactly once.
        unsafe { libc::munmap(shm.ptr as *mut libc::c_void, shm.len) };
        if shm.owner
            && let Ok(c) = cname(&shm.name)
        {
            // SAFETY: removing the name we created.
            unsafe { libc::shm_unlink(c.as_ptr()) };
        }
    }
}

#[cfg(windows)]
mod sys {
    use super::Shm;
    use std::io;
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Memory::{
        CreateFileMappingW, FILE_MAP_ALL_ACCESS, MEMORY_MAPPED_VIEW_ADDRESS, MapViewOfFile,
        OpenFileMappingW, PAGE_READWRITE, UnmapViewOfFile,
    };

    pub(crate) fn wide(name: &str) -> Vec<u16> {
        format!("Local\\{name}")
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect()
    }

    fn view(
        handle: windows_sys::Win32::Foundation::HANDLE,
        len: usize,
        name: &str,
        owner: bool,
    ) -> io::Result<Shm> {
        // SAFETY: mapping a view of a mapping object we hold.
        let v = unsafe { MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, len) };
        if v.Value.is_null() {
            let err = io::Error::last_os_error();
            // SAFETY: closing our handle.
            unsafe { CloseHandle(handle) };
            return Err(err);
        }
        Ok(Shm {
            ptr: v.Value as *mut u8,
            len,
            name: name.to_owned(),
            owner,
            handle,
        })
    }

    pub fn create(name: &str, len: usize) -> io::Result<Shm> {
        let w = wide(name);
        let size = len as u64;
        // SAFETY: `w` is NUL-terminated; a pagefile-backed mapping.
        let h = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                std::ptr::null(),
                PAGE_READWRITE,
                (size >> 32) as u32,
                size as u32,
                w.as_ptr(),
            )
        };
        if h.is_null() {
            return Err(io::Error::last_os_error());
        }
        if io::Error::last_os_error().raw_os_error() == Some(183) {
            // ERROR_ALREADY_EXISTS
            // SAFETY: closing our handle.
            unsafe { CloseHandle(h) };
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("shared memory {name} already exists"),
            ));
        }
        view(h, len, name, true)
    }

    pub fn open(name: &str, len: usize) -> io::Result<Shm> {
        let w = wide(name);
        // SAFETY: `w` is NUL-terminated.
        let h = unsafe { OpenFileMappingW(FILE_MAP_ALL_ACCESS, 0, w.as_ptr()) };
        if h.is_null() {
            return Err(io::Error::last_os_error());
        }
        view(h, len, name, false)
    }

    pub fn close(shm: &mut Shm) {
        // SAFETY: unmapping our view and closing our handle, once each. The mapping
        // disappears when the last handle closes.
        unsafe {
            UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                Value: shm.ptr as *mut core::ffi::c_void,
            });
            CloseHandle(shm.handle);
        }
        let _ = shm.owner;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_open_share_bytes() {
        let name = unique_name("t");
        let a = Shm::create(&name, 8192).unwrap();
        let b = Shm::open(&name, 8192).unwrap();
        // SAFETY: both mappings are 8192 bytes; this test is the only user.
        unsafe {
            assert_eq!(*b.as_ptr().add(5000), 0);
            *a.as_ptr().add(5000) = 42;
            assert_eq!(*b.as_ptr().add(5000), 42);
        }
        assert!(Shm::create(&name, 8192).is_err(), "names are exclusive");
        drop(b);
        drop(a);
        assert!(
            Shm::open(&name, 8192).is_err(),
            "the owner removes the name"
        );
    }

    /// A name left behind, as by a killed process that had this process's ID, does not
    /// stop a new region.
    #[cfg(unix)]
    #[test]
    fn a_name_left_behind_is_replaced() {
        let name = unique_name("t");
        let old = Shm::create(&name, 8192).unwrap();
        // SAFETY: the mapping is 8192 bytes.
        unsafe { *old.as_ptr().add(5000) = 7 };
        // Never removed, as if the process had died.
        std::mem::forget(old);
        let new = Shm::create_unique(&name, 8192).unwrap();
        // SAFETY: as above.
        assert_eq!(unsafe { *new.as_ptr().add(5000) }, 0, "a new region");
        drop(new);
        assert!(Shm::open(&name, 8192).is_err(), "and its owner removes it");
    }
}
