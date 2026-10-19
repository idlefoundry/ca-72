//! The arpeggiator (decisions.md R-ARP): the keys held played one at a time, in turn, on a grid
//! of the host's tempo, through the same path a key played takes (the engine's `Event::Note`),
//! so that MONO, POLY, UNISON, NOTE PRIORITY, RETRIGGER and GLIDE take its notes as they take
//! the player's. Off, every key goes straight through, as it always has; but with HOLD on
//! (the parameter `arp_latch`, once LATCH), the keys played are held as played, a drone, until
//! HOLD goes off or a new chord is played (the owner, 2026-10-19).
//!
//! - **The keys it plays**: those held (and, with the sustain pedal down, those let go since it
//!   went down), or with HOLD the chord last played, kept after its keys are let go until a
//!   key is pressed again with none held (a new chord) or HOLD goes off. OCTAVES repeats them
//!   an octave up each time; a pitch two keys reach is played once (but in ORDER). A note an
//!   octave would put past MIDI's 127 plays in the highest octave it has instead (the owner:
//!   "repeat the highest octave available of the selected note rather than go silent"): the
//!   pattern is the one it would be with room above, each of its notes brought down by whole
//!   octaves into range as it is played.
//! - **The order**: UP, the next higher pitch than the last played (from the lowest again past
//!   the highest); DOWN, the next lower; UP/DOWN, up to the highest and down to the lowest, the
//!   ends played once each; RANDOM, any of them, but never the note last played again while
//!   there is another; ORDER, the keys in the order they were pressed (one pressed while others
//!   are held goes on the end, one let go leaves it), then again an octave up for each of
//!   OCTAVES. A chord changed while it plays carries on from the note it last played; started
//!   again from silence, it starts from the pattern's beginning.
//! - **The grid**: a step every RATE (a note value in quarter notes; dotted and triplet ones
//!   between), every second one late by SWING (at 100 % three quarters of the way through the
//!   pair, at 67 % a triplet's shuffle). While the host plays, the grid is the host's bars: the
//!   steps counted from each bar's start, the last cut short at the bar's end; a key pressed up
//!   to a quarter step late plays at once, as that step's. Stopped (and in the standalone), it
//!   runs at the host's tempo (the standalone's TEMPO) from the moment a key is pressed.
//!   A jump of the host's position (a loop, a locate) finds the grid again there.
//! - **GATE**: a share of what the circuit can play in a step (the owner, 2026-10-19): each note
//!   held at least [`SHORTEST`], so that its contours start (the trigger closes 7.8 ms after a
//!   key goes down), and let go at least [`REARM`] before the next, so that they start again
//!   (the trigger re-arms once the keys have been up about 12 ms); GATE sets where between the
//!   two the note is let go. A step too short for both keeps the note's and shortens the gap; one
//!   shorter than the note itself holds it over the next's start (the owner: "the gate in the arp
//!   should NEVER go below the minimum gate length to trigger a sound in the device"), the two
//!   overlapping until it has had its [`SHORTEST`]. At 100 % tied, the next note pressed before
//!   the last is let go (legato: single triggering and GLIDE take it so). The same key again
//!   (one key held, a pattern's repeat, a note folded down from past 127) is let go and pressed
//!   again once it has been held [`SHORTEST`]; before that it cannot be pressed twice, so the
//!   step is tied into it, the note held on to that step's release. Letting the keys go (or
//!   HOLD off) lets a note go at once only once it has had its [`SHORTEST`]; switching ON off,
//!   All Notes Off and a reset let every note go at once.
//!
//! Nothing here allocates: the keys are bit sets and a list of 128, the times beats (quarter
//! notes) turned into the block's samples as each block begins.

/// The sustain pedal's control change.
pub const SUSTAIN: u8 = 64;

/// The shortest note it plays, seconds (GATE at 0 %): the trigger's 7.8 ms from a key down to
/// the contours' start (`ca72::light::laws::TRIGGER_ON`, the circuit's) and room for them to be
/// heard.
pub const SHORTEST: f64 = 0.020;

/// The shortest gap it leaves before the next note but at GATE 100 %, seconds: the keys up long
/// enough for the trigger to start the contours again (`ca72::voice::RETRIGGER_GAP`).
pub const REARM: f64 = 0.013;

/// OCTAVES' most.
pub const MOST_OCTAVES: u8 = 5;

/// A key pressed this share of a step after its time plays at once, as that step's.
const GRACE: f64 = 0.25;

/// Beats close enough to be one time (rounding in the host's positions and ours).
const EPS: f64 = 1e-6;

/// How the keys are played in turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    Up,
    Down,
    UpDown,
    Random,
    /// As the keys were pressed.
    Order,
}

/// RATE's note values, slow to fast, as the dial has them: each straight value, then its dotted
/// and its triplet between it and the next (the dotted the longer), the readout's text and the
/// step in quarter notes.
pub const RATES: [(&str, f64); 13] = [
    ("2", 2.0),
    ("4.", 1.5),
    ("2t", 4.0 / 3.0),
    ("4", 1.0),
    ("8.", 0.75),
    ("4t", 2.0 / 3.0),
    ("8", 0.5),
    ("16.", 0.375),
    ("8t", 1.0 / 3.0),
    ("16", 0.25),
    ("32.", 0.1875),
    ("16t", 1.0 / 6.0),
    ("32", 0.125),
];

/// What the arpeggiator is set to, as the parameters have it for a block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub on: bool,
    pub latch: bool,
    pub direction: Direction,
    /// A step, quarter notes.
    pub step: f64,
    /// 1 to [`MOST_OCTAVES`].
    pub octaves: u8,
    /// 0 to 1.
    pub gate: f64,
    /// 0 to 1.
    pub swing: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            on: false,
            latch: false,
            direction: Direction::Up,
            step: 0.25,
            octaves: 1,
            gate: 0.5,
            swing: 0.0,
        }
    }
}

/// The host's transport as a block begins.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clock {
    /// Beats (quarter notes) a minute: the host's, else the standalone's TEMPO.
    pub tempo: f64,
    pub playing: bool,
    /// Where the host is, quarter notes, if it says (and plays).
    pub beats: Option<f64>,
    /// Where its bar began, and how long a bar is, quarter notes, if it says.
    pub bar_start: Option<f64>,
    pub bar: Option<f64>,
    /// Samples a second.
    pub rate: f64,
}

impl Clock {
    /// Stopped, at `tempo`, `rate` samples a second.
    pub fn stopped(tempo: f64, rate: f64) -> Clock {
        Clock {
            tempo,
            playing: false,
            beats: None,
            bar_start: None,
            bar: None,
            rate,
        }
    }
}

/// A step of the grid: the bar it is counted from, its number in it and its time (beats).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Step {
    bar: f64,
    n: i64,
    beat: f64,
}

/// The notes of a pattern, a bit a pitch, from 0 to 255: those OCTAVES puts past MIDI's 127
/// kept as they would be, to be brought down into range as each is played ([`audible`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Wide([u128; 2]);

impl Wide {
    /// `keys` raised `by` semitones (under 128).
    fn raised(keys: u128, by: u32) -> Wide {
        match by {
            0 => Wide([keys, 0]),
            b => Wide([keys << b, keys >> (128 - b)]),
        }
    }

    /// Pitch `p` alone.
    fn one(p: u8) -> Wide {
        let mut w = Wide::default();
        w.0[usize::from(p >> 7)] = 1u128 << (p & 127);
        w
    }

    fn or(self, o: Wide) -> Wide {
        Wide([self.0[0] | o.0[0], self.0[1] | o.0[1]])
    }

    fn is_empty(self) -> bool {
        self.0 == [0, 0]
    }

    fn has(self, p: u8) -> bool {
        self.0[usize::from(p >> 7)] >> (p & 127) & 1 == 1
    }

    fn count(self) -> u32 {
        self.0[0].count_ones() + self.0[1].count_ones()
    }

    fn lowest(self) -> Option<u8> {
        match self.0 {
            [0, 0] => None,
            [0, h] => Some(128 + h.trailing_zeros() as u8),
            [l, _] => Some(l.trailing_zeros() as u8),
        }
    }

    fn highest(self) -> Option<u8> {
        match self.0 {
            [0, 0] => None,
            [l, 0] => Some(127 - l.leading_zeros() as u8),
            [_, h] => Some(255 - h.leading_zeros() as u8),
        }
    }

