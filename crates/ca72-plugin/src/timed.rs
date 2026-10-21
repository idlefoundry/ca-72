//! Oscillator 3 timed (decisions.md R-OSC-TIMED): at RANGE's LO, its cycle a note value of the
//! host's tempo (the standalone's TEMPO), counted from the host's place while its transport
//! plays, so that it never drifts from the host (the owner, 2026-10-20: "it can't drift. It
//! has to stay synced to the host").
//!
//! - **The clock**: where the music is as each block begins, in quarter notes, and how many a
//!   sample: the arpeggiator's ([`crate::arp::Arp::position`]), so the two always agree. While
//!   the host plays and says where it is, that is the host's place, taken afresh every block
//!   (the place a block's samples are at worked out from it, never added up from block to
//!   block): a tempo changed, a loop or a locate is followed as the host has it. Stopped, and
//!   in the standalone, it runs on from where it was at the tempo.
//! - **The cycle**: a note value counts quarter notes from the song's start; a bar value counts
//!   the host's bars (its bar number and where its bar began, at its time signature), else
//!   four quarter notes a bar from the start. Its phase is the count's fraction: 0, where
//!   oscillator 3's ramp resets, on the beat or bar that starts a cycle.
//! - **Where it stops applying**: off, or RANGE not at LO, nothing here reaches the voices.
//!
//! Restarting the cycle with each note (a later option, not built) would be a phase taken from
//! each voice's own attack instead of from [`Position`].

use nih_plug::prelude::Transport;

/// A timed cycle's length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    /// So many of the host's bars.
    Bars(f64),
    /// So many quarter notes.
    Beats(f64),
}

/// The divisions, slow to fast as the dial turns (the arpeggiator's RATE's order, R-ARP): 4,
/// 2 and 1 bars, then each straight note value with its dotted and its triplet between it and
/// the next (the dotted the longer), the host's text and the length. At 120 BPM, 0.125 to
/// 16 Hz: LO's travel (0.21 to 11.4 Hz at FREQUENCY's ends with OSC. 3 CONTROL off) with a
/// little under an octave beyond it each way.
pub const DIVISIONS: [(&str, Length); 18] = [
    ("4 bars", Length::Bars(4.0)),
    ("2 bars", Length::Bars(2.0)),
    ("1 bar", Length::Bars(1.0)),
    ("1/2 dotted", Length::Beats(3.0)),
    ("1/1 triplet", Length::Beats(8.0 / 3.0)),
    ("1/2", Length::Beats(2.0)),
    ("1/4 dotted", Length::Beats(1.5)),
    ("1/2 triplet", Length::Beats(4.0 / 3.0)),
    ("1/4", Length::Beats(1.0)),
    ("1/8 dotted", Length::Beats(0.75)),
    ("1/4 triplet", Length::Beats(2.0 / 3.0)),
    ("1/8", Length::Beats(0.5)),
    ("1/16 dotted", Length::Beats(0.375)),
    ("1/8 triplet", Length::Beats(1.0 / 3.0)),
    ("1/16", Length::Beats(0.25)),
    ("1/32 dotted", Length::Beats(0.1875)),
    ("1/16 triplet", Length::Beats(1.0 / 6.0)),
    ("1/32", Length::Beats(0.125)),
];

/// The division a new patch has, and one saved before timed mode: a quarter note (2 Hz at
/// 120 BPM, where FREQUENCY's centre puts LO: 1.89 Hz).
pub const DEFAULT_DIVISION: usize = 8;

/// The tempo where the host gives none and the standalone's TEMPO is not set: 120 BPM (the
/// arpeggiator's, `params::DEFAULT_TEMPO`).
pub const FALLBACK_TEMPO: f64 = crate::params::DEFAULT_TEMPO;

/// Where the music is as a block begins.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    /// Quarter notes from the song's start.
    pub beats: f64,
    /// Bars from the song's start (fractional).
    pub bars: f64,
    /// Quarter notes a sample.
    pub per: f64,
    /// Quarter notes a bar.
    pub bar: f64,
}

