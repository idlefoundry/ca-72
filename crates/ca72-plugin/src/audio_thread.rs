//! The standalone application's own audio thread made an audio thread (decisions.md
//! R-DROPOUTS), RTKit asked where Linux refuses (R-RTKIT), and what became of it and of POLY's
//! workers, for the editor to say.
//!
//! The plug-ins play on their hosts' audio threads, which are the hosts' to schedule:
//! `ca72_rt` never touches them. The standalone's audio thread is CPAL's, and on Linux CPAL's
//! ALSA backend leaves it an ordinary thread (`SCHED_OTHER`), while POLY's workers are made
//! audio threads (`SCHED_FIFO`, decisions.md R11). With other work on the computer, the
//! ordinary thread was preempted past its period: the device ran dry (an xrun), and POLY's
//! voices sat runs out while it waited for workers it could not get back from. It is made an
//! audio thread here, as the workers are, as its first block begins, wherever the system allows
//! real-time priority (an rtprio allowance, `ca72_rt::promote_in`). Where it does not, in the
//! standalone, the thread and the workers are asked of RTKit (`ca72_rt::use_rtkit`), by
//! `ca72_rt`'s own thread: the audio thread only leaves its id. Where RTKit cannot or will not,
//! they stay as they were, and the editor says so ([`warning`]). The watchdog that makes the
//! threads promoted here ordinary again, should one never stop, watches it too. On macOS
//! CoreAudio's I/O thread and on Windows CPAL's boosted WASAPI thread are already the system's
//! audio threads: nothing is done there, and only a worker refused is said.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// The standalone's audio thread, as promoted (Linux).
pub static AUDIO: ca72_rt::Group = ca72_rt::Group::new("the audio thread");

/// POLY's workers and ULTRA's split crews, as promoted (`crate::pool`).
pub static WORKERS: ca72_rt::Group = ca72_rt::Group::new("POLY's workers");

/// The system refused to make the standalone's audio thread an audio thread, and RTKit was
/// not asked (said once, off the audio thread: [`report_refusal`]).
static REFUSED: AtomicBool = AtomicBool::new(false);

/// Says once, on standard error, that the audio thread could not be made an audio thread, and
/// what that means: called off the audio thread (the plug-in's helper), after
/// [`AudioThread::begin`] was refused. (RTKit's answers are said by `ca72_rt`'s thread.)
pub fn report_refusal() {
    static SAID: AtomicBool = AtomicBool::new(false);
    if REFUSED.load(Ordering::Acquire) && !SAID.swap(true, Ordering::AcqRel) {
        eprintln!(
            "CA-72: the audio thread runs at ordinary priority: this system gives this user no \
             real-time priority (on Linux an rtprio allowance: the `realtime` or `audio` group, \
             or limits.conf; or RTKit). With other work on the computer the sound may drop out \
             (decisions.md R-DROPOUTS, R-RTKIT)."
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
    /// for blocks of at most `period`. The watchdog, and RTKit's thread, are started here, where
    /// starting a thread is allowed. A plug-in never comes past the first line.
    pub fn prepare(&mut self, standalone: bool, period: Duration) {
        self.asked = (standalone && cfg!(target_os = "linux")).then_some(period);
        if self.asked.is_some() {
            AUDIO.clear();
            // (Not in the tests: a test process is not to ask the system's RTKit for its
            // threads, nor have its RLIMIT_RTTIME set. `ca72_rt`'s own tests and the
            // standalone's check it: decisions.md R-RTKIT.)
            #[cfg(not(test))]
            ca72_rt::use_rtkit();
            ca72_rt::watch();
        }
    }

    /// At a block's start, on the audio thread: its thread promoted, the first time after
    /// [`AudioThread::prepare`] asked; the system's answer (None: nothing asked). Asked of
    /// RTKit is an answer to come (`AUDIO`); a refusal is for [`report_refusal`] to say.
    pub fn begin(&mut self) -> Option<Result<ca72_rt::Promoted, String>> {
        let period = self.asked.take()?;
        // (The watchdog's note of the thread, the first time it is promoted, is its one
        // allocation, and a refusal's explanation another; the plug-ins never come here.)
        let r = nih_plug::util::permit_alloc(|| ca72_rt::promote_in(&AUDIO, period, period / 2));
        if r.is_err() {
            REFUSED.store(true, Ordering::Release);
        }
        Some(r)
    }
}

/// What the standalone's editor says when its audio is not real-time (decisions.md R-RTKIT):
/// once on the walnut, as a tip ([`Warning::tip`]), and under SETTINGS' STANDALONE and in AUDIO
/// & MIDI while it lasts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    /// The walnut's line: which threads, what it means, and the fix (in the panel's capitals).
    pub tip: String,
    /// SETTINGS' row under AUDIO & MIDI….
    pub row: &'static str,
    /// AUDIO & MIDI's lines: which threads and why; the fix in full.
    pub detail: String,
    pub remedy: &'static str,
}

/// How the standalone's audio threads stand: real-time, waiting for RTKit, or some ordinary
/// (and why): AUDIO & MIDI's status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// Nothing promoted or asked yet (no stream, or POLY off on macOS and Windows).
    Unknown,
    /// Waiting for RTKit's answer.
    Asking,
    /// Every thread asked is real-time; some through RTKit.
    RealTime { rtkit: bool },
    /// Some are not.
    Ordinary,
}

