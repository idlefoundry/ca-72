//! The real-time watchdog (Linux): a thread at real-time priority that spins on one CPU
//! for good is made ordinary within about [`STARVED`] of the CPU's ordinary threads last
//! running, and counted. Only one CPU is held, and for at most three seconds whatever
//! happens. Where real-time priority is not allowed, there is nothing to watch: skipped.

#![cfg(target_os = "linux")]
#![allow(clippy::unwrap_used)]

use daw_ipc::rt::{self, STARVED};
use std::time::{Duration, Instant};

#[test]
fn a_real_time_thread_that_never_stops_is_made_ordinary() {
    let cpus = std::thread::available_parallelism().map_or(1, |n| n.get());
    let spinner = std::thread::spawn(move || {
        rt::promote(Duration::from_millis(5), Duration::from_millis(2))?;
        // The last CPU, so as to hold only one.
        rt::pin(0, cpus - 1).unwrap();
        let t = Instant::now();
        while rt::is_realtime() && t.elapsed() < Duration::from_secs(3) {
            std::hint::spin_loop();
        }
        Ok::<_, String>((t.elapsed(), rt::is_realtime()))
    });
    let (took, still) = match spinner.join().unwrap() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("real-time priority is not allowed here ({e}): skipped");
            return;
        }
    };
    eprintln!(
        "made ordinary after {took:?} of spinning; demotions {}",
        rt::demotions()
    );
    assert!(!still, "still real-time after {took:?}");
    assert!(rt::demotions() >= 1);
    assert!(took < STARVED + Duration::from_millis(400), "{took:?}");
}