impl Position {
    /// At the song's start, stopped, at `tempo` (BPM) and `rate` samples a second, four
    /// quarter notes a bar.
    pub fn start(tempo: f64, rate: f64) -> Position {
        Position {
            beats: 0.0,
            bars: 0.0,
            per: per_sample(tempo, rate),
            bar: 4.0,
        }
    }

    /// A cycle of `length` from here: its phase now (0 to 1) and the cycles a sample.
    pub fn cycle(&self, length: Length) -> (f64, f64) {
        let (count, per) = match length {
            Length::Beats(q) => (self.beats / q, self.per / q),
            Length::Bars(n) => (self.bars / n, self.per / (self.bar * n)),
        };
        (count - count.floor(), per)
    }
}

/// Quarter notes a sample at `tempo` (BPM; [`FALLBACK_TEMPO`] where it is not a tempo) and
/// `rate` samples a second.
pub fn per_sample(tempo: f64, rate: f64) -> f64 {
    let tempo = if tempo.is_finite() && tempo > 0.0 {
        tempo
    } else {
        FALLBACK_TEMPO
    };
    let rate = if rate.is_finite() && rate > 0.0 {
        rate
    } else {
        48_000.0
    };
    tempo / 60.0 / rate
}

/// The clock's bars between blocks: where the next block's start is while the host does not
/// say (stopped, or no host).
#[derive(Clone, Copy, Debug, Default)]
pub struct Clock {
    next_bars: Option<f64>,
    /// The block's place as last given ([`Clock::begin`], [`Clock::retime`]).
    last: Option<Position>,
}

impl Clock {
    /// A block of `len` samples begins: the host's transport `t`, and the arpeggiator's clock,
    /// `beats` (the host's place while it plays, else run on) at `per` quarter notes a sample.
    pub fn begin(&mut self, t: &Transport, beats: f64, per: f64, len: usize) -> Position {
        let bar = match (t.time_sig_numerator, t.time_sig_denominator) {
            (Some(n), Some(d)) if n > 0 && d > 0 => f64::from(n) * 4.0 / f64::from(d),
            _ => 4.0,
        };
        // (As the arpeggiator takes the host's place: while it plays, if it says.)
        let host = t.playing && t.pos_beats().is_some_and(f64::is_finite);
        let bars = if host {
            match (t.bar_number(), t.bar_start_pos_beats()) {
                (Some(n), Some(start)) if start.is_finite() => f64::from(n) + (beats - start) / bar,
                _ => beats / bar,
            }
        } else {
            self.next_bars.unwrap_or(beats / bar)
        };
        self.next_bars = Some(bars + len as f64 * per / bar);
        let p = Position {
            beats,
            bars,
            per,
            bar,
        };
        self.last = Some(p);
        p
    }

    /// The tempo changed `at` samples into the block of `len` (the standalone's TEMPO moved by a
    /// modulation source: decisions.md R-LFO), to `per` quarter notes a sample: the block's
    /// place as the samples from here have it, the beats and bars going on from where they had
    /// come at the new tempo (the block's start as it would have been, so that a sample's place
    /// is still the block's start's and its offset), as the arpeggiator's
    /// ([`crate::arp::Arp::retime`]). None before a block has begun.
    pub fn retime(&mut self, at: usize, per: f64, len: usize) -> Option<Position> {
        let p = self.last?;
        let moved = at as f64 * (p.per - per);
        let q = Position {
            beats: p.beats + moved,
            bars: p.bars + moved / p.bar,
            per,
            bar: p.bar,
        };
        self.next_bars = Some(q.bars + len as f64 * per / q.bar);
        self.last = Some(q);
        Some(q)
    }

