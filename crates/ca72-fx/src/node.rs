//! One effect in the chain, on the audio thread (docs/fx-host.md).
//!
//! **Synchronous** hands each block to the helper and waits for it, up to the block's
//! deadline: no latency added. **Pipelined** collects the input into chunks of the device's
//! block size and runs a chunk behind: the helper processes chunk *k* while the synth plays
//! the next, and the node outputs chunk *k − 1*, one block of latency added.
//!
//! In real time the node never waits past its deadline: a late or lost block, a helper
//! that died, an effect given up on, passes its input instead (dry), and a miss is counted.
//! Offline (the tests) it waits for every block, and for a restarted helper, and does the
//! lost block again, so that a render has no gap.
//!
//! The processing path does not allocate: every buffer is sized when the node is made.
//! DAW-v3's `daw-sandbox/src/node.rs` (commit cb15558, its D19 and D25), effects only: no
//! notes, modulation or fault injection (the test plug-ins' crash and hang are parameters).

use crate::sandbox::{Instance, RUNNING};
use daw_ipc::layout::{EV_PARAM, SEQ_MASK, STATUS_OK};
use daw_ipc::{Done, Event, MAX_BLOCK, MAX_EVENTS, SLOTS};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

pub use crate::Mode;

/// How long an offline node waits for a helper it killed to be seen gone.
const KILL_WAIT: Duration = Duration::from_secs(10);

/// A fixed-capacity stereo FIFO.
struct Fifo {
    l: Vec<f32>,
    r: Vec<f32>,
    head: usize,
    len: usize,
}

impl Fifo {
    fn new(cap: usize) -> Fifo {
        Fifo {
            l: vec![0.0; cap],
            r: vec![0.0; cap],
            head: 0,
            len: 0,
        }
    }

    fn push(&mut self, l: &[f32], r: &[f32]) {
        let cap = self.l.len();
        for (a, b) in l.iter().zip(r) {
            if self.len == cap {
                return;
            }
            let i = (self.head + self.len) % cap;
            self.l[i] = *a;
            self.r[i] = *b;
            self.len += 1;
        }
    }

    fn push_zeros(&mut self, n: usize) {
        let cap = self.l.len();
        for _ in 0..n {
            if self.len == cap {
                return;
            }
            let i = (self.head + self.len) % cap;
            self.l[i] = 0.0;
            self.r[i] = 0.0;
            self.len += 1;
        }
    }

    /// Pops up to `l.len()` frames; missing frames are zero.
    fn pop(&mut self, l: &mut [f32], r: &mut [f32]) {
        let cap = self.l.len();
        for (a, b) in l.iter_mut().zip(r.iter_mut()) {
            if self.len == 0 {
                *a = 0.0;
                *b = 0.0;
                continue;
            }
            *a = self.l[self.head];
            *b = self.r[self.head];
            self.head = (self.head + 1) % cap;
            self.len -= 1;
        }
    }
}

/// A block handed to the helper (or due to be).
struct Flight {
    active: bool,
    seq: u32,
    frame: i64,
    len: usize,
    l: Vec<f32>,
    r: Vec<f32>,
    events: Vec<Event>,
}

/// An effect's node.
pub struct Node {
    inst: Arc<Instance>,
    mode: Mode,
    realtime: bool,
    hang_timeout: Duration,
    seq: u32,
    generation: u32,
    /// A real-time request that missed its deadline and may still be running.
    outstanding: bool,
    /// The helper was restored and no block has come back since.
    recovering: bool,
    /// Parameter changes not yet delivered: (parameter ID, plain value).
    pending: Vec<(u32, f64)>,
    flight: Flight,
    out_l: Vec<f32>,
    out_r: Vec<f32>,
    // Pipelined.
    chunk: usize,
    acc_l: Vec<f32>,
    acc_r: Vec<f32>,
    acc_len: usize,
    acc_start: i64,
    fifo: Fifo,
    /// The deadline of the block being processed (real time).
    deadline: Option<Instant>,
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("effect", &self.inst.label)
            .field("mode", &self.mode)
            .finish_non_exhaustive()
    }
}

