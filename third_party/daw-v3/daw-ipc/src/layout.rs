//! The shared-memory layout of one plugin instance's block transport.
//!
//! The host writes a block's input audio and events into a slot, then stores the block's
//! sequence number in `request` and wakes the helper. The helper processes the slot in
//! place and stores the same number in `done`. Two slots let the pipelined mode fill one
//! while the helper works on the other. The host's supervisor sets [`DEAD`] in `done` when
//! the helper dies, which wakes a host that is waiting for a block.

#![allow(unsafe_code)]

use crate::shm::{Shm, unique_name};
use crate::wake::{Signal, WaitError};
use std::io;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

/// Frames per block, the engine's maximum.
pub const MAX_BLOCK: usize = 4096;
/// Events per block.
pub const MAX_EVENTS: usize = 512;
pub const SLOTS: usize = 2;
const MAGIC: u64 = 0x4441_5731_5348_4d31; // "DAW1SHM1"

/// Set in `done` by the host when the helper has died.
pub const DEAD: u32 = 1 << 31;
/// Set in `request` by the host to stop the helper's processing thread.
pub const QUIT: u32 = 1 << 31;
/// Sequence numbers use the low 31 bits.
pub const SEQ_MASK: u32 = !(1 << 31);

pub const EV_NOTE_ON: u32 = 1;
pub const EV_NOTE_OFF: u32 = 2;
/// A parameter's plain value.
pub const EV_PARAM: u32 = 3;
/// A non-destructive modulation amount, in plain units (DESIGN 11.4).
pub const EV_PARAM_MOD: u32 = 4;

/// A block event.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Event {
    pub kind: u32,
    /// Frame offset in the block.
    pub time: u32,
    /// Parameter ID, for parameter events.
    pub param: u32,
    pub note_id: i32,
    pub key: i16,
    pub channel: i16,
    pub _pad: u32,
    /// Plain value, modulation amount or velocity.
    pub value: f64,
}

/// Status the helper reports for a block.
pub const STATUS_OK: i32 = 0;
pub const STATUS_ERROR: i32 = 1;

#[repr(C)]
pub struct Slot {
    pub steady_time: i64,
    pub frames: u32,
    pub n_events: u32,
    pub status: i32,
    pub _pad: [u32; 11],
    pub events: [Event; MAX_EVENTS],
    pub input: [[f32; MAX_BLOCK]; 2],
    pub output: [[f32; MAX_BLOCK]; 2],
}

#[repr(C)]
pub struct Header {
    pub magic: u64,
    /// Written by the host.
    pub request: AtomicU32,
    _pad0: [u32; 13],
    /// Written by the helper (and the host's supervisor, for [`DEAD`]).
    pub done: AtomicU32,
    _pad1: [u32; 15],
}

#[repr(C)]
pub struct Region {
    pub header: Header,
    pub slots: [Slot; SLOTS],
}

pub const REGION_SIZE: usize = std::mem::size_of::<Region>();

impl std::fmt::Debug for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Slot").finish_non_exhaustive()
    }
}

impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Header").finish_non_exhaustive()
    }
}

impl std::fmt::Debug for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Region").finish_non_exhaustive()
    }
}

/// Names a transport: the shared memory and the two Windows events.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TransportName {
    pub shm: String,
    pub request: String,
    pub done: String,
}

/// One instance's transport, as mapped into either process.
#[derive(Debug)]
pub struct Transport {
    shm: Shm,
    request: Signal,
    done: Signal,
    name: TransportName,
}

impl Transport {
    fn region(&self) -> *mut Region {
        self.shm.as_ptr() as *mut Region
    }

    /// Creates a new transport (host side).
    pub fn create() -> io::Result<Transport> {
        let name = TransportName {
            shm: unique_name("m"),
            request: unique_name("q"),
            done: unique_name("d"),
        };
        let shm = Shm::create_unique(&name.shm, REGION_SIZE)?;
        let r = shm.as_ptr() as *mut Region;
        // SAFETY: `r` points to a fresh zeroed mapping of `REGION_SIZE` bytes; the helper
        // has not been told its name yet. Signals point into the mapping, which the
        // transport keeps alive.
        let (request, done) = unsafe {
            (*r).header.magic = MAGIC;
            (
                Signal::create(&raw const (*r).header.request, &name.request)?,
                Signal::create(&raw const (*r).header.done, &name.done)?,
            )
        };
        Ok(Transport {
            shm,
            request,
            done,
            name,
        })
    }

