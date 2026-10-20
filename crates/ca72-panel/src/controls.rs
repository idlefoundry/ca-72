//! The panel's controls: where each is, what it looks like, and the parameter it operates.

/// A knob's printed dial.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dial {
    /// 0 to 10.
    Ten,
    /// TUNE: –2 to 2 (semitones, as printed).
    Tune,
    /// CUTOFF FREQUENCY: –4 to 4.
    Cutoff,
    /// An oscillator's FREQUENCY: –7 to 7 semitones.
    Freq,
    /// The contours' ATTACK and DECAY TIME, in time.
    Time,
    /// MODULATION MIX: 0 to 10 between OSC. 3 and NOISE.
    ModMix,
    /// The strip's (A6): VOICES, 2 to 10, a step a voice.
    Voices,
    /// DETUNE, DOUBLE's: 0 to 100 cents.
    Detune,
    /// DRIVE: 0 to 24 dB.
    Drive,
    /// LEVEL: -30 to +12 dB.
    Level,
    /// The arpeggiator's (decisions.md R-ARP): RATE, its note values 2 (a half note) to 32, the
    /// straight ones numbered, the dotted and triplet ones between them unnumbered ticks.
    ArpRate,
    /// OCTAVES, 1 to 5.
    ArpOctaves,
    /// GATE and SWING: 0 to 100 %.
    Percent,
    /// An oscillator's FREQUENCY while it is timed (decisions.md R-OSC-TIMED): its DIVISION's
    /// eighteen note values, 4 and 2 bars (4×, 2×) and 1 bar to a thirty-second (1 to 32), the
    /// dotted and triplet ones between them unnumbered ticks.
    Timed,
}

impl Dial {
    /// How many steps its travel is in, for a knob of discrete positions (it goes from one to
    /// the next, never between them: the owner, 2026-10-10); none for one that turns smoothly.
    pub fn steps(self) -> Option<usize> {
        match self {
            Dial::Voices => Some(8),
            Dial::ArpRate => Some(12),
            Dial::ArpOctaves => Some(4),
            Dial::Timed => Some(TIMED_STEPS),
            _ => None,
        }
    }
}

/// The timed dial's steps: its eighteen note values' seventeen intervals (R-OSC-TIMED).
pub const TIMED_STEPS: usize = 17;

/// The time dial's labelled marks: angle in degrees and milliseconds (R-KNOB-ENTRY).
pub const TIME_MARKS: [(f64, f64); 6] = [
    (-120.0, 10.0),
    (-90.0, 200.0),
    (-30.0, 600.0),
    (30.0, 1000.0),
    (67.0, 5000.0),
    (108.0, 10000.0),
];

/// A selector's legends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Legends {
    Ranges,
    /// Oscillators 1 and 2's waveforms.
    Waves12,
    /// Oscillator 3's (a reverse sawtooth second).
    Waves3,
}

/// A rocker's colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Colour {
    Blue,
    Red,
    Black,
    Ivory,
}

/// Where a rocker's "on" end is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orient {
    Right,
    Top,
    Bottom,
}

/// A wheel's mark: the PITCH wheel's line, the MODULATION wheel's dot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Line,
    Dot,
}

/// What a control is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Knob {
        dial: Dial,
        big: bool,
    },
    /// Six positions (RANGE, WAVEFORM).
    Selector(Legends),
    /// A switch, `w` by `h` as seen.
    Rocker {
        w: f64,
        h: f64,
        colour: Colour,
        orient: Orient,
    },
    /// A three-position toggle (QUALITY, NOTE PRIORITY), `w` by `h` as seen: its lever down
    /// (0), out (0.5) or up (1); `across`, on its side, left (0), out or right (1).
    Toggle {
        w: f64,
        h: f64,
        across: bool,
    },
    /// `detent`: centred (PITCH); `span`: its turn over its travel, in degrees.
    Wheel {
        mark: Mark,
        detent: bool,
        span: f64,
    },
    /// A small amber key lit from inside while on (`art::KEY` in size), the note value its
    /// oscillator is timed at printed beside it: an oscillator's TIMED (decisions.md
    /// R-OSC-TIMED). It flips at a click, as a switch.
    Key,
}