    /// Forgets the bars run on (a reset): the next block's counted from its beats.
    pub fn clear(&mut self) {
        self.next_bars = None;
        self.last = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The divisions are slow to fast, and each length is what its name says.
    #[test]
    fn the_divisions_run_slow_to_fast() {
        let beats = |l: Length| match l {
            Length::Bars(n) => 4.0 * n,
            Length::Beats(q) => q,
        };
        for w in DIVISIONS.windows(2) {
            assert!(beats(w[0].1) > beats(w[1].1), "{} {}", w[0].0, w[1].0);
        }
        assert_eq!(DIVISIONS[DEFAULT_DIVISION].0, "1/4");
        for (name, l) in DIVISIONS {
            let q = beats(l);
            let expected = match name {
                "4 bars" => 16.0,
                "2 bars" => 8.0,
                "1 bar" => 4.0,
                _ => {
                    let (value, kind) = name.split_once(' ').unwrap_or((name, ""));
                    let d: f64 = value.trim_start_matches("1/").parse().unwrap();
                    4.0 / d
                        * match kind {
                            "dotted" => 1.5,
                            "triplet" => 2.0 / 3.0,
                            _ => 1.0,
                        }
                }
            };
            assert!((q - expected).abs() < 1e-12, "{name}");
        }
    }

    /// A bar value counts bars of the time signature; a note value quarter notes.
    #[test]
    fn bars_follow_the_time_signature() {
        let p = Position {
            beats: 7.5,
            bars: 2.5,
            per: 0.001,
            bar: 3.0,
        };
        let (phase, per) = p.cycle(Length::Bars(1.0));
        assert!((phase - 0.5).abs() < 1e-12 && (per - 0.001 / 3.0).abs() < 1e-15);
        let (phase, _) = p.cycle(Length::Bars(2.0));
        assert!((phase - 0.25).abs() < 1e-12);
        let (phase, per) = p.cycle(Length::Beats(2.0));
        assert!((phase - 0.75).abs() < 1e-12 && (per - 0.0005).abs() < 1e-15);
    }
}

/// Through the plug-in as a host plays it: the clock every block, the voices' oscillator 3 on
/// it at each QUALITY, and what timed mode leaves alone.
#[cfg(test)]
mod played {
    use super::*;
    use crate::Ca72;
    use crate::params::{Ca72Params, Division, Footage, QualityMode};
    use crate::tests::{block, initialised_with};
    use nih_plug::context::process::TestProcessContext;
    use nih_plug::prelude::*;
    use std::sync::Arc;

    /// Oscillator `n` (1 or 2: oscillator 2 or 3) heard, its TIMED `on` at `division`, its RANGE at `range`, at
    /// `quality`.
    fn timed_osc(
        n: usize,
        on: bool,
        range: Footage,
        division: Division,
        quality: QualityMode,
    ) -> Ca72Params {
        let d = Ca72Params {
            quality: EnumParam::new("Quality", quality),
            ..Ca72Params::default()
        };
        let (t, dv, r, o) = (
            BoolParam::new("Timed", on),
            EnumParam::new("Division", division),
            EnumParam::new("Range", range),
            BoolParam::new("On", true),
        );
        match n {
            1 => Ca72Params {
                osc2_timed: t,
                osc2_division: dv,
                osc2_range: r,
                osc2_on: o,
                ..d
            },
            _ => Ca72Params {
                osc3_timed: t,
                osc3_division: dv,
                osc3_range: r,
                osc3_on: o,
                ..d
            },
        }
    }

    /// Oscillator 3 timed at `division`, RANGE at LO, a sawtooth heard, at `quality`.
    fn timed(division: Division, quality: QualityMode) -> Ca72Params {
        timed_osc(2, true, Footage::Lo, division, quality)
    }

    fn context(p: &Ca72, rate: f32, mode: ProcessMode) -> TestProcessContext<Ca72> {
        let params: Arc<dyn Params> = p.params.clone();
        TestProcessContext::new(params, rate, mode)
    }

    fn key(note: u8) -> NoteEvent<()> {
        NoteEvent::NoteOn {
            timing: 0,
            voice_id: None,
            channel: 0,
            note,
            velocity: 1.0,
        }
    }