    /// Opens a transport the host created (helper side).
    pub fn open(name: &TransportName) -> io::Result<Transport> {
        let shm = Shm::open(&name.shm, REGION_SIZE)?;
        let r = shm.as_ptr() as *mut Region;
        // SAFETY: the mapping is `REGION_SIZE` bytes; the magic was written before the
        // host passed the name on.
        let (magic, request, done) = unsafe {
            (
                (*r).header.magic,
                Signal::open(&raw const (*r).header.request, &name.request)?,
                Signal::open(&raw const (*r).header.done, &name.done)?,
            )
        };
        if magic != MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "not a DAW transport",
            ));
        }
        Ok(Transport {
            shm,
            request,
            done,
            name: name.clone(),
        })
    }

    pub fn name(&self) -> &TransportName {
        &self.name
    }

    pub fn request(&self) -> &Signal {
        &self.request
    }

    pub fn done(&self) -> &Signal {
        &self.done
    }

    /// Clears both words, for a fresh helper.
    pub fn reset(&self) {
        self.request.word().store(0, Ordering::Release);
        self.done.word().store(0, Ordering::Release);
    }

    /// Host: writes a block into `slot`. Must only be called while the helper is not
    /// processing that slot (no request for it is outstanding).
    pub fn write_block(
        &self,
        slot: usize,
        steady_time: i64,
        events: &[Event],
        left: &[f32],
        right: &[f32],
    ) {
        let frames = left.len().min(right.len()).min(MAX_BLOCK);
        let n_events = events.len().min(MAX_EVENTS);
        let s = self.slot_ptr(slot);
        // SAFETY: `s` is inside the mapping; the protocol gives the host exclusive access
        // to a slot with no outstanding request. Writes go through raw pointers because
        // the other process may map the same memory.
        unsafe {
            (&raw mut (*s).steady_time).write_volatile(steady_time);
            (&raw mut (*s).frames).write_volatile(frames as u32);
            (&raw mut (*s).n_events).write_volatile(n_events as u32);
            (&raw mut (*s).status).write_volatile(STATUS_OK);
            std::ptr::copy_nonoverlapping(
                events.as_ptr(),
                (&raw mut (*s).events) as *mut Event,
                n_events,
            );
            std::ptr::copy_nonoverlapping(
                left.as_ptr(),
                (&raw mut (*s).input[0]) as *mut f32,
                frames,
            );
            std::ptr::copy_nonoverlapping(
                right.as_ptr(),
                (&raw mut (*s).input[1]) as *mut f32,
                frames,
            );
        }
    }

    /// Host: requests processing of the block with sequence number `seq` (in the slot
    /// `seq % SLOTS`).
    pub fn submit(&self, seq: u32) {
        self.request.store(seq & SEQ_MASK);
    }

    /// Host: waits for the helper to finish block `seq`, or to die.
    pub fn wait_done(&self, seq: u32, deadline: Option<Instant>) -> Result<Done, WaitError> {
        let seq = seq & SEQ_MASK;
        // A plugin's block usually comes back within microseconds: spin briefly first.
        let v = self.done.wait_spinning(
            |v| v & DEAD != 0 || v == seq,
            deadline,
            std::time::Duration::from_micros(50),
        )?;
        Ok(if v & DEAD != 0 { Done::Dead } else { Done::Ok })
    }

    /// Host: whether block `seq` is finished, without waiting.
    pub fn poll_done(&self, seq: u32) -> Option<Done> {
        let v = self.done.word().load(Ordering::Acquire);
        if v & DEAD != 0 {
            Some(Done::Dead)
        } else if v == seq & SEQ_MASK {
            Some(Done::Ok)
        } else {
            None
        }
    }

    /// Host: reads a finished block's output and status. Non-finite samples become zero
    /// and are counted.
    pub fn read_output(&self, slot: usize, left: &mut [f32], right: &mut [f32]) -> (i32, usize) {
        let frames = left.len().min(right.len()).min(MAX_BLOCK);
        let s = self.slot_ptr(slot);
        // SAFETY: the helper finished this slot (`done` was read with acquire ordering)
        // and does not touch it again until the next request.
        let status = unsafe {
            std::ptr::copy_nonoverlapping(
                (&raw const (*s).output[0]) as *const f32,
                left.as_mut_ptr(),
                frames,
            );
            std::ptr::copy_nonoverlapping(
                (&raw const (*s).output[1]) as *const f32,
                right.as_mut_ptr(),
                frames,
            );
            (&raw const (*s).status).read_volatile()
        };
        let mut bad = 0;
        for x in left[..frames].iter_mut().chain(right[..frames].iter_mut()) {
            if !x.is_finite() {
                *x = 0.0;
                bad += 1;
            }
        }
        (status, bad)
    }

    /// Host: marks the helper dead and wakes a waiting host thread.
    pub fn mark_dead(&self) {
        self.done.set_bits(DEAD);
    }

    /// Host: asks the helper's processing thread to stop.
    pub fn quit(&self) {
        self.request.set_bits(QUIT);
    }

    fn slot_ptr(&self, slot: usize) -> *mut Slot {
        let r = self.region();
        // SAFETY: the index is reduced modulo `SLOTS`, so the slot is inside the mapping.
        unsafe { &raw mut (*r).slots[slot % SLOTS] }
    }

    /// Helper: waits for a request different from `last`. Returns the new request word
    /// (which may carry [`QUIT`]).
    pub fn wait_request(&self, last: u32, deadline: Option<Instant>) -> Result<u32, WaitError> {
        self.request.wait_until(|v| v != last, deadline)
    }

    /// Helper: the slot for a request, for processing in place.
    ///
    /// # Safety
    /// Only between receiving the request for this slot and calling [`Transport::finish`];
    /// the returned pointer must not outlive the transport.
    pub unsafe fn slot(&self, slot: usize) -> *mut Slot {
        self.slot_ptr(slot)
    }

    /// Helper: reports block `seq` finished.
    pub fn finish(&self, seq: u32) {
        self.done.store(seq & SEQ_MASK);
    }
}