/// The audio thread's and the workers' standing now ([`ca72_rt::Group::standing`]): for the
/// editor, at its frames (atomics only).
#[cfg(not(test))]
pub fn standing() -> (ca72_rt::Standing, ca72_rt::Standing) {
    (AUDIO.standing(), WORKERS.standing())
}

/// In the unit tests, nothing asked: the groups are the process's, and the tests' own POLY
/// workers are refused real-time where the system allows none (CI's runners), so an editor that
/// read them would say so, its tip drawn whole, as other tests run. A test of what is said gives
/// its own (the editor's `stood`, AUDIO & MIDI's `review`).
#[cfg(test)]
pub fn standing() -> (ca72_rt::Standing, ca72_rt::Standing) {
    Default::default()
}

/// The standalone's audio as `audio` and `workers` stand.
pub fn summary(audio: ca72_rt::Standing, workers: ca72_rt::Standing) -> Standing {
    if !audio.settled() || !workers.settled() {
        Standing::Asking
    } else if audio.ordinary + workers.ordinary > 0 {
        Standing::Ordinary
    } else if audio.direct + audio.rtkit + workers.direct + workers.rtkit > 0 {
        Standing::RealTime {
            rtkit: audio.rtkit + workers.rtkit > 0,
        }
    } else {
        Standing::Unknown
    }
}