    /// Oscillator 3's resets ([`resets_of`]).
    fn resets(
        p: &mut Ca72,
        c: &mut TestProcessContext<Ca72>,
        len: usize,
        blocks: usize,
        host: impl Fn(usize, &mut TestProcessContext<Ca72>),
    ) -> Vec<f64> {
        resets_of(2, p, c, len, blocks, host)
    }

    /// `blocks` blocks of `len` with a key held from the first, `host(b, c)` setting the host's
    /// transport for block `b`: each of oscillator `n`'s resets, host samples from the first
    /// block's start (the probe read after each block: a block holds one at most here).
    fn resets_of(
        n: usize,
        p: &mut Ca72,
        c: &mut TestProcessContext<Ca72>,
        len: usize,
        blocks: usize,
        host: impl Fn(usize, &mut TestProcessContext<Ca72>),
    ) -> Vec<f64> {
        c.push_event(key(60));
        let mut out = Vec::new();
        // (Counted from here: the voices may have played before, as they are made.)
        let mut seen = p.engine.osc_reset(n).map_or(0, |r| r.1);
        for b in 0..blocks {
            host(b, c);
            let y = block(p, c, len);
            assert!(y.iter().all(|v| v.is_finite()));
            let (ago, count) = p.engine.osc_reset(n).unwrap();
            assert!(count <= seen + 1, "block {b}: two resets in a block");
            if count > seen {
                seen = count;
                out.push(((b + 1) * len) as f64 - 1.0 - ago);
            }
        }
        out
    }

    /// The host's place at sample `s`, quarter notes: playing from `from` at `bpm`, 48 kHz.
    fn host_beats(from: f64, bpm: f64, s: f64) -> f64 {
        from + s * bpm / 60.0 / 48_000.0
    }