    /// Those below `p`.
    fn below(self, p: u8) -> Wide {
        let under = |w: u128, p: u8| w & ((1u128 << p) - 1);
        if p < 128 {
            Wide([under(self.0[0], p), 0])
        } else {
            Wide([self.0[0], under(self.0[1], p - 128)])
        }
    }

    /// Those above `p`.
    fn above(self, p: u8) -> Wide {
        let over = |w: u128, p: u8| if p >= 127 { 0 } else { w & !((2u128 << p) - 1) };
        if p < 128 {
            Wide([over(self.0[0], p), self.0[1]])
        } else {
            Wide([0, over(self.0[1], p - 128)])
        }
    }

    /// The pitches, low to high.
    fn pitches(self) -> impl Iterator<Item = u8> {
        keys(self.0[0]).chain(keys(self.0[1]).map(|k| k + 128))
    }

    /// The `i`th (from 0, low to high).
    fn nth(self, i: u32) -> Option<u8> {
        let low = self.0[0].count_ones();
        if i < low {
            Some(nth(self.0[0], i))
        } else if i - low < self.0[1].count_ones() {
            Some(128 + nth(self.0[1], i - low))
        } else {
            None
        }
    }
}

/// A pattern's note as it is played: brought down by whole octaves into MIDI's range, the
/// highest octave it has there.
pub fn audible(p: u8) -> u8 {
    if p <= 127 {
        p
    } else {
        p - 12 * (p - 127).div_ceil(12)
    }
}

/// A note it sounds: its key, when it was pressed and when it is let go (beats; none: tied to
/// the next step, GATE 100 %).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Held {
    key: u8,
    on: f64,
    off: Option<f64>,
}

/// How many notes may sound at once: steps under a quarter of [`SHORTEST`] (2.5 ms; past the
/// tempos and RATEs it plays, but for a bar's last step cut very short) let the oldest go early.
const OVERLAP: usize = 8;

/// The arpeggiator. Every key the note sources pass goes through it ([`Arp::key`]); what it
/// plays it hands `out` (key, on), the engine's `Event::Note`.
#[derive(Debug)]
pub struct Arp {
    s: Settings,
    /// The keys held, a bit a key; those let go with the pedal down; the chord HOLD holds
    /// (while ON is off, the drone).
    held: u128,
    sustained: u128,
    latched: u128,
    pedal: bool,
    /// The keys in the order they were last pressed, each once (ORDER's; those no longer
    /// played passed over).
    order: [u8; 128],
    pressed: usize,
    /// The block: where it begins (beats), beats a sample and a second.
    pos: f64,
    per: f64,
    bps: f64,
    /// Where the next block begins, as this one ran (the clock while the host is stopped).
    expected: f64,
    /// Whether the last block's grid was the host's.
    locked: bool,
    /// The grid: the bar the steps are counted from and a bar's length (infinite: none).
    anchor: f64,
    bar: f64,
    /// The next step, while it plays.
    next: Option<Step>,
    /// The notes sounding (more than one only while a note shorter than its step's allows is
    /// held to [`SHORTEST`] over the next's start).
    sounding: [Option<Held>; OVERLAP],
    /// The note last played (as the pattern has it, past 127 too), UP/DOWN's way (up), and
    /// ORDER's place: the octave, the key and its place in the order.
    last: Option<u8>,
    rising: bool,
    place: Option<(u8, u8, usize)>,
    rng: u64,
}

impl Default for Arp {
    fn default() -> Self {
        Arp::new(0x5eed_ca72)
    }
}

impl Arp {
    /// An arpeggiator whose RANDOM begins from `seed`.
    pub fn new(seed: u64) -> Arp {
        Arp {
            s: Settings::default(),
            held: 0,
            sustained: 0,
            latched: 0,
            pedal: false,
            order: [0; 128],
            pressed: 0,
            pos: 0.0,
            per: 0.0,
            bps: 2.0,
            expected: 0.0,
            locked: false,
            anchor: 0.0,
            bar: f64::INFINITY,
            next: None,
            sounding: [None; OVERLAP],
            last: None,
            rising: true,
            place: None,
            rng: seed | 1,
        }
    }

    /// RANDOM begun again from `seed` (a session's own, so that it renders the same again).
    pub fn seed(&mut self, seed: u64) {
        self.rng = seed | 1;
    }

    /// Whether it is on.
    pub fn on(&self) -> bool {
        self.s.on
    }

    /// The key it sounds now (the last pressed), if any.
    pub fn sounding(&self) -> Option<u8> {
        self.sounding
            .iter()
            .flatten()
            .max_by(|a, b| a.on.total_cmp(&b.on))
            .map(|h| h.key)
    }

    /// When the next note sounding is let go (beats), if one is to be.
    fn next_off(&self) -> Option<f64> {
        self.sounding
            .iter()
            .flatten()
            .filter_map(|h| h.off)
            .min_by(f64::total_cmp)
    }

    /// A block begins: `s` its settings, `clock` the host's transport, `len` samples long.
    /// ON or HOLD changed are taken here, at its first sample: switched on, the keys sounding
    /// (held, or held by HOLD) go from the engine to the arpeggiator, HOLD's chord its own; off,
    /// its note is let go and the keys held pressed again, as played, with HOLD its chord too.
    /// HOLD switched off while ON is off lets go of what it held.
    pub fn begin(&mut self, s: Settings, clock: Clock, len: usize, out: &mut impl FnMut(u8, bool)) {
        let tempo = if clock.tempo.is_finite() && clock.tempo > 0.0 {
            clock.tempo.clamp(1.0, 1000.0)
        } else {
            120.0
        };
        let rate = if clock.rate.is_finite() && clock.rate > 0.0 {
            clock.rate
        } else {
            48_000.0
        };
        self.bps = tempo / 60.0;
        self.per = self.bps / rate;
        let host = clock
            .playing
            .then_some(clock.beats)
            .flatten()
            .filter(|b| b.is_finite());
        let was = self.s;
        let s = Settings {
            step: if s.step.is_finite() && s.step > 0.0 {
                s.step
            } else {
                0.25
            },
            octaves: s.octaves.clamp(1, MOST_OCTAVES),
            gate: s.gate.clamp(0.0, 1.0),
            swing: s.swing.clamp(0.0, 1.0),
            ..s
        };
        // Where the block begins: the host's place while it plays, else where the last left
        // off (the clock running on at the tempo).
        let start = host.unwrap_or(self.expected);
        let jumped = start - self.expected;
        self.pos = start;
        self.expected = start + len as f64 * self.per;
        self.s = s;
        if !s.on {
            self.locked = false;
            if was.on {
                self.silence(out);
                let played = if s.latch {
                    let kept = if was.latch { self.latched } else { 0 };
                    kept | self.held | self.sustained
                } else {
                    self.held
                };
                if s.latch {
                    self.latched = played;
                } else {
                    (self.latched, self.sustained) = (0, 0);
                }
                for k in keys(played) {
                    out(k, true);
                }
            } else if s.latch != was.latch {
                if s.latch {
                    self.latched = self.held;
                } else {
                    for k in keys(self.latched & !self.held) {
                        out(k, false);
                    }
                    (self.latched, self.sustained) = (0, 0);
                }
            }
            return;
        }
        // The host's grid, its bars, while it plays and says where it is: found again where it
        // is when it starts, or its place jumps (a loop, a locate); the note sounding keeps
        // what was left of it.
        match host {
            Some(p) => {
                let bar = clock
                    .bar
                    .filter(|b| b.is_finite() && *b > EPS)
                    .unwrap_or(f64::INFINITY);
                (self.anchor, self.bar) = match clock.bar_start.filter(|b| b.is_finite()) {
                    Some(a) => (a, bar),
                    None => (0.0, f64::INFINITY),
                };
                let moved = !self.locked || jumped.abs() > 1e-4;
                self.locked = true;
                if moved {
                    for h in self.sounding.iter_mut().flatten() {
                        h.on += jumped;
                        if let Some(off) = &mut h.off {
                            *off += jumped;
                        }
                    }
                }
                let stale = self
                    .next
                    .is_some_and(|n| n.beat < p - 1e-4 || n.beat > p + 2.0 * s.step + 1e-4);
                if self.next.is_some() && (moved || stale) {
                    self.next = Some(self.at_or_after(p - EPS));
                }
            }
            None => self.locked = false,
        }
        if !was.on {
            let sounding = self.held | if was.latch { self.latched } else { 0 };
            for k in keys(sounding) {
                out(k, false);
            }
            self.latched = if s.latch { sounding } else { 0 };
            self.sustained = 0;
            self.silence(out);
            if !self.notes().is_empty() {
                self.start(self.pos);
            }
            return;
        }
        if s.latch != was.latch {
            self.latched = if s.latch {
                self.held | self.sustained
            } else {
                0
            };
            if self.notes().is_empty() {
                self.stop(self.pos, out);
            }
        }
        // RATE or SWING moved: the next step on the grid as it now is.
        if self.next.is_some() && (s.step != was.step || s.swing != was.swing) {
            self.next = Some(self.at_or_after(self.pos - EPS));
        }
    }

