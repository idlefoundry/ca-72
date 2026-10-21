//! The LFO page and modulation in the editor (decisions.md R-LFO; docs/lfo.md): the page opened
//! from the rail's LFO key over the lower page; its scopes, orbs and MODULATES screens from the
//! engine's [`crate::lfo::Scope`] and [`crate::lfo::Patch`]; an assignment dragged from an OUT
//! jack to a knob; the knobs' rings, their hover pop-up, their ends dragged, Alt/Option-drag on a
//! knob, and the knobs' menus' heads; an LFO's colour and EDGES from its own menu.
//!
//! Everything here is over **modulation sources** ([`crate::lfo::Source`]), not LFOs as such:
//! a source has a colour ([`colour`]) and a label ([`label`]), and the patch's assignments are
//! by source. The macros (next) are a source each: give [`colour`] and [`label`] their arms,
//! a jack (or another handle) that starts a [`Patching`], and they take the cord, the rings, the
//! pop-up, the menus and the tips as they are.

use std::time::{Duration, Instant};

use ca72_panel::controls::{Place as ControlPlace, lfo_index};
use ca72_panel::modulation::{self as mods, Cord, KnobRings, ModScene, PopDial, Popup, Ring};
use ca72_panel::strip::lfo::{self as lfo_ui, Column, LfoScene, LfoTarget, TRACE_POINTS};
use ca72_panel::{CONTROLS, Target, art, interact};
use keyboard_types::Modifiers;
use nih_plug::prelude::*;

use super::{DOUBLE_CLICK, Editing, MENU_TEXT, Operated};
use crate::learning::{ModItem, Place, Row};
use crate::lfo::{self, Edges, LFOS, Phosphor, Reading, Source, Target as ModTarget};
use crate::params::WaveMode;
use crate::settings::{Change, Settings};

/// How often the scopes' orbs and the rings' points move while an LFO moves: at most 30 frames
/// a second (the owner's budget: "≤30 fps").
const ORB_FRAME: Duration = Duration::from_millis(33);
/// A depth this near 0 when its drag ends takes the assignment away (as a ring's end dragged
/// to the knob's pointer does).
const NO_DEPTH: f32 = 0.005;
/// How far a tip moved clear of the scopes keeps above their bezels' tops, and in from the
/// window's sides (panel units).
const SCOPE_CLEAR: f64 = 4.0;
/// Logical pixels a depth's drag up or down takes for a whole knob's travel (100 %), as a knob's
/// drag takes for its own.
const DEPTH_DRAG: f64 = 240.0;
/// A source's colour: an LFO's the computer's phosphor for it, a macro's the computer's colour
/// for it (decisions.md R-MACROS).
pub(super) fn colour(settings: &Settings, s: Source) -> [u8; 3] {
    match s {
        Source::Lfo(n) => settings.lfo_colors[n.min(LFOS - 1)].rgb(),
        Source::Macro(n) => settings.macro_colors[n.min(lfo::MACROS - 1)].rgb(),
    }
}

/// A source's label, as the pop-up, the tip and the menus name it: a macro's typed name where it
/// has one (BRIGHT), else MACRO n.
pub(super) fn label(patch: &lfo::Patch, s: Source) -> String {
    match s {
        Source::Lfo(n) => format!("LFO {}", n + 1),
        Source::Macro(n) => {
            let name = patch.name(n);
            if name.is_empty() {
                format!("MACRO {}", n + 1)
            } else {
                name
            }
        }
    }
}

/// Where a scope's orb is along its one cycle (0..1 from PHASE; decisions.md R-LFO): where the
/// LFO is in its cycle, its reading's `phase`. An ENV holds at its cycle's end: there once its
/// cycle is over (`assigned`, no longer moving) or never begun; one not assigned, as far
/// through it as its time since the note has it.
pub(super) fn orb_at(set: &lfo::Settings, r: &Reading, tempo: f64, assigned: bool) -> f64 {
    let through = f64::from(r.since) * set.hz_at(tempo);
    // (PHASE in f32 as the reading's phase is, so that a cycle's start reads 0, not just short
    // of 1.)
    let x = (f64::from(r.phase) - f64::from(set.phase as f32)).rem_euclid(1.0);
    if set.mode == lfo::Mode::Env {
        return if r.moving {
            // (Its last instant in f32 can read as the next cycle's start.)
            if x < 0.25 && through > 0.75 { 1.0 } else { x }
        } else if assigned || r.since <= 0.0 {
            1.0
        } else {
            through.min(1.0)
        };
    }
    if x > 1.0 - 1e-6 { 0.0 } else { x }
}

/// What an LFO's trace is drawn from (decisions.md R-LFO's corrections; the owner, 2026-10-20:
/// "the oscilloscope wave isn't changing to match the actual pattern that's happening at the
/// moment. The floating orb however is following the correct path"): its settings as its knobs
/// are set, and LEVEL's 1; a source moving its PHASE, its CONTINUOUS shape or its LEVEL, those
/// as the engine has them now (`r`), so that the orb rides on the line.
pub(super) fn drawn_settings(set: &lfo::Settings, r: &Reading) -> (lfo::Settings, f64) {
    if !r.moved {
        return (*set, 1.0);
    }
    (
        lfo::Settings {
            at: f64::from(r.shape),
            phase: f64::from(r.offset),
            ..*set
        },
        f64::from(r.level).clamp(0.0, 1.0),
    )
}

/// How finely a moved trace's samples are kept: a change smaller is under half a pixel of the
/// trace's height at the largest size, and draws nothing again.
const TRACE_STEP: f64 = 1.0 / 256.0;

/// The trace of one cycle from PHASE ([`TRACE_POINTS`] samples) at LEVEL `level`; a moved one's
/// (`moved`) kept to [`TRACE_STEP`].
pub(super) fn trace_of(set: &lfo::Settings, level: f64, moved: bool) -> Vec<f32> {
    (0..TRACE_POINTS)
        .map(|i| {
            let v = set.cycle_at(i as f64 / (TRACE_POINTS - 1) as f64) * level;
            if moved {
                ((v / TRACE_STEP).round() * TRACE_STEP) as f32
            } else {
                v as f32
            }
        })
        .collect()
}

/// An assignment being dragged from a source's jack: the pointer's place (the drawing's), and
/// whether it has left the jack (an LFO's cord let go on its own jack without, a click, gives
/// its LEVEL nothing: R-LFO's addendum).
#[derive(Debug)]
pub(super) struct Patching {
    pub(super) source: Source,
    pub(super) at: (f64, f64),
    pub(super) from: (f64, f64),
    pub(super) away: bool,
}

/// How far a cord's plug goes from its jack before it has left it (as the page finds the jack).
const JACK_LEFT: f64 = lfo_ui::JACK_R + 10.0;

/// How a depth is being dragged: up or down (the pop-up's dial, Alt/Option on the knob), or a
/// ring's end round the knob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DepthWay {
    Vertical,
    Ring,
}

/// A depth being dragged: whose, on what (and its control), from where (logical pixels) and
/// what it was.
#[derive(Debug)]
pub(super) struct DepthDrag {
    pub(super) source: Source,
    pub(super) target: ModTarget,
    pub(super) control: usize,
    pub(super) from: (f64, f64),
    pub(super) start: f32,
    pub(super) way: DepthWay,
}