    /// Each reset against where the host's cycles of `q` quarter notes start (the host at
    /// `beats(s)`, found by bisection): the largest error, samples.
    fn worst(resets: &[f64], q: f64, beats: impl Fn(f64) -> f64) -> f64 {
        resets
            .iter()
            .map(|&r| {
                let m = (beats(r) / q).round();
                let (mut lo, mut hi) = (r - 100.0, r + 100.0);
                for _ in 0..80 {
                    let mid = 0.5 * (lo + hi);
                    if beats(mid) / q < m {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                (r - lo).abs()
            })
            .fold(0.0, f64::max)
    }

    /// Ten minutes of the host playing at 123.7 BPM: every block, oscillator 3's place on the
    /// clock is the host's to within a billionth of a cycle at its first sample and its last,
    /// as at the start, nothing added up (the owner, 2026-10-20: "it can't drift"); stopped,
    /// ten minutes run on at the tempo end where the tempo puts them.
    #[test]
    fn ten_minutes_at_an_odd_tempo_are_the_hosts_to_the_end() {
        const LEN: usize = 256;
        let blocks = (600.0 * 48_000.0 / LEN as f64) as usize;
        for division in [
            Division::EighthTriplet,
            Division::TwoBars,
            Division::QuarterDotted,
        ] {
            let mut p = initialised_with(
                timed(division, QualityMode::Lo),
                48_000.0,
                ProcessMode::Realtime,
            );
            let mut c = context(&p, 48_000.0, ProcessMode::Realtime);
            let length = DIVISIONS[division.to_index()].1;
            let count = |s: f64| {
                let beats = host_beats(0.0, 123.7, s);
                match length {
                    Length::Beats(q) => beats / q,
                    Length::Bars(n) => beats / 4.0 / n,
                }
            };
            let mut most = 0.0f64;
            let silence = vec![0.0f32; LEN];
            for b in 0..blocks {
                let s = (b * LEN) as f64;
                c.set_transport(
                    true,
                    Some(123.7),
                    Some(host_beats(0.0, 123.7, s)),
                    Some((4, 4)),
                );
                assert_eq!(block(&mut p, &mut c, LEN), silence);
                for k in [0, LEN - 1] {
                    let t = p.engine.timed_probe(k as f64)[2].unwrap();
                    let exact = count(s + k as f64);
                    let mut e = t.phase - (exact - exact.floor());
                    e -= e.round();
                    most = most.max(e.abs());
                }
            }
            assert!(most < 1e-9, "{division:?}: {most} cycles");
            // Stopped: run on from the start at the tempo for ten minutes.
            let mut p = initialised_with(
                timed(division, QualityMode::Lo),
                48_000.0,
                ProcessMode::Realtime,
            );
            let mut c = context(&p, 48_000.0, ProcessMode::Realtime);
            for _ in 0..blocks {
                c.set_transport(false, Some(123.7), None, Some((4, 4)));
                block(&mut p, &mut c, LEN);
            }
            // (The last block's start.)
            let t = p.engine.timed_probe(0.0)[2].unwrap();
            let exact = count(((blocks - 1) * LEN) as f64);
            let mut e = t.phase - (exact - exact.floor());
            e -= e.round();
            assert!(e.abs() < 1e-6, "{division:?} stopped: {e} cycles");
        }
    }

    /// Every division, playing from an odd place in 3/4: the clock's phase is the host's count
    /// of its note value, or of its bars (three quarter notes each), and its rate the tempo's.
    #[test]
    fn every_division_counts_the_hosts_beats_or_bars() {
        for (i, (name, length)) in DIVISIONS.iter().enumerate() {
            let division = Division::from_index(i);
            let mut p = initialised_with(
                timed(division, QualityMode::Lo),
                48_000.0,
                ProcessMode::Realtime,
            );
            let mut c = context(&p, 48_000.0, ProcessMode::Realtime);
            let from = 37.3;
            for b in 0..20 {
                let s = (b * 256) as f64;
                let beats = host_beats(from, 97.0, s);
                c.set_transport(true, Some(97.0), Some(beats), Some((3, 4)));
                block(&mut p, &mut c, 256);
                let t = p.engine.timed_probe(100.0)[2].unwrap();
                let at = host_beats(from, 97.0, s + 100.0);
                let (count, per) = match *length {
                    Length::Beats(q) => (at / q, 97.0 / 60.0 / 48_000.0 / q),
                    // (nih-plug's bar count from the place and the signature: bar 12 starts at
                    // quarter note 36.)
                    Length::Bars(n) => (at / 3.0 / n, 97.0 / 60.0 / 48_000.0 / 3.0 / n),
                };
                let mut e = t.phase - (count - count.floor());
                e -= e.round();
                assert!(e.abs() < 1e-9, "{name}, block {b}: {e}");
                assert!(
                    (t.per / per - 1.0).abs() < 1e-12,
                    "{name}: {} against {per}",
                    t.per
                );
            }
        }
    }

    /// At LO, HI and ULTRA, oscillators 2's and 3's resets fall where the host's cycles start (a
    /// thirty-second at 123.7 BPM, from an odd place), the circuit's within a thousandth of a
    /// sample and the light voice's within a tenth; ENTROPY (at its default) moves them not
    /// at all.
    #[test]
    fn its_resets_are_on_the_hosts_grid_at_every_quality() {
        // (ULTRA's split crews draw on the process's workers: one such test at a time.)
        let _one = crate::tests::BUDGET_TESTS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (n, quality) in (1..3)
            .flat_map(|n| [QualityMode::Lo, QualityMode::Hi, QualityMode::Ultra].map(|q| (n, q)))
        {
            let mut p = initialised_with(
                timed_osc(n, true, Footage::Lo, Division::ThirtySecond, quality),
                48_000.0,
                ProcessMode::Realtime,
            );
            let mut c = context(&p, 48_000.0, ProcessMode::Realtime);
            let beats = |s: f64| host_beats(5.3, 123.7, s);
            let rs = resets_of(n, &mut p, &mut c, 128, 200, |b, c| {
                c.set_transport(
                    true,
                    Some(123.7),
                    Some(beats((b * 128) as f64)),
                    Some((4, 4)),
                );
            });
            // (The first is the oscillator taking its place on the clock.)
            let rs = &rs[1..];
            assert!(rs.len() >= 7, "oscillator {}, {quality:?}: {rs:?}", n + 1);
            let limit = if quality == QualityMode::Lo {
                0.1
            } else {
                1e-3
            };
            let w = worst(rs, 0.125, beats);
            assert!(w < limit, "oscillator {}, {quality:?}: {w} samples", n + 1);
        }
    }

    /// The transport stopped: oscillator 3 runs on at the tempo from where it was, its cycles
    /// a whole number of the tempo's apart; started again elsewhere, it takes the host's place
    /// at once and is on its grid from its next reset. A loop (the host's place back) the
    /// same.
    #[test]
    fn it_runs_on_when_stopped_and_takes_the_hosts_place_again() {
        for quality in [QualityMode::Lo, QualityMode::Hi] {
            let mut p = initialised_with(
                timed(Division::ThirtySecond, quality),
                48_000.0,
                ProcessMode::Realtime,
            );
            let mut c = context(&p, 48_000.0, ProcessMode::Realtime);
            // 120 BPM: a thirty-second is 3000 samples. Playing from 0 for 300 blocks of 128, stopped
            // for 300, then playing from beat 100.37 for 300, then looping back to beat 4.
            let host = |b: usize| -> Option<f64> {
                let s = |from: usize| ((b - from) * 128) as f64;
                match b {
                    0..300 => Some(host_beats(0.0, 120.0, s(0))),
                    300..600 => None,
                    600..900 => Some(host_beats(100.37, 120.0, s(600))),
                    _ => Some(host_beats(4.0, 120.0, s(900))),
                }
            };
            let rs = resets(&mut p, &mut c, 128, 1200, |b, c| {
                c.set_transport(host(b).is_some(), Some(120.0), host(b), Some((4, 4)));
            });
            let limit = if quality == QualityMode::Lo {
                0.1
            } else {
                1e-3
            };
            let within = |from: usize, to: usize| -> Vec<f64> {
                rs.iter()
                    .copied()
                    .filter(|&r| r > (from * 128) as f64 && r < (to * 128) as f64)
                    .collect()
            };
            // Playing: on the grid from the start.
            let w = worst(&within(1, 300), 0.125, |s| host_beats(0.0, 120.0, s));
            assert!(w < limit, "{quality:?} playing: {w}");
            // Stopped: the grid run on, at the tempo (the clock from where it was).
            let w = worst(&within(300, 600), 0.125, |s| host_beats(0.0, 120.0, s));
            assert!(w < limit, "{quality:?} stopped: {w}");
            // Started elsewhere, then looped: the host's grid again from its first reset there.
            for (from, to, at) in [(600, 900, 100.37), (900, 1200, 4.0)] {
                let start = (from * 128) as f64;
                let got = within(from, to);
                assert!(got.len() >= 8, "{quality:?}: {got:?}");
                // (Its first reset there may be the jump's own, its ramp put at once.)
                let w = worst(&got[1..], 0.125, |s| at + (s - start) * 2.0 / 48_000.0);
                assert!(w < limit, "{quality:?} from {at}: {w}");
            }
        }
    }

    /// The tempo changed while a note plays, the host's place following it: the resets on the
    /// host's grid as it now falls.
    #[test]
    fn a_tempo_change_mid_note_is_followed() {
        let mut p = initialised_with(
            timed(Division::Sixteenth, QualityMode::Hi),
            48_000.0,
            ProcessMode::Realtime,
        );
        let mut c = context(&p, 48_000.0, ProcessMode::Realtime);
        // 100 BPM for 250 blocks of 128, then 151.3 from where that left the host.
        let turn = 250.0 * 128.0;
        let beats = move |s: f64| {
            if s < turn {
                host_beats(0.0, 100.0, s)
            } else {
                host_beats(host_beats(0.0, 100.0, turn), 151.3, s - turn)
            }
        };
        let rs = resets(&mut p, &mut c, 128, 600, |b, c| {
            let s = (b * 128) as f64;
            let bpm = if s < turn { 100.0 } else { 151.3 };
            c.set_transport(true, Some(bpm), Some(beats(s)), Some((4, 4)));
        });
        assert!(rs.len() > 10);
        let w = worst(&rs[1..], 0.25, beats);
        assert!(w < 1e-3, "{w} samples");
    }

    /// At any other RANGE, an oscillator's TIMED does nothing: the samples are those with it
    /// off, to the bit, for each oscillator at HI and LO.
    #[test]
    fn at_any_other_range_timed_does_nothing() {
        for n in 1..3 {
            for quality in [QualityMode::Hi, QualityMode::Lo] {
                for range in [Footage::R32, Footage::R8, Footage::R2] {
                    let render = |on: bool| {
                        let params = timed_osc(n, on, range, Division::Sixteenth, quality);
                        let mut p = initialised_with(params, 48_000.0, ProcessMode::Realtime);
                        p.params
                            .noise_seed
                            .store(7, std::sync::atomic::Ordering::Relaxed);
                        p.engine.prepare(48_000.0, p.params.seed());
                        let mut c = context(&p, 48_000.0, ProcessMode::Realtime);
                        c.push_event(key(57));
                        (0..40)
                            .flat_map(|b| {
                                c.set_transport(true, Some(120.0), Some(b as f64 * 0.0053), None);
                                block(&mut p, &mut c, 128)
                            })
                            .collect::<Vec<f32>>()
                    };
                    assert!(render(true) == render(false), "{n} {quality:?} {range:?}");
                }
            }
        }
    }

    /// The key played (each oscillator follows the keys; oscillator 3 with OSC. 3 CONTROL on)
    /// and the PITCH wheel move a timed oscillator not at all: its resets are the same wherever.
    #[test]
    fn the_keys_and_the_wheel_leave_it_on_its_clock() {
        for n in 1..3 {
            let render = |note: u8, bend: f32| {
                let params = Ca72Params {
                    osc3_control: BoolParam::new("Osc 3 Control", true),
                    ..timed_osc(n, true, Footage::Lo, Division::Sixteenth, QualityMode::Hi)
                };
                let mut p = initialised_with(params, 48_000.0, ProcessMode::Realtime);
                let mut c = context(&p, 48_000.0, ProcessMode::Realtime);
                c.push_event(NoteEvent::MidiPitchBend {
                    timing: 0,
                    channel: 0,
                    value: bend,
                });
                c.push_event(key(note));
                let mut out = Vec::new();
                let mut seen = p.engine.osc_reset(n).map_or(0, |r| r.1);
                for b in 0..200 {
                    let at = host_beats(1.1, 133.0, (b * 128) as f64);
                    c.set_transport(true, Some(133.0), Some(at), None);
                    block(&mut p, &mut c, 128);
                    let (ago, count) = p.engine.osc_reset(n).unwrap();
                    if count > seen {
                        seen = count;
                        out.push(((b + 1) * 128) as f64 - 1.0 - ago);
                    }
                }
                out
            };
            let low = render(36, 0.5);
            assert!(low.len() > 3, "{n}");
            for (note, bend) in [(84, 0.5), (60, 1.0), (60, 0.0)] {
                let other = render(note, bend);
                assert_eq!(other.len(), low.len(), "{n}");
                for (a, b) in low.iter().zip(&other).skip(1) {
                    assert!(
                        (a - b).abs() < 1e-6,
                        "{n}: key {note}, bend {bend}: {a} against {b}"
                    );
                }
            }
        }
    }

    /// At 96 kHz and at 44.1, rendered offline as in real time, a host giving no tempo (the
    /// standalone's TEMPO then, 120 BPM by default) and the extremes of tempo: on the grid.
    #[test]
    fn rates_offline_no_tempo_and_extreme_tempos() {
        // (ULTRA's split crews draw on the process's workers: one such test at a time.)
        let _one = crate::tests::BUDGET_TESTS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // At 96 and 44.1 kHz, the host's place in its own samples.
        for rate in [96_000.0f32, 44_100.0] {
            let mut p = initialised_with(
                timed(Division::ThirtySecond, QualityMode::Hi),
                rate,
                ProcessMode::Realtime,
            );
            let mut c = context(&p, rate, ProcessMode::Realtime);
            let beats = move |s: f64| 2.2 + s * 140.0 / 60.0 / f64::from(rate);
            let rs = resets(&mut p, &mut c, 256, 160, |b, c| {
                c.set_transport(
                    true,
                    Some(140.0),
                    Some(beats((b * 256) as f64)),
                    Some((4, 4)),
                );
            });
            assert!(rs.len() > 5);
            let w = worst(&rs[1..], 0.125, beats);
            assert!(w < 1e-3, "{rate} Hz: {w}");
        }
        // Offline: the same samples as in real time.
        let render = |mode: ProcessMode| {
            let mut p = initialised_with(
                timed(Division::SixteenthTriplet, QualityMode::Lo),
                48_000.0,
                mode,
            );
            p.params
                .noise_seed
                .store(3, std::sync::atomic::Ordering::Relaxed);
            p.engine.prepare(48_000.0, p.params.seed());
            let mut c = context(&p, 48_000.0, mode);
            c.push_event(key(48));
            (0..100)
                .flat_map(|b| {
                    c.set_transport(
                        true,
                        Some(111.0),
                        Some(host_beats(0.5, 111.0, (b * 256) as f64)),
                        None,
                    );
                    block(&mut p, &mut c, 256)
                })
                .collect::<Vec<f32>>()
        };
        assert!(render(ProcessMode::Offline) == render(ProcessMode::Realtime));
        // No tempo, stopped: 120 BPM, a sixteenth 6000 samples.
        let mut p = initialised_with(
            timed(Division::Sixteenth, QualityMode::Hi),
            48_000.0,
            ProcessMode::Realtime,
        );
        let mut c = context(&p, 48_000.0, ProcessMode::Realtime);
        let rs = resets(&mut p, &mut c, 128, 300, |_, c| {
            c.set_transport(false, None, None, None)
        });
        for w in rs[1..].windows(2) {
            assert!((w[1] - w[0] - 6000.0).abs() < 1e-3, "{w:?}");
        }
        // 1000 BPM (the most the clock takes), a thirty-second: 360 samples a cycle; and 1 BPM's
        // four bars, finite and quiet about it.
        let mut p = initialised_with(
            timed(Division::ThirtySecond, QualityMode::Hi),
            48_000.0,
            ProcessMode::Realtime,
        );
        let mut c = context(&p, 48_000.0, ProcessMode::Realtime);
        let beats = |s: f64| s * 1000.0 / 60.0 / 48_000.0;
        let rs = resets(&mut p, &mut c, 128, 40, |b, c| {
            c.set_transport(true, Some(1000.0), Some(beats((b * 128) as f64)), None);
        });
        assert!(rs.len() > 10);
        assert!(worst(&rs[1..], 0.125, beats) < 1e-3);
        let mut p = initialised_with(
            timed(Division::FourBars, QualityMode::Ultra),
            48_000.0,
            ProcessMode::Realtime,
        );
        let mut c = context(&p, 48_000.0, ProcessMode::Realtime);
        let rs = resets(&mut p, &mut c, 128, 40, |b, c| {
            c.set_transport(
                true,
                Some(1.0),
                Some((b * 128) as f64 / 60.0 / 48_000.0),
                None,
            );
        });
        assert!(rs.len() <= 1);
    }
}