impl Kind {
    /// Whether every fraction of a drag or scroll can change the value, rather than only
    /// whole positions. VOICES and the arpeggiator's RATE and OCTAVES have a knob's shape but
    /// discrete options.
    pub fn is_continuous(&self) -> bool {
        matches!(self, Self::Knob { dial, .. } if dial.steps().is_none())
            || matches!(self, Self::Wheel { .. })
    }
}

/// The part of the drawing a control's position is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// The panel, its origin at its top left corner.
    Panel,
    /// The left hand controller's, its origin where its column's was, now on the strip
    /// (`strip::LEFT_HAND`).
    Column,
    /// The strip's, its origin the drawing's.
    Strip,
    /// The arpeggiator's drawer's (`strip::arp`), its origin the drawing's with the drawer
    /// open: drawn by the drawer, found only while it is open.
    Arp,
}

/// A control of the panel, operating the parameter of the same function.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Control {
    /// Its parameter's id.
    pub param: &'static str,
    /// Its name, as its tip gives it.
    pub label: &'static str,
    pub place: Place,
    /// Its centre in its place.
    pub x: f64,
    pub y: f64,
    pub kind: Kind,
}

impl Control {
    /// Its centre in the drawing.
    pub fn centre(&self) -> (f64, f64) {
        let (ox, oy) = match self.place {
            Place::Panel => (crate::art::COL, crate::art::TOP),
            Place::Column => crate::strip::LEFT_HAND,
            Place::Strip | Place::Arp => (0.0, 0.0),
        };
        (ox + self.x, oy + self.y)
    }
}

const fn knob(param: &'static str, label: &'static str, x: f64, y: f64, dial: Dial) -> Control {
    Control {
        param,
        label,
        place: Place::Panel,
        x,
        y,
        kind: Kind::Knob { dial, big: false },
    }
}

/// A knob of the arpeggiator's drawer, the `at`th of its units (`strip::arp::UNITS`).
const fn arp_knob(param: &'static str, label: &'static str, at: usize, dial: Dial) -> Control {
    Control {
        param,
        label,
        place: Place::Arp,
        x: crate::strip::arp::UNITS[at] + crate::strip::UNIT_KNOB,
        y: crate::strip::arp::ROW,
        kind: Kind::Knob { dial, big: false },
    }
}

/// A knob on the strip, at its place there (`strip::KNOBS`).
const fn strip_knob(param: &'static str, label: &'static str, at: usize, dial: Dial) -> Control {
    let (_, x, y) = crate::strip::KNOBS[at];
    Control {
        param,
        label,
        place: Place::Strip,
        x,
        y,
        kind: Kind::Knob { dial, big: false },
    }
}

const fn big(param: &'static str, label: &'static str, x: f64, y: f64) -> Control {
    Control {
        param,
        label,
        place: Place::Panel,
        x,
        y,
        kind: Kind::Knob {
            dial: Dial::Freq,
            big: true,
        },
    }
}

const fn selector(
    param: &'static str,
    label: &'static str,
    x: f64,
    y: f64,
    legends: Legends,
) -> Control {
    Control {
        param,
        label,
        place: Place::Panel,
        x,
        y,
        kind: Kind::Selector(legends),
    }
}

#[allow(clippy::too_many_arguments)]
const fn rocker(
    param: &'static str,
    label: &'static str,
    place: Place,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    colour: Colour,
    orient: Orient,
) -> Control {
    Control {
        param,
        label,
        place,
        x,
        y,
        kind: Kind::Rocker {
            w,
            h,
            colour,
            orient,
        },
    }
}

const fn switch(
    param: &'static str,
    label: &'static str,
    x: f64,
    y: f64,
    colour: Colour,
) -> Control {
    rocker(
        param,
        label,
        Place::Panel,
        x,
        y,
        108.0,
        50.0,
        colour,
        Orient::Right,
    )
}

/// An oscillator's TIMED key (R-OSC-TIMED), centred under its RANGE, or its PWM key
/// (R-VARIABLE-RECT), centred under its WAVEFORM.
const fn key(param: &'static str, label: &'static str, x: f64, y: f64) -> Control {
    Control {
        param,
        label,
        place: Place::Panel,
        x,
        y,
        kind: Kind::Key,
    }
}