/// The editor's modulation state.
#[derive(Debug, Default)]
pub(super) struct Modulating {
    /// The LFO page shows (over the lower page; with the window's lower part).
    pub(super) open: bool,
    /// The source last touched on its page: Alt/Option-drag edits its depth.
    pub(super) last: Option<Source>,
    pub(super) patching: Option<Patching>,
    pub(super) depth: Option<DepthDrag>,
    /// The knob whose pop-up shows, and its dial under the pointer.
    pub(super) popup: Option<usize>,
    pub(super) popup_hover: Option<usize>,
    /// The page's part under the pointer (its controls apart: `Editing::hover`).
    pub(super) hover: Option<LfoTarget>,
    /// The orbs' clock, and each LFO as last read.
    pub(super) due: Option<Instant>,
    pub(super) readings: [Reading; LFOS],
    pub(super) last_popup_press: Option<(Instant, usize)>,
}

/// An LFO's RATE knob: its Hz, or its DIVISION while timed (a value of its own, so that RATE's
/// place is kept for when it is off, as a timed oscillator's FREQUENCY: R-OSC-TIMED).
pub(super) struct LfoRate {
    pub(super) params: std::sync::Arc<crate::params::Ca72Params>,
    pub(super) n: usize,
}

impl LfoRate {
    fn now(&self) -> &dyn Operated {
        let q = self.params.lfo(self.n);
        if q.timed.value() {
            &q.division
        } else {
            &q.rate
        }
    }
}

impl Operated for LfoRate {
    fn value(&self) -> f32 {
        self.now().value()
    }
    fn default(&self) -> f32 {
        self.now().default()
    }
    fn text(&self, v: f32) -> String {
        self.now().text(v)
    }
    fn begin(&self, s: &ParamSetter<'_>) {
        self.now().begin(s);
    }
    fn set(&self, s: &ParamSetter<'_>, v: f32) {
        self.now().set(s, v);
    }
    fn end(&self, s: &ParamSetter<'_>) {
        self.now().end(s);
    }
}

/// An LFO's WAVEFORM switch, as an oscillator's (R-CONTINUOUS-WAVE): WAVEFORM while STEPPED;
/// CONTINUOUS, its shape along the switch's whole travel, WAVEFORM following it to the nearest
/// of the switch's positions.
pub(super) struct LfoWave {
    pub(super) params: std::sync::Arc<crate::params::Ca72Params>,
    pub(super) n: usize,
}

impl LfoWave {
    fn continuous(&self) -> bool {
        self.params.lfo(self.n).shape_mode.value() == WaveMode::Continuous
    }
}

impl Operated for LfoWave {
    fn value(&self) -> f32 {
        let q = self.params.lfo(self.n);
        if self.continuous() {
            q.shape.unmodulated_normalized_value()
        } else {
            q.waveform.unmodulated_normalized_value()
        }
    }
    fn default(&self) -> f32 {
        let q = self.params.lfo(self.n);
        if self.continuous() {
            q.shape.default_normalized_value()
        } else {
            q.waveform.default_normalized_value()
        }
    }
    fn text(&self, v: f32) -> String {
        let q = self.params.lfo(self.n);
        if self.continuous() {
            q.shape.normalized_value_to_string(v, true)
        } else {
            q.waveform.normalized_value_to_string(v, true)
        }
    }
    fn begin(&self, s: &ParamSetter<'_>) {
        let q = self.params.lfo(self.n);
        s.begin_set_parameter(&q.waveform);
        if self.continuous() {
            s.begin_set_parameter(&q.shape);
        }
    }
    fn set(&self, s: &ParamSetter<'_>, v: f32) {
        let q = self.params.lfo(self.n);
        if !self.continuous() {
            s.set_parameter_normalized(&q.waveform, v);
            return;
        }
        let at = f64::from(v) * interact::CONTINUOUS_PLACES;
        let k = (at.round().max(0.0) as usize).min(5);
        s.set_parameter_normalized(&q.waveform, k as f32 / 5.0);
        s.set_parameter_normalized(&q.shape, v);
    }
    fn end(&self, s: &ParamSetter<'_>) {
        let q = self.params.lfo(self.n);
        s.end_set_parameter(&q.waveform);
        if self.continuous() {
            s.end_set_parameter(&q.shape);
        }
    }
}

/// The LFO and part (`LFO_PARTS`' word, or another of its parameters') of an LFO's parameter's
/// ID, `lfoN_<part>`.
pub(super) fn lfo_part(id: &str) -> Option<(usize, &str)> {
    let rest = id.strip_prefix("lfo")?;
    let (n, part) = rest.split_once('_')?;
    let n: usize = n.parse().ok()?;
    (1..=LFOS).contains(&n).then_some((n - 1, part))
}

/// LFO `n`'s parameter `part` as the editor operates it (its RATE and WAVEFORM apart: theirs).
pub(super) fn lfo_param<'a>(
    p: &'a crate::params::Ca72Params,
    n: usize,
    part: &str,
) -> Option<&'a dyn Operated> {
    let (q, keys) = (p.lfo(n), p.lfo_keys(n));
    Some(match part {
        "waveform" => &q.waveform,
        "shape_mode" => &q.shape_mode,
        "shape" => &q.shape,
        "rate" => &q.rate,
        "timed" => &q.timed,
        "division" => &q.division,
        "phase" => &q.phase,
        "rise" => &q.rise,
        "mode" => &q.mode,
        "mono" => &q.mono,
        "polarity" => &keys.polarity,
        "reverse" => &keys.reverse,
        "on" => &keys.on,
        "level" => &p.lfo_level(n).level,
        _ => return None,
    })
}

/// The standalone's TEMPO screen as a control the sources take (decisions.md R-LFO): past the
/// panel's (`CONTROLS` has none for it), so that the cord, the rails, the pop-up, the menus and
/// a typed depth take it as they take a knob, by this number.
pub(super) const TEMPO_CONTROL: usize = usize::MAX;

/// A control's name as the menus and the tips give it: TEMPO's screen's too.
pub(super) fn control_label(i: usize) -> &'static str {
    if i == TEMPO_CONTROL {
        "TEMPO"
    } else {
        CONTROLS.get(i).map_or("", |c| c.label)
    }
}

/// The LFO whose LEVEL control `i` is (its OUT jack: R-LFO's addendum), if it is one.
pub(super) fn level_of_control(i: usize) -> Option<usize> {
    (0..LFOS).find(|&n| lfo_index(n, ca72_panel::controls::LFO_LEVEL_PART) == i)
}

/// The control showing modulation target `t`: its knob's (an oscillator's shape its WAVEFORM).
pub(super) fn control_of(t: ModTarget) -> Option<usize> {
    let id = match t {
        ModTarget::Osc1Shape => "osc1_waveform",
        ModTarget::Osc2Shape => "osc2_waveform",
        ModTarget::Osc3Shape => "osc3_waveform",
        // (An LFO's CONTINUOUS shape its WAVEFORM's, as an oscillator's: R-LFO's addendum.)
        ModTarget::LfoShape(n) => {
            return Some(lfo_index(n.min(LFOS - 1), 0));
        }
        ModTarget::Tempo => return Some(TEMPO_CONTROL),
        ModTarget::Fx(n) => return Some(super::fx_pins::FIRST + n),
        t => t.id(),
    };
    ca72_panel::controls::index(id)
}