    /// A key pressed or let go `at` samples into the block.
    pub fn key(&mut self, key: u8, on: bool, at: usize, out: &mut impl FnMut(u8, bool)) {
        if key > 127 {
            return;
        }
        let bit = 1u128 << key;
        if on {
            self.ordered(key);
        }
        if !self.s.on {
            if self.s.latch {
                self.drone(key, on, out);
                return;
            }
            self.held = if on {
                self.held | bit
            } else {
                self.held & !bit
            };
            out(key, on);
            return;
        }
        let before = self.notes();
        if on {
            if self.s.latch && self.held | self.sustained == 0 {
                // A new chord: the latched one let go.
                self.latched = 0;
            }
            self.held |= bit;
            if self.s.latch {
                self.latched |= bit;
            }
        } else {
            self.held &= !bit;
            if self.pedal {
                self.sustained |= bit;
            }
        }
        self.follow(before, at, out);
    }

    /// A key pressed or let go with ON off and HOLD on: pressed, as played, the chord held let go
    /// first if it is a new one (none held), the key let go first if HOLD holds it; let go,
    /// held on.
    fn drone(&mut self, key: u8, on: bool, out: &mut impl FnMut(u8, bool)) {
        let bit = 1u128 << key;
        if on {
            if self.held | self.sustained == 0 {
                for k in keys(self.latched) {
                    out(k, false);
                }
                self.latched = 0;
            } else if self.latched & bit != 0 {
                out(key, false);
            }
            self.held |= bit;
            self.latched |= bit;
            out(key, true);
        } else {
            self.held &= !bit;
            if self.pedal {
                self.sustained |= bit;
            }
        }
    }

    /// `key` pressed: at the end of the order.
    fn ordered(&mut self, key: u8) {
        let n = self.pressed;
        if let Some(i) = self.order[..n].iter().position(|&k| k == key) {
            self.order.copy_within(i + 1..n, i);
            self.order[n - 1] = key;
        } else if n < self.order.len() {
            self.order[n] = key;
            self.pressed += 1;
        }
    }

    /// The sustain pedal down or up `at` samples into the block: the keys let go while it is
    /// down held as if they were still.
    pub fn pedal(&mut self, down: bool, at: usize, out: &mut impl FnMut(u8, bool)) {
        if down == self.pedal {
            return;
        }
        let before = self.notes();
        self.pedal = down;
        if !down {
            self.sustained = 0;
        }
        if self.s.on {
            self.follow(before, at, out);
        }
    }

    /// Everything forgotten (All Notes Off, All Sound Off, a reset), the engine having let go
    /// of every key itself: nothing held, latched or sounding.
    pub fn clear(&mut self) {
        self.held = 0;
        self.sustained = 0;
        self.latched = 0;
        self.pedal = false;
        self.pressed = 0;
        self.next = None;
        self.sounding = [None; OVERLAP];
        self.last = None;
        self.place = None;
    }

    /// The pedal up and what it held let go (Reset All Controllers).
    pub fn pedal_up(&mut self, at: usize, out: &mut impl FnMut(u8, bool)) {
        self.pedal(false, at, out);
    }

    /// The sample of this block when it next acts, if it will (at or past the block's end
    /// too: the caller cuts it there).
    pub fn due(&self) -> Option<usize> {
        if !self.s.on {
            return None;
        }
        let step = self.next.map(|n| n.beat);
        let off = self.next_off();
        let t = match (step, off) {
            (Some(a), Some(b)) => a.min(b),
            (a, b) => a.or(b)?,
        };
        Some(self.sample(t))
    }

    /// Everything due at or before sample `at` of the block done, in time.
    pub fn advance(&mut self, at: usize, out: &mut impl FnMut(u8, bool)) {
        if !self.s.on {
            return;
        }
        // (A bounded loop: a step's length is never below a sample's worth of beats for long,
        // and each pass moves the next step on.)
        for _ in 0..64 {
            let off = self.next_off().filter(|&o| self.sample(o) <= at);
            let step = self.next.filter(|n| self.sample(n.beat) <= at);
            match (off, step) {
                (Some(o), Some(n)) if o <= n.beat + EPS => self.let_go(o, out),
                (Some(o), None) => self.let_go(o, out),
                (_, Some(n)) => self.play(n, out),
                (None, None) => return,
            }
        }
    }

    /// The sample of the block at beat `t` (the first at or after it; 0 for one before the
    /// block).
    fn sample(&self, t: f64) -> usize {
        if self.per <= 0.0 {
            return 0;
        }
        let k = ((t - self.pos) / self.per - 1e-6).ceil();
        if k <= 0.0 { 0 } else { k.min(1e12) as usize }
    }

    /// The beat at sample `at` of the block.
    fn beat(&self, at: usize) -> f64 {
        self.pos + at as f64 * self.per
    }

    /// The keys it plays.
    fn base(&self) -> u128 {
        if self.s.latch {
            self.latched | self.held | self.sustained
        } else {
            self.held | self.sustained
        }
    }

    /// The keys it plays, the octaves over them.
    fn notes(&self) -> Wide {
        let base = self.base();
        (0..u32::from(self.s.octaves))
            .fold(Wide::default(), |m, o| m.or(Wide::raised(base, 12 * o)))
    }

    /// The keys changed `at` samples into the block, `before` what it played: started from
    /// silence, or silent now.
    fn follow(&mut self, before: Wide, at: usize, out: &mut impl FnMut(u8, bool)) {
        let now = self.notes();
        if now.is_empty() {
            if !before.is_empty() || self.sounding().is_some() {
                self.stop(self.beat(at), out);
            }
        } else if self.next.is_none() {
            self.start(self.beat(at));
        }
    }

    /// Nothing more to play from beat `t` (the keys let go, HOLD off): each note let go now, but
    /// one not yet held [`SHORTEST`], which is let go then (the owner: "NEVER below the minimum
    /// gate length").
    fn stop(&mut self, t: f64, out: &mut impl FnMut(u8, bool)) {
        let shortest = SHORTEST * self.bps;
        for slot in &mut self.sounding {
            if let Some(h) = slot {
                if t - h.on >= shortest - EPS {
                    out(h.key, false);
                    *slot = None;
                } else {
                    h.off = Some(h.on + shortest);
                }
            }
        }
        self.next = None;
        self.last = None;
        self.rising = true;
        self.place = None;
    }

    /// Every note let go at once, nothing more to play (ON switched off, its keys handed back).
    fn silence(&mut self, out: &mut impl FnMut(u8, bool)) {
        for h in self.sounding.iter_mut().filter_map(Option::take) {
            out(h.key, false);
        }
        self.next = None;
        self.last = None;
        self.rising = true;
        self.place = None;
    }

    /// Started from silence at beat `t`: on the host's grid, the step a key pressed now
    /// belongs to (a quarter step late at most) or the next; stopped, a grid from now.
    fn start(&mut self, t: f64) {
        self.last = None;
        self.rising = true;
        self.place = None;
        if self.locked {
            let first = self.at_or_after(t - GRACE * self.s.step);
            self.next = Some(if first.beat < t {
                Step { beat: t, ..first }
            } else {
                first
            });
        } else {
            (self.anchor, self.bar) = (t, f64::INFINITY);
            self.next = Some(Step {
                bar: t,
                n: 0,
                beat: t,
            });
        }
    }

    /// A step's offset from its bar's start: its number of steps, the odd ones late by SWING.
    fn offset(&self, n: i64) -> f64 {
        let l = self.s.step;
        n as f64 * l
            + if n.rem_euclid(2) == 1 {
                self.s.swing * l / 2.0
            } else {
                0.0
            }
    }

    /// The first step of the grid at or after beat `p`.
    fn at_or_after(&self, p: f64) -> Step {
        let bar = if self.bar.is_finite() {
            self.anchor + ((p - self.anchor) / self.bar).floor() * self.bar
        } else {
            self.anchor
        };
        let from = ((p - bar) / self.s.step).floor() as i64 - 1;
        let mut n = from;
        while bar + self.offset(n) < p - EPS && n < from + 4 {
            n += 1;
        }
        let step = Step {
            bar,
            n,
            beat: bar + self.offset(n),
        };
        self.within_bar(step)
    }

