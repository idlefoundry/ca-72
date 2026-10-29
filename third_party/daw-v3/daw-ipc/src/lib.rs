//! The sandbox transport (DESIGN 10.2): what the host and a plugin helper process share.
//!
//! - [`shm`]: named shared memory;
//! - [`wake`]: cross-process wake-ups on a word in that memory;
//! - [`layout`]: the block transport of one plugin instance (audio, events, sequence
//!   numbers);
//! - [`protocol`]: the line-based control channel (load, activate, state).
//!
//! Unsafe code is confined to `shm`, `wake` and the raw slot access in `layout`; each
//! block carries a `SAFETY` comment and the modules are listed in `docs/unsafe.md`.

pub mod layout;
pub mod protocol;
pub mod rt;
pub mod shm;
pub mod wake;

pub use layout::{Done, Event, MAX_BLOCK, MAX_EVENTS, SLOTS, Timing, Transport, TransportName};
pub use wake::WaitError;