/// How far under its RANGE's centre an oscillator's TIMED key is (the mock-ups', the owner's
/// "under the oscillator range selector", 2026-10-20), and its PWM key under its WAVEFORM's.
pub const KEY_BELOW_RANGE: f64 = 84.0;

/// The MIXER's rows.
const ROWS: [f64; 5] = [170.0, 274.0, 377.0, 481.0, 583.0];

/// Every control, in the order they are drawn.
pub const CONTROLS: [Control; 64] = [
    // CONTROLLERS.
    knob("tune", "TUNE", 229.0, 273.0, Dial::Tune),
    switch(
        "osc_mod",
        "OSCILLATOR MODULATION",
        444.0,
        269.0,
        Colour::Red,
    ),
    knob("glide", "GLIDE", 126.0, 480.0, Dial::Ten),
    knob("mod_mix", "MODULATION MIX", 332.0, 480.0, Dial::ModMix),
    // OSCILLATOR BANK.
    selector(
        "osc1_range",
        "OSCILLATOR-1 RANGE",
        617.0,
        163.0,
        Legends::Ranges,
    ),
    selector(
        "osc2_range",
        "OSCILLATOR-2 RANGE",
        617.0,
        370.0,
        Legends::Ranges,
    ),
    selector(
        "osc3_range",
        "OSCILLATOR-3 RANGE",
        617.0,
        576.0,
        Legends::Ranges,
    ),
    big("osc2_frequency", "OSCILLATOR-2 FREQUENCY", 820.0, 373.0),
    big("osc3_frequency", "OSCILLATOR-3 FREQUENCY", 820.0, 581.0),
    selector(
        "osc1_waveform",
        "OSCILLATOR-1 WAVEFORM",
        1021.0,
        163.0,
        Legends::Waves12,
    ),
    selector(
        "osc2_waveform",
        "OSCILLATOR-2 WAVEFORM",
        1021.0,
        370.0,
        Legends::Waves12,
    ),
    selector(
        "osc3_waveform",
        "OSCILLATOR-3 WAVEFORM",
        1021.0,
        576.0,
        Legends::Waves3,
    ),
    rocker(
        "osc3_control",
        "OSC. 3 CONTROL",
        Place::Panel,
        480.0,
        585.0,
        55.0,
        111.0,
        Colour::Red,
        Orient::Top,
    ),
    // MIXER: the oscillators' VOLUME at the left, EXTERNAL INPUT and NOISE at the right.
    knob(
        "osc1_volume",
        "OSCILLATOR-1 VOLUME",
        1220.0,
        ROWS[0],
        Dial::Ten,
    ),
    switch("osc1_on", "OSCILLATOR-1 ON", 1419.0, ROWS[0], Colour::Blue),
    knob(
        "ext_volume",
        "EXTERNAL INPUT VOLUME",
        1621.0,
        ROWS[1],
        Dial::Ten,
    ),
    switch("ext_on", "EXTERNAL INPUT ON", 1419.0, ROWS[1], Colour::Blue),
    knob(
        "osc2_volume",
        "OSCILLATOR-2 VOLUME",
        1220.0,
        ROWS[2],
        Dial::Ten,
    ),
    switch("osc2_on", "OSCILLATOR-2 ON", 1419.0, ROWS[2], Colour::Blue),
    knob("noise_volume", "NOISE VOLUME", 1621.0, ROWS[3], Dial::Ten),
    switch("noise_on", "NOISE ON", 1419.0, ROWS[3], Colour::Blue),
    knob(
        "osc3_volume",
        "OSCILLATOR-3 VOLUME",
        1220.0,
        ROWS[4],
        Dial::Ten,
    ),
    switch("osc3_on", "OSCILLATOR-3 ON", 1419.0, ROWS[4], Colour::Blue),
    rocker(
        "noise_type",
        "NOISE (WHITE or PINK)",
        Place::Panel,
        1766.0,
        481.0,
        44.0,
        114.0,
        Colour::Blue,
        Orient::Bottom,
    ),
    // Across MIXER and MODIFIERS: the filter's mode, modulation and keyboard control. FILTER
    // MODE, which the original does not have, at the column's head, where the hardware
    // reference has it (decisions.md R46): LO, or HI pressed.
    rocker(
        "filter_mode",
        "FILTER MODE (LO or HI)",
        Place::Panel,
        1907.0,
        169.0,
        108.0,
        48.0,
        Colour::Red,
        Orient::Right,
    ),
    switch(
        "filter_mod",
        "FILTER MODULATION",
        1907.0,
        291.0,
        Colour::Red,
    ),
    rocker(
        "keyboard_control_1",
        "KEYBOARD CONTROL 1",
        Place::Panel,
        1907.0,
        386.0,
        108.0,
        48.0,
        Colour::Red,
        Orient::Right,
    ),
    rocker(
        "keyboard_control_2",
        "KEYBOARD CONTROL 2",
        Place::Panel,
        1907.0,
        486.0,
        108.0,
        48.0,
        Colour::Red,
        Orient::Right,
    ),
    // MODIFIERS.
    knob("cutoff", "CUTOFF FREQUENCY", 2081.0, 170.0, Dial::Cutoff),
    knob("emphasis", "EMPHASIS", 2281.0, 170.0, Dial::Ten),
    knob(
        "contour_amount",
        "AMOUNT OF CONTOUR",
        2483.0,
        170.0,
        Dial::Ten,
    ),
    knob(
        "filter_attack",
        "FILTER CONTOUR ATTACK TIME",
        2080.0,
        358.0,
        Dial::Time,
    ),
    knob(
        "filter_decay",
        "FILTER CONTOUR DECAY TIME",
        2281.0,
        358.0,
        Dial::Time,
    ),
    knob(
        "filter_sustain",
        "FILTER CONTOUR SUSTAIN LEVEL",
        2483.0,
        358.0,
        Dial::Ten,
    ),
    knob(
        "loudness_attack",
        "LOUDNESS CONTOUR ATTACK TIME",
        2080.0,
        579.0,
        Dial::Time,
    ),
    knob(
        "loudness_decay",
        "LOUDNESS CONTOUR DECAY TIME",
        2281.0,
        579.0,
        Dial::Time,
    ),
    knob(
        "loudness_sustain",
        "LOUDNESS CONTOUR SUSTAIN LEVEL",
        2483.0,
        579.0,
        Dial::Ten,
    ),
    // OUTPUT.
    knob("volume", "MAIN OUTPUT VOLUME", 2680.0, 167.0, Dial::Ten),
    rocker(
        "main_output",
        "MAIN OUTPUT",
        Place::Panel,
        2853.0,
        165.0,
        110.0,
        48.0,
        Colour::Blue,
        Orient::Right,
    ),
    rocker(
        "a440",
        "A-440",
        Place::Panel,
        2682.0,
        375.0,
        112.0,
        50.0,
        Colour::Blue,
        Orient::Right,
    ),
    // The phones' VOLUME is the plug-in's FEEDBACK (decisions.md R8): the PHONES output
    // cabled into EXTERNAL INPUT, as players overdrive the instrument.
    knob(
        "feedback",
        "FEEDBACK (the phones' VOLUME: the output into EXTERNAL INPUT)",
        2680.0,
        580.0,
        Dial::Ten,
    ),
    // QUALITY (decisions.md R47, R48): a chrome toggle over POWER's lamp, under
    // ULTRA's: its lever up for ULTRA, out for HI, down for LO.
    Control {
        param: "quality",
        label: "QUALITY (ULTRA, HI or LO)",
        place: Place::Panel,
        x: crate::art::QUALITY_AT.0,
        y: crate::art::QUALITY_AT.1,
        kind: Kind::Toggle {
            w: crate::art::TOGGLE.0,
            h: crate::art::TOGGLE.1,
            across: false,
        },
    },
    // The left hand controller.
    rocker(
        "glide_on",
        "GLIDE",
        Place::Column,
        crate::art::LH_ROCKER,
        crate::art::LH_ROWS[0],
        82.0,
        38.0,
        Colour::Ivory,
        Orient::Right,
    ),
    rocker(
        "decay_on",
        "DECAY",
        Place::Column,
        crate::art::LH_ROCKER,
        crate::art::LH_ROWS[1],
        82.0,
        38.0,
        Colour::Ivory,
        Orient::Right,
    ),
    Control {
        param: "pitch_wheel",
        label: "PITCH",
        place: Place::Column,
        x: crate::art::LH_WHEELS[0],
        y: crate::art::WHEEL_Y,
        kind: Kind::Wheel {
            mark: Mark::Line,
            detent: true,
            span: 70.0,
        },
    },
    Control {
        param: "mod_wheel",
        label: "MODULATION",
        place: Place::Column,
        x: crate::art::LH_WHEELS[1],
        y: crate::art::WHEEL_Y,
        kind: Kind::Wheel {
            mark: Mark::Dot,
            detent: false,
            span: 76.0,
        },
    },
    // The strip's knobs (A6: decisions.md R44, R45, R49), the plug-in's own; INNER
    // last, so that every other control keeps its index.
    strip_knob("voices", "VOICES", 0, Dial::Voices),
    strip_knob("entropy", "ENTROPY", 1, Dial::Ten),
    strip_knob("spread", "WIDTH", 2, Dial::Ten),
    strip_knob("double", "DETUNE (DOUBLE)", 3, Dial::Detune),
    strip_knob("drive", "DRIVE", 4, Dial::Drive),
    strip_knob("level", "LEVEL", 5, Dial::Level),
    strip_knob("inner", "INNER", 6, Dial::Ten),
    // NOTE PRIORITY (decisions.md R50), the plug-in's own: QUALITY's toggle on its side,
    // the left hand's first row, over RETRIGGER, GLIDE and DECAY where their switches are, its
    // lever left for LOW, out for LAST, right for HIGH. Last, so that every other control keeps
    // its index.
    Control {
        param: "note_priority",
        label: "NOTE PRIORITY (LOW, LAST or HIGH)",
        place: Place::Column,
        x: crate::art::LH_ROCKER,
        y: crate::art::LH_PRIORITY,
        kind: Kind::Toggle {
            w: crate::art::TOGGLE.1,
            h: crate::art::TOGGLE.0,
            across: true,
        },
    },
    // RETRIGGER (R50), the plug-in's own: an ivory switch as GLIDE's and DECAY's, the
    // left hand's second row, over theirs. After NOTE PRIORITY, for the same reason.
    rocker(
        "retrigger",
        "RETRIGGER",
        Place::Column,
        crate::art::LH_ROCKER,
        crate::art::LH_RETRIGGER,
        82.0,
        38.0,
        Colour::Ivory,
        Orient::Right,
    ),
    // The arpeggiator's drawer's knobs (decisions.md R-ARP), the plug-in's own, after every
    // other, so that no control moves.
    arp_knob("arp_rate", "RATE", 0, Dial::ArpRate),
    arp_knob("arp_octaves", "OCTAVES", 1, Dial::ArpOctaves),
    arp_knob("arp_gate", "GATE", 2, Dial::Percent),
    arp_knob("arp_swing", "SWING", 3, Dial::Percent),
    // Oscillators 2's and 3's TIMED keys (decisions.md R-OSC-TIMED), each centred under its
    // own RANGE, after every other, so that no control moves. Oscillator 1 has none: it has no
    // FREQUENCY to turn into note values.
    key(
        "osc2_timed",
        "OSCILLATOR-2 TIMED",
        617.0,
        370.0 + KEY_BELOW_RANGE,
    ),
    key(
        "osc3_timed",
        "OSCILLATOR-3 TIMED",
        617.0,
        576.0 + KEY_BELOW_RANGE,
    ),
    // Each oscillator's PWM key (decisions.md R-VARIABLE-RECT), centred under its own
    // WAVEFORM, after every other, so that no control moves: on the panel only while its
    // oscillator's rectangle is VARIABLE.
    key(
        "osc1_pwm",
        "OSCILLATOR-1 PWM",
        1021.0,
        163.0 + KEY_BELOW_RANGE,
    ),
    key(
        "osc2_pwm",
        "OSCILLATOR-2 PWM",
        1021.0,
        370.0 + KEY_BELOW_RANGE,
    ),
    key(
        "osc3_pwm",
        "OSCILLATOR-3 PWM",
        1021.0,
        576.0 + KEY_BELOW_RANGE,
    ),
];