impl Node {
    /// A node for a loaded instance; `block` is the device's largest block (the pipelined
    /// chunk).
    pub fn new(
        inst: Arc<Instance>,
        mode: Mode,
        realtime: bool,
        block: usize,
        hang_timeout: Duration,
    ) -> Node {
        let chunk = block.clamp(1, MAX_BLOCK);
        let generation = inst.generation.load(Ordering::Acquire);
        let mut fifo = Fifo::new(2 * chunk + MAX_BLOCK);
        if mode == Mode::Pipelined {
            fifo.push_zeros(chunk);
        }
        let n_params = inst.info.params.len();
        Node {
            seq: inst.transport.request().word().load(Ordering::Acquire) & SEQ_MASK,
            inst,
            mode,
            realtime,
            hang_timeout,
            generation,
            outstanding: false,
            recovering: false,
            // Room for every parameter's value at once, besides a block's events.
            pending: Vec::with_capacity(MAX_EVENTS + n_params),
            flight: Flight {
                active: false,
                seq: 0,
                frame: 0,
                len: 0,
                l: vec![0.0; MAX_BLOCK],
                r: vec![0.0; MAX_BLOCK],
                events: Vec::with_capacity(MAX_EVENTS),
            },
            out_l: vec![0.0; MAX_BLOCK],
            out_r: vec![0.0; MAX_BLOCK],
            chunk,
            acc_l: vec![0.0; chunk],
            acc_r: vec![0.0; chunk],
            acc_len: 0,
            acc_start: 0,
            fifo,
            deadline: None,
        }
    }