/// How a block ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Done {
    Ok,
    /// The helper died; the block was not processed.
    Dead,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn a_block_round_trips_through_a_second_mapping() {
        let host = Transport::create().unwrap();
        let helper = Transport::open(host.name()).unwrap();
        let left: Vec<f32> = (0..64).map(|i| i as f32).collect();
        let right: Vec<f32> = (0..64).map(|i| -(i as f32)).collect();
        let ev = Event {
            kind: EV_PARAM,
            time: 3,
            param: 7,
            value: 0.25,
            ..Event::default()
        };
        host.write_block(1, 1000, &[ev], &left, &right);
        let t = std::thread::spawn(move || {
            let req = helper.wait_request(0, None).unwrap();
            assert_eq!(req, 1);
            // SAFETY: we hold the request for slot 1.
            unsafe {
                let s = helper.slot(1);
                assert_eq!((*s).frames, 64);
                assert_eq!((*s).steady_time, 1000);
                assert_eq!((*s).events[0], ev);
                for i in 0..64 {
                    (*s).output[0][i] = (*s).input[0][i] * 2.0;
                    (*s).output[1][i] = if i == 5 { f32::NAN } else { (*s).input[1][i] };
                }
            }
            helper.finish(1);
            helper
        });
        host.submit(1);
        assert_eq!(host.wait_done(1, None), Ok(Done::Ok));
        let (mut l, mut r) = (vec![0.0; 64], vec![0.0; 64]);
        let (status, bad) = host.read_output(1, &mut l, &mut r);
        assert_eq!((status, bad), (STATUS_OK, 1));
        assert_eq!(l[10], 20.0);
        assert_eq!(r[5], 0.0, "non-finite output is zeroed");
        let helper = t.join().unwrap();
        // A dead helper wakes the host.
        host.submit(2);
        let marker = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(10));
            host.mark_dead();
            host
        });
        assert_eq!(helper.wait_request(1, None), Ok(2));
        let host = marker.join().unwrap();
        assert_eq!(host.wait_done(2, None), Ok(Done::Dead));
        assert_eq!(
            host.wait_done(3, Some(Instant::now() + Duration::from_millis(5))),
            Ok(Done::Dead)
        );
    }
}