/// What to say when the standalone's audio threads stand as `audio` and `workers` do on
/// `linux` or not: None while every one asked is real-time, or an answer is still to come.
pub fn warning(
    audio: ca72_rt::Standing,
    workers: ca72_rt::Standing,
    linux: bool,
) -> Option<Warning> {
    use ca72_rt::Why;
    if summary(audio, workers) != Standing::Ordinary {
        return None;
    }
    let (a, w) = (audio.ordinary > 0, workers.ordinary > 0);
    let why = if a { audio.why } else { workers.why };
    let who = match (a, w) {
        (true, true) => "AUDIO",
        (true, false) => "AUDIO THREAD",
        _ => "POLY'S THREADS",
    };
    // (Logging in again for RTKit too: the desktop portal, through which PipeWire's own client in
    // this process asks it, reads RTKit as it was at login: decisions.md R-RTKIT.)
    let hint = match why {
        _ if !linux => "THE SYSTEM REFUSED",
        Some(Why::RtkitRefused) => {
            "RTKIT REFUSED: JOIN THE REALTIME OR AUDIO GROUP, THEN LOG IN AGAIN"
        }
        Some(Why::RtkitFailed) => {
            "RTKIT FAILED: JOIN THE REALTIME OR AUDIO GROUP, THEN LOG IN AGAIN"
        }
        Some(Why::Full) => "TOO MANY REAL-TIME THREADS",
        _ => "INSTALL RTKIT OR JOIN THE REALTIME OR AUDIO GROUP, THEN LOG IN AGAIN",
    };
    let tip = format!("{who} NOT REAL-TIME: MAY DROP OUT · {hint}");
    let threads = |s: &ca72_rt::Standing| s.direct + s.rtkit + s.ordinary;
    let mut whom = Vec::new();
    if a {
        whom.push("the audio thread".to_owned());
    }
    if w {
        let n = threads(&workers);
        whom.push(if workers.ordinary == n {
            "POLY's workers".to_owned()
        } else {
            format!("{} of POLY's {n} workers", workers.ordinary)
        });
    }
    let reason = match why {
        Some(Why::NoBus) => "no rtprio allowance; no system bus to ask RTKit on",
        Some(Why::NoRtkit) => "no rtprio allowance; RTKit not running",
        Some(Why::RtkitRefused) => "no rtprio allowance; RTKit refused",
        Some(Why::RtkitFailed) => "no rtprio allowance; RTKit could not be asked",
        Some(Why::Full) => "too many real-time threads",
        Some(Why::Refused) | None if linux => "no rtprio allowance",
        Some(Why::Refused) | None => "the system refused",
    };
    let remedy = match why {
        _ if !linux => "The log says more.",
        Some(Why::Full) => "The log says more.",
        Some(Why::RtkitRefused | Why::RtkitFailed) => {
            "Join the realtime or audio group (where it has an rtprio limit), then log in again and restart the CA-72."
        }
        _ => {
            "Install rtkit, or join the realtime or audio group (where it has an rtprio limit); then log in again and restart the CA-72."
        }
    };
    let mut who = whom.join(" and ");
    if let Some(c) = who.get(..1) {
        who = c.to_uppercase() + &who[1..];
    }
    Some(Warning {
        tip,
        row: "NOT REAL-TIME: SEE AUDIO & MIDI…",
        detail: format!("{who} at ordinary priority ({reason})."),
        remedy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ca72_rt::Why;

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
            if let Some(Ok(p)) = r {
                // (RTKit is not used in the tests: `prepare`.)
                assert_eq!(p, ca72_rt::Promoted::Now);
                assert!(ca72_rt::is_realtime(), "said promoted, but ordinary");
                ca72_rt::demote();
            }
            assert!(a.begin().is_none(), "promoted at every block");
        })
        .join()
        .unwrap();
    }

    fn stood(
        direct: u32,
        rtkit: u32,
        waiting: u32,
        ordinary: u32,
        why: Option<Why>,
    ) -> ca72_rt::Standing {
        ca72_rt::Standing {
            direct,
            rtkit,
            waiting,
            ordinary,
            why,
        }
    }

    /// The warning says which threads are ordinary, why and the fix, on Linux; nothing while
    /// RTKit has yet to answer or every thread is real-time; on macOS and Windows only a worker
    /// refused, without Linux's fix.
    #[test]
    fn the_warning_says_which_threads_why_and_the_fix() {
        let none = ca72_rt::Standing::default();
        let rt = stood(1, 0, 0, 0, None);
        assert_eq!(warning(none, none, true), None);
        assert_eq!(warning(rt, stood(6, 0, 0, 0, None), true), None);
        assert_eq!(
            summary(rt, stood(0, 6, 0, 0, None)),
            Standing::RealTime { rtkit: true }
        );
        // Asking: nothing yet.
        let asking = stood(0, 0, 1, 0, None);
        assert_eq!(summary(asking, none), Standing::Asking);
        assert_eq!(
            warning(asking, stood(0, 0, 0, 6, Some(Why::NoRtkit)), true),
            None
        );
        // Both, RTKit absent.
        let gone = stood(0, 0, 0, 1, Some(Why::NoRtkit));
        let w = warning(gone, stood(0, 0, 0, 6, Some(Why::NoRtkit)), true).expect("a warning");
        assert_eq!(
            w.tip,
            "AUDIO NOT REAL-TIME: MAY DROP OUT · INSTALL RTKIT OR JOIN THE REALTIME OR AUDIO GROUP, THEN LOG IN AGAIN"
        );
        assert_eq!(
            (w.detail.as_str(), w.remedy),
            (
                "The audio thread and POLY's workers at ordinary priority (no rtprio allowance; RTKit not running).",
                "Install rtkit, or join the realtime or audio group (where it has an rtprio limit); then log in again and restart the CA-72."
            )
        );
        // The audio thread alone, RTKit refusing it.
        let w = warning(
            stood(0, 0, 0, 1, Some(Why::RtkitRefused)),
            stood(6, 0, 0, 0, None),
            true,
        )
        .expect("a warning");
        assert!(w.tip.starts_with("AUDIO THREAD NOT REAL-TIME"), "{}", w.tip);
        assert!(
            w.tip
                .ends_with("RTKIT REFUSED: JOIN THE REALTIME OR AUDIO GROUP, THEN LOG IN AGAIN"),
            "{}",
            w.tip
        );
        // Some workers, RTKit's limits reached part way.
        let w = warning(rt, stood(0, 4, 0, 2, Some(Why::RtkitRefused)), true).expect("a warning");
        assert!(
            w.tip.starts_with("POLY'S THREADS NOT REAL-TIME"),
            "{}",
            w.tip
        );
        assert!(
            w.detail.starts_with(
                "2 of POLY's 6 workers at ordinary priority (no rtprio allowance; RTKit refused)"
            ),
            "{}",
            w.detail
        );
        // macOS and Windows: no audio thread asked; a worker refused, no Linux fix.
        let w = warning(none, stood(5, 0, 0, 1, Some(Why::Refused)), false).expect("a warning");
        assert_eq!(
            w.tip,
            "POLY'S THREADS NOT REAL-TIME: MAY DROP OUT · THE SYSTEM REFUSED"
        );
        assert!(w.detail.ends_with("(the system refused)."), "{}", w.detail);
        assert_eq!(w.remedy, "The log says more.");
        assert_eq!(warning(none, stood(6, 0, 0, 0, None), false), None);
    }
}