    pub fn instance(&self) -> &Arc<Instance> {
        &self.inst
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// The node's latency in frames: the plug-in's, and a pipelined node's chunk.
    pub fn latency(&self) -> u32 {
        let plugin = self.inst.latency.load(Ordering::Relaxed);
        match self.mode {
            Mode::Sync => plugin,
            Mode::Pipelined => plugin + self.chunk as u32,
        }
    }

    /// Sets a parameter (plain value) from the next block on. Never allocates: the list has
    /// room for every parameter once, and a parameter pending already takes the new value.
    pub fn set_param(&mut self, id: u32, plain: f64) {
        if self.inst.param_index(id).is_none() {
            return;
        }
        if let Some(slot) = self.pending.iter_mut().find(|(pid, _)| *pid == id) {
            slot.1 = plain;
        } else if self.pending.len() < self.pending.capacity() {
            self.pending.push((id, plain));
        }
    }

    fn next_seq(&mut self) -> u32 {
        self.seq = (self.seq + 1) & SEQ_MASK;
        if self.seq == 0 {
            self.seq = 1;
        }
        self.seq
    }

    /// Whether the helper can take a block. Offline, waits for a restore under way.
    fn ensure_running(&mut self) -> bool {
        loop {
            match self.inst.status() {
                RUNNING => {
                    let g = self.inst.generation.load(Ordering::Acquire);
                    if g != self.generation {
                        self.generation = g;
                        self.outstanding = false;
                        self.recovering = true;
                        // Restored: its last snapshot and the values it took before. What
                        // waited is dropped: a change the helper died or hung on (the test
                        // plug-ins' crash and hang) would only take it down again.
                        self.pending.clear();
                        self.flight.events.clear();
                    }
                    return true;
                }
                crate::sandbox::DOWN if !self.realtime => {
                    if self.inst.wait_recovered(self.generation).is_none() {
                        return false;
                    }
                }
                _ => return false,
            }
        }
    }

    /// The pending parameter changes, at the block's start, as many as fit.
    fn build_events(&mut self) {
        let ev = &mut self.flight.events;
        ev.clear();
        let mut kept = 0;
        for k in 0..self.pending.len() {
            let (id, v) = self.pending[k];
            if ev.len() < ev.capacity() {
                ev.push(Event {
                    kind: EV_PARAM,
                    time: 0,
                    param: id,
                    value: v,
                    ..Event::default()
                });
            } else {
                self.pending[kept] = (id, v);
                kept += 1;
            }
        }
        self.pending.truncate(kept);
    }

    fn submit_flight(&mut self) {
        let seq = self.next_seq();
        self.flight.seq = seq;
        let n = self.flight.len;
        self.inst
            .position
            .store(self.flight.frame, Ordering::Relaxed);
        self.inst.transport.write_block(
            seq as usize % SLOTS,
            self.flight.frame,
            &self.flight.events,
            &self.flight.l[..n],
            &self.flight.r[..n],
        );
        self.inst
            .submitted_at
            .store(self.inst.now_us(), Ordering::Release);
        self.inst.transport.submit(seq);
        self.flight.active = true;
    }

    /// Whether the helper the flight went to has gone since.
    fn restarted(&self) -> bool {
        self.inst.status() != RUNNING
            || self.inst.generation.load(Ordering::Acquire) != self.generation
    }

    /// Records the parameter values a finished block delivered, what a restore gives the
    /// effect again; a hidden parameter's is not (the test plug-ins' crash and hang, which
    /// the plug-in takes a block later, would take the restored one down again).
    fn delivered(&mut self) {
        for e in &self.flight.events {
            if e.kind == EV_PARAM
                && let Some(i) = self.inst.param_index(e.param)
                && !self.inst.info.params[i].hidden
            {
                self.inst.set_value(i, e.value);
            }
        }
        self.inst.submitted_at.store(0, Ordering::Release);
    }

    /// Waits for the flight's output (into `out_l`/`out_r`): offline until the block is
    /// done or the instance fails, in real time until the deadline. Whether real output came.
    fn land(&mut self) -> bool {
        loop {
            if !self.flight.active {
                // Not sent (the helper was down): offline, try now; in real time, lost.
                if self.realtime || !self.ensure_running() {
                    return false;
                }
                // The new instance has its snapshot and the values the host set (given at
                // its activation); the flight's own changes go with it again.
                self.submit_flight();
            }
            let until = if self.realtime {
                self.deadline
                    .unwrap_or_else(|| Instant::now() + Duration::from_millis(2))
            } else {
                Instant::now() + self.hang_timeout
            };
            let done = match self.inst.transport.wait_done(self.flight.seq, Some(until)) {
                // A restore stores the last request's number in `done`, which reads as this
                // block done if this thread did not run between the death and the restore:
                // a block whose helper has gone since is lost (DAW-v3's finding).
                Ok(Done::Ok) if self.restarted() => Ok(Done::Dead),
                r => r,
            };
            match done {
                Ok(Done::Ok) => {
                    let n = self.flight.len;
                    let (status, _bad) = self.inst.transport.read_output(
                        self.flight.seq as usize % SLOTS,
                        &mut self.out_l[..n],
                        &mut self.out_r[..n],
                    );
                    self.flight.active = false;
                    self.delivered();
                    if self.recovering {
                        self.recovering = false;
                        self.inst
                            .resumed_at
                            .store(self.flight.frame, Ordering::Release);
                    }
                    return status == STATUS_OK;
                }
                Ok(Done::Dead) => {
                    self.flight.active = false;
                    self.inst.submitted_at.store(0, Ordering::Release);
                    if self.realtime {
                        return false;
                    }
                }
                Err(_) if self.realtime => {
                    self.outstanding = true;
                    self.flight.active = false;
                    self.inst.misses.fetch_add(1, Ordering::Relaxed);
                    return false;
                }
                Err(_) => {
                    // Offline watchdog: the helper hangs; kill it, wait to see it go.
                    self.inst.hung.store(true, Ordering::Release);
                    self.inst.kill_helper();
                    let _ = self
                        .inst
                        .transport
                        .wait_done(self.flight.seq, Some(Instant::now() + KILL_WAIT));
                    self.inst
                        .wait_down(self.generation, Instant::now() + KILL_WAIT);
                    self.inst.submitted_at.store(0, Ordering::Release);
                    self.flight.active = false;
                }
            }
        }
    }

    /// Whether a block may be sent now (real time: none may still be running).
    fn may_send(&mut self) -> bool {
        if !self.ensure_running() {
            return false;
        }
        if self.outstanding {
            match self.inst.transport.poll_done(self.seq) {
                Some(_) => {
                    self.outstanding = false;
                    self.inst.submitted_at.store(0, Ordering::Release);
                }
                None => {
                    self.inst.misses.fetch_add(1, Ordering::Relaxed);
                    return false;
                }
            }
        }
        true
    }

    /// The flight was not processed: its input goes out instead, and its parameter changes
    /// wait for the next block (a newer pending value wins).
    fn lost(&mut self) {
        for k in 0..self.flight.events.len() {
            let e = self.flight.events[k];
            if e.kind == EV_PARAM
                && !self.pending.iter().any(|(id, _)| *id == e.param)
                && self.pending.len() < self.pending.capacity()
            {
                self.pending.push((e.param, e.value));
            }
        }
        let n = self.flight.len;
        self.out_l[..n].copy_from_slice(&self.flight.l[..n]);
        self.out_r[..n].copy_from_slice(&self.flight.r[..n]);
    }

    fn process_sync(&mut self, frame: i64, l: &mut [f32], r: &mut [f32]) {
        let n = l.len();
        self.flight.frame = frame;
        self.flight.len = n;
        self.flight.l[..n].copy_from_slice(l);
        self.flight.r[..n].copy_from_slice(r);
        self.flight.active = false;
        self.build_events();
        let ok = if self.may_send() {
            self.submit_flight();
            self.land()
        } else {
            false
        };
        if !ok {
            self.lost();
        }
        l.copy_from_slice(&self.out_l[..n]);
        r.copy_from_slice(&self.out_r[..n]);
    }

    /// Collects the chunk in flight into the FIFO.
    fn collect(&mut self) {
        if self.flight.len == 0 {
            return;
        }
        let ok = if self.flight.active {
            self.land()
        } else {
            !self.realtime && self.land()
        };
        if !ok {
            self.lost();
        }
        let n = self.flight.len;
        self.fifo.push(&self.out_l[..n], &self.out_r[..n]);
        self.flight.len = 0;
    }

    /// Sends the accumulated chunk.
    fn launch(&mut self) {
        let n = self.acc_len;
        self.flight.frame = self.acc_start;
        self.flight.len = n;
        self.flight.l[..n].copy_from_slice(&self.acc_l[..n]);
        self.flight.r[..n].copy_from_slice(&self.acc_r[..n]);
        self.flight.active = false;
        self.build_events();
        if self.may_send() {
            self.submit_flight();
        }
    }

    fn process_pipelined(&mut self, frame: i64, l: &mut [f32], r: &mut [f32]) {
        let n = l.len();
        let mut off = 0;
        while off < n {
            if self.acc_len == 0 {
                self.acc_start = frame + off as i64;
            }
            let take = (self.chunk - self.acc_len).min(n - off);
            self.acc_l[self.acc_len..self.acc_len + take].copy_from_slice(&l[off..off + take]);
            self.acc_r[self.acc_len..self.acc_len + take].copy_from_slice(&r[off..off + take]);
            self.acc_len += take;
            off += take;
            if self.acc_len == self.chunk {
                self.collect();
                self.launch();
                self.acc_len = 0;
            }
        }
        if self.fifo.len < n {
            self.collect();
        }
        self.fifo.pop(l, r);
    }

    /// Processes `l` and `r` (at most `MAX_BLOCK` frames, the block at `frame`) in place.
    /// In real time the helper's block is waited for until `deadline`.
    pub fn process(&mut self, frame: i64, l: &mut [f32], r: &mut [f32], deadline: Option<Instant>) {
        debug_assert!(l.len() == r.len() && l.len() <= MAX_BLOCK);
        let n = l.len().min(r.len()).min(MAX_BLOCK);
        self.deadline = deadline;
        let (l, r) = (&mut l[..n], &mut r[..n]);
        match self.mode {
            Mode::Sync => self.process_sync(frame, l, r),
            Mode::Pipelined => self.process_pipelined(frame, l, r),
        }
    }
}
