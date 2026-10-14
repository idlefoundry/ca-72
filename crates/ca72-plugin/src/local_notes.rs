//! Locally played notes cross into the audio thread through a bounded queue. A generation
//! independent of the queue releases notes after focus loss or overflow, even if no NoteOff
//! could fit. The audio thread owns the merged physical/local note state.

use plugin_kit_queue::AtomicQueue;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// A callback takes at most this many local events, including discarded stale events.
pub const CAPACITY: usize = 256;

#[derive(Clone, Copy, Debug)]
pub struct LocalEvent {
    pub note: u8,
    pub on: bool,
    pub generation: u64,
}

#[derive(Debug)]
pub struct LocalNotes {
    queue: AtomicQueue,
    generation: AtomicU64,
    panic: AtomicU64,
    standalone_panic: AtomicBool,
    held: [AtomicU64; 2],
}

impl Default for LocalNotes {
    fn default() -> Self {
        Self {
            queue: AtomicQueue::new(CAPACITY),
            generation: AtomicU64::new(0),
            panic: AtomicU64::new(0),
            standalone_panic: AtomicBool::new(false),
            held: std::array::from_fn(|_| AtomicU64::new(0)),
        }
    }
}

impl LocalNotes {
    /// Nonblocking: failure invalidates all queued local notes and requests their release.
    pub fn note(&self, note: u8, on: bool) -> bool {
        if note > 127 {
            return false;
        }
        // Capture the epoch before any producer reservation. If this producer is interrupted
        // by a release, its resumed publication stays stale instead of reviving that note.
        let generation = self.generation();
        let packet = (generation << 9) | (1 << 8) | (u64::from(on) << 7) | u64::from(note);
        if !self.queue.try_push(packet) {
            self.release();
            return false;
        }
        true
    }

    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire) & ((1u64 << 55) - 1)
    }

    /// Local notes only. Physical notes stay held when an editor closes or changes view.
    pub fn release(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
    }

    /// A player's explicit panic: physical and local notes, controllers and old events.
    pub fn panic(&self) {
        self.release();
        self.panic.fetch_add(1, Ordering::AcqRel);
    }

    pub fn panic_generation(&self) -> u64 {
        self.panic.load(Ordering::Acquire)
    }

    /// The UI pairs its epoch change with a hardware-source panic. Audio must hold fresh
    /// queued local notes until that boundary, even if the requests span two callbacks.
    pub fn await_standalone_panic(&self) {
        self.standalone_panic.store(true, Ordering::Release);
    }

    pub(crate) fn standalone_panic_pending(&self) -> bool {
        self.standalone_panic.load(Ordering::Acquire)
    }

    pub(crate) fn finish_standalone_panic(&self) {
        self.standalone_panic.store(false, Ordering::Release);
    }

    /// Only the plugin's single audio consumer calls this. The producer never advances its
    /// tail while a slot is incomplete, so an interrupted producer cannot stall the callback.
    pub(crate) fn pop(&self) -> Option<LocalEvent> {
        let packet = self.queue.try_pop()?;
        Some(LocalEvent {
            note: (packet & 127) as u8,
            on: packet & (1 << 7) != 0,
            generation: packet >> 9,
        })
    }

    pub fn publish(&self, held: [u64; 2]) {
        for (a, v) in self.held.iter().zip(held) {
            a.store(v, Ordering::Relaxed);
        }
    }

    pub fn held(&self) -> [u64; 2] {
        self.held.each_ref().map(|a| a.load(Ordering::Relaxed))
    }
}

/// Ownership, rather than voice allocation: the existing engine keeps its original attacks,
/// priorities and voice stealing. A release reaches it only after the last source lets go.
#[derive(Debug)]
pub struct Sources {
    physical: [[bool; 128]; 16],
    local: [bool; 128],
    held: [u64; 2],
}

impl Default for Sources {
    fn default() -> Self {
        Self {
            physical: [[false; 128]; 16],
            local: [false; 128],
            held: [0, 0],
        }
    }
}

impl Sources {
    pub fn physical(&mut self, channel: u8, note: u8, on: bool) -> bool {
        let (channel, note) = (usize::from(channel), usize::from(note));
        if channel >= 16 || note >= 128 {
            return false;
        }
        self.physical[channel][note] = on;
        self.refresh(note);
        // Keep every incoming physical attack, including repeated attacks, as before.
        on || !self.down(note)
    }