/// The panel's controls' history (decisions.md R-SKIN-PACKS), so that a skin pack's layout made
/// for one CA-72 works on later ones: the revision of the panel's controls this build has, and
/// each change since the first. A layout says which revision it was made for (`controls`):
/// a control added since is placed by the plug-in, a renamed one found by its old name, a
/// retired one passed over, and a control whose kind changed drawn in its own kind's look.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlSet {
    pub revision: u32,
    /// Controls added after the first revision: each with the revision it came in.
    pub added: &'static [(&'static str, u32)],
    /// Controls renamed: the old name, the new, and the revision of the change.
    pub renamed: &'static [(&'static str, &'static str, u32)],
    /// Controls removed: the name, and the revision it went in.
    pub retired: &'static [(&'static str, u32)],
    /// Controls whose kind changed (a switch become a toggle): the name, and the revision.
    pub kind_changed: &'static [(&'static str, u32)],
}

/// This build's panel controls: the third revision; oscillators 2's and 3's TIMED keys added in
/// the second (decisions.md R-OSC-TIMED), each oscillator's PWM key in the third
/// (R-VARIABLE-RECT). Never edit an entry once released: add to it (and raise `revision`) as
/// controls come, go, or change.
pub const CONTROL_SET: ControlSet = ControlSet {
    revision: 3,
    added: &[
        ("osc2_timed", 2),
        ("osc3_timed", 2),
        ("osc1_pwm", 3),
        ("osc2_pwm", 3),
        ("osc3_pwm", 3),
    ],
    renamed: &[],
    retired: &[],
    kind_changed: &[],
};

