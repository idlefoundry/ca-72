//! The standalone application's own audio thread made an audio thread (decisions.md
//! R-DROPOUTS).
//!
//! The plug-ins play on their hosts' audio threads, which are the hosts' to schedule:
//! `ca72_rt` never touches them. The standalone's audio thread is CPAL's, and on Linux CPAL's
//! ALSA backend leaves it an ordinary thread (`SCHED_OTHER`), while POLY's workers are made
//! audio threads (`SCHED_FIFO`, decisions.md R11). With other work on the computer, the
//! ordinary thread was preempted past its period: the device ran dry (an xrun), and POLY's
//! voices sat runs out while it waited for workers it could not get back from. It is made an
//! audio thread here, as the workers are, as its first block begins, wherever the system allows
//! real-time priority (an rtprio allowance, `ca72_rt::promote`); where it does not, it stays as
//! it was. The watchdog that makes the threads promoted here ordinary again, should one never
//! stop, watches it too. On macOS CoreAudio's I/O thread and on Windows CPAL's boosted WASAPI
//! thread are already the system's audio threads: nothing is done there.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// The system refused to make the standalone's audio thread an audio thread (said once, off
/// the audio thread: [`report_refusal`]).
static REFUSED: AtomicBool = AtomicBool::new(false);

/// Says once, on standard error, that the audio thread could not be made an audio thread, and
/// what that means: called off the audio thread (the plug-in's helper), after
/// [`AudioThread::begin`] was refused.
pub fn report_refusal() {
    static SAID: AtomicBool = AtomicBool::new(false);
    if REFUSED.load(Ordering::Acquire) && !SAID.swap(true, Ordering::AcqRel) {
        eprintln!(
            "CA-72: the audio thread runs at ordinary priority: this system gives this user no \
             real-time priority (on Linux an rtprio allowance: the `realtime` or `audio` group, \
             or limits.conf). With other work on the computer the sound may drop out \
             (decisions.md R-DROPOUTS)."
        );
    }
}

/// Whether the next block is to promote the thread it is played on, and for what period.
#[derive(Debug, Default)]
pub struct AudioThread {
    asked: Option<Duration>,
}

impl AudioThread {
    /// Before a stream's first block, off the audio thread (`Plugin::initialize`): the next
    /// block promotes its thread if this is the standalone application (`standalone`) on Linux,
    /// for blocks of at most `period`. The watchdog is started here, where starting a thread is
    /// allowed.
    pub fn prepare(&mut self, standalone: bool, period: Duration) {
        self.asked = (standalone && cfg!(target_os = "linux")).then_some(period);
        if self.asked.is_some() {
            ca72_rt::watch();
        }
    }

    /// At a block's start, on the audio thread: its thread promoted, the first time after
    /// [`AudioThread::prepare`] asked; the system's answer (None: nothing asked). A refusal is
    /// for [`report_refusal`] to say.
    pub fn begin(&mut self) -> Option<Result<(), String>> {
        let period = self.asked.take()?;
        // (The watchdog's note of the thread, the first time it is promoted, is its one
        // allocation, and a refusal's explanation another; the plug-ins never come here.)
        let r = nih_plug::util::permit_alloc(|| ca72_rt::promote(period, period / 2));
        if r.is_err() {
            REFUSED.store(true, Ordering::Release);
        }
        Some(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only the standalone's thread, only on Linux, and only once after each `prepare`: a
    /// plug-in's host's thread is never touched.
    #[test]
    fn only_the_standalone_s_thread_is_promoted_and_once() {
        let period = Duration::from_secs_f64(256.0 / 48_000.0);
        let mut a = AudioThread::default();
        assert!(a.begin().is_none(), "promoted before it was asked");
        a.prepare(false, period);
        assert!(a.begin().is_none(), "a plug-in's host's thread promoted");
        // (On a thread of its own, made ordinary again after: the test's threads are not
        // to be left real-time.)
        let linux = cfg!(target_os = "linux");
        std::thread::spawn(move || {
            a.prepare(true, period);
            let r = a.begin();
            assert_eq!(r.is_some(), linux);
            if let Some(Ok(())) = r {
                assert!(ca72_rt::is_realtime(), "said promoted, but ordinary");
                ca72_rt::demote();
            }
            assert!(a.begin().is_none(), "promoted at every block");
        })
        .join()
        .unwrap();
    }
}