    /// The step after `s`.
    fn after(&self, s: Step) -> Step {
        self.within_bar(Step {
            bar: s.bar,
            n: s.n + 1,
            beat: s.bar + self.offset(s.n + 1),
        })
    }

    /// A step past its bar's end the next bar's first.
    fn within_bar(&self, s: Step) -> Step {
        if self.bar.is_finite() && s.beat >= s.bar + self.bar - EPS {
            let bar = s.bar + self.bar;
            Step {
                bar,
                n: 0,
                beat: bar,
            }
        } else {
            s
        }
    }

    /// Step `n` played: its key pressed, its release set, and the next step. The notes tied to
    /// it (GATE 100 %) let go after it is pressed (legato), or at their [`SHORTEST`] if they are
    /// younger. Its key still sounding: let go and pressed again if it has been held
    /// [`SHORTEST`], else tied into this step, held on to the step's release (a key cannot be
    /// down twice, and no note is cut short).
    fn play(&mut self, n: Step, out: &mut impl FnMut(u8, bool)) {
        let next = self.after(n);
        self.next = Some(next);
        let Some(p) = self.pick() else {
            self.stop(n.beat, out);
            return;
        };
        let (k, t) = (audible(p), n.beat);
        let shortest = SHORTEST * self.bps;
        let off = (self.s.gate < 1.0).then(|| t + self.held_for(next.beat - t));
        self.last = Some(p);
        let same = self
            .sounding
            .iter()
            .position(|h| h.is_some_and(|h| h.key == k));
        let tied = match same.and_then(|i| self.sounding[i].map(|h| (i, h))) {
            Some((i, h)) if t - h.on < shortest - EPS => {
                self.sounding[i] = Some(Held {
                    off: off.map(|o| o.max(h.on + shortest)),
                    ..h
                });
                true
            }
            Some((i, h)) => {
                out(h.key, false);
                self.sounding[i] = None;
                false
            }
            None => false,
        };
        if !tied {
            out(k, true);
            let slot = match self.sounding.iter().position(Option::is_none) {
                Some(i) => i,
                // (All full: the oldest let go early.)
                None => {
                    let i = (0..OVERLAP)
                        .min_by(|&a, &b| {
                            let on = |i: usize| self.sounding[i].map_or(0.0, |h| h.on);
                            on(a).total_cmp(&on(b))
                        })
                        .unwrap_or(0);
                    if let Some(h) = self.sounding[i].take() {
                        out(h.key, false);
                    }
                    i
                }
            };
            self.sounding[slot] = Some(Held { key: k, on: t, off });
        }
        for slot in &mut self.sounding {
            if let Some(h) = slot
                && h.key != k
                && h.off.is_none()
            {
                if t - h.on >= shortest - EPS {
                    out(h.key, false);
                    *slot = None;
                } else {
                    h.off = Some(h.on + shortest);
                }
            }
        }
    }

    /// How long a note is held in a step `length` beats long: GATE's share of what lies between
    /// the shortest note and the shortest gap; in a step too short for both, the shortest note,
    /// over the next step's start if need be (the owner: "the gate in the arp should NEVER go
    /// below the minimum gate length to trigger a sound in the device").
    fn held_for(&self, length: f64) -> f64 {
        let (on, gap) = (SHORTEST * self.bps, REARM * self.bps);
        let room = length - on - gap;
        if room >= 0.0 {
            on + self.s.gate * room
        } else {
            on
        }
    }

    /// The note due to be let go at beat `o` let go.
    fn let_go(&mut self, o: f64, out: &mut impl FnMut(u8, bool)) {
        if let Some(slot) = self
            .sounding
            .iter_mut()
            .find(|h| h.is_some_and(|h| h.off == Some(o)))
            && let Some(h) = slot.take()
        {
            out(h.key, false);
        }
    }

    /// The next note to play, as the direction has it (past 127 as the pattern has it).
    fn pick(&mut self) -> Option<u8> {
        if self.s.direction == Direction::Order {
            return self.in_order();
        }
        let notes = self.notes();
        let (lowest, highest) = (notes.lowest()?, notes.highest()?);
        let above = |k: u8| notes.above(k).lowest();
        let below = |k: u8| notes.below(k).highest();
        Some(match (self.s.direction, self.last) {
            (Direction::Up, Some(l)) => above(l).unwrap_or(lowest),
            (Direction::Down, Some(l)) => below(l).unwrap_or(highest),
            (Direction::Up | Direction::UpDown, None) => lowest,
            (Direction::Down, None) => highest,
            (Direction::UpDown, Some(l)) => {
                let (ahead, back) = if self.rising {
                    (above(l), below(l))
                } else {
                    (below(l), above(l))
                };
                match (ahead, back) {
                    (Some(k), _) => k,
                    (None, Some(k)) => {
                        self.rising = !self.rising;
                        k
                    }
                    // (Alone, or the last gone from a chord: from the end it is going to.)
                    (None, None) if notes.has(l) => l,
                    (None, None) => {
                        if self.rising {
                            lowest
                        } else {
                            highest
                        }
                    }
                }
            }
            // Never the note last heard again while another may be: those that sound as it
            // does passed over (several of the pattern's may, past 127).
            (Direction::Random | Direction::Order, last) => {
                let heard = last.map(audible);
                let others = notes
                    .pitches()
                    .filter(|&p| Some(audible(p)) != heard)
                    .fold(Wide::default(), |m, p| m.or(Wide::one(p)));
                let choices = if others.is_empty() { notes } else { others };
                let i = (self.random() % u64::from(choices.count())) as u32;
                choices.nth(i)?
            }
        })
    }

    /// ORDER's next note: the key after the last played in the order, past the last the first an
    /// octave higher, past the top octave the first again; the last let go, the one now in its
    /// place.
    fn in_order(&mut self) -> Option<u8> {
        let base = self.base();
        let order = self.order[..self.pressed]
            .iter()
            .copied()
            .filter(move |&k| base >> k & 1 == 1);
        let n = order.clone().count();
        if n == 0 {
            return None;
        }
        let (mut octave, mut i) = match self.place {
            None => (0, 0),
            Some((o, key, at)) => match order.clone().position(|k| k == key) {
                Some(j) => (o, j + 1),
                None => (o, at),
            },
        };
        if i >= n {
            (octave, i) = (octave + 1, 0);
        }
        if octave >= self.s.octaves {
            octave = 0;
        }
        let key = order.clone().nth(i)?;
        self.place = Some((octave, key, i));
        Some(key + 12 * octave)
    }

    /// The next of RANDOM's numbers (xorshift64*).
    fn random(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 11
    }
}

/// The keys of a set, low to high.
fn keys(set: u128) -> impl Iterator<Item = u8> {
    (0..128u8).filter(move |k| set >> k & 1 == 1)
}