impl ControlSet {
    /// The revision a control came in (1 for the first's).
    pub fn since(&self, param: &str) -> u32 {
        self.added
            .iter()
            .find(|(p, _)| *p == param)
            .map_or(1, |(_, r)| *r)
    }

    /// A name a layout of revision `made_for` uses, as this build knows it: renamed since
    /// (`Some(new)`), or the same.
    pub fn current<'a>(&self, name: &'a str, made_for: u32) -> &'a str
    where
        'static: 'a,
    {
        let mut name = name;
        // (A chain of renames followed in order.)
        for (old, new, at) in self.renamed {
            if *old == name && *at > made_for {
                name = new;
            }
        }
        name
    }

    /// Whether `param` was retired after revision `made_for`.
    pub fn retired_since(&self, param: &str, made_for: u32) -> bool {
        self.retired
            .iter()
            .any(|(p, at)| *p == param && *at > made_for)
    }

    /// Whether `param`'s kind changed after revision `made_for`.
    pub fn kind_changed_since(&self, param: &str, made_for: u32) -> bool {
        self.kind_changed
            .iter()
            .any(|(p, at)| *p == param && *at > made_for)
    }
}

/// The index of the control operating `param`.
pub fn index(param: &str) -> Option<usize> {
    CONTROLS.iter().position(|c| c.param == param)
}