/// What an LFO's MODULATES screen reads (its ten characters): the one knob and its depth, the
/// count, or NOT IN USE (dim); MUTED (dim) while its ON is off, whatever it modulates (R-LFO's
/// corrections).
pub(super) fn screen_text(
    patch: &lfo::Patch,
    n: usize,
    on: bool,
    standalone: bool,
) -> (String, bool) {
    if !on {
        return ("MUTED".into(), false);
    }
    let a: Vec<_> = patch
        .assignments(Source::Lfo(n))
        .into_iter()
        .flatten()
        .collect();
    match a.len() {
        0 => ("NOT IN USE".into(), false),
        // (TEMPO in a plug-in: kept, doing nothing, dim; its tip and menu say why.)
        1 if a[0].target == ModTarget::Tempo && !standalone => ("TEMPO".into(), false),
        1 => {
            let s = a[0].target.short();
            let d = format!("{:+.0}", f64::from(a[0].depth) * 100.0);
            if s.len() + 1 + d.len() <= 10 {
                (format!("{s} {d}"), true)
            } else {
                (s.to_owned(), true)
            }
        }
        k => (format!("{k} KNOBS"), true),
    }
}

impl Editing {
    /// Whether the LFO page shows (with the window's lower part).
    pub(super) fn page_shows(&self) -> bool {
        self.modulating.open && self.expanded()
    }

    /// Whether the keyboard shows (the LFO page is in its place while it shows).
    pub(super) fn keyboard_shown(&self) -> bool {
        self.playing.visible && !self.modulating.open
    }

    /// The rail's LFO key: the page shown, or (lit) put away for the page under it; the lower
    /// part shown first if it is not, the presets' drawer put away for it.
    pub(super) fn toggle_lfo_page(&mut self) {
        if !self.expanded() {
            self.toggle_panel();
            if !self.expanded() {
                return;
            }
            self.modulating.open = true;
        } else if self.browser.open {
            self.browser.show(false, &self.params);
            self.follow_drawer();
            self.modulating.open = true;
        } else {
            self.modulating.open = !self.modulating.open;
        }
        self.playing.release_pointer();
        self.typing = None;
        self.hover = None;
        self.strip_scene.hover = None;
        self.modulating.hover = None;
        self.modulating.popup = None;
        self.composed = None;
    }

    /// The modulation target control `i` is now, if it can take one (TEMPO's screen, the
    /// standalone's: [`TEMPO_CONTROL`]).
    pub(super) fn mod_target(&self, i: usize) -> Option<ModTarget> {
        if let Some(n) = super::fx_pins::target(i) {
            return self.fx_view.pin_info(n).map(|_| ModTarget::Fx(n));
        }
        if i == TEMPO_CONTROL {
            return self.playing.standalone.then_some(ModTarget::Tempo);
        }
        CONTROLS
            .get(i)
            .and_then(|c| lfo::target_of(&self.params, c.param))
    }

    /// Whether control `i` is in view: the panel's always, the strip's while the strip's page
    /// shows (neither the keyboard nor the LFO page over it), the LFO page's while it shows (its
    /// knobs and jacks take the sources too: R-LFO's addendum).
    fn in_view(&self, i: usize) -> bool {
        if let Some(n) = super::fx_pins::target(i) {
            return self.fx_pins_show()
                && !self.fx_view.list_open()
                && self.fx_view.pin_info(n).is_some();
        }
        if i == TEMPO_CONTROL {
            return self.playing.standalone;
        }
        match CONTROLS.get(i).map(|c| c.place) {
            Some(ControlPlace::Panel) => true,
            Some(ControlPlace::Strip) => {
                self.expanded() && !self.modulating.open && !self.playing.visible
            }
            Some(ControlPlace::Lfo) => self.page_shows(),
            _ => false,
        }
    }

    /// The control under (`x`, `y`) a source can be given to: the LFO page's while it shows and
    /// the pointer is over it, else the panel's and the strip's.
    pub(super) fn mod_hit(&self, x: f64, y: f64) -> Option<Target> {
        if self.in_fx((x, y)) {
            return self
                .fx_pin_hit((x, y))
                .map(|n| Target::Control(super::fx_pins::FIRST + n));
        }
        if self.tempo_hovered((x, y)) {
            return Some(Target::Control(TEMPO_CONTROL));
        }
        if self.page_shows() && lfo_ui::inside(x, y) {
            interact::hit_lfo(self.renderer.layout(), x, y)
        } else {
            self.hit(x, y)
        }
    }

    /// Each source on `t` and its depth, in the sources' order.
    pub(super) fn sources_on(&self, t: ModTarget) -> Vec<(Source, f32)> {
        Source::all()
            .filter_map(|s| self.params.lfo_patch.depth(s, t).map(|d| (s, d)))
            .collect()
    }

    /// Whether (`x`, `y`) is on the standalone's TEMPO screen.
    pub(super) fn tempo_hovered(&self, (x, y): (f64, f64)) -> bool {
        self.playing.standalone && art::tempo_hit(self.renderer.arrangement(), x, y)
    }

    /// The TEMPO screen's window (left, top, width, height), and where its rails hang from (its
    /// surround's foot, its middle) and half their length.
    fn tempo_place(&self) -> ((f64, f64, f64, f64), (f64, f64), f64) {
        let (x, y, w, h) = art::tempo_window(self.renderer.arrangement().plate_at());
        let o = art::TEMPO_SURROUND;
        ((x, y, w, h), (x + w / 2.0, y + h + o), w / 2.0)
    }

    /// Control `i`'s knob's travel now (0..1, its pointer: a CONTINUOUS WAVEFORM's its shape's;
    /// TEMPO's as it is set, along its range).
    fn travel_of(&self, i: usize) -> f64 {
        if let Some(n) = super::fx_pins::target(i) {
            return self.fx_view.pin_info(n).map_or(0.0, |p| p.value);
        }
        if i == TEMPO_CONTROL {
            return lfo::tempo_travel(self.params.tempo());
        }
        if self.continuous_switch(i) {
            self.value_of(i)
        } else {
            self.scene.values[i]
        }
    }

    /// A source's present output (-1..1; a macro's 0..1, its knob's) and whether it moves (a
    /// macro's point always shows).
    fn reading_of(&self, s: Source) -> (f64, bool) {
        match s {
            Source::Lfo(n) if n < LFOS => {
                let r = self.modulating.readings[n];
                (f64::from(r.value), r.moving)
            }
            Source::Macro(n) if n < lfo::MACROS => (self.macro_value(n), true),
            _ => (0.0, false),
        }
    }

    /// The orbs' clock: each LFO read again at most 30 times a second while the page shows.
    pub(super) fn read_lfos(&mut self, now: Instant) {
        if !self.page_shows() {
            return;
        }
        if self.modulating.due.is_some_and(|d| now < d) {
            return;
        }
        self.modulating.readings = std::array::from_fn(|n| self.params.lfo_scope.read(n));
        // (Due a thirtieth after it was due, not after now, and never before now.)
        let next = self.modulating.due.map_or(now, |d| d + ORB_FRAME);
        self.modulating.due = Some(next.max(now));
    }