    pub fn local(&mut self, note: u8, on: bool) -> bool {
        let note = usize::from(note);
        if note >= 128 {
            return false;
        }
        if self.local[note] == on {
            return false;
        }
        self.local[note] = on;
        self.refresh(note);
        on || !self.down(note)
    }

    fn down(&self, note: usize) -> bool {
        self.local[note] || self.physical.iter().any(|channel| channel[note])
    }

    fn refresh(&mut self, note: usize) {
        let mask = 1 << (note % 64);
        if self.down(note) {
            self.held[note / 64] |= mask;
        } else {
            self.held[note / 64] &= !mask;
        }
    }

    pub fn release_local(&mut self, mut off: impl FnMut(u8)) {
        for note in 0..128 {
            if std::mem::take(&mut self.local[note]) && !self.down(note) {
                off(note as u8);
                self.refresh(note);
            }
        }
    }

    pub fn clear(&mut self) {
        self.physical = [[false; 128]; 16];
        self.local = [false; 128];
        self.held = [0, 0];
    }

    pub fn held(&self) -> [u64; 2] {
        self.held
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn releasing_one_source_preserves_the_other_sources_pitch() {
        let mut s = Sources::default();
        assert!(s.physical(0, 60, true));
        assert!(s.physical(1, 60, true));
        assert!(s.local(60, true));
        assert!(!s.physical(0, 60, false));
        assert!(!s.physical(1, 60, false));
        assert!(s.local(60, false));
        assert_eq!(s.held(), [0, 0]);
        s.physical(0, 60, true);
        s.local(60, true);
        s.local(67, true);
        let mut off = Vec::new();
        s.release_local(|n| off.push(n));
        assert_eq!(off, [67]);
        assert_eq!(s.held(), [1 << 60, 0]);
    }

    #[test]
    fn overflow_invalidates_pending_attacks_without_needing_queue_space() {
        let q = LocalNotes::default();
        for _ in 0..CAPACITY {
            assert!(q.note(60, true));
        }
        assert!(!q.note(60, false));
        assert_eq!(q.generation(), 1);
        let stale: Vec<_> = (0..CAPACITY).filter_map(|_| q.pop()).collect();
        assert_eq!(stale.len(), CAPACITY);
        assert!(stale.iter().all(|e| e.generation != q.generation()));
        assert!(q.pop().is_none());
        assert!(q.note(62, true));
        assert_eq!(q.pop().expect("one fresh event").generation, q.generation());
        q.panic();
        assert_eq!(q.panic_generation(), 1);
        assert_eq!(q.generation(), 2);
    }

    #[test]
    fn queue_wrap_and_resumed_stale_producer_never_revive_a_released_note() {
        let q = LocalNotes::default();
        for _ in 0..10 {
            for note in 0..CAPACITY {
                assert!(q.note((note % 128) as u8, true));
            }
            for note in 0..CAPACITY {
                assert_eq!(q.pop().expect("wrapped event").note, (note % 128) as u8);
            }
            assert!(q.pop().is_none());
        }
        // A producer captured this packet before it was preempted by a release. Resuming
        // after the release cannot promote it into the new epoch.
        let packet = (q.generation() << 9) | (1 << 8) | (1 << 7) | 60;
        q.release();
        assert!(q.queue.try_push(packet));
        assert_ne!(
            q.pop().expect("resumed producer").generation,
            q.generation()
        );
        assert!(q.note(62, true));
        assert_eq!(q.pop().expect("fresh producer").generation, q.generation());
    }

    #[test]
    fn concurrent_publication_preserves_complete_events_and_per_producer_order() {
        let q = std::sync::Arc::new(LocalNotes::default());
        let done = std::sync::Arc::new(AtomicBool::new(false));
        let producer = {
            let q = q.clone();
            let done = done.clone();
            std::thread::spawn(move || {
                for _ in 0..1000 {
                    for (note, on) in [(60, true), (64, true), (60, false), (64, false)] {
                        while !q.note(note, on) {
                            std::thread::yield_now();
                        }
                    }
                }
                done.store(true, Ordering::Release);
            })
        };
        let mut count = 0;
        while !done.load(Ordering::Acquire) || !q.queue.is_empty() {
            if let Some(e) = q.pop() {
                let (note, on) = [(60, true), (64, true), (60, false), (64, false)][count % 4];
                assert_eq!((e.note, e.on), (note, on));
                count += 1;
            } else {
                std::thread::yield_now();
            }
        }
        producer.join().expect("producer joined");
        assert_eq!(count, 4000);
    }
}