/// The `i`th key of a set (from 0, low to high).
fn nth(mut set: u128, i: u32) -> u8 {
    for _ in 0..i {
        set &= set - 1;
    }
    set.trailing_zeros() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: f64 = 48_000.0;

    /// What it played, by the sample (from the first block's start): key, on.
    type Played = Vec<(usize, u8, bool)>;

    /// The same run, the samples recorded exactly: each event's sample from a counter the
    /// closure reads.
    fn played(s: Settings, block: usize, until: usize, events: &[(usize, u8, bool)]) -> Played {
        played_with(s, block, until, events, |_| {})
    }

    fn played_with(
        s: Settings,
        block: usize,
        until: usize,
        events: &[(usize, u8, bool)],
        set: impl FnOnce(&mut Exact),
    ) -> Played {
        let mut r = Exact::new(s, block);
        set(&mut r);
        r.run(until, events);
        r.played
    }

    /// A run that records each note's exact sample.
    struct Exact {
        arp: Arp,
        s: Settings,
        block: usize,
        now: usize,
        played: Played,
        tempo: f64,
        rate: f64,
        host: Option<f64>,
        bar: Option<(f64, f64)>,
        /// The host's position jumps to `.1` at sample `.0` (a loop).
        jump: Option<(usize, f64)>,
    }

    impl Exact {
        fn new(s: Settings, block: usize) -> Exact {
            Exact {
                arp: Arp::default(),
                s,
                block,
                now: 0,
                played: Vec::new(),
                tempo: 120.0,
                rate: RATE,
                host: None,
                bar: None,
                jump: None,
            }
        }

        fn per(&self) -> f64 {
            self.tempo / 60.0 / self.rate
        }

        fn clock(&self) -> Clock {
            let beats = self.host.map(|h| match self.jump {
                Some((at, to)) if self.now >= at => to + (self.now - at) as f64 * self.per(),
                _ => h + self.now as f64 * self.per(),
            });
            Clock {
                tempo: self.tempo,
                playing: self.host.is_some(),
                beats,
                bar_start: beats.and_then(|b| {
                    let (start, len) = self.bar?;
                    Some(start + ((b - start) / len).floor() * len)
                }),
                bar: self.bar.map(|(_, l)| l),
                rate: self.rate,
            }
        }

        fn run(&mut self, until: usize, events: &[(usize, u8, bool)]) {
            let mut events: Vec<_> = events.to_vec();
            events.sort_by_key(|e| e.0);
            let mut k = events
                .iter()
                .position(|e| e.0 >= self.now)
                .unwrap_or(events.len());
            while self.now < until {
                let mut len = self.block.min(until - self.now);
                if let Some((at, _)) = self.jump
                    && self.now < at
                {
                    len = len.min(at - self.now);
                }
                let base = self.now;
                let clock = self.clock();
                let at = std::cell::Cell::new(base);
                let played = std::cell::RefCell::new(std::mem::take(&mut self.played));
                let mut out = |key: u8, on: bool| played.borrow_mut().push((at.get(), key, on));
                self.arp.begin(self.s, clock, len, &mut out);
                let mut i = 0;
                loop {
                    at.set(base + i);
                    while k < events.len() && events[k].0 <= base + i {
                        self.arp.key(events[k].1, events[k].2, i, &mut out);
                        k += 1;
                    }
                    self.arp.advance(i, &mut out);
                    let mut end = len;
                    if k < events.len() {
                        end = end.min(events[k].0 - base);
                    }
                    if let Some(d) = self.arp.due() {
                        end = end.min(d.max(i + 1));
                    }
                    if end >= len {
                        break;
                    }
                    i = end;
                }
                self.played = played.into_inner();
                self.now += len;
            }
        }
    }

    fn ons(p: &Played) -> Vec<(usize, u8)> {
        p.iter().filter(|e| e.2).map(|e| (e.0, e.1)).collect()
    }

    fn on_keys(p: &Played) -> Vec<u8> {
        p.iter().filter(|e| e.2).map(|e| e.1).collect()
    }

    fn arp(direction: Direction) -> Settings {
        Settings {
            on: true,
            direction,
            ..Settings::default()
        }
    }

    /// Every key pressed is let go, at most one sounds at a time (but the instant a tie hands
    /// over), and nothing is let go that was not pressed.
    fn balanced(p: &Played) {
        let mut down = [0i32; 128];
        for &(at, k, on) in p {
            down[usize::from(k)] += if on { 1 } else { -1 };
            assert!(
                (0..=1).contains(&down[usize::from(k)]),
                "key {k} at {at}: {p:?}"
            );
        }
        assert!(down.iter().all(|d| *d == 0), "stuck: {p:?}");
    }

    /// Off, every key goes straight through, at its sample, in its order: as it always has.
    #[test]
    fn off_it_passes_every_key_through_as_played() {
        let events = [
            (10, 60, true),
            (10, 64, true),
            (300, 60, false),
            (301, 64, false),
        ];
        let p = played(Settings::default(), 128, 1024, &events);
        assert_eq!(p, events.to_vec());
    }

    /// Stopped, 1/16 at 120 beats a minute is a step every 6000 samples at 48 kHz, the first at
    /// the key's press; UP plays the chord low to high and again; GATE 50 % lets each go half way
    /// between the shortest note (960 samples) and the shortest gap's start (624 before the next).
    #[test]
    fn up_plays_the_keys_held_in_pitch_order_from_the_press() {
        let events = [(1000, 67, true), (1000, 60, true), (1000, 64, true)];
        let p = played(arp(Direction::Up), 256, 1000 + 6000 * 6, &events);
        assert_eq!(
            ons(&p),
            [60, 64, 67, 60, 64, 67]
                .iter()
                .enumerate()
                .map(|(i, &k)| (1000 + 6000 * i, k))
                .collect::<Vec<_>>()
        );
        let offs: Vec<usize> = p.iter().filter(|e| !e.2).map(|e| e.0).collect();
        assert_eq!(offs[0], 1000 + 960 + (6000 - 960 - 624) / 2);
    }

    /// DOWN high to low; UP/DOWN up and down, its ends once; OCTAVES 2 the chord again an octave
    /// up.
    #[test]
    fn down_up_down_and_octaves_order_the_pattern() {
        let chord = [(0, 60, true), (0, 64, true), (0, 67, true)];
        let n = |p: &Played, k: usize| on_keys(p)[..k].to_vec();
        let p = played(arp(Direction::Down), 128, 6000 * 6, &chord);
        assert_eq!(n(&p, 6), [67, 64, 60, 67, 64, 60]);
        let p = played(arp(Direction::UpDown), 128, 6000 * 9, &chord);
        assert_eq!(n(&p, 9), [60, 64, 67, 64, 60, 64, 67, 64, 60]);
        let s = Settings {
            octaves: 2,
            ..arp(Direction::Up)
        };
        let p = played(s, 128, 6000 * 7, &chord);
        assert_eq!(n(&p, 7), [60, 64, 67, 72, 76, 79, 60]);
        // One key, UP/DOWN: the key again and again.
        let p = played(arp(Direction::UpDown), 128, 6000 * 3, &[(0, 60, true)]);
        assert_eq!(on_keys(&p), [60, 60, 60]);
    }

    /// RANDOM never plays the same key twice running while two or more are held, and plays each
    /// of them; with one, that one; and the same seed plays the same again (a render repeats).
    #[test]
    fn random_avoids_an_immediate_repeat() {
        let chord = [(0, 60, true), (0, 62, true), (0, 64, true), (0, 65, true)];
        let p = played(arp(Direction::Random), 128, 6000 * 400, &chord);
        let k = on_keys(&p);
        assert!(k.len() >= 399);
        assert!(k.windows(2).all(|w| w[0] != w[1]), "{k:?}");
        for key in [60, 62, 64, 65] {
            assert!(k.contains(&key));
        }
        assert_eq!(
            k,
            on_keys(&played(arp(Direction::Random), 128, 6000 * 400, &chord))
        );
        let p = played(arp(Direction::Random), 128, 6000 * 5, &[(0, 60, true)]);
        assert_eq!(on_keys(&p), [60; 5]);
    }

    /// The step's sample is the same whatever the blocks (1, 64, 1024 and 4096 samples), and
    /// lands within a sample of the grid at several tempos and sample rates.
    #[test]
    fn steps_land_on_their_samples_whatever_the_block() {
        for (tempo, rate) in [
            (120.0, 48_000.0),
            (97.3, 44_100.0),
            (174.0, 96_000.0),
            (60.0, 22_050.0),
        ] {
            let mut want: Option<Vec<(usize, u8)>> = None;
            for block in [1, 64, 1024, 4096] {
                let p = played_with(
                    arp(Direction::Up),
                    block,
                    200_000,
                    &[(0, 60, true), (0, 63, true)],
                    |r| {
                        r.tempo = tempo;
                        r.rate = rate;
                    },
                );
                let got = ons(&p);
                let step = 0.25 * 60.0 / tempo * rate;
                for (i, (at, _)) in got.iter().enumerate() {
                    assert!(
                        (*at as f64 - i as f64 * step).abs() <= 1.0,
                        "{tempo} {rate} {block}: step {i} at {at}"
                    );
                }
                match &want {
                    None => want = Some(got),
                    Some(w) => assert_eq!(&got, w, "{tempo} {rate} block {block}"),
                }
            }
        }
    }

    /// While the host plays, the steps are its sixteenths from the bar's start, whatever the
    /// key's moment: a key pressed between two waits for the next, one pressed a little late
    /// plays at once as that step's.
    #[test]
    fn playing_it_locks_to_the_hosts_grid() {
        // The host at beat 10.1 (a beat 24 000 samples): beat 10.25 is 3600 samples on.
        let host = |r: &mut Exact| {
            r.host = Some(10.1);
            r.bar = Some((0.0, 4.0));
        };
        let p = played_with(arp(Direction::Up), 512, 30_000, &[(1000, 60, true)], host);
        let at: Vec<usize> = ons(&p).iter().map(|e| e.0).collect();
        assert!((at[0] as f64 - 3600.0).abs() <= 1.0, "{at:?}");
        assert!((at[1] as f64 - 9600.0).abs() <= 1.0, "{at:?}");
        // A key 500 samples after a step (a twelfth of one): at once, as that step's.
        let p = played_with(arp(Direction::Up), 512, 30_000, &[(4100, 60, true)], host);
        let at: Vec<usize> = ons(&p).iter().map(|e| e.0).collect();
        assert_eq!(at[0], 4100);
        assert!((at[1] as f64 - 9600.0).abs() <= 1.0, "{at:?}");
    }

    /// A dotted quarter in 4/4: steps at 0, 1.5 and 3 beats of each bar, the last cut short at
    /// the bar's end, the next bar's first on its downbeat.
    #[test]
    fn a_bar_starts_the_grid_again() {
        let s = Settings {
            step: 1.5,
            ..arp(Direction::Up)
        };
        let p = played_with(s, 256, 24_000 * 9, &[(0, 60, true)], |r| {
            r.host = Some(0.0);
            r.bar = Some((0.0, 4.0));
        });
        let beats: Vec<f64> = ons(&p)
            .iter()
            .map(|(at, _)| *at as f64 / 24_000.0)
            .collect();
        let want = [0.0, 1.5, 3.0, 4.0, 5.5, 7.0, 8.0];
        for (got, want) in beats.iter().zip(want) {
            assert!((got - want).abs() < 1e-4, "{beats:?}");
        }
    }

    /// SWING delays every second step: at 100 % three quarters through the pair, at 0 straight.
    #[test]
    fn swing_delays_every_second_step() {
        let s = Settings {
            swing: 1.0,
            ..arp(Direction::Up)
        };
        let p = played(s, 128, 24_000, &[(0, 60, true)]);
        let at: Vec<usize> = ons(&p).iter().map(|e| e.0).collect();
        assert_eq!(&at[..4], [0, 9000, 12_000, 21_000]);
        let s = Settings {
            swing: 2.0 / 3.0,
            ..arp(Direction::Up)
        };
        let at: Vec<usize> = ons(&played(s, 128, 24_000, &[(0, 60, true)]))
            .iter()
            .map(|e| e.0)
            .collect();
        assert_eq!(&at[..4], [0, 8000, 12_000, 20_000], "the triplet's shuffle");
    }

    /// GATE: 0 % the shortest note; 100 % tied, the next key pressed before the last is let go
    /// (the same key let go first).
    #[test]
    fn gate_sets_each_notes_length() {
        let s = Settings {
            gate: 0.0,
            ..arp(Direction::Up)
        };
        let p = played(s, 128, 12_000, &[(0, 60, true)]);
        assert_eq!(&p[..2], [(0, 60, true), (960, 60, false)]);
        let s = Settings {
            gate: 1.0,
            ..arp(Direction::Up)
        };
        let p = played(s, 128, 12_001, &[(0, 60, true), (0, 64, true)]);
        assert_eq!(
            p,
            [
                (0, 60, true),
                (6000, 64, true),
                (6000, 60, false),
                (12_000, 60, true),
                (12_000, 64, false)
            ]
        );
        let p = played(s, 128, 6001, &[(0, 60, true)]);
        assert_eq!(p, [(0, 60, true), (6000, 60, false), (6000, 60, true)]);
    }

    /// GATE is a share of what the circuit can play (the owner, 2026-10-19): at every RATE, at
    /// tempos from 30 to 300 and GATE from 0 to 99 %, each note is held at least `SHORTEST` and
    /// let go at least `REARM` before the next; in a step too short for both (1/32 above 225
    /// beats a minute) the note keeps its length and the gap is shorter. Longer as GATE grows.
    #[test]
    fn gate_leaves_the_circuit_room_to_start_and_start_again() {
        let (on, gap) = (SHORTEST * RATE, REARM * RATE);
        for (_, step) in RATES {
            for tempo in [30.0, 120.0, 180.0, 225.0, 240.0, 300.0] {
                let length = step * 60.0 / tempo * RATE;
                let mut was = 0;
                for g in [0.0, 0.12, 0.5, 0.9, 0.99] {
                    let s = Settings {
                        gate: g,
                        step,
                        ..arp(Direction::Up)
                    };
                    let p =
                        played_with(s, 256, length.ceil() as usize + 1, &[(0, 60, true)], |r| {
                            r.tempo = tempo;
                        });
                    let off = p.iter().find(|e| !e.2).expect("let go").0;
                    let next = p.iter().filter(|e| e.2).nth(1).expect("the next").0;
                    assert!(
                        off as f64 >= on.min(length) - 1.0,
                        "{step} {tempo} {g}: {p:?}"
                    );
                    if length >= on + gap {
                        assert!(
                            (next - off) as f64 >= gap - 1.0,
                            "{step} {tempo} {g}: {p:?}"
                        );
                    } else {
                        assert!((off as f64 - on).abs() <= 1.0, "{step} {tempo} {g}: {p:?}");
                    }
                    assert!(off >= was, "{step} {tempo} {g}");
                    was = off;
                }
            }
        }
        // 1/32 at 120 with GATE 12 %, the owner's silent case: 23.5 ms now, not 7.5.
        let s = Settings {
            gate: 0.12,
            step: 0.125,
            ..arp(Direction::Up)
        };
        let p = played(s, 64, 3001, &[(0, 60, true)]);
        assert_eq!(p[1], (1130, 60, false));
    }

    /// The keys let go: silent at once (the note let go there); pressed again, from the start.
    #[test]
    fn letting_go_stops_it_and_leaves_nothing_stuck() {
        let events = [
            (0, 60, true),
            (0, 64, true),
            (8000, 60, false),
            (8100, 64, false),
            (20_000, 67, true),
            (21_000, 67, false),
        ];
        let p = played(arp(Direction::Up), 100, 40_000, &events);
        balanced(&p);
        assert!(p.contains(&(8100, 64, false)));
        assert_eq!(ons(&p), [(0, 60), (6000, 64), (20_000, 67)]);
    }

    /// LATCH: the chord plays on after its keys are let go; a key pressed with none held starts
    /// a new chord, one pressed with keys held adds to it; LATCH off lets it go.
    #[test]
    fn latch_holds_the_chord_until_a_new_one_or_latch_off() {
        let s = Settings {
            latch: true,
            ..arp(Direction::Up)
        };
        let mut r = Exact::new(s, 128);
        r.run(
            30_000,
            &[
                (0, 60, true),
                (0, 64, true),
                (100, 60, false),
                (100, 64, false),
            ],
        );
        assert_eq!(on_keys(&r.played), [60, 64, 60, 64, 60]);
        // A new chord: 62 and 65 pressed together with none held, then 67 added while they are.
        r.run(
            60_000,
            &[
                (30_100, 62, true),
                (30_100, 65, true),
                (31_000, 67, true),
                (31_100, 62, false),
                (31_100, 65, false),
                (31_100, 67, false),
            ],
        );
        let after: Vec<u8> = ons(&r.played)
            .iter()
            .filter(|e| e.0 > 30_100)
            .map(|e| e.1)
            .collect();
        assert_eq!(after, [65, 67, 62, 65], "the new chord, 67 added to it");
        // LATCH off: let go at once, nothing more played.
        r.s.latch = false;
        let before = r.played.len();
        r.run(90_000, &[]);
        assert!(ons(&r.played[before..].to_vec()).is_empty());
        assert_eq!(r.arp.sounding(), None);
        balanced(&r.played);
    }

    /// The pedal down keeps the keys let go playing until it comes up.
    #[test]
    fn the_sustain_pedal_holds_keys_as_if_held() {
        let mut r = Exact::new(arp(Direction::Up), 128);
        let mut out = |_: u8, _: bool| {};
        r.arp.begin(r.s, Clock::stopped(120.0, RATE), 0, &mut out);
        r.arp.pedal(true, 0, &mut out);
        r.run(
            30_000,
            &[
                (0, 60, true),
                (0, 64, true),
                (10, 60, false),
                (10, 64, false),
            ],
        );
        assert_eq!(on_keys(&r.played), [60, 64, 60, 64, 60]);
        let played = std::cell::RefCell::new(std::mem::take(&mut r.played));
        r.arp.pedal(false, 0, &mut |k, on| {
            played.borrow_mut().push((30_000, k, on))
        });
        let mut p = std::mem::take(&mut r.played);
        p.extend(played.into_inner());
        r.run(60_000, &[]);
        assert!(ons(&r.played).is_empty(), "{:?}", r.played);
        p.extend(r.played.clone());
        assert_eq!(r.arp.sounding(), None);
        balanced(&p);
    }

    /// ON switched with keys held: on, the engine's keys let go and the arpeggiator plays them;
    /// off, its note let go and the keys pressed again as played. Nothing stuck either way.
    #[test]
    fn switching_on_and_off_hands_the_keys_over() {
        let mut r = Exact::new(Settings::default(), 128);
        r.run(1000, &[(0, 60, true), (0, 64, true)]);
        assert_eq!(r.played, [(0, 60, true), (0, 64, true)]);
        r.s.on = true;
        r.run(13_000, &[]);
        assert_eq!(&r.played[2..4], [(1000, 60, false), (1000, 64, false)]);
        assert_eq!(&ons(&r.played)[2..4], [(1000, 60), (7000, 64)]);
        r.s.on = false;
        r.run(14_000, &[(13_500, 60, false), (13_600, 64, false)]);
        balanced(&r.played);
        assert!(r.played.contains(&(13_000, 60, true)) && r.played.contains(&(13_000, 64, true)));
    }

    /// Cleared (All Notes Off, a reset), nothing plays on and nothing is let go again.
    #[test]
    fn clearing_forgets_everything() {
        let s = Settings {
            latch: true,
            ..arp(Direction::Up)
        };
        let mut r = Exact::new(s, 128);
        r.run(
            7000,
            &[
                (0, 60, true),
                (0, 64, true),
                (10, 60, false),
                (10, 64, false),
            ],
        );
        r.arp.clear();
        r.played.clear();
        r.run(40_000, &[]);
        assert!(r.played.is_empty(), "{:?}", r.played);
    }

    /// The host stopped while it plays: on at the same tempo, in time; started again, on its
    /// grid. A loop back (the position jumping) finds the grid there, the step on the loop's
    /// start played there.
    #[test]
    fn the_transport_stopping_starting_and_looping() {
        let mut r = Exact::new(arp(Direction::Up), 300);
        r.host = Some(0.0);
        r.bar = Some((0.0, 4.0));
        r.run(30_000, &[(0, 60, true), (0, 64, true)]);
        // Stopped at sample 30 000 (beat 1.25): the steps go on every 6000 samples.
        r.host = None;
        r.run(60_000, &[]);
        let at: Vec<usize> = ons(&r.played).iter().map(|e| e.0).collect();
        for (i, a) in at.iter().enumerate() {
            assert!((*a as f64 - 6000.0 * i as f64).abs() <= 1.0, "{at:?}");
        }
        // A loop: at sample 72 001 the host goes back to beat 4.0.
        let mut l = Exact::new(arp(Direction::Up), 256);
        l.host = Some(0.0);
        l.bar = Some((0.0, 4.0));
        l.jump = Some((72_001, 4.0));
        l.run(90_000, &[(0, 60, true), (0, 64, true)]);
        assert!(
            ons(&l.played).iter().any(|e| e.0 == 72_001),
            "{:?}",
            ons(&l.played)
        );
        l.run(91_000, &[(90_500, 60, false), (90_500, 64, false)]);
        balanced(&l.played);
        // One key: the loop's step a sample after the last is the same key, not yet held its
        // shortest: tied into it (not pressed again), the next step 6000 on.
        let mut l = Exact::new(arp(Direction::Up), 256);
        l.host = Some(0.0);
        l.bar = Some((0.0, 4.0));
        l.jump = Some((72_001, 4.0));
        l.run(90_000, &[(0, 60, true)]);
        let at: Vec<usize> = ons(&l.played).iter().map(|e| e.0).collect();
        assert!(
            at.contains(&72_000) && !at.contains(&72_001) && at.contains(&78_001),
            "{at:?}"
        );
    }

    /// A tempo changed between blocks: the next steps follow it from where they were.
    #[test]
    fn a_tempo_change_moves_the_steps_on_at_the_new_tempo() {
        let mut r = Exact::new(arp(Direction::Up), 128);
        r.run(12_000, &[(0, 60, true)]);
        r.tempo = 60.0;
        r.run(48_000, &[]);
        let at: Vec<usize> = ons(&r.played).iter().map(|e| e.0).collect();
        assert_eq!(&at[..4], [0, 6000, 12_000, 24_000]);
    }

    /// RATE's values, slow to fast, their steps each shorter than the last.
    #[test]
    fn rates_run_slow_to_fast() {
        assert!(RATES.windows(2).all(|w| w[0].1 > w[1].1));
        assert_eq!(RATES[9], ("16", 0.25));
    }

    /// ORDER plays the keys as they were pressed, whatever their pitch; a key pressed while the
    /// others are held goes on the end; one let go leaves the order, the one after it next.
    /// OCTAVES plays the order again an octave up for each, a pitch two keys reach played by
    /// each (the order is its keys'); and with HOLD it holds the order as pressed.
    #[test]
    fn order_plays_the_keys_as_they_were_pressed() {
        let o = arp(Direction::Order);
        let chord = [(0, 67, true), (10, 60, true), (20, 64, true)];
        let p = played(o, 128, 6000 * 6, &chord);
        assert_eq!(on_keys(&p), [67, 60, 64, 67, 60, 64]);
        // 62 added while the chord is held, at the end; then 60 let go: 64, 62, 67 ...
        let mut events = chord.to_vec();
        events.extend([(6500, 62, true), (19_000, 60, false)]);
        let p = played(o, 128, 6000 * 9, &events);
        assert_eq!(on_keys(&p), [67, 60, 64, 62, 67, 64, 62, 67, 64]);
        // Pressed again (let go and pressed), a key goes to the end: 60, 67, 64 from then.
        let p = played(
            o,
            128,
            6000 * 6,
            &[
                (0, 60, true),
                (0, 64, true),
                (0, 67, true),
                (100, 64, false),
                (200, 64, true),
            ],
        );
        assert_eq!(on_keys(&p), [60, 67, 64, 60, 67, 64]);
        let s = Settings { octaves: 2, ..o };
        let p = played(s, 128, 6000 * 8, &[(0, 72, true), (10, 60, true)]);
        assert_eq!(on_keys(&p), [72, 60, 84, 72, 72, 60, 84, 72]);
        let s = Settings { latch: true, ..o };
        let mut events = chord.to_vec();
        events.extend([(100, 67, false), (100, 60, false), (100, 64, false)]);
        let p = played(s, 128, 6000 * 6, &events);
        assert_eq!(on_keys(&p), [67, 60, 64, 67, 60, 64]);
        balanced(&played(
            o,
            128,
            40_000,
            &[
                (0, 60, true),
                (10, 64, true),
                (30_000, 60, false),
                (30_000, 64, false),
            ],
        ));
    }

    /// OCTAVES 5 over a high key: a note past MIDI's 127 plays in the highest octave it has (the
    /// owner, 2026-10-19: "repeat the highest octave available of the selected note rather than
    /// go silent"), every step sounding, in each DIRECTION; RANDOM never repeats a note while
    /// there is another, and plays the top one's repeats; UP/DOWN turns at the pattern's ends.
    #[test]
    fn notes_past_the_top_play_in_the_highest_octave_they_have() {
        assert_eq!(
            (audible(127), audible(128), audible(139), audible(151)),
            (127, 116, 127, 127)
        );
        assert_eq!(audible(175), 127);
        let five = |d: Direction| Settings {
            octaves: 5,
            ..arp(d)
        };
        // G8 (115): G8, G9 and G9 again three times (the octaves above G9).
        let g = [(0, 115, true)];
        let p = played(five(Direction::Up), 128, 6000 * 10, &g);
        assert_eq!(
            on_keys(&p),
            [115, 127, 127, 127, 127, 115, 127, 127, 127, 127]
        );
        let p = played(five(Direction::Down), 128, 6000 * 5, &g);
        assert_eq!(on_keys(&p), [127, 127, 127, 127, 115]);
        let p = played(five(Direction::UpDown), 128, 6000 * 9, &g);
        assert_eq!(on_keys(&p), [115, 127, 127, 127, 127, 127, 127, 127, 115]);
        let p = played(five(Direction::Order), 128, 6000 * 5, &g);
        assert_eq!(on_keys(&p), [115, 127, 127, 127, 127]);
        let p = played(five(Direction::Random), 128, 6000 * 40, &g);
        let k = on_keys(&p);
        assert!(k.windows(2).all(|w| w[0] != w[1]), "{k:?}");
        assert!(k.contains(&115) && k.contains(&127));
        // G9 alone: every step G9, none silent.
        for d in [
            Direction::Up,
            Direction::Down,
            Direction::UpDown,
            Direction::Random,
            Direction::Order,
        ] {
            let p = played(five(d), 128, 6000 * 10, &[(0, 127, true)]);
            assert_eq!(on_keys(&p), [127; 10], "{d:?}");
            balanced(&played(
                five(d),
                128,
                6000 * 10,
                &[(0, 127, true), (59_000, 127, false)],
            ));
            // A chord near the top: as many steps as with room, none past 127.
            let chord = [(0, 100, true), (0, 104, true), (0, 107, true)];
            let p = played(five(d), 128, 6000 * 30, &chord);
            assert_eq!(on_keys(&p).len(), 30, "{d:?}");
            assert!(on_keys(&p).iter().all(|&k| k <= 127));
            if d == Direction::Random {
                let k = on_keys(&p);
                assert!(k.windows(2).all(|w| w[0] != w[1]), "{k:?}");
            }
        }
        // With HOLD, the same: the chord let go, the pattern plays on.
        let s = Settings {
            latch: true,
            ..five(Direction::Up)
        };
        let p = played(s, 128, 6000 * 5, &[(0, 115, true), (10, 115, false)]);
        assert_eq!(on_keys(&p), [115, 127, 127, 127, 127]);
    }

    /// The shortest note is a floor at every RATE, tempo, GATE and SWING (the owner, 2026-10-19:
    /// "the gate in the arp should NEVER go below the minimum gate length to trigger a sound in
    /// the device"): every note, pressed to let go, is held at least `SHORTEST`, the keys let go
    /// at the end included; a step shorter than that overlaps the last note, or, the same key,
    /// ties into it; every step sounds (a key is down at each); nothing is stuck. One key, a
    /// chord, ORDER with a pitch two keys reach, and notes folded down from past 127.
    #[test]
    fn no_note_is_shorter_than_the_shortest() {
        let floor = SHORTEST * RATE - 1.0;
        let patterns: [(&str, Direction, u8, &[u8]); 4] = [
            ("one key", Direction::Up, 1, &[60]),
            ("chord", Direction::Up, 2, &[60, 64, 67]),
            ("order, C4 C5", Direction::Order, 2, &[60, 72]),
            ("folded", Direction::UpDown, 4, &[115]),
        ];
        let mut notes = 0;
        for (name, direction, octaves, keys) in patterns {
            for (label, beats) in RATES {
                for tempo in [30.0, 60.0, 97.0, 120.0, 180.0, 225.0, 240.0, 300.0, 1000.0] {
                    for gate in [0.0, 0.12, 0.5, 0.9, 0.99, 1.0] {
                        for swing in [0.0, 1.0] {
                            let s = Settings {
                                gate,
                                swing,
                                octaves,
                                step: beats,
                                ..arp(direction)
                            };
                            let step = beats * 60.0 / tempo * RATE;
                            let end = (step * 12.0) as usize + 7;
                            let mut events: Vec<(usize, u8, bool)> =
                                keys.iter().map(|&key| (0, key, true)).collect();
                            events.extend(keys.iter().map(|&key| (end, key, false)));
                            let p = played_with(s, 4096, end + 4000, &events, |r| {
                                r.tempo = tempo;
                            });
                            let what = format!(
                                "{name}, RATE {label}, {tempo} BPM, GATE {gate}, SWING {swing}"
                            );
                            balanced(&p);
                            notes += p.iter().filter(|e| e.2).count();
                            let mut down = [None::<usize>; 128];
                            for &(at, key, on) in &p {
                                if on {
                                    down[usize::from(key)] = Some(at);
                                } else if let Some(from) = down[usize::from(key)].take() {
                                    assert!(
                                        (at - from) as f64 >= floor,
                                        "{what}: {key} held {} samples: {p:?}",
                                        at - from
                                    );
                                }
                            }
                            // Every step: a key down just after it.
                            let mut n = 0;
                            loop {
                                let t = n as f64 * step
                                    + if n % 2 == 1 { swing * step / 2.0 } else { 0.0 };
                                let t = t.ceil() as usize;
                                if t >= end {
                                    break;
                                }
                                let mut d = [0i32; 128];
                                for &(_, key, on) in p.iter().filter(|e| e.0 <= t) {
                                    d[usize::from(key)] += if on { 1 } else { -1 };
                                }
                                let sounding = d.iter().any(|v| *v > 0);
                                assert!(sounding, "{what}: step {n} at {t} silent: {p:?}");
                                n += 1;
                            }
                        }
                    }
                }
            }
        }
        // (Every case played its dozen steps: 4 patterns, 13 RATEs, 9 tempos, 6 GATEs, 2 SWINGs.)
        assert!(notes > 4 * 13 * 9 * 6 * 2 * 8, "{notes} notes");
    }

    /// HOLD with ON off, a drone (the owner, 2026-10-19): the keys pass as played, held after
    /// they are let go; a key pressed while others are held is added; one pressed with none held
    /// starts a new chord, the old one let go first; HOLD off lets go of what it held (not of
    /// what is still held); nothing stuck, nothing let go twice.
    #[test]
    fn hold_with_the_arpeggiator_off_holds_the_chord_as_played() {
        let hold = Settings {
            latch: true,
            ..Settings::default()
        };
        let mut r = Exact::new(hold, 128);
        r.run(
            2000,
            &[
                (0, 60, true),
                (5, 64, true),
                (100, 60, false),
                (100, 64, false),
            ],
        );
        assert_eq!(r.played, [(0, 60, true), (5, 64, true)], "held as played");
        // A new chord: 60 and 64 let go as 62 is pressed; 65 added while 62 is held; 62 pressed
        // again (it is held by HOLD) let go and pressed.
        r.run(
            4000,
            &[
                (2000, 62, true),
                (2100, 65, true),
                (2200, 62, false),
                (2300, 62, true),
                (2400, 62, false),
            ],
        );
        assert_eq!(
            r.played[2..],
            [
                (2000, 60, false),
                (2000, 64, false),
                (2000, 62, true),
                (2100, 65, true),
                (2300, 62, false),
                (2300, 62, true),
            ]
        );
        // HOLD off with 65 still held: 62 let go, 65 sounds on as played.
        r.run(4500, &[]);
        r.s.latch = false;
        r.run(5000, &[]);
        assert_eq!(r.played.last(), Some(&(4500, 62, false)));
        assert!(!r.played[r.played.len() - 3..].contains(&(4500, 65, false)));
        r.run(6000, &[(5500, 65, false)]);
        balanced(&r.played);
    }

    /// HOLD's drone and the arpeggiator hand the chord over: ON on, the drone's keys let go and
    /// arpeggiated; ON off again, the chord held as played again; the sustain pedal keeps a chord
    /// let go under it one chord; cleared, nothing more. Nothing stuck at any point.
    #[test]
    fn the_drone_and_the_arpeggiator_hand_the_chord_over() {
        let hold = Settings {
            latch: true,
            ..Settings::default()
        };
        let mut r = Exact::new(hold, 128);
        r.run(
            1000,
            &[
                (0, 60, true),
                (0, 64, true),
                (10, 60, false),
                (10, 64, false),
            ],
        );
        r.s.on = true;
        r.run(13_000, &[]);
        assert_eq!(&r.played[2..4], [(1000, 60, false), (1000, 64, false)]);
        assert_eq!(&ons(&r.played)[2..4], [(1000, 60), (7000, 64)]);
        r.s.on = false;
        r.run(14_000, &[]);
        assert!(r.played.contains(&(13_000, 60, true)) && r.played.contains(&(13_000, 64, true)));
        // The pedal: down, a chord let go under it and another key pressed is one chord.
        let mut out = |_: u8, _: bool| {};
        r.arp.pedal(true, 0, &mut out);
        r.run(
            16_000,
            &[(14_100, 67, true), (14_200, 67, false), (14_300, 69, true)],
        );
        assert!(r.played.contains(&(14_100, 60, false)), "a new chord");
        assert!(
            !r.played.contains(&(14_300, 67, false)),
            "one chord under the pedal"
        );
        r.arp.pedal(false, 0, &mut out);
        r.run(17_000, &[(16_500, 69, false)]);
        r.s.latch = false;
        r.run(18_000, &[]);
        balanced(&r.played);
        // Cleared while it holds: nothing let go after (the engine let go itself).
        r.s.latch = true;
        r.run(19_000, &[(18_100, 60, true), (18_200, 60, false)]);
        r.arp.clear();
        let n = r.played.len();
        r.s.latch = false;
        r.run(20_000, &[]);
        assert_eq!(r.played.len(), n);
    }
}