    /// The LFO page's scene: each column's controls as the panel's scene has them (a drag's
    /// where the drag has it), its trace from its settings, its orb from its last reading, what
    /// MODULATES reads, its colour.
    pub(super) fn lfo_scene_now(&self) -> LfoScene {
        // (The host's tempo as the audio thread last played at it, else the standalone's.)
        let tempo = self.meters.tempo().unwrap_or_else(|| self.params.tempo());
        let mut s = LfoScene::default();
        for (n, col) in s.columns.iter_mut().enumerate() {
            let q = self.params.lfo(n);
            let set = lfo::settings(&self.params, n, self.params.lfo_patch.edges(n));
            let r = self.modulating.readings[n];
            // (One cycle across the scope from PHASE: the owner, 2026-10-20, "it should be 1";
            // a source moving its PHASE, shape or LEVEL, the cycle as the engine plays it now.)
            let (drawn, level) = drawn_settings(&set, &r);
            let trace = trace_of(&drawn, level, r.moved);
            let assigned = (self.params.lfo_patch.assignments(Source::Lfo(n)).iter())
                .flatten()
                .any(|a| a.depth != 0.0);
            let x = orb_at(&drawn, &r, tempo, assigned);
            // (Moving, at its output, RISE's fade and EDGES in it; still, on the trace.)
            let (y, gain) = if r.moving {
                (f64::from(r.value), f64::from(r.rise).clamp(0.0, 1.0))
            } else {
                (drawn.cycle_at(x) * level, 1.0)
            };
            let v = |k: usize| self.scene.values[lfo_index(n, k)];
            let continuous = q.shape_mode.value() == WaveMode::Continuous;
            let keys = self.params.lfo_keys(n);
            let on = keys.on.value();
            let (screen, in_use) =
                screen_text(&self.params.lfo_patch, n, on, self.playing.standalone);
            *col = Column {
                colour: colour(&self.settings, Source::Lfo(n)),
                wave: if continuous {
                    f64::from(q.waveform.unmodulated_normalized_value())
                } else {
                    v(0)
                },
                continuous,
                shape_at: if continuous {
                    v(0) * interact::CONTINUOUS_PLACES
                } else {
                    0.0
                },
                rate: v(1),
                timed: q.timed.value(),
                division: q.division.value().to_index(),
                phase: v(3),
                rise: v(4),
                mode: v(5),
                mono: q.mono.value(),
                trace,
                // (Muted and faded out: none.)
                orb: (on || r.moving).then_some((x, y)),
                uni: keys.polarity.value() == lfo::Polarity::Uni,
                on,
                orb_rise: gain,
                screen,
                in_use,
                live: r.moved,
            };
        }
        s
    }

    /// The modulation overlay's scene: the rings on the knobs in view (all while the LFO page
    /// shows or an assignment is dragged, else only the one under the pointer's; every macro's
    /// while the macros show: decisions.md R-MACROS), a dragged assignment's cord and targets,
    /// the hover pop-up.
    pub(super) fn mod_scene_now(&self) -> ModScene {
        let page = self.page_shows();
        let macros = self.macros_show();
        let patching = self.modulating.patching.as_ref();
        let hovered = match self.hover {
            Some(Target::Control(i) | Target::Legend(i, _)) => Some(i),
            _ => None,
        };
        let dragged = self.modulating.depth.as_ref().map(|d| d.control);
        let mut knobs = Vec::new();
        let mut targets = Vec::new();
        let mut over = None;
        let (mut boxes, mut over_box, mut under_room) = (Vec::new(), None, false);
        let pointer = patching.map(|p| p.at);
        let under = pointer.and_then(|(x, y)| match self.mod_hit(x, y) {
            Some(Target::Control(i) | Target::Legend(i, _)) => Some(i),
            _ => None,
        });
        // (The panel's knobs, then the standalone's TEMPO screen: R-LFO.)
        let tempo = self.playing.standalone.then_some(TEMPO_CONTROL);
        for i in (0..CONTROLS.len()).chain(tempo).chain(
            self.fx_view
                .pins()
                .iter()
                .map(|p| super::fx_pins::FIRST + p.target),
        ) {
            if !self.in_view(i) {
                continue;
            }
            let Some(t) = self.mod_target(i) else {
                continue;
            };
            // Where it is: a knob's centre, skirt and sweep; TEMPO's screen and its rails.
            let (place, sweep, bar, screen) = if let Some(n) = super::fx_pins::target(i) {
                let Some(at) = self.fx_pin_at(n) else {
                    continue;
                };
                ((at, art::STD.skirt), (-150.0, 150.0), None, None)
            } else if i == TEMPO_CONTROL {
                let (window, foot, half) = self.tempo_place();
                ((foot, 0.0), (-150.0, 150.0), Some(half), Some(window))
            } else {
                let c = self.renderer.arrangement().control(i);
                let place = (c.centre(), art::skirt(&c.kind));
                (place, art::sweep(&c.kind), None, None)
            };
            if let Some(p) = patching {
                let room = self.params.lfo_patch.depth(p.source, t).is_some()
                    || self.params.lfo_patch.count(p.source) < lfo::MOST;
                if room && self.mod_live(i) {
                    under_room |= under == Some(i);
                    match (screen, under == Some(i)) {
                        (Some(w), true) => over_box = Some(w),
                        (Some(w), false) => boxes.push(w),
                        (None, true) => over = Some(place),
                        (None, false) => targets.push(place),
                    }
                }
            }
            let shown = super::fx_pins::target(i).is_some()
                || page
                || patching.is_some()
                || hovered == Some(i)
                || self.modulating.popup == Some(i)
                || dragged == Some(i);
            if !shown && !macros {
                continue;
            }
            let travel = self.travel_of(i);
            let mut on = self.sources_on(t);
            // (The macros showing: their rings alone where nothing else shows them.)
            if !shown {
                on.retain(|(s, _)| s.macro_n().is_some());
            }
            // (A drop's preview: the depth it would be given.)
            if let Some(p) = patching
                && under == Some(i)
                && !on.iter().any(|(s, _)| *s == p.source)
                && under_room
            {
                on.push((p.source, lfo::DEFAULT_DEPTH));
                on.sort_by_key(|(s, _)| s.index());
            }
            if on.is_empty() {
                continue;
            }
            let rings = on
                .iter()
                .map(|&(s, d)| {
                    let (value, moving) = self.reading_of(s);
                    let one_way = self.one_way(s);
                    Ring {
                        colour: colour(&self.settings, s),
                        depth: f64::from(d),
                        point: ((page || s.macro_n().is_some()) && moving)
                            .then(|| (travel + f64::from(d) * value).clamp(0.0, 1.0)),
                        one_way,
                        dim: self.muted(s) || !self.mod_live(i),
                    }
                })
                .collect();
            knobs.push(KnobRings {
                control: i,
                at: place.0,
                skirt: place.1,
                sweep,
                travel,
                rings,
                bar,
            });
        }
        let cord = patching.and_then(|p| {
            let from = match p.source {
                Source::Lfo(n) if n < LFOS => lfo_ui::places(n).jack,
                Source::Macro(n) if n < lfo::MACROS => ca72_panel::strip::macros::jack_at(n),
                _ => return None,
            };
            Some(Cord {
                from,
                to: p.at,
                colour: colour(&self.settings, p.source),
                targets,
                over,
                boxes,
                over_box,
            })
        });
        let popup = self
            .modulating
            .popup
            .filter(|_| patching.is_none())
            .and_then(|i| {
                let t = self.mod_target(i)?;
                let on = self.sources_on(t);
                if on.is_empty() {
                    return None;
                }
                // (Under TEMPO's rails, its dials on a knob's 300 degrees.)
                let (knob, span) = if let Some(n) = super::fx_pins::target(i) {
                    ((self.fx_pin_at(n)?, art::STD.skirt), 300.0)
                } else if i == TEMPO_CONTROL {
                    let (_, foot, _) = self.tempo_place();
                    ((foot, mods::rails_depth(on.len())), 300.0)
                } else {
                    let c = self.renderer.arrangement().control(i);
                    let (a0, a1) = art::sweep(&c.kind);
                    ((c.centre(), art::skirt(&c.kind)), a1 - a0)
                };
                Some(Popup {
                    knob,
                    span,
                    dials: on
                        .iter()
                        .map(|&(s, d)| PopDial {
                            colour: colour(&self.settings, s),
                            label: label(&self.params.lfo_patch, s),
                            depth: f64::from(d),
                            shared: lfo::takes_shared(&self.params, s, t),
                            uni: s.lfo().is_some() && self.one_way(s),
                            muted: self.muted(s) || !self.mod_live(i),
                        })
                        .collect(),
                    hover: self.modulating.popup_hover,
                    size: self.renderer.text_size(),
                })
            });
        ModScene { knobs, cord, popup }
    }