/// FEEDBACK's tip while it is silent ([`feedback_silent`]).
pub const FEEDBACK_SILENT: &str =
    "FEEDBACK (silent: switch EXTERNAL INPUT on in the MIXER and turn its VOLUME up)";

/// Whether FEEDBACK is silent at these values (normalized, by index in [`CONTROLS`]): it is
/// heard through the MIXER's EXTERNAL INPUT, so nothing goes back while that is switched off
/// or its VOLUME is at 0. Its knob is then dimmed, still turnable, and its tip says why (the
/// owner, 2026-10-02).
pub fn feedback_silent(values: &[f64; CONTROLS.len()]) -> bool {
    let at = |param| index(param).map_or(0.0, |i| values[i]);
    !(at("ext_on") >= 0.5 && at("ext_volume") > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feedback_is_silent_while_external_input_is_closed() {
        let mut v = [0.5; CONTROLS.len()];
        let (on, volume) = (
            index("ext_on").expect("a control"),
            index("ext_volume").expect("a control"),
        );
        assert!(!feedback_silent(&v));
        v[on] = 0.0;
        assert!(feedback_silent(&v));
        v[on] = 1.0;
        v[volume] = 0.0;
        assert!(feedback_silent(&v));
        v[volume] = 0.01;
        assert!(!feedback_silent(&v));
    }
}