    /// What a control's tip adds: each source on it and its depth (" · LFO 1 +40 % · LFO 2
    /// –25 %").
    pub(super) fn depths_tip(&self, i: usize) -> String {
        let Some(t) = self.mod_target(i) else {
            return String::new();
        };
        self.sources_on(t)
            .iter()
            .map(|&(s, d)| {
                let uni = if s.lfo().is_some() && self.one_way(s) {
                    " UNI"
                } else {
                    ""
                };
                let muted = if self.muted(s) { " MUTED" } else { "" };
                format!(
                    " \u{b7} {} {}{uni}{muted}",
                    label(&self.params.lfo_patch, s),
                    mods::percent(f64::from(d))
                )
            })
            .collect()
    }

    /// What a menu or a tip adds to an assignment that does nothing here: TEMPO's in a plug-in,
    /// which follows its host's tempo (decisions.md R-LFO).
    pub(super) fn standalone_only(&self, t: ModTarget) -> &'static str {
        if t == ModTarget::Tempo && !self.playing.standalone {
            " (STANDALONE ONLY)"
        } else {
            ""
        }
    }

    /// Whether source `s` moves its knobs one way: a macro, or an LFO whose POLARITY is UNI
    /// (R-LFO's corrections).
    pub(super) fn one_way(&self, s: Source) -> bool {
        match s {
            Source::Lfo(n) => self.params.lfo_keys(n).polarity.value() == lfo::Polarity::Uni,
            Source::Macro(_) => true,
        }
    }

    /// Whether source `s` is an LFO turned off (ON off: R-LFO's corrections).
    pub(super) fn muted(&self, s: Source) -> bool {
        s.lfo().is_some_and(|n| !self.params.lfo_keys(n).on.value())
    }

    /// The tip of the page's part under the pointer (its controls' are the panel's): what an
    /// LFO's screen lists, what its jack does.
    pub(super) fn page_tip(&self) -> Option<(String, f64, f64)> {
        let t = self.modulating.hover.filter(|_| self.page_shows())?;
        let n = t.lfo();
        let (x, y) = lfo_ui::target_centre(t);
        let text = match t {
            LfoTarget::Screen(_) => {
                let a: Vec<String> = self
                    .params
                    .lfo_patch
                    .assignments(Source::Lfo(n))
                    .into_iter()
                    .flatten()
                    .map(|a| {
                        let name = self.target_label(a.target);
                        format!(
                            "{name} {}{}",
                            mods::percent(f64::from(a.depth)),
                            self.standalone_only(a.target)
                        )
                    })
                    .collect();
                if a.is_empty() {
                    format!(
                        "LFO {} MODULATES NOTHING: DRAG FROM ITS OUT JACK TO A KNOB",
                        n + 1
                    )
                } else {
                    format!("LFO {} MODULATES {}", n + 1, a.join(" \u{b7} "))
                }
            }
            LfoTarget::Jack(_) => format!("LFO {} OUT: DRAG TO A KNOB TO MODULATE IT", n + 1),
            LfoTarget::Title(_) => {
                let on = self.params.lfo_keys(n).on.value();
                format!(
                    "LFO {} ON: {}",
                    n + 1,
                    if on { "ON" } else { "OFF (MUTED)" }
                )
            }
            _ => return None,
        };
        Some((text, x, y - 30.0))
    }

    /// A tip of the page's (a control's, its jack's, its screen's) where the panel's tips are,
    /// centred above what it names; but where it would cover a scope (MODE's, longer than its
    /// toggle is wide, lay over its own scope's corner and the next one's: the owner,
    /// 2026-10-20), its foot lifted to just above the scopes' tops; and moved in where it would
    /// run past the window's side (as the grip's is: R-SIZE), LFO 4's MODE's.
    pub(super) fn clear_of_scopes(&self, (t, x, y): (String, f64, f64)) -> (String, f64, f64) {
        let b = art::tip_bounds(self.renderer.fonts(), &t, self.renderer.text_size());
        let x = x.min(art::W - SCOPE_CLEAR - b[2]).max(SCOPE_CLEAR - b[0]);
        let (x0, y0, x1, y1) = (x + b[0], y + b[1], x + b[2], y + b[3]);
        let covers = (0..LFOS).any(|m| {
            let (sx, sy, sw, sh) = lfo_ui::places(m).scope;
            x1 > sx && x0 < sx + sw && y1 > sy && y0 < sy + sh
        });
        if !covers {
            return (t, x, y);
        }
        let top = lfo_ui::places(0).scope.1;
        (t, x, y.min(top - b[3] - SCOPE_CLEAR))
    }

    /// A press on the LFO page (it showing, the pointer over it): its controls as the panel's,
    /// its OUT jack an assignment dragged, its screen the list of what it modulates.
    pub(super) fn page_press(&mut self, (x, y): (f64, f64), modifiers: Modifiers) {
        if let Some(t) = interact::hit_lfo(self.renderer.layout(), x, y) {
            if let Target::Control(i) | Target::Legend(i, _) = t
                && let Some((n, _)) = lfo_part(CONTROLS[i].param)
            {
                self.modulating.last = Some(Source::Lfo(n));
            }
            // (The OUT jack is its LEVEL's control: a press there drags a cord as before, a
            // double click puts LEVEL back at 100 %, Ctrl or Cmd and a click types it: R-LFO's
            // addendum.)
            if let Target::Control(i) = t
                && let Some(n) = level_of_control(i)
                && !super::entry::shortcut(modifiers)
            {
                let now = Instant::now();
                let double = self
                    .last_press
                    .is_some_and(|(at, last)| last == t && now - at < DOUBLE_CLICK);
                self.last_press = Some((now, t));
                if double {
                    let level = &self.params.lfo_level(n).level;
                    self.set_once(i, f64::from(level.default_normalized_value()));
                    self.last_press = None;
                    return;
                }
                self.modulating.patching = Some(Patching {
                    source: Source::Lfo(n),
                    at: (x, y),
                    from: (x, y),
                    away: false,
                });
                self.modulating.popup = None;
                return;
            }
            self.press_target(t, modifiers);
            return;
        }
        let Some(t) = lfo_ui::hit(x, y) else { return };
        let n = t.lfo();
        self.modulating.last = Some(Source::Lfo(n));
        match t {
            LfoTarget::Jack(_) => {
                self.modulating.patching = Some(Patching {
                    source: Source::Lfo(n),
                    at: (x, y),
                    from: (x, y),
                    away: false,
                });
                self.modulating.popup = None;
            }
            LfoTarget::Screen(_) => self.open_modulates_menu(n),
            // (The title is ON's key's word: pressed as the key is.)
            LfoTarget::Title(_) => self.press_target(Target::Control(lfo_index(n, 8)), modifiers),
            _ => {}
        }
    }

    /// A press on what modulation shows over the panel: a pop-up's dial, a ring's end, or a knob
    /// with Alt/Option held (the last touched source's depth). Whether it was one.
    pub(super) fn mod_press(&mut self, (x, y): (f64, f64), modifiers: Modifiers) -> bool {
        let scene = self.mod_scene.clone();
        if let (Some(p), Some(i)) = (&scene.popup, self.modulating.popup) {
            if let Some(k) = mods::hit_popup(p, x, y)
                && self.mod_live(i)
                && let Some(t) = self.mod_target(i)
                && let Some(&(s, d)) = self.sources_on(t).get(k)
            {
                let now = Instant::now();
                // A double click: taken away.
                if self
                    .modulating
                    .last_popup_press
                    .is_some_and(|(at, j)| j == k && now - at < DOUBLE_CLICK)
                {
                    self.params.lfo_patch.remove(s, t);
                    self.modulating.last_popup_press = None;
                    return true;
                }
                self.modulating.last_popup_press = Some((now, k));
                self.modulating.depth = Some(DepthDrag {
                    source: s,
                    target: t,
                    control: i,
                    from: self.pointer,
                    start: d,
                    way: DepthWay::Vertical,
                });
                return true;
            }
            if mods::on_popup(p, x, y) {
                return true;
            }
        }
        if let Some((k, j)) = mods::hit_ring_end(&scene.knobs, x, y) {
            let i = scene.knobs[k].control;
            if self.mod_live(i)
                && let Some(t) = self.mod_target(i)
                && let Some(&(s, d)) = self.sources_on(t).get(j)
            {
                self.modulating.depth = Some(DepthDrag {
                    source: s,
                    target: t,
                    control: i,
                    from: self.pointer,
                    start: d,
                    way: DepthWay::Ring,
                });
                return true;
            }
        }
        if modifiers.contains(Modifiers::ALT)
            && let Some(Target::Control(i)) = self.mod_hit(x, y)
            && self.in_view(i)
            && self.mod_live(i)
            && let Some(t) = self.mod_target(i)
        {
            let s = self.modulating.last.unwrap_or(Source::Lfo(0));
            let start = self.params.lfo_patch.depth(s, t).unwrap_or(0.0);
            // (A press that does not move is still a click: the first of a double click, which
            // resets the knob, R-KNOB-ENTRY.)
            self.last_press = Some((Instant::now(), Target::Control(i)));
            self.modulating.depth = Some(DepthDrag {
                source: s,
                target: t,
                control: i,
                from: self.pointer,
                start,
                way: DepthWay::Vertical,
            });
            return true;
        }
        false
    }

    /// The pointer moved during an assignment's or a depth's drag: whether it was one.
    pub(super) fn mod_moved(&mut self, at: (f64, f64)) -> bool {
        if let Some(p) = &mut self.modulating.patching {
            p.at = at;
            p.away |= (at.0 - p.from.0).hypot(at.1 - p.from.1) > JACK_LEFT;
            return true;
        }
        let Some(d) = &self.modulating.depth else {
            return false;
        };
        if !self.mod_live(d.control) {
            return true;
        }
        let depth = match d.way {
            DepthWay::Vertical => {
                (f64::from(d.start) + (d.from.1 - self.pointer.1) / DEPTH_DRAG) as f32
            }
            DepthWay::Ring => {
                let Some(k) = self.mod_scene.knobs.iter().find(|k| k.control == d.control) else {
                    return true;
                };
                mods::depth_at(k, at) as f32
            }
        }
        .clamp(-1.0, 1.0);
        let (s, t) = (d.source, d.target);
        if self.params.lfo_patch.depth(s, t) != Some(depth) {
            // (Past the most a source takes: nothing.)
            let _ = self.params.lfo_patch.assign(s, t, depth);
        }
        true
    }

    /// The pointer let go: an assignment dropped on the knob under it (at the default depth,
    /// if it has none from that source and there is room), a depth's drag ended (taken away at
    /// 0). Whether it was one.
    pub(super) fn mod_released(&mut self) -> bool {
        if let Some(p) = self.modulating.patching.take() {
            // (A cord drawn away is no click: the next press is no double click.)
            if p.away {
                self.last_press = None;
            }
            if let Some(Target::Control(i) | Target::Legend(i, _)) = self.mod_hit(p.at.0, p.at.1)
                && self.in_view(i)
                && self.mod_live(i)
                && let Some(t) = self.mod_target(i)
                && self.params.lfo_patch.depth(p.source, t).is_none()
                // (Let go on its own jack without leaving it: a click, not a cord.)
                && (p.away || level_of_control(i).map(Source::Lfo) != Some(p.source))
            {
                let _ = self
                    .params
                    .lfo_patch
                    .assign(p.source, t, lfo::DEFAULT_DEPTH);
            }
            return true;
        }
        if let Some(d) = self.modulating.depth.take() {
            if self
                .params
                .lfo_patch
                .depth(d.source, d.target)
                .is_some_and(|v| v.abs() < NO_DEPTH)
            {
                self.params.lfo_patch.remove(d.source, d.target);
            }
            return true;
        }
        false
    }

    /// The pop-up shown for the knob under the pointer (with something on it), kept while the
    /// pointer is over the pop-up itself.
    pub(super) fn follow_popup(&mut self, at: (f64, f64)) {
        if self.modulating.depth.is_some() {
            return;
        }
        if let (Some(p), Some(_)) = (&self.mod_scene.popup, self.modulating.popup)
            && mods::on_popup(p, at.0, at.1)
        {
            self.modulating.popup_hover = mods::hit_popup(p, at.0, at.1);
            self.hover = None;
            return;
        }
        self.modulating.popup_hover = None;
        // (TEMPO's screen, the standalone's, with something on it: R-LFO.)
        if self.tempo_hovered(at)
            && self.modulating.patching.is_none()
            && !self.sources_on(ModTarget::Tempo).is_empty()
        {
            self.modulating.popup = Some(TEMPO_CONTROL);
            return;
        }
        self.modulating.popup = match self.hover {
            Some(Target::Control(i) | Target::Legend(i, _))
                if self.in_view(i)
                    && self.modulating.patching.is_none()
                    && self
                        .mod_target(i)
                        .is_some_and(|t| !self.sources_on(t).is_empty()) =>
            {
                Some(i)
            }
            _ => None,
        };
    }

    /// The rows a knob's right-click menu gains: a head for each source on it with DEPTH… and
    /// REMOVE; a strip knob's LFO head (nothing can be dragged to it under the page); an LFO's
    /// WAVEFORM's modes.
    pub(super) fn knob_rows(&self, i: usize) -> Vec<Row> {
        // (A macro's own knob: RENAME… and what it modulates, decisions.md R-MACROS.)
        if let Some(n) = super::macros::macro_of_control(i) {
            return self.macro_rows(n);
        }
        let mut rows = Vec::new();
        if i == TEMPO_CONTROL {
            // (What moves it, then a source to give it, as a strip knob's: R-LFO.)
            rows.extend(self.depth_rows(ModTarget::Tempo, i));
            rows.extend(self.assign_rows(ModTarget::Tempo));
            return rows;
        }
        let Some(control) = CONTROLS.get(i) else {
            return rows;
        };
        if let Some((n, "waveform")) = lfo_part(control.param) {
            // (CONTINUOUS, what moves its shape first: R-LFO's addendum.)
            if let Some(t) = self.mod_target(i) {
                rows.extend(self.depth_rows(t, i));
            }
            let continuous = self.params.lfo(n).shape_mode.value() == WaveMode::Continuous;
            rows.push(Row::Head("WAVEFORM".into()));
            rows.push(Row::Item(
                ModItem::Wave(n, false),
                "STEPPED".into(),
                continuous,
                !continuous,
            ));
            rows.push(Row::Item(
                ModItem::Wave(n, true),
                "CONTINUOUS".into(),
                !continuous,
                continuous,
            ));
            // (Read backwards, STEPPED or CONTINUOUS: R-LFO's corrections.)
            let reverse = self.params.lfo_keys(n).reverse.value();
            rows.push(Row::Item(
                ModItem::Reverse(n),
                "REVERSE".into(),
                true,
                reverse,
            ));
            return rows;
        }
        let Some(t) = self.mod_target(i) else {
            return rows;
        };
        // (An LFO's OUT jack, its LEVEL's: what moves its LEVEL, then the LFO's own menu's
        // COLOR and EDGES, which a right click on its jack gave before: R-LFO's addendum.)
        if let Some(n) = level_of_control(i) {
            rows.extend(self.depth_rows(t, i));
            rows.extend(self.lfo_rows(n));
            return rows;
        }
        rows.extend(self.depth_rows(t, i));
        if control.place == ControlPlace::Strip {
            rows.extend(self.assign_rows(t));
        }
        rows
    }

    /// An LFO head and a MACRO head, each source an item that gives it `t` (those with room
    /// that do not move it yet).
    pub(super) fn assign_rows(&self, t: ModTarget) -> Vec<Row> {
        let mut rows = Vec::new();
        for (head, macros) in [("LFO", false), ("MACRO", true)] {
            rows.push(Row::Head(head.into()));
            for s in Source::all().filter(|s| s.macro_n().is_some() == macros) {
                let free = self.params.lfo_patch.depth(s, t).is_none()
                    && self.params.lfo_patch.count(s) < lfo::MOST;
                rows.push(Row::Item(
                    ModItem::Assign(s, t),
                    label(&self.params.lfo_patch, s),
                    free,
                    false,
                ));
            }
        }
        rows
    }

    /// The TEMPO screen's menu (a right click on it, the standalone's): what moves it, with
    /// DEPTH… and REMOVE, and the sources that can.
    pub(super) fn open_tempo_menu(&mut self, at: (f64, f64)) {
        let rows = self.knob_rows(TEMPO_CONTROL);
        let size = self.renderer.text_size() * MENU_TEXT;
        self.learning
            .open_rows_menu(self.renderer.fonts(), at, size, "TEMPO".into(), rows);
    }

    /// A head for each source on target `t` (control `i`'s) with DEPTH… and REMOVE.
    pub(super) fn depth_rows(&self, t: ModTarget, i: usize) -> Vec<Row> {
        let mut rows = Vec::new();
        for (s, d) in self.sources_on(t) {
            rows.push(Row::Head(format!(
                "{} \u{b7} {}",
                label(&self.params.lfo_patch, s),
                mods::percent(f64::from(d))
            )));
            rows.push(Row::Item(
                ModItem::Depth(s, t, i),
                "DEPTH\u{2026}".into(),
                true,
                false,
            ));
            rows.push(Row::Item(
                ModItem::Remove(s, t),
                "REMOVE".into(),
                true,
                false,
            ));
        }
        rows
    }

    /// LFO `n`'s own rows: its colour from the phosphors (the computer's), its EDGES (the
    /// patch's).
    fn lfo_rows(&self, n: usize) -> Vec<Row> {
        let shown = self.settings.lfo_colors[n];
        let edges = self.params.lfo_patch.edges(n);
        let mut rows = vec![Row::Head("COLOR".into())];
        for p in Phosphor::ALL {
            rows.push(Row::Item(
                ModItem::Color(n, p),
                p.name().into(),
                p != shown,
                p == shown,
            ));
        }
        rows.push(Row::Head("EDGES".into()));
        for (e, t) in [(Edges::Soft, "SOFT"), (Edges::Sharp, "SHARP")] {
            rows.push(Row::Item(
                ModItem::Edges(n, e),
                t.into(),
                e != edges,
                e == edges,
            ));
        }
        rows
    }

    /// An LFO's own menu (a right click on its scope, its screen or its column; its jack's is
    /// its LEVEL's, with these rows): its colour from the phosphors (the computer's), its EDGES
    /// (the patch's).
    pub(super) fn open_lfo_menu(&mut self, n: usize, at: (f64, f64)) {
        let rows = self.lfo_rows(n);
        let size = self.renderer.text_size() * MENU_TEXT;
        self.learning.open_rows_menu(
            self.renderer.fonts(),
            at,
            size,
            format!("LFO {}", n + 1),
            rows,
        );
    }

    /// What LFO `n` modulates, dropped from its screen: each knob a head with DEPTH… and REMOVE.
    pub(super) fn open_modulates_menu(&mut self, n: usize) {
        let s = Source::Lfo(n);
        let mut rows = Vec::new();
        for a in self.params.lfo_patch.assignments(s).into_iter().flatten() {
            let Some(i) = control_of(a.target) else {
                continue;
            };
            rows.push(Row::Head(format!(
                "{} \u{b7} {}{}",
                self.target_label(a.target),
                mods::percent(f64::from(a.depth)),
                self.standalone_only(a.target)
            )));
            rows.push(Row::Item(
                ModItem::Depth(s, a.target, i),
                "DEPTH\u{2026}".into(),
                true,
                false,
            ));
            rows.push(Row::Item(
                ModItem::Remove(s, a.target),
                "REMOVE".into(),
                true,
                false,
            ));
        }
        if rows.is_empty() {
            return;
        }
        let (x, y, w, h) = lfo_ui::places(n).screen;
        let size = self.renderer.text_size() * MENU_TEXT;
        self.learning.open_rows_menu(
            self.renderer.fonts(),
            (x + w / 2.0, y + h),
            size,
            format!("LFO {} MODULATES", n + 1),
            rows,
        );
    }

    /// A right click on the LFO page: a control's menu (MIDI Learn, a WAVEFORM's modes), else the
    /// LFO's own.
    pub(super) fn page_menu(&mut self, (x, y): (f64, f64)) {
        let size = self.renderer.text_size() * MENU_TEXT;
        if let Some(target @ (Target::Control(i) | Target::Legend(i, _))) =
            interact::hit_lfo(self.renderer.layout(), x, y)
        {
            let rows = self.knob_rows(i);
            self.learning.open_menu_with(
                self.renderer.fonts(),
                &self.params.midi_map,
                Some(Place::Panel(i)),
                Some(target),
                (x, y),
                size,
                rows,
            );
            return;
        }
        match lfo_ui::hit(x, y) {
            Some(t) => self.open_lfo_menu(t.lfo(), (x, y)),
            None => self.learning.close_menu(),
        }
    }

    /// A modulation item of a menu chosen.
    pub(super) fn mod_item(&mut self, m: ModItem) {
        let patch = std::sync::Arc::clone(&self.params.lfo_patch);
        match m {
            ModItem::Unpin(n) => self.fx_view.unpin(n),
            ModItem::Depth(s, t, i) => self.start_depth_entry(i, s, t),
            ModItem::Remove(s, t) => {
                patch.remove(s, t);
            }
            ModItem::Assign(s, t) => {
                if let ModTarget::Fx(n) = t
                    && !self.mod_live(super::fx_pins::FIRST + n)
                {
                    return;
                }
                let _ = patch.assign(s, t, lfo::DEFAULT_DEPTH);
                self.modulating.last = Some(s);
            }
            ModItem::Color(n, p) => self.keep_settings(Change::LfoColor(n, p)),
            ModItem::Edges(n, e) => patch.set_edges(n, e),
            ModItem::Wave(n, continuous) => self.set_lfo_wave_mode(n, continuous),
            ModItem::Reverse(n) => {
                let p = &self.params.lfo_keys(n).reverse;
                let s = self.setter();
                s.begin_set_parameter(p);
                s.set_parameter(p, !p.value());
                s.end_set_parameter(p);
            }
            ModItem::MacroColor(n, c) => self.keep_settings(Change::MacroColor(n, c)),
            ModItem::Rename(n) => self.start_rename(n),
            ModItem::RemoveAll(s) => {
                for a in patch.assignments(s).into_iter().flatten() {
                    patch.remove(s, a.target);
                }
            }
        }
    }

    /// LFO `n`'s waveform STEPPED or CONTINUOUS (its WAVEFORM's menu), as an oscillator's: going
    /// CONTINUOUS, its shape where its switch was.
    pub(super) fn set_lfo_wave_mode(&self, n: usize, continuous: bool) {
        let q = self.params.lfo(n);
        let to = if continuous {
            WaveMode::Continuous
        } else {
            WaveMode::Stepped
        };
        if q.shape_mode.value() == to {
            return;
        }
        let s = self.setter();
        if continuous {
            let k = ca72_panel::art::position(f64::from(q.waveform.unmodulated_normalized_value()));
            let t = k as f64 / interact::CONTINUOUS_PLACES;
            s.begin_set_parameter(&q.shape);
            s.set_parameter_normalized(&q.shape, t as f32);
            s.end_set_parameter(&q.shape);
        }
        s.begin_set_parameter(&q.shape_mode);
        s.set_parameter(&q.shape_mode, to);
        s.end_set_parameter(&q.shape_mode);
    }

    /// Whether control `i` is an LFO's WAVEFORM switch whose waveform is CONTINUOUS.
    pub(super) fn lfo_continuous(&self, i: usize) -> bool {
        CONTROLS.get(i).is_some_and(|c| {
            matches!(lfo_part(c.param), Some((n, "waveform"))
                if self.params.lfo(n).shape_mode.value() == WaveMode::Continuous)
        })
    }

    /// Whether control `i` is an LFO's RATE timed now.
    pub(super) fn lfo_timed(&self, i: usize) -> bool {
        CONTROLS.get(i).is_some_and(
            |c| matches!(lfo_part(c.param), Some((n, "rate")) if self.params.lfo(n).timed.value()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{Controls, Engine, Event};
    use crate::lfo::{Assignment, Edges, LfoKnob, Mode, Settings, Target};

    /// LFO 1 CONTINUOUS, its shape moved by LFO 2 (and its PHASE and LEVEL by LFO 3), as the
    /// owner's picture had it (decisions.md R-LFO's corrections: "the oscilloscope wave isn't
    /// changing to match the actual pattern that's happening at the moment"): at every reading,
    /// the trace drawn from what the engine has in force passes through the orb within a pixel of
    /// the largest scope; the trace from the knobs as set, as before, does not.
    #[test]
    fn a_moved_trace_passes_through_its_orb() {
        let sharp = |at: f64, hz: f64| Settings {
            at,
            hz,
            mode: Mode::Free,
            edges: Edges::Sharp,
            continuous: true,
            ..Settings::default()
        };
        let mut c = Controls {
            entropy: 0.0,
            potato: true,
            ..Controls::default()
        };
        c.lfo.lfo = [
            sharp(1.0, 1.7),
            sharp(0.0, 0.31),
            sharp(2.0, 0.13),
            sharp(0.0, 1.0),
        ];
        let mut give = |from: usize, t: Target, depth: f32, base: f32| {
            let slot = c.lfo.to[from].iter_mut().find(|a| a.is_none()).unwrap();
            *slot = Some(Assignment { target: t, depth });
            c.lfo.base[t.index()] = base;
        };
        give(0, Target::Cutoff, 0.3, 0.5);
        give(1, Target::of_lfo(0, LfoKnob::Shape), 0.4, 0.4);
        give(2, Target::of_lfo(0, LfoKnob::Phase), 0.3, 0.2);
        give(2, Target::of_lfo(0, LfoKnob::Level), -0.3, 0.8);
        let mut e = Engine::new();
        e.set(&c);
        e.prepare(48_000.0, 3);
        e.event(Event::Note { key: 60, on: true });
        let (mut l, mut r) = (vec![0.0f32; 256], vec![0.0f32; 256]);
        // (A pixel at 0.87 pixels a unit, in the trace's values.)
        let pixel = 1.0 / (ca72_panel::strip::lfo::trace_amplitude() * 0.87);
        let (mut worst, mut as_set) = (0.0f64, 0.0f64);
        let mut seen = 0;
        for b in 0..1200 {
            e.render(&[], &mut l, &mut r);
            e.end_block(256);
            if b % 7 != 0 {
                continue;
            }
            let reading = e.lfo_readings()[0];
            assert!(reading.moved && reading.moving);
            let set = c.lfo.lfo[0];
            let (drawn, level) = drawn_settings(&set, &reading);
            let trace = trace_of(&drawn, level, true);
            let x = orb_at(&drawn, &reading, 120.0, true);
            let at = |trace: &[f32]| {
                let f = x * (TRACE_POINTS - 1) as f64;
                let i = (f.floor() as usize).min(TRACE_POINTS - 2);
                let k = f - i as f64;
                f64::from(trace[i]) * (1.0 - k) + f64::from(trace[i + 1]) * k
            };
            let y = f64::from(reading.value);
            // (A square's or a sawtooth's edge under the orb: the beam's jump, any height.)
            let jump = (f64::from(trace[(x * 479.0) as usize])
                - f64::from(trace[((x * 479.0) as usize + 1).min(479)]))
            .abs();
            if jump > 0.1 {
                continue;
            }
            seen += 1;
            worst = worst.max((at(&trace) - y).abs());
            let still = trace_of(&set, 1.0, false);
            as_set = as_set.max((at(&still) - y).abs());
        }
        assert!(seen > 100, "{seen} readings");
        assert!(
            worst < pixel,
            "the orb {worst} off its trace, a pixel {pixel}"
        );
        assert!(as_set > 10.0 * pixel, "as set it was off by {as_set}");
    }
}
