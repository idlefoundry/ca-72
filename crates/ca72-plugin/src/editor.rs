//! The editor: the front panel (`ca72-panel`) in a window of its own (baseview), its frames
//! shown with softbuffer, and under it the plug-in's own strip (`ca72_panel::strip`, A6:
//! decisions.md R44): the presets' rail, the left hand's GLIDE, DECAY and wheels, MODE,
//! VOICES and ENTROPY, how the voices are played across the field, WIDTH, INNER, DETUNE and
//! where the voices sound, DRIVE, AUTO GAIN and LEVEL. Every control operates its parameter
//! through the host, a drag one gesture (one undo step in hosts that keep them); the POWER
//! switch is the bypass; the OVERLOAD lamp, the wheels and the voices' display show what the
//! audio thread reports. The grip at the window's bottom right corner resizes it. The presets'
//! drawer drops down from under the rail over the strip (`crate::presets`, decisions.md R10),
//! the window keeping its size, and takes the keyboard while open; in it the update check
//! (`crate::update`, R27). A right click opens a control's menu, for MIDI Learn; the drawer's
//! MIDI button shows its list of assignments (`crate::learning`, R34).
//! PANEL on the rail shows or hides the whole lower panel. The window snaps to its height;
//! opening presets temporarily reveals a hidden panel, closing them hides it again.

use std::any::Any;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::local_notes::LocalNotes;
use crate::playing::Playing;
use baseview::{
    Event, EventStatus, MouseButton, MouseCursor, MouseEvent, ScrollDelta, Size, Window,
    WindowEvent, WindowHandler, WindowInfo, WindowOpenOptions, WindowScalePolicy,
};
use ca72::freq_dial::{Law, Pot};
use ca72_panel::art::{self, POWER};
use ca72_panel::controls::{FEEDBACK_SILENT, feedback_silent};
use ca72_panel::fonts::Weight;
use ca72_panel::keyboard;
use ca72_panel::learn::{Note, Tone};
use ca72_panel::presets::{
    BarTarget, DRAWER_H, DRAWER_TOP, DrawerRenderer, DrawerTarget, PresetKey, ROW_H, drawer_hit,
    in_tags,
};
use ca72_panel::strip::arp::{self as arp_ui, ArpBank, ArpRenderer, ArpScene, ArpTarget};
use ca72_panel::strip::fx::{self as fx_ui, FxRenderer, FxScene};
use ca72_panel::strip::lower as lower_ui;
use ca72_panel::strip::{self, Bank, Field, Rail, Readout, StripRenderer, StripScene, StripTarget};
use ca72_panel::{CONTROLS, Kind, Lettering, Renderer, Scene, Skin, Target, interact};
use keyboard_types::{Key, KeyState, KeyboardEvent, Modifiers};
use nih_plug::editor::HostKey;
use nih_plug::prelude::*;
use plugin_kit_stereo::place;

use crate::engine::LEVELS;
use crate::fx_view::FxView;
use crate::learning::{Learning, Place, Pressed};
use crate::library::Library;
use crate::opening::{ANIMATE_S, Motion, level_of};
use crate::params::{ArpDirection, ArpRate, Ca72Params, QualityMode, Scatter};
use crate::presets::Browser;
use crate::settings::{Change, Settings};
use crate::update::Update;

mod agent;

/// The share of the usable screen the editor opens at until it is resized.
const SCREEN_SHARE: f64 = 0.8;
/// The editor's width at first where the screen's size is unknown, in logical pixels.
pub const DEFAULT_WIDTH: u32 = 1380;
/// How small and large the grip makes it.
const MIN_WIDTH: u32 = 860;
const MAX_WIDTH: u32 = 3108;

/// The compact view's grip's square in the rail's bottom right corner (drawing units): where
/// it is drawn (`render::compact_grip`) and found.
const COMPACT_GRIP: f64 = 44.0;
/// The SIZE menu's sizes (R-SIZE): shares of the size the editor opens at on a 1920 by 1080
/// screen ([`DEFAULT_WIDTH`]); after them, the size it opens at on this screen.
const SIZES: [(&str, f64); 5] = [
    ("75%", 0.75),
    ("100%", 1.0),
    ("125%", 1.25),
    ("150%", 1.5),
    ("200%", 2.0),
];
/// The grip's tip.
const GRIP_TIP: &str = "DRAG TO RESIZE";

/// Where the settings' menu and the SIZE menu open: their top right corner under the rail's
/// settings key, at its right end (in the same place on either rail).
fn settings_menu_corner() -> (f64, f64) {
    let (x, y) = strip::centre(Rail::Plugin, StripTarget::Bar(BarTarget::Settings));
    (x + strip::KEY_W / 2.0, y + strip::KEY_H / 2.0 + 6.0)
}

/// Two presses within this long are a double click.
const DOUBLE_CLICK: Duration = Duration::from_millis(400);
/// A mouse wheel's turns are one gesture until it rests this long.
const SCROLL_REST: Duration = Duration::from_millis(400);
/// A trackpad's travel that makes a notch, in pixels.
const NOTCH: f64 = 40.0;
/// A control's menu's lettering, against the tip's.
const MENU_TEXT: f64 = 1.15;

/// The editor's height at `width`: the panel and the strip under it, in the drawing's
/// proportions.
pub fn height_for(width: u32) -> u32 {
    (f64::from(width) * art::H / art::W).round() as u32
}

fn height_for_panel(width: u32, expanded: bool) -> u32 {
    if expanded {
        height_for(width)
    } else {
        (f64::from(width) * DRAWER_TOP / art::W).round() as u32
    }
}

fn clamp_width(width: u32) -> u32 {
    width.clamp(MIN_WIDTH, MAX_WIDTH)
}

/// The width the editor opens at before it is resized: its share of the usable screen
/// (logical pixels), its height kept within the same share.
fn fitted_width(screen: Option<(f64, f64)>) -> u32 {
    screen.map_or(DEFAULT_WIDTH, |(w, h)| {
        let w = (w * SCREEN_SHARE).min(h * SCREEN_SHARE * art::W / art::H);
        clamp_width(w.round() as u32)
    })
}

/// What the audio thread reports to the editor after each block.
#[derive(Debug)]
pub struct Meters {
    overload: AtomicU32,
    bend: AtomicU32,
    modulation: AtomicU32,
    /// The voices sounding, a bit a voice (`Engine::sounding_mask`).
    voices: AtomicU32,
    /// Each voice's output's peak since the editor last took them (`Engine::levels`), and the
    /// plug-in's output's, the highest kept: a peak is never negative, and such a float's bits
    /// order as its value. Only while an editor watches them (`watchers`, how many).
    levels: [AtomicU32; LEVELS],
    output: AtomicU32,
    watchers: AtomicU32,
}

impl Default for Meters {
    fn default() -> Self {
        Meters {
            overload: AtomicU32::new(0.0f32.to_bits()),
            bend: AtomicU32::new(0.5f32.to_bits()),
            modulation: AtomicU32::new(0.0f32.to_bits()),
            voices: AtomicU32::new(0),
            levels: std::array::from_fn(|_| AtomicU32::new(0)),
            output: AtomicU32::new(0),
            watchers: AtomicU32::new(0),
        }
    }
}

impl Meters {
    /// The OVERLOAD lamp (0..1), a MIDI keyboard's wheels as the panel draws them, and the
    /// voices sounding (a bit a voice).
    pub fn publish(&self, overload: f32, (bend, modulation): (f32, f32), voices: u32) {
        self.overload.store(overload.to_bits(), Ordering::Relaxed);
        self.bend.store(bend.to_bits(), Ordering::Relaxed);
        self.modulation
            .store(modulation.to_bits(), Ordering::Relaxed);
        self.voices.store(voices, Ordering::Relaxed);
    }

    /// Whether an editor watches the levels (the audio thread measures the output's then).
    pub fn watched(&self) -> bool {
        self.watchers.load(Ordering::Relaxed) > 0
    }

    /// Each voice's output's peak over the block, and the output's (the audio thread's: a
    /// compare each).
    pub fn publish_levels(&self, levels: &[f32; LEVELS], output: f32) {
        let held = |a: &AtomicU32, v: f32| {
            let v = if v.is_finite() && v > 0.0 { v } else { 0.0 };
            a.fetch_max(v.to_bits(), Ordering::Relaxed);
        };
        for (a, &v) in self.levels.iter().zip(levels) {
            held(a, v);
        }
        held(&self.output, output);
    }

    /// The voices' peaks and the output's since they were last taken, taken.
    fn take_levels(&self) -> ([f64; LEVELS], f64) {
        let take = |a: &AtomicU32| {
            let v = f64::from(f32::from_bits(a.swap(0, Ordering::Relaxed)));
            if v.is_finite() { v } else { 0.0 }
        };
        (
            std::array::from_fn(|k| take(&self.levels[k])),
            take(&self.output),
        )
    }

    fn load(a: &AtomicU32) -> f64 {
        let v = f64::from(f32::from_bits(a.load(Ordering::Relaxed)));
        if v.is_finite() { v } else { 0.0 }
    }
}

/// A parameter as the editor operates it, whatever its type.
trait Operated {
    fn value(&self) -> f32;
    fn default(&self) -> f32;
    fn text(&self, normalized: f32) -> String;
    fn begin(&self, s: &ParamSetter<'_>);
    fn set(&self, s: &ParamSetter<'_>, normalized: f32);
    fn end(&self, s: &ParamSetter<'_>);
}

impl<P: Param> Operated for P {
    fn value(&self) -> f32 {
        self.unmodulated_normalized_value()
    }
    fn default(&self) -> f32 {
        self.default_normalized_value()
    }
    fn text(&self, normalized: f32) -> String {
        self.normalized_value_to_string(normalized, true)
    }
    fn begin(&self, s: &ParamSetter<'_>) {
        s.begin_set_parameter(self);
    }
    fn set(&self, s: &ParamSetter<'_>, normalized: f32) {
        s.set_parameter_normalized(self, normalized);
    }
    fn end(&self, s: &ParamSetter<'_>) {
        s.end_set_parameter(self);
    }
}

/// An oscillator's FREQUENCY knob (decisions.md R54): where it points, read through
/// the computer's law (`ca72::freq_dial`), its pot's position the parameter's (normalized,
/// the wiper). A pot beyond the law's reach shows the knob at its stop.
struct FreqKnob {
    params: Arc<Ca72Params>,
    pot: Pot,
}

impl FreqKnob {
    fn param(&self) -> &FloatParam {
        match self.pot {
            Pot::Two => &self.params.osc2_frequency,
            Pot::Three => &self.params.osc3_frequency,
        }
    }

    /// The knob at `knob` of its travel: its pot's position.
    fn wiper(&self, knob: f32) -> f32 {
        self.pot.wiper_at(self.params.freq_law(), f64::from(knob)) as f32
    }

    fn knob(&self, wiper: f32) -> f32 {
        self.pot.knob_at(self.params.freq_law(), f64::from(wiper)) as f32
    }
}

impl Operated for FreqKnob {
    fn value(&self) -> f32 {
        self.knob(self.param().unmodulated_normalized_value())
    }
    fn default(&self) -> f32 {
        self.knob(self.param().default_normalized_value())
    }
    fn text(&self, knob: f32) -> String {
        self.param()
            .normalized_value_to_string(self.wiper(knob), true)
    }
    fn begin(&self, s: &ParamSetter<'_>) {
        s.begin_set_parameter(self.param());
    }
    fn set(&self, s: &ParamSetter<'_>, knob: f32) {
        s.set_parameter_normalized(self.param(), self.wiper(knob));
    }
    fn end(&self, s: &ParamSetter<'_>) {
        s.end_set_parameter(self.param());
    }
}

/// The parameter a control of the panel operates, by its id.
fn param<'a>(p: &'a Ca72Params, id: &str) -> Option<&'a dyn Operated> {
    Some(match id {
        "tune" => &p.tune,
        "glide" => &p.glide,
        "mod_mix" => &p.mod_mix,
        "osc_mod" => &p.osc_mod,
        "osc3_control" => &p.osc3_control,
        "osc1_range" => &p.osc1_range,
        "osc1_waveform" => &p.osc1_waveform,
        "osc2_range" => &p.osc2_range,
        "osc2_frequency" => &p.osc2_frequency,
        "osc2_waveform" => &p.osc2_waveform,
        "osc3_range" => &p.osc3_range,
        "osc3_frequency" => &p.osc3_frequency,
        "osc3_waveform" => &p.osc3_waveform,
        "osc1_on" => &p.osc1_on,
        "osc1_volume" => &p.osc1_volume,
        "ext_on" => &p.ext_on,
        "ext_volume" => &p.ext_volume,
        "osc2_on" => &p.osc2_on,
        "osc2_volume" => &p.osc2_volume,
        "noise_on" => &p.noise_on,
        "noise_volume" => &p.noise_volume,
        "noise_type" => &p.noise_type,
        "osc3_on" => &p.osc3_on,
        "osc3_volume" => &p.osc3_volume,
        "filter_mode" => &p.filter_mode,
        "quality" => &p.quality,
        "filter_mod" => &p.filter_mod,
        "keyboard_control_1" => &p.keyboard_control_1,
        "keyboard_control_2" => &p.keyboard_control_2,
        "cutoff" => &p.cutoff,
        "emphasis" => &p.emphasis,
        "contour_amount" => &p.contour_amount,
        "filter_attack" => &p.filter_attack,
        "filter_decay" => &p.filter_decay,
        "filter_sustain" => &p.filter_sustain,
        "loudness_attack" => &p.loudness_attack,
        "loudness_decay" => &p.loudness_decay,
        "loudness_sustain" => &p.loudness_sustain,
        "volume" => &p.volume,
        "feedback" => &p.feedback,
        "main_output" => &p.main_output,
        "a440" => &p.a440,
        "glide_on" => &p.glide_on,
        "decay_on" => &p.decay_on,
        "note_priority" => &p.note_priority,
        "retrigger" => &p.retrigger,
        "pitch_wheel" => &p.pitch_wheel,
        "mod_wheel" => &p.mod_wheel,
        // The strip's knobs (A6).
        "voices" => &p.voices,
        "entropy" => &p.entropy,
        "spread" => &p.spread,
        "inner" => &p.inner,
        "double" => &p.double,
        "drive" => &p.drive,
        "level" => &p.level,
        // The arpeggiator's drawer's (decisions.md R-ARP).
        "arp_rate" => &p.arp_rate,
        "arp_octaves" => &p.arp_octaves,
        "arp_gate" => &p.arp_gate,
        "arp_swing" => &p.arp_swing,
        _ => return None,
    })
}

/// A readout a value is typed into (decisions.md R52): the strip's, or the arpeggiator's
/// drawer's (RATE, OCTAVES, GATE, SWING; R-ARP).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Typed {
    Strip(Readout),
    Arp(usize),
}

/// The drawers dropped from the rail: the arpeggiator's (decisions.md R-ARP), and the
/// standalone's effects' (R-FX-DRAWER).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Lower {
    #[default]
    Arp,
    Fx,
}

impl Lower {
    /// How deep it is.
    fn depth(self) -> f64 {
        match self {
            Lower::Arp => arp_ui::DEPTH,
            Lower::Fx => fx_ui::DEPTH,
        }
    }
}

/// The drawer dropped from the rail, one at a time: which, open or not, and its slide, as the
/// presets' drawer's (`presets::SLIDE`).
#[derive(Debug, Default)]
struct Dropped {
    which: Lower,
    open: bool,
    slide: Option<(Instant, bool)>,
}

impl Dropped {
    /// `which` opened or put away, sliding there; another open put away for it at once (it
    /// drops from the rail anew).
    fn show(&mut self, which: Lower, on: bool) {
        if !on {
            if self.which == which && self.open {
                self.open = false;
                self.slide = Some((Instant::now(), false));
            }
            return;
        }
        if self.open && self.which == which {
            return;
        }
        self.which = which;
        self.open = true;
        self.slide = Some((Instant::now(), true));
    }

    /// Whether `which` is open (or opening).
    fn is_open(&self, which: Lower) -> bool {
        self.open && self.which == which
    }

    /// Put away at once (the lower panel hidden).
    fn shut(&mut self) {
        self.open = false;
        self.slide = None;
    }

    /// How far down it is, 0 to 1.
    fn reveal(&self) -> f64 {
        match self.slide {
            Some((t0, opening)) => {
                let k = (t0.elapsed().as_secs_f64() / crate::presets::SLIDE.as_secs_f64()).min(1.0);
                let k = 1.0 - (1.0 - k) * (1.0 - k);
                if opening { k } else { 1.0 - k }
            }
            None => f64::from(u8::from(self.open)),
        }
    }

    fn sliding(&self) -> bool {
        self.slide
            .is_some_and(|(t0, _)| t0.elapsed() < crate::presets::SLIDE)
    }

    fn tick(&mut self) {
        if self.slide.is_some() && !self.sliding() {
            self.slide = None;
        }
    }

    /// Open and still: its parts answer the pointer.
    fn settled(&self) -> bool {
        self.open && self.slide.is_none()
    }
}

/// The editor, as the plug-in hands it to the host.
pub struct Ca72Editor {
    params: Arc<Ca72Params>,
    meters: Arc<Meters>,
    local_notes: Arc<LocalNotes>,
    /// The host's scale (f32 bits; 0 none given: the system's, as on macOS).
    scale: AtomicU32,
    open: Arc<AtomicBool>,
    /// The width fitted to the screen for this opening (0 not yet measured).
    fitted: Arc<AtomicU32>,
    /// The keys the host gives the editor's view rather than its window (decisions.md R53).
    host_keys: Arc<HostKeys>,
}

/// Keys the host gives the editor's view rather than its window (decisions.md R53: Cubase
/// keeps the keyboard from the window and gives a VST3 plug-in its keys through
/// `IPlugView::onKeyDown()`): whether the editor holds the keyboard now (a value typed, the
/// drawer, MIDI Learn), and those taken for it, for its window at its next frame.
#[derive(Debug, Default)]
pub struct HostKeys {
    holding: AtomicBool,
    queue: Mutex<Vec<KeyboardEvent>>,
    overflow: AtomicBool,
}

impl HostKeys {
    /// A key the host gave: taken while the editor holds the keyboard (every key then, so that
    /// none is the host's key command meanwhile), else the host's.
    pub fn offer(&self, k: HostKey) -> bool {
        if !self.holding.load(Ordering::Acquire) {
            return false;
        }
        if let Some(e) = host_key_event(k)
            && let Ok(mut q) = self.queue.lock()
        {
            if q.len() < 64 {
                q.push(e);
            } else {
                self.overflow.store(true, Ordering::Release);
            }
        }
        true
    }

    /// The keys taken since the last time.
    fn take(&self) -> Vec<KeyboardEvent> {
        self.queue
            .lock()
            .map(|mut q| std::mem::take(&mut *q))
            .unwrap_or_default()
    }
}

/// A host's key as the window's own come: a key VST3 names by its code (Backspace, Enter,
/// Escape, the arrows and so on), else its character (a numeric keypad's digit, point or minus
/// sign too when the host gives none); its modifiers. None for what the editor has no use for.
pub fn host_key_event(k: HostKey) -> Option<KeyboardEvent> {
    // VST3's `VirtualKeyCodes` (pluginterfaces/base/keycodes.h).
    let named = match k.key_code {
        1 => Some(Key::Backspace),
        2 => Some(Key::Tab),
        4 | 19 => Some(Key::Enter),
        6 => Some(Key::Escape),
        9 => Some(Key::End),
        10 => Some(Key::Home),
        11 => Some(Key::ArrowLeft),
        12 => Some(Key::ArrowUp),
        13 => Some(Key::ArrowRight),
        14 => Some(Key::ArrowDown),
        15 => Some(Key::PageUp),
        16 => Some(Key::PageDown),
        22 => Some(Key::Delete),
        54 => Some(Key::Shift),
        55 => Some(Key::Control),
        56 => Some(Key::Alt),
        _ => None,
    };
    let key = match (named, k.character) {
        (Some(n), _) => n,
        (None, Some(c)) if !c.is_control() => Key::Character(c.to_string()),
        (None, _) => match k.key_code {
            7 => Key::Character(" ".into()),
            24..=33 => Key::Character(char::from(b'0' + (k.key_code - 24) as u8).to_string()),
            37 => Key::Character("-".into()),
            38 => Key::Character(".".into()),
            _ => return None,
        },
    };
    // VST3's `KeyModifier`: shift, alt, command (Ctrl on Windows), control (the Windows key).
    let mut modifiers = Modifiers::empty();
    if k.modifiers & 1 != 0 {
        modifiers |= Modifiers::SHIFT;
    }
    if k.modifiers & 2 != 0 {
        modifiers |= Modifiers::ALT;
    }
    let (command, control) = if cfg!(target_os = "macos") {
        (Modifiers::META, Modifiers::CONTROL)
    } else {
        (Modifiers::CONTROL, Modifiers::META)
    };
    if k.modifiers & 4 != 0 {
        modifiers |= command;
    }
    if k.modifiers & 8 != 0 {
        modifiers |= control;
    }
    Some(KeyboardEvent {
        state: if k.down { KeyState::Down } else { KeyState::Up },
        key,
        modifiers,
        ..KeyboardEvent::default()
    })
}

/// How long a key coming both ways (the host's and the window's own) waits for its twin.
const TWIN: Duration = Duration::from_millis(150);

/// Keys taken once though a host gives them both ways: each kept for [`TWIN`] to be matched
/// with its twin from the other way, which is then not taken again.
#[derive(Debug, Default)]
struct KeyTwins {
    host: VecDeque<(Instant, Key, KeyState)>,
    own: VecDeque<(Instant, Key, KeyState)>,
}

impl KeyTwins {
    /// Whether a key the host gave is new (not one the window had just now).
    fn host_new(&mut self, k: &KeyboardEvent, now: Instant) -> bool {
        Self::new_one(&mut self.own, &mut self.host, k, now)
    }

    /// Whether a key the window had is new (not one the host gave just now).
    fn own_new(&mut self, k: &KeyboardEvent, now: Instant) -> bool {
        Self::new_one(&mut self.host, &mut self.own, k, now)
    }

    fn new_one(
        other: &mut VecDeque<(Instant, Key, KeyState)>,
        mine: &mut VecDeque<(Instant, Key, KeyState)>,
        k: &KeyboardEvent,
        now: Instant,
    ) -> bool {
        let fresh = |t: &Instant| now.saturating_duration_since(*t) < TWIN;
        other.retain(|(t, ..)| fresh(t));
        mine.retain(|(t, ..)| fresh(t));
        if let Some(i) = other
            .iter()
            .position(|(_, key, state)| *key == k.key && *state == k.state)
        {
            other.remove(i);
            return false;
        }
        mine.push_back((now, k.key.clone(), k.state));
        true
    }
}

impl std::fmt::Debug for Ca72Editor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ca72Editor")
            .field("size", &self.size())
            .finish()
    }
}

impl Ca72Editor {
    pub fn new(params: Arc<Ca72Params>, meters: Arc<Meters>) -> Self {
        Self::with_notes(params, meters, Arc::new(LocalNotes::default()))
    }

    pub fn with_notes(
        params: Arc<Ca72Params>,
        meters: Arc<Meters>,
        local_notes: Arc<LocalNotes>,
    ) -> Self {
        Ca72Editor {
            params,
            meters,
            local_notes,
            scale: AtomicU32::new(0),
            open: Arc::new(AtomicBool::new(false)),
            fitted: Arc::new(AtomicU32::new(0)),
            host_keys: Arc::new(HostKeys::default()),
        }
    }

    fn host_scale(&self) -> Option<f32> {
        let s = f32::from_bits(self.scale.load(Ordering::Relaxed));
        (s > 0.0 && s.is_finite()).then_some(s)
    }

    /// The editor opening: the drawer shut (the window does not open grown), the screen
    /// measured again, its size.
    fn opening(&self) -> (u32, u32) {
        self.fitted.store(0, Ordering::Relaxed);
        self.params.editor_presets.store(false, Ordering::Relaxed);
        self.params.editor_panel_applied.store(
            self.params.editor_panel.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        self.params.editor_expanded.store(
            self.params.editor_panel.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        self.size()
    }

    /// The width fitted to the screen, measured once an opening and kept until the editor
    /// closes. Measured afresh at each of the host's questions, it changed with the drawer
    /// (on Linux it is the monitor under the pointer), and the panel was drawn again at
    /// another scale as the drawer opened and shut (the owner, 2026-10-03; decisions.md R19).
    fn fitted(&self) -> u32 {
        match self.fitted.load(Ordering::Relaxed) {
            0 => {
                let w = fitted_width(screen::usable(self.host_scale().map(f64::from)));
                self.fitted.store(w, Ordering::Relaxed);
                w
            }
            w => w,
        }
    }

    /// What resets the editor as its window closes (the host may ask its size before it
    /// opens it again: the drawer is shut then).
    fn closing(&self) -> Closing {
        Closing {
            open: Arc::clone(&self.open),
            fitted: Arc::clone(&self.fitted),
            resizing: Arc::clone(&self.params.editor_resizing),
        }
    }
}

/// The editor's window closed: not open, the drawer shut, the screen to be measured again.
struct Closing {
    open: Arc<AtomicBool>,
    fitted: Arc<AtomicU32>,
    resizing: Arc<crate::resizing::Resizing>,
}

impl Drop for Closing {
    fn drop(&mut self) {
        self.open.store(false, Ordering::Release);
        self.fitted.store(0, Ordering::Relaxed);
        self.resizing.close();
    }
}

/// How baseview scales the window: on Windows and Linux by the host's scale, else by none.
/// nih-plug gives the host the editor's size times the host's scale (1 when it set none:
/// Ableton Live sets none), and the window must fill that frame; baseview's own scale there
/// (the monitor's, Xft.dpi) made the window draw at one scale, size itself at another and
/// place the pointer at a third (decisions.md R18). macOS scales by itself.
fn scale_policy(host: Option<f32>) -> WindowScalePolicy {
    if cfg!(target_os = "macos") {
        WindowScalePolicy::SystemScaleFactor
    } else {
        WindowScalePolicy::ScaleFactor(host.map_or(1.0, f64::from))
    }
}

impl Editor for Ca72Editor {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        context: Arc<dyn GuiContext>,
    ) -> Box<dyn Any + Send> {
        let (width, height) = self.opening();
        let scale = self.host_scale();
        let params = Arc::clone(&self.params);
        let meters = Arc::clone(&self.meters);
        let host_keys = Arc::clone(&self.host_keys);
        let local_notes = Arc::clone(&self.local_notes);
        let parent = window::Parent(parent);
        // No window to open in (no parent, no X11 display): nothing shown, rather than
        // baseview's panic in the host's thread (decisions.md R18).
        if !parent.can_open() {
            return Box::new(window::Handle::new(None, self.closing()));
        }
        let window = Window::open_parented(
            &parent,
            WindowOpenOptions {
                title: String::from("CA-72"),
                size: Size::new(f64::from(width), f64::from(height)),
                scale: scale_policy(scale),
            },
            move |window: &mut Window<'_>| {
                let mut panel = PanelWindow::new(
                    window,
                    context,
                    params,
                    meters,
                    width,
                    f64::from(scale.unwrap_or(1.0)),
                    Library::shared(),
                    host_keys,
                );
                panel.editing.playing.notes = local_notes;
                panel.editing.playing.sync();
                panel
            },
        );
        self.open.store(true, Ordering::Release);
        Box::new(window::Handle::new(Some(window), self.closing()))
    }

    fn size(&self) -> (u32, u32) {
        let width = match self.params.editor_width.load(Ordering::Relaxed) {
            0 => self.fitted(),
            w => clamp_width(w),
        };
        let preference = self.params.editor_panel.load(Ordering::Relaxed);
        let expanded = if !self.open.load(Ordering::Acquire) {
            preference
        } else if preference != self.params.editor_panel_applied.load(Ordering::Relaxed) {
            preference || self.params.editor_presets.load(Ordering::Relaxed)
        } else {
            self.params.editor_expanded.load(Ordering::Relaxed)
        };
        (width, height_for_panel(width, expanded))
    }

    fn resize_request(&self) -> (u64, (u32, u32)) {
        self.params
            .editor_resizing
            .requested()
            .unwrap_or_else(|| (0, self.size()))
    }

    fn resize_result(&self, request: u64, accepted: bool) {
        if self.open.load(Ordering::Acquire) {
            self.params.editor_resizing.answer(request, accepted);
        }
    }

    fn set_scale_factor(&self, factor: f32) -> bool {
        // The window cannot follow a new scale once open (Ableton Live asks).
        if self.open.load(Ordering::Acquire) {
            return false;
        }
        self.scale.store(factor.to_bits(), Ordering::Relaxed);
        true
    }

    // The window reads the parameters every frame.
    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {}
    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {}
    fn param_values_changed(&self) {}

    // Keys the host gives the view (Cubase): the editor's while it holds the keyboard.
    fn on_host_key(&self, key: HostKey) -> bool {
        self.host_keys.offer(key)
    }
}

/// A control being dragged.
#[derive(Debug)]
struct Drag {
    control: usize,
    /// Where the pointer pressed (logical pixels), and what it pressed.
    from: (f64, f64),
    pressed: Target,
    /// Its value at the press; the drag counts from `base` at `at` (Shift changes the rate
    /// mid-drag: the travel then counts from where it changed).
    value: f64,
    base: f64,
    at: (f64, f64),
    fine: bool,
    moved: bool,
}

/// A mouse wheel turning a control.
#[derive(Debug)]
struct Scroll {
    control: usize,
    value: f64,
    last: Instant,
    travel: f64,
}

/// The editor's window: its surface, and the panel being operated in it.
struct PanelWindow {
    surface: window::Surface,
    physical: (u32, u32),
    /// Its size in logical pixels, as last set or reported.
    logical: (u32, u32),
    /// When the frame was last shown on the window (none since it opened or resized).
    shown: Option<Instant>,
    /// When the frame was last drawn while the window could not be seen ([`HIDDEN_DRAW`]).
    hidden_drawn: Option<Instant>,
    editing: Editing,
    /// The keyboard the drawer took from another window (Windows), given back as it shuts.
    keys: Option<window::Keys>,
    /// The keys the host gives the editor's view (decisions.md R53), each taken once if they
    /// come both ways.
    host_keys: Arc<HostKeys>,
    twins: KeyTwins,
    /// The pointer's look as last set (R-SIZE: the resize arrows over the grip).
    cursor: MouseCursor,
}

impl Drop for PanelWindow {
    // Closed: the host's keys are the host's again.
    fn drop(&mut self) {
        self.host_keys.holding.store(false, Ordering::Release);
    }
}

/// The panel being operated: what the pointer does to the parameters, and the scene drawn.
struct Editing {
    context: Arc<dyn GuiContext>,
    params: Arc<Ca72Params>,
    meters: Arc<Meters>,
    /// Physical pixels a logical pixel, and the window's width in logical pixels.
    dpr: f64,
    width: u32,
    physical_width: u32,
    renderer: Renderer,
    scene: Scene,
    /// The strip's own parts (over the drawing's strip): their renderer and scene, and
    /// ENTROPY's, WIDTH's, DETUNE's and INNER's last amount but 0 (their switches, and SCATTER |
    /// DOUBLE, turn them back on there; `amount` gives each its place).
    strip: StripRenderer,
    strip_scene: StripScene,
    last: [f32; 4],
    /// The presets: the drawer's renderer, what the rail and the drawer show and do, and the
    /// window's frame: the panel's, the strip's parts under its foot, the drawer over them.
    drawer: DrawerRenderer,
    pub(crate) browser: Browser,
    /// The arpeggiator's drawer (decisions.md R-ARP): its renderer, what it shows. The drawer
    /// dropped from the rail: which, whether it is open, and how far down it was in the last frame
    /// composed (and which it was).
    arp: ArpRenderer,
    arp_scene: ArpScene,
    lower: Dropped,
    lower_shown: (f64, Lower),
    last_arp_press: Option<(Instant, ArpTarget)>,
    /// The standalone's effects' drawer (decisions.md R-FX-DRAWER): the chain as it shows it and
    /// what a pointer does there, its renderer (none without a chain), what it shows; the rail's
    /// keys (its FX key with a chain).
    fx_view: FxView,
    fx: Option<FxRenderer>,
    fx_scene: FxScene,
    rail: Rail,
    composed: Option<ca72_panel::Pixmap>,
    /// Where the window's frame changed in the last `draw` that changed it (pixels: left,
    /// top, right, bottom), or none when it may have changed anywhere (decisions.md R58).
    damage: Option<Vec<[i32; 4]>>,
    /// A size asked of the window.
    resizing_window: bool,
    /// The drawer placed for its opening (below the strip or over the panel), asked once.
    placed: bool,
    /// The update check in the drawer; dropped with the editor, it ends a check under way.
    update: Update,
    /// MIDI Learn: a control's menu, the note over the control being learned, the drawer's
    /// list (decisions.md R34).
    pub(crate) learning: Learning,
    /// The pointer, in logical pixels.
    pointer: (f64, f64),
    hover: Option<Target>,
    drag: Option<Drag>,
    scroll: Option<Scroll>,
    /// The presets' list's scrolling not yet a whole row, in rows (+ up).
    list_travel: f64,
    last_press: Option<(Instant, Target)>,
    /// The grip dragged: the pointer's x and the width at the press; the width it last took
    /// the window to, asked of the host at the next frame (once a frame, however often the
    /// pointer moves).
    resizing: Option<(f64, u32)>,
    /// A press, release, wheel or key here since AUTO GAIN's curve was last asked for (as the
    /// editor opens too): asked again once nothing is held ([`Editing::measure_when_done`]).
    touched: bool,
    gripped: Option<u32>,
    /// The usable screen (logical pixels) at the window's scale (the editor's opening size, the
    /// SIZE menu's FIT TO SCREEN and the sizes that fit).
    screen: fn(f64) -> Option<(f64, f64)>,
    /// How far the drawer was down (`Browser::reveal`) in the last frame composed.
    reveal_shown: f64,
    /// QUALITY's opening (decisions.md R48; `opening.rs`), moved on by the time since it
    /// last was, at most [`ANIMATE_S`] apart (the strip's drops move on with it: the two are
    /// drawn in one frame); the editor opens with QUALITY's setting up. The voices' levels as last taken (0 to 1), for the strip's drops, and the output's
    /// (the one instrument's drop's, and the lamps').
    motion: Motion,
    motion_at: Option<Instant>,
    /// When the opening is next due to move on: a thirtieth of a second after the last time it
    /// was due, not after it last moved, so that baseview's 15 ms frames (which reach a
    /// thirtieth only every third) still move it thirty times a second on average, not 22; and
    /// never due before the present (after a stalled frame it moves once, and once more at the
    /// next frame at most, not in a burst).
    motion_due: Option<Instant>,
    levels: [f64; LEVELS],
    output: f64,
    /// What this computer keeps (`settings.rs`): ULTRA's shutter or always open, ULTRA's
    /// note read. The note shows once ULTRA's lamp is lit, until it is read.
    settings: Settings,
    /// Where they are kept (none in the tests' editors, which keep nothing on the computer).
    settings_file: Option<std::path::PathBuf>,
    ultra_note: bool,
    whats_new: Option<ca72_panel::whats_new::View>,
    whats_new_checked: bool,
    /// A value being typed into a readout (decisions.md R52): which, and what so far; the
    /// keyboard wanted for it, asked once; the strip's last press (where, when, and the amount
    /// a switch had before it), for a double click.
    typing: Option<(Typed, String)>,
    typing_keys: bool,
    last_strip_press: Option<(Instant, StripTarget, f32)>,
    /// Oscillators 2 and 3's FREQUENCY knobs, through the computer's law.
    freq_knobs: [FreqKnob; 2],
    playing: Playing,
    #[cfg(feature = "standalone-production")]
    standalone_dialog: Option<crate::standalone_settings::Dialog>,
    /// AI agents operating the editor through MCP, while allowed (`editor/agent.rs`;
    /// decisions.md R-MCP).
    agents: agent::Agents,
}

impl PanelWindow {
    /// The editor's window: `width` logical pixels wide, `dpr` physical pixels a logical one,
    /// the presets from `library` (the user's, but in the tests).
    #[allow(clippy::too_many_arguments)]
    fn new(
        window: &mut Window<'_>,
        context: Arc<dyn GuiContext>,
        params: Arc<Ca72Params>,
        meters: Arc<Meters>,
        width: u32,
        dpr: f64,
        library: Library,
        host_keys: Arc<HostKeys>,
    ) -> Self {
        let height = height_for_panel(width, params.editor_panel.load(Ordering::Relaxed));
        let physical = (
            (f64::from(width) * dpr).round() as u32,
            (f64::from(height) * dpr).round() as u32,
        );
        let surface = window::Surface::new(window, physical);
        PanelWindow {
            surface,
            physical,
            logical: (width, height),
            shown: None,
            hidden_drawn: None,
            editing: Editing::new(
                context,
                params,
                meters,
                (width, physical.0, dpr),
                library,
                |k| screen::usable(Some(k)),
            ),
            keys: None,
            host_keys,
            twins: KeyTwins::default(),
            cursor: MouseCursor::Default,
        }
    }

    /// A key, the window's own or the host's: the editor's to act on (whether it took it), the
    /// window's size and keyboard following.
    fn key(&mut self, window: &mut Window<'_>, k: &KeyboardEvent) -> bool {
        self.editing.touched = true;
        let taken = self.editing.keyed(k);
        self.follow(window);
        self.keyboard(window);
        taken
    }

    /// The window now `info`'s size: the surface and every frame drawn to it.
    fn resized(&mut self, info: &WindowInfo) {
        let size = info.physical_size();
        let physical = (size.width.max(1), size.height.max(1));
        let logical = info.logical_size();
        self.logical = (logical.width.round() as u32, logical.height.round() as u32);
        if physical == self.physical && info.scale() == self.editing.dpr && self.shown.is_some() {
            return;
        }
        self.physical = physical;
        self.editing.fit(
            info.scale(),
            info.logical_size().width.round() as u32,
            physical.0,
        );
        self.surface.resize(self.physical);
        self.shown = None;
    }

    /// The window resized to what the editor asks (the grip, the drawer below the bar),
    /// when it asked: on macOS baseview reports no resize of its own making, so the frames
    /// follow here; elsewhere the report that follows finds the size already taken.
    fn follow(&mut self, window: &mut Window<'_>) {
        if !std::mem::take(&mut self.editing.resizing_window) {
            return;
        }
        let want = (self.editing.width, self.editing.window_height());
        if want != self.logical {
            let size = Size::new(f64::from(want.0), f64::from(want.1));
            window.resize(size);
            self.resized(&WindowInfo::from_logical_size(size, self.editing.dpr));
        }
    }

    /// The pointer's look: the resize arrows over the grip and while it is dragged (R-SIZE),
    /// else the system's (on macOS through baseview's change 7, `R-MAC-FIXES`).
    fn cursor(&mut self, window: &mut Window<'_>) {
        let want = if self.editing.on_grip() {
            MouseCursor::NwseResize
        } else {
            MouseCursor::Default
        };
        if want == self.cursor {
            return;
        }
        self.cursor = want;
        window.set_mouse_cursor(want);
    }

    /// The keyboard as the drawer wants it: taken when a field takes the caret (baseview's
    /// focus; not on X11, where baseview has none and the keys go to the window under the
    /// pointer), and on Windows given back to the window that had it as the drawer shuts
    /// (baseview there keeps every key its window is given, the host's shortcuts too:
    /// decisions.md R18).
    fn keyboard(&mut self, window: &mut Window<'_>) {
        // (MIDI Learn too, while a control's menu is open or a controller is awaited: Escape
        // closes or cancels it; decisions.md R34.)
        let asked = std::mem::take(&mut self.editing.browser.wants_keys)
            | std::mem::take(&mut self.editing.learning.take_keys)
            | std::mem::take(&mut self.editing.typing_keys)
            | std::mem::take(&mut self.editing.playing.wants_keys);
        if asked {
            let had = window::take_keys(window);
            if self.keys.is_none() {
                self.keys = had;
            }
        }
        let learning = self
            .editing
            .learning
            .holds_keys(&self.editing.params.midi_map);
        let holding = self.editing.browser.open
            || learning
            || self.editing.typing.is_some()
            || self.editing.note_typing_allowed()
            || self.editing.standalone_dialog_open()
            || self.editing.whats_new.is_some();
        // (And the keys the host gives the view: the editor's while it holds them.)
        self.host_keys.holding.store(holding, Ordering::Release);
        // Every key while it holds them, those a host's dialog keeps for itself too (Windows:
        // in REAPER's FX window an arrow moved REAPER's focus to one of its own buttons); the
        // host's again once it lets them go (decisions.md R34).
        window.set_wants_keys(holding);
        if !holding && let Some(keys) = self.keys.take() {
            window::give_keys_back(window, keys);
        }
    }

    /// The frame shown: where it changed in the last `draw` when `partial`, or all of it.
    fn present(&mut self, partial: bool) {
        let e = &self.editing;
        let frame = e.composed.as_ref().unwrap_or_else(|| e.renderer.frame());
        let damage = e.damage.as_deref().filter(|_| partial);
        self.surface.present(self.physical, &[frame], damage);
    }
}

impl Editing {
    /// An editor `width` logical pixels wide, `physical_width` physical ones, `dpr`
    /// physical pixels a logical one; `screen` the usable screen at a scale.
    fn new(
        context: Arc<dyn GuiContext>,
        params: Arc<Ca72Params>,
        meters: Arc<Meters>,
        (width, physical_width, dpr): (u32, u32, f64),
        library: Library,
        screen: fn(f64) -> Option<(f64, f64)>,
    ) -> Self {
        // (The tests' editors keep nothing on the computer.)
        params.editor_presets.store(false, Ordering::Relaxed);
        params.editor_panel_applied.store(
            params.editor_panel.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        params.editor_expanded.store(
            params.editor_panel.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        params.editor_resizing.opening(crate::resizing::View {
            width,
            saved_width: params.editor_width.load(Ordering::Relaxed),
            expanded: params.editor_expanded.load(Ordering::Relaxed),
            panel: params.editor_panel.load(Ordering::Relaxed),
            applied: params.editor_panel_applied.load(Ordering::Relaxed),
            presets: false,
        });
        let settings = if cfg!(test) {
            Settings::default()
        } else {
            Settings::load()
        };
        params.set_freq_law(settings.frequency);
        params.set_tempo(f64::from(settings.tempo));
        let standalone = context.plugin_api() == PluginApi::Standalone;
        let mut playing = Playing::default();
        playing.standalone = standalone;
        playing.visible = settings.keyboard.unwrap_or(standalone);
        playing.root = settings.keyboard_root;
        playing.typing_root = settings.typing_root;
        let freq_knobs = [Pot::Two, Pot::Three].map(|pot| FreqKnob {
            params: Arc::clone(&params),
            pot,
        });
        // The standalone's effects (decisions.md R-FX-DRAWER): the plug-ins have none.
        let effects = if standalone {
            crate::fx_view::installed()
        } else {
            None
        };
        let mut editing = Editing {
            context,
            params,
            meters,
            dpr,
            width,
            physical_width,
            renderer: Renderer::with_look(
                settings.skin,
                settings.lettering,
                f64::from(physical_width) / art::W,
                dpr,
            ),
            scene: Scene::default(),
            strip: StripRenderer::new(f64::from(physical_width) / art::W),
            strip_scene: StripScene::default(),
            last: [0.5; 4],
            drawer: DrawerRenderer::new(f64::from(physical_width) / art::W),
            browser: Browser::new(library),
            arp: ArpRenderer::new(
                f64::from(physical_width) / art::W,
                settings.skin,
                settings.lettering,
            ),
            arp_scene: ArpScene::default(),
            lower: Dropped::default(),
            lower_shown: (0.0, Lower::Arp),
            last_arp_press: None,
            fx_view: FxView::new(None),
            fx: None,
            fx_scene: FxScene::default(),
            rail: Rail::Plugin,
            composed: None,
            damage: None,
            resizing_window: false,
            placed: false,
            update: Update::default(),
            learning: Learning::default(),
            pointer: (0.0, 0.0),
            hover: None,
            drag: None,
            scroll: None,
            list_travel: 0.0,
            last_press: None,
            resizing: None,
            gripped: None,
            screen,
            touched: true,
            reveal_shown: 0.0,
            motion: Motion::default(),
            motion_at: None,
            motion_due: None,
            levels: [0.0; LEVELS],
            output: 0.0,
            settings,
            settings_file: (!cfg!(test)).then(crate::settings::file),
            ultra_note: false,
            whats_new: None,
            whats_new_checked: false,
            typing: None,
            typing_keys: false,
            last_strip_press: None,
            freq_knobs,
            playing,
            #[cfg(feature = "standalone-production")]
            standalone_dialog: None,
            agents: agent::Agents::default(),
        };
        editing.use_effects(FxView::new(effects));
        editing.opened().listening()
    }

    /// The effects' drawer for `view`'s chain: with one, its renderer (its still parts begun on
    /// their thread) and the rail's FX key; without, the plug-ins' rail.
    fn use_effects(&mut self, view: FxView) {
        self.rail = if view.available() {
            Rail::Effects
        } else {
            Rail::Plugin
        };
        self.fx = view.available().then(|| {
            FxRenderer::new(
                self.renderer.scale(),
                self.renderer.skin(),
                self.renderer.lettering(),
            )
        });
        self.fx_view = view;
        self.strip.set_rail(self.rail);
        self.composed = None;
    }

    /// Listening for agents if they are allowed.
    fn listening(mut self) -> Self {
        self.sync_agents();
        self
    }

    /// Agents allowed or not from now on (SETTINGS: AGENTS), on this computer; off, for this
    /// session too (`--agents`).
    fn set_agents(&mut self, on: bool) {
        self.keep_settings(Change::Agents(on));
        if !on {
            crate::agent::force(false);
        }
        self.agents.stop = false;
        self.agents.error = None;
        self.sync_agents();
    }

    /// As it opens: what QUALITY's setting shows up in its opening, not moving (the hamster
    /// asleep); it watches the levels, and the voices' peaks held while it was shut are let go.
    fn opened(mut self) -> Self {
        self.meters.watchers.fetch_add(1, Ordering::Relaxed);
        let _ = self.meters.take_levels();
        self.motion = Motion::at(self.params.quality.value(), 0.0);
        self.scene.opening = self.motion.opening(!self.settings.shutter);
        self
    }

    /// Called only by an editor window's frame, once it is actually visible. Neither
    /// constructing the plug-in/editor nor drawing a hidden editor consumes the claim.
    fn release_notes_when_visible(&mut self, visible: bool) {
        if !visible || self.whats_new_checked {
            return;
        }
        self.whats_new_checked = true;
        if let Some(path) = &self.settings_file {
            match crate::settings::claim_release_notes(path, crate::update::VERSION) {
                Ok(Some(notes)) => self.show_release_notes(notes),
                Ok(None) => {}
                Err(error) => nih_log!("Could not keep What's New history: {error}"),
            }
        }
    }

    fn show_release_notes(&mut self, notes: crate::whats_new::Notes) {
        self.whats_new = Some(notes.view());
        let height = f64::from(self.physical_height()) / self.renderer.scale();
        if let Some(view) = &mut self.whats_new {
            view.fit(self.renderer.fonts(), height, self.settings.lettering);
        }
        self.list_travel = 0.0;
        self.typing_keys = true;
        self.composed = None;
    }

    /// The editor drawn in `skin` from now on, on this computer (decisions.md R51): its
    /// panel's renderer made again in it (its pictures decoded as it first draws).
    fn set_look(&mut self, skin: Skin) {
        self.keep_settings(Change::Skin(skin));
        if self.renderer.skin() != skin {
            let lettering = self.renderer.lettering();
            self.renderer = Renderer::with_look(skin, lettering, self.renderer.scale(), self.dpr);
            self.arp.set_look(skin, lettering);
            if let Some(fx) = &mut self.fx {
                fx.set_look(skin, lettering);
            }
            self.composed = None;
        }
    }

    /// The SIZE menu's sizes (R-SIZE): [`SIZES`], then FIT TO SCREEN, the size the editor
    /// opens at on this screen; each its label, its width (logical pixels) and whether the
    /// window at it, as it is now (the panel shown or not), fits the screen (all, where the
    /// screen is not known).
    fn sizes(&self) -> Vec<(&'static str, u32, bool)> {
        let screen = (self.screen)(self.dpr);
        let expanded = self.expanded();
        let fits = |w: u32| {
            screen.is_none_or(|(sw, sh)| {
                f64::from(w) <= sw && f64::from(height_for_panel(w, expanded)) <= sh
            })
        };
        let mut sizes: Vec<_> = SIZES
            .iter()
            .map(|&(t, k)| {
                let w = clamp_width((f64::from(DEFAULT_WIDTH) * k).round() as u32);
                (t, w, fits(w))
            })
            .collect();
        sizes.push(("FIT TO SCREEN", fitted_width(screen), true));
        sizes
    }

    /// The editor's lettering printed at `lettering`'s sizes from now on, on this computer
    /// (R-LETTERING): its panel's renderer made again with them, in the look it is in (the
    /// tip's, notes' and menus' lettering follow it).
    fn set_lettering(&mut self, lettering: Lettering) {
        self.keep_settings(Change::Lettering(lettering));
        if self.renderer.lettering() != lettering {
            let skin = self.renderer.skin();
            self.renderer = Renderer::with_look(skin, lettering, self.renderer.scale(), self.dpr);
            self.arp.set_look(skin, lettering);
            if let Some(fx) = &mut self.fx {
                fx.set_look(skin, lettering);
            }
            self.composed = None;
        }
    }

    fn standalone_dialog_open(&self) -> bool {
        #[cfg(feature = "standalone-production")]
        {
            self.standalone_dialog.is_some()
        }
        #[cfg(not(feature = "standalone-production"))]
        {
            false
        }
    }

    fn note_typing_allowed(&self) -> bool {
        self.playing.holds_keys()
            && !self.browser.open
            && !self.browser.sliding()
            && self.typing.is_none()
            && !self.learning.holds_keys(&self.params.midi_map)
            && !self.standalone_dialog_open()
            && self.whats_new.is_none()
    }

    /// The FREQUENCY knobs read through `law` from now on, on this computer (decisions.md
    /// R54): each points where its pot is through it, the sound as it was.
    fn set_frequency_dials(&mut self, law: Law) {
        self.keep_settings(Change::Frequency(law));
        self.params.set_freq_law(law);
    }

    /// `change` made to this computer's settings: this editor's copy, and that key alone in the
    /// file (not by the tests' editors), so that another editor's changes since this one opened
    /// stay as they are.
    fn keep_settings(&mut self, change: Change) {
        self.settings.apply(change);
        if let Some(path) = &self.settings_file {
            let _ = Settings::keep(path, change);
        }
    }

    /// QUALITY's opening moved on towards what its setting `q` shows, by the time since it
    /// last was (at most 50 ms of it: a stalled frame does not jump it), and the voices' levels
    /// taken: at most [`ANIMATE_S`] apart (thirty times a second, whatever the screen's rate).
    /// ULTRA's note shown once its lamp is up, until it is read.
    fn move_opening(&mut self, now: Instant, q: QualityMode) {
        if self.motion_due.is_some_and(|due| now < due) {
            return;
        }
        let tick = Duration::from_secs_f64(ANIMATE_S);
        self.motion_due = Some(match self.motion_due {
            Some(due) => (due + tick).max(now),
            None => now + tick,
        });
        let since = self
            .motion_at
            .map(|t| now.saturating_duration_since(t).as_secs_f64());
        let (peaks, output) = self.meters.take_levels();
        self.levels = peaks.map(level_of);
        self.output = level_of(output);
        if self.settings.shutter {
            self.motion
                .step(since.unwrap_or(0.0).min(0.05), q, self.output);
        } else {
            // (None of it shown: what the setting shows kept up, so that shown again it is
            // there, still.)
            self.motion = Motion::at(q, self.output);
        }
        self.motion_at = Some(now);
        let on = q == QualityMode::Ultra;
        if on && self.motion.coil == 1.0 && !self.settings.ultra_note_read && !self.ultra_note {
            // (Read in another editor since this one opened: not shown again, once a computer.)
            let read = self
                .settings_file
                .as_ref()
                .is_some_and(|p| Settings::read(p).ultra_note_read);
            if read {
                self.settings.apply(Change::NoteRead);
            } else {
                self.ultra_note = true;
            }
        }
        if !on {
            self.ultra_note = false;
        }
        self.scene.opening = self.motion.opening(!self.settings.shutter);
    }

    /// ULTRA's note: what it is for (the owner: "only intended for offline renders, or small
    /// projects on powerful PCs") and what it costs, as this computer plays it (whether it
    /// splits an ULTRA voice over six threads: `engine::split_voices`, decisions.md R-THREADS),
    /// left of its lamp; a press anywhere closes it, read.
    fn ultra_note(&self) -> Note {
        self.ultra_note_for(crate::engine::split_voices() > 0)
    }

    /// [`Editing::ultra_note`] for a computer that splits an ULTRA voice (`splits`) or not.
    fn ultra_note_for(&self, splits: bool) -> Note {
        let size = self.renderer.text_size();
        let lines = ultra_note_lines(splits)
            .iter()
            .map(|&(t, tone)| (t.to_owned(), tone))
            .collect();
        let mut note = Note {
            x: 0.0,
            y: 0.0,
            size,
            lines,
        };
        let (w, h) = note.extent(self.renderer.fonts());
        // (Its box's right edge a little left of the lamp's column, its top at the lamp's.)
        note.x = art::COL + 2940.0 - w / 2.0;
        note.y = art::TOP + 40.0 + h + size * 0.4;
        note
    }

    /// A change made here done (nothing held, no wheel's gesture open): AUTO GAIN's curve asked
    /// for the sound as it now is, measured unless it is already its (decisions.md R45,
    /// the CA-74's R29). Only the editor asks: the host's automation and learned controllers
    /// never do, so that a render does not depend on when a measurement finished.
    fn measure_when_done(&mut self) {
        if self.touched && !self.dragging() && self.scroll.is_none() {
            self.touched = false;
            self.params.drive_curve.ask(&self.params.controls());
        }
    }

    /// The renderers at the window's scale: `dpr` physical pixels a logical one, `width`
    /// logical pixels wide, `physical_width` physical ones.
    fn fit(&mut self, dpr: f64, width: u32, physical_width: u32) {
        // A height-only snap keeps the scale and the frame just composed. In particular,
        // state recall can ask for that height during draw(), before the window follows it.
        if self.dpr == dpr && self.width == width && self.physical_width == physical_width {
            return;
        }
        self.dpr = dpr;
        self.width = width;
        self.physical_width = physical_width;
        let scale = f64::from(physical_width) / art::W;
        self.renderer.rescale(scale, dpr);
        self.strip.rescale(scale);
        self.drawer.rescale(scale);
        self.arp.rescale(scale);
        if let Some(fx) = &mut self.fx {
            fx.rescale(scale);
        }
        self.composed = None;
    }

    fn setter(&self) -> ParamSetter<'_> {
        ParamSetter::new(self.context.as_ref())
    }

    /// The pointer's place in the drawing.
    fn in_drawing(&self, (x, y): (f64, f64)) -> (f64, f64) {
        let k = self.dpr / self.renderer.scale();
        (x * k, y * k)
    }

    /// The compact view's grip is on the rail; controls outside the view cannot be hit.
    fn hit(&self, x: f64, y: f64) -> Option<Target> {
        if !self.expanded() {
            let bottom = f64::from(self.physical_height()) / self.renderer.scale();
            if !(0.0..bottom).contains(&y) {
                return None;
            }
            if x >= art::W - COMPACT_GRIP && y >= bottom - COMPACT_GRIP {
                return Some(Target::Grip);
            }
            if y >= DRAWER_TOP {
                return None;
            }
        }
        interact::hit(self.renderer.layout(), x, y)
    }

    /// The drawer's top now (drawing units) and where it is cut off, while it shows: dropping
    /// down from under the rail over the strip's sections (A6), the window as it is.
    fn drawer_span(&self) -> Option<(f64, f64, f64)> {
        let k = self.browser.reveal();
        if k <= 0.0 {
            return None;
        }
        Some((DRAWER_TOP - (1.0 - k) * DRAWER_H, DRAWER_TOP, art::H))
    }

    /// The pointer's place in the drawer (drawer units), if it is over it.
    fn in_drawer(&self, (x, y): (f64, f64)) -> Option<(f64, f64)> {
        let (top, from, to) = self.drawer_span()?;
        (y >= from && y < to).then_some((x, y - top))
    }

    /// The drawer dropped from the rail: its top now (drawing units), while it shows, dropping
    /// down from under the rail, over the lower page.
    fn lower_top(&self) -> Option<f64> {
        let k = self.lower.reveal();
        (k > 0.0 && self.expanded()).then_some(lower_ui::TOP - (1.0 - k) * self.lower.which.depth())
    }

    /// Whether the pointer (the drawing's units) is over the drawer dropped from the rail, as much
    /// of it as shows, or the effects' list hanging from it (the presets' drawer, over it, apart).
    fn in_lower(&self, (x, y): (f64, f64)) -> bool {
        self.lower_top().is_some_and(|top| {
            let plate = (lower_ui::LEFT..lower_ui::RIGHT).contains(&x)
                && y >= DRAWER_TOP
                && y < top + self.lower.which.depth();
            let list = self.lower.which == Lower::Fx
                && self.lower.settled()
                && self
                    .fx_scene
                    .list
                    .as_ref()
                    .is_some_and(|l| fx_ui::list_hit(l, x, y).is_some());
            plate || list
        }) && self.in_drawer((x, y)).is_none()
    }

    /// Whether the pointer is over the arpeggiator's drawer.
    fn in_arp(&self, at: (f64, f64)) -> bool {
        self.lower.which == Lower::Arp && self.in_lower(at)
    }

    /// Whether the pointer is over the effects' drawer or its list.
    fn in_fx(&self, at: (f64, f64)) -> bool {
        self.lower.which == Lower::Fx && self.in_lower(at)
    }

    fn expanded(&self) -> bool {
        self.params.editor_expanded.load(Ordering::Relaxed)
    }

    /// The window's height (logical pixels): the synth and rail, with the lower panel shown.
    fn window_height(&self) -> u32 {
        height_for_panel(self.width, self.expanded())
    }

    fn resize_view(&self) -> crate::resizing::View {
        crate::resizing::View {
            width: self.width,
            saved_width: self.params.editor_width.load(Ordering::Relaxed),
            expanded: self.expanded(),
            panel: self.params.editor_panel.load(Ordering::Relaxed),
            applied: self.params.editor_panel_applied.load(Ordering::Relaxed),
            presets: self.browser.open,
        }
    }

    fn reconcile_resize(&mut self) {
        self.params.editor_resizing.finish(self.resize_view());
        if let Some(view) = self.params.editor_resizing.take_restore() {
            if view == self.resize_view() {
                return;
            }
            // draw() can consume a delayed refusal before the native window follows it.
            // Refit now so that following the restored width preserves the composed frame.
            let physical = WindowInfo::from_logical_size(
                Size::new(
                    f64::from(view.width),
                    f64::from(height_for_panel(view.width, view.expanded)),
                ),
                self.dpr,
            )
            .physical_size();
            self.fit(self.dpr, view.width, physical.width);
            self.params
                .editor_width
                .store(view.saved_width, Ordering::Relaxed);
            self.params
                .editor_expanded
                .store(view.expanded, Ordering::Relaxed);
            self.params
                .editor_panel
                .store(view.panel, Ordering::Relaxed);
            self.params
                .editor_panel_applied
                .store(view.applied, Ordering::Relaxed);
            self.browser.show(view.presets, &self.params);
            self.placed = view.presets;
            self.params
                .editor_presets
                .store(view.presets, Ordering::Relaxed);
            if !view.expanded {
                self.playing.release_pointer();
                self.playing.hover = None;
            }
            self.playing.wants_keys = self.playing.holds_keys();
            self.typing = None;
            self.hover = None;
            self.strip_scene.hover = None;
            self.learning.fit_menu(
                self.renderer.fonts(),
                [
                    0.0,
                    0.0,
                    art::W,
                    if view.expanded { art::H } else { DRAWER_TOP },
                ],
            );
            self.composed = None;
            self.resizing_window = true;
        }
    }

    fn ask_resize(&self, width: u32) -> bool {
        let request = self
            .params
            .editor_resizing
            .issue((width, height_for_panel(width, self.expanded())));
        let accepted = self.context.request_resize();
        if !accepted || !self.context.resize_response_will_follow() {
            self.params.editor_resizing.answer(request, accepted);
        }
        accepted
    }

    fn physical_height(&self) -> u32 {
        (f64::from(self.window_height()) * self.dpr)
            .round()
            .max(1.0) as u32
    }

    /// Publish the height before asking the host: its size query sees the requested view.
    /// Refusing a resize leaves the old view intact.
    fn set_panel_height(&mut self, expanded: bool) -> bool {
        let was = self
            .params
            .editor_expanded
            .swap(expanded, Ordering::Relaxed);
        if was == expanded {
            return true;
        }
        if !self.ask_resize(self.width) {
            self.params.editor_expanded.store(was, Ordering::Relaxed);
            return false;
        }
        if !expanded {
            self.playing.release_pointer();
            self.playing.hover = None;
        }
        self.playing.wants_keys = self.playing.holds_keys();
        self.resizing_window = true;
        self.composed = None;
        true
    }

    /// Show or hide all of the lower panel at once, outside the sound's saved parameters.
    fn toggle_panel(&mut self) {
        self.reconcile_resize();
        let expanded = !self.expanded();
        if !self.set_panel_height(expanded) {
            return;
        }
        self.params.editor_panel.store(expanded, Ordering::Relaxed);
        self.params
            .editor_panel_applied
            .store(expanded, Ordering::Relaxed);
        if !expanded {
            self.lower.shut();
            self.fx_view.close_list();
        }
        self.typing = None;
        self.hover = None;
        self.strip_scene.hover = None;
        self.browser.show(false, &self.params);
        self.learning.close_menu();
        self.follow_drawer();
        self.params.editor_resizing.finish(self.resize_view());
    }

    /// The drawer placed as it opens: always over the strip, the window as it is (A6; it had
    /// grown the window below the strip: decisions.md R18, R19).
    fn follow_drawer(&mut self) {
        let open = self.browser.open;
        if open == self.placed {
            return;
        }
        self.placed = open;
        self.params.editor_presets.store(open, Ordering::Relaxed);
        if open {
            self.playing.release();
        }
        self.browser.below = false;
        let expanded = self.params.editor_panel.load(Ordering::Relaxed) || open;
        if !self.set_panel_height(expanded) {
            // If a host stops accepting sizes mid-session, keep the view it still owns.
            if open {
                self.browser.show(false, &self.params);
                self.placed = false;
                self.params.editor_presets.store(false, Ordering::Relaxed);
            } else {
                self.params.editor_panel.store(true, Ordering::Relaxed);
                self.params
                    .editor_panel_applied
                    .store(true, Ordering::Relaxed);
            }
        }
        self.composed = None;
        self.params.editor_resizing.finish(self.resize_view());
    }

    /// The rail's part under the pointer (the drawing's units), if it is over one.
    fn on_rail(&self, (x, y): (f64, f64)) -> Option<BarTarget> {
        match strip::hit(self.rail, x, y) {
            Some(StripTarget::Bar(t)) => Some(t),
            _ => None,
        }
    }

    fn operated(&self, control: usize) -> Option<&dyn Operated> {
        match CONTROLS.get(control)?.param {
            "osc2_frequency" => Some(&self.freq_knobs[0]),
            "osc3_frequency" => Some(&self.freq_knobs[1]),
            id => param(&self.params, id),
        }
    }

    fn value_of(&self, control: usize) -> f64 {
        self.operated(control).map_or(0.0, |p| f64::from(p.value()))
    }

    /// One change of a control, a gesture of its own.
    fn set_once(&self, control: usize, v: f64) {
        if let Some(p) = self.operated(control) {
            let s = self.setter();
            p.begin(&s);
            p.set(&s, v as f32);
            p.end(&s);
        }
    }

    /// A value to be typed into readout `r` (decisions.md R52): not AUTO GAIN's, which reads
    /// what is measured, nor DETUNE's in SCATTER or VOICES's where it does nothing
    /// ([`voices_inert`]), their knobs doing nothing either.
    fn start_typing(&mut self, r: Readout) {
        self.playing.release();
        if !r.typed()
            || (r == Readout::Detune && !self.params.doubled.value())
            || (r == Readout::Voices && voices_inert(&self.params))
        {
            return;
        }
        self.typing = Some((Typed::Strip(r), String::new()));
        self.typing_keys = true;
    }

    /// A value to be typed into the arpeggiator's drawer's readout `i` (RATE, OCTAVES, GATE,
    /// SWING; decisions.md R-ARP).
    fn start_arp_typing(&mut self, i: usize) {
        self.playing.release();
        self.typing = Some((Typed::Arp(i.min(3)), String::new()));
        self.typing_keys = true;
    }

    /// A key while a value is typed: a digit, the point or the sign added where they fit, the
    /// last taken back, Enter setting the value, Escape leaving it; every key is the editor's
    /// meanwhile.
    fn type_key(&mut self, k: &KeyboardEvent) -> bool {
        if k.state != KeyState::Down {
            return true;
        }
        let Some((r, text)) = &mut self.typing else {
            return false;
        };
        match &k.key {
            Key::Character(c) => {
                for ch in c.chars() {
                    let cells = text.chars().filter(|c| *c != '.').count();
                    // (RATE's note value: dotted with a point after it, a triplet with a T;
                    // OCTAVES one digit.)
                    let ended = text.ends_with(['.', 't']);
                    let fits = match (*r, ch) {
                        (Typed::Arp(0), '0'..='9') => cells < 2 && !ended,
                        (Typed::Arp(0), '.') => cells > 0 && !ended,
                        (Typed::Arp(0), 't' | 'T') => cells > 0 && !ended,
                        (Typed::Arp(1), '0'..='9') => cells < 1,
                        (Typed::Arp(1), _) => false,
                        (_, '0'..='9') => cells < 3,
                        (_, '.') => !text.contains('.') && cells < 3,
                        (Typed::Strip(r), '-') => text.is_empty() && matches!(r, Readout::Level),
                        _ => false,
                    };
                    let ch = ch.to_ascii_lowercase();
                    if fits {
                        text.push(ch);
                    }
                }
            }
            Key::Backspace => {
                text.pop();
            }
            Key::Enter => {
                let (r, text) = (*r, text.clone());
                self.typing = None;
                match r {
                    Typed::Strip(r) => self.set_typed(r, &text),
                    Typed::Arp(i) => self.set_arp_typed(i, &text),
                }
            }
            Key::Escape => self.typing = None,
            _ => {}
        }
        true
    }

    /// The value typed into readout `r` set, as its readout reads it (VOICES a count, ENTROPY,
    /// WIDTH and INNER percent, DETUNE cents, DRIVE and LEVEL dB), within its knob's travel, a
    /// gesture of its own; nothing for what is not a number.
    fn set_typed(&mut self, r: Readout, text: &str) {
        let Ok(v) = text.parse::<f64>() else {
            return;
        };
        if !v.is_finite() {
            return;
        }
        let p = &self.params;
        let (id, normalized) = match r {
            Readout::Voices => (
                "voices",
                p.voices
                    .preview_normalized(v.round().clamp(2.0, 10.0) as i32),
            ),
            Readout::Entropy => ("entropy", p.entropy.preview_normalized(v as f32)),
            Readout::Width => ("spread", p.spread.preview_normalized(v as f32)),
            Readout::Inner => ("inner", p.inner.preview_normalized(v as f32)),
            Readout::Detune => (
                "double",
                p.double
                    .preview_normalized((v / crate::engine::DOUBLE_CENTS * 100.0) as f32),
            ),
            Readout::Drive => ("drive", p.drive.preview_normalized(v as f32)),
            Readout::Level => ("level", p.level.preview_normalized(v as f32)),
            Readout::Auto => return,
        };
        if let Some(i) = ca72_panel::controls::index(id) {
            self.set_once(i, f64::from(normalized.clamp(0.0, 1.0)));
        }
    }

    /// The value typed into the arpeggiator's drawer's readout `i` set, as it reads it: RATE a
    /// note value (16, 16. dotted, 16t a triplet: one of its thirteen), OCTAVES 1 to 4, GATE and
    /// SWING percent; nothing for what is not one.
    fn set_arp_typed(&mut self, i: usize, text: &str) {
        let p = &self.params;
        let normalized = match i {
            0 => {
                let Some(k) = crate::arp::RATES.iter().position(|(t, _)| *t == text) else {
                    return;
                };
                p.arp_rate.preview_normalized(ArpRate::from_index(k))
            }
            1 => {
                let Ok(v) = text.parse::<i32>() else { return };
                p.arp_octaves.preview_normalized(
                    v.clamp(crate::params::ARP_OCTAVES.0, crate::params::ARP_OCTAVES.1),
                )
            }
            _ => {
                let Ok(v) = text.parse::<f32>() else { return };
                if !v.is_finite() {
                    return;
                }
                let q = if i == 2 { &p.arp_gate } else { &p.arp_swing };
                q.preview_normalized(v.clamp(0.0, 100.0))
            }
        };
        if let Some(c) = ca72_panel::controls::index(arp_ui::KNOB_PARAMS[i.min(3)]) {
            self.set_once(c, f64::from(normalized.clamp(0.0, 1.0)));
        }
    }

    /// A press on the arpeggiator's drawer's tabs and readouts (decisions.md R-ARP): ON and
    /// LATCH each switched, DIRECTION's choice set, a readout double-clicked typed into.
    fn arp_press(&mut self, t: ArpTarget) {
        let now = Instant::now();
        let double = self
            .last_arp_press
            .is_some_and(|(at, last)| last == t && now - at < DOUBLE_CLICK);
        self.last_arp_press = Some((now, t));
        let p = &self.params;
        match t {
            ArpTarget::Tab(ArpBank::Arp, 0) => self.set_switch(&p.arp_on, !p.arp_on.value()),
            ArpTarget::Tab(ArpBank::Arp, _) => {
                self.set_switch(&p.arp_latch, !p.arp_latch.value());
            }
            ArpTarget::Tab(ArpBank::Direction, i) => {
                let q = &p.arp_direction;
                let last = ArpDirection::variants().len() - 1;
                let to = q.preview_normalized(ArpDirection::from_index(i.min(last)));
                if q.unmodulated_normalized_value() != to {
                    let s = self.setter();
                    s.begin_set_parameter(q);
                    s.set_parameter_normalized(q, to);
                    s.end_set_parameter(q);
                }
            }
            ArpTarget::Readout(i) => {
                if double {
                    self.last_arp_press = None;
                    self.start_arp_typing(i);
                }
            }
        }
    }

    /// The rail's ARP key, or the standalone's FX key: its drawer dropped down over the lower
    /// page (the other's put away for it), or put away (the presets' drawer, if open, shut for it;
    /// the lower panel shown if hidden).
    fn toggle_lower(&mut self, which: Lower) {
        if !self.expanded() {
            self.toggle_panel();
            if !self.expanded() {
                return;
            }
            self.lower.show(which, true);
        } else if self.browser.open {
            self.browser.show(false, &self.params);
            self.follow_drawer();
            self.lower.show(which, true);
        } else {
            self.lower.show(which, !self.lower.is_open(which));
        }
        self.typing = None;
        self.hover = None;
        self.strip_scene.hover = None;
        self.arp_scene.hover = None;
        self.fx_view.hover = None;
        self.fx_view.cancel_press();
        self.fx_view.close_list();
        self.composed = None;
    }

    fn set_power(&self, on: bool) {
        let s = self.setter();
        s.begin_set_parameter(&self.params.bypass);
        s.set_parameter(&self.params.bypass, !on);
        s.end_set_parameter(&self.params.bypass);
    }

    /// An amount the strip turns off and back on (ENTROPY's, WIDTH's and INNER's readouts,
    /// SCATTER | DOUBLE): its parameter and its place in `last`.
    fn amount(&self, a: Readout) -> (&FloatParam, usize) {
        match a {
            Readout::Width => (&self.params.spread, 1),
            Readout::Detune => (&self.params.double, 2),
            Readout::Inner => (&self.params.inner, 3),
            _ => (&self.params.entropy, 0),
        }
    }

    /// An amount turned off (0, kept) or back on at it (half way if it never was), a gesture of
    /// its own; nothing if it is so already.
    fn switch_amount(&mut self, a: Readout, on: bool) {
        let (p, k) = self.amount(a);
        let now = p.unmodulated_normalized_value();
        if (now > 0.0) == on {
            return;
        }
        let to = if on {
            self.last[k]
        } else {
            self.last[k] = now;
            0.0
        };
        let (p, _) = self.amount(a);
        let s = self.setter();
        s.begin_set_parameter(p);
        s.set_parameter_normalized(p, to);
        s.end_set_parameter(p);
    }

    /// Whether control `i` answers the pointer: all but DETUNE in SCATTER (the owner: "make that
    /// knob unresponsive unless the user is in double mode") and VOICES where it does nothing
    /// ([`voices_inert`]), each drawn dimmed then.
    fn operable(&self, i: usize) -> bool {
        CONTROLS.get(i).is_none_or(|c| match c.param {
            "double" => self.params.doubled.value(),
            "voices" => !voices_inert(&self.params),
            _ => true,
        })
    }

    /// A switch set, a gesture of its own; nothing if it is so already.
    fn set_switch(&self, p: &BoolParam, on: bool) {
        if p.value() != on {
            let s = self.setter();
            s.begin_set_parameter(p);
            s.set_parameter(p, on);
            s.end_set_parameter(p);
        }
    }

    /// A press on the strip's own parts (the drawing's units): a tab, a readout that is a
    /// switch, the rail's keys and the name.
    fn strip_press(&mut self, t: StripTarget) {
        let now = Instant::now();
        let double = self
            .last_strip_press
            .filter(|(at, last, _)| *last == t && now - *at < DOUBLE_CLICK);
        // A readout double-clicked: a value typed into it (decisions.md R52); a switch's
        // first press, which switched its amount, undone.
        if let (Some((_, _, was)), StripTarget::Switch(r) | StripTarget::Readout(r)) = (double, t) {
            self.last_strip_press = None;
            if let StripTarget::Switch(a) = t {
                let (p, _) = self.amount(a);
                if p.unmodulated_normalized_value() != was {
                    let s = self.setter();
                    s.begin_set_parameter(p);
                    s.set_parameter_normalized(p, was);
                    s.end_set_parameter(p);
                }
            }
            self.start_typing(r);
            return;
        }
        let before = match t {
            StripTarget::Switch(a) => self.amount(a).0.unmodulated_normalized_value(),
            _ => 0.0,
        };
        self.last_strip_press = Some((now, t, before));
        match t {
            StripTarget::Bar(BarTarget::Panel) => self.toggle_panel(),
            StripTarget::Bar(BarTarget::Arp) => self.toggle_lower(Lower::Arp),
            StripTarget::Bar(BarTarget::Fx) if self.fx_view.available() => {
                self.toggle_lower(Lower::Fx);
            }
            StripTarget::Readout(_) => {}
            StripTarget::Switch(a) => {
                let on = self.amount(a).0.unmodulated_normalized_value() <= 0.0;
                self.switch_amount(a, on);
            }
            // MONO: POLY and UNISON off; POLY: POLY on, UNISON off; UNISON: on (POLY as it was,
            // which UNISON outranks: the CA-74's R31).
            StripTarget::Tab(Bank::Mode, i) => {
                let p = &self.params;
                match i {
                    0 => {
                        self.set_switch(&p.unison, false);
                        self.set_switch(&p.poly, false);
                    }
                    1 => {
                        self.set_switch(&p.unison, false);
                        self.set_switch(&p.poly, true);
                    }
                    _ => self.set_switch(&p.unison, true),
                }
            }
            StripTarget::Tab(Bank::Stereo, i) => self.set_switch(&self.params.doubled, i == 1),
            // The tabs are the placement's choices in their order: EVEN, CENTER.
            StripTarget::Tab(Bank::Placement, i) => {
                let p = &self.params.placement;
                let last = Scatter::variants().len() - 1;
                let to = p.preview_normalized(Scatter::from_index(i.min(last)));
                if p.unmodulated_normalized_value() != to {
                    let s = self.setter();
                    s.begin_set_parameter(p);
                    s.set_parameter_normalized(p, to);
                    s.end_set_parameter(p);
                }
            }
            StripTarget::Tab(Bank::Auto, _) => {
                let on = !self.params.auto_gain.value();
                self.set_switch(&self.params.auto_gain, on);
            }
            // The settings' key: its menu, dropping down from under it to its left.
            StripTarget::Bar(BarTarget::Keyboard) => {
                self.playing.release_pointer();
                self.playing.visible = !self.playing.visible;
                self.keep_settings(Change::Keyboard(self.playing.visible));
                self.playing.wants_keys = self.playing.holds_keys();
                self.hover = None;
                self.strip_scene.hover = None;
                self.composed = None;
            }
            StripTarget::Bar(BarTarget::Settings) => {
                self.playing.release();
                let size = self.renderer.text_size() * MENU_TEXT;
                self.learning.open_settings_menu(
                    self.renderer.fonts(),
                    settings_menu_corner(),
                    size,
                    (
                        self.settings.skin,
                        self.settings.lettering,
                        self.settings.frequency,
                        self.playing.standalone.then(|| self.agents_allowed()),
                    ),
                );
            }
            StripTarget::Bar(b) => {
                if matches!(
                    b,
                    BarTarget::Prev | BarTarget::Next | BarTarget::Name | BarTarget::Save
                ) {
                    self.playing.release();
                }
                let setter = ParamSetter::new(self.context.as_ref());
                self.browser.bar_press(b, &self.params, &setter);
                self.follow_drawer();
            }
            StripTarget::Display => {}
        }
    }

    fn press(&mut self, modifiers: Modifiers) {
        let (x, y) = self.in_drawing(self.pointer);
        if let Some(notes) = &mut self.whats_new {
            if notes.press(x, y) {
                self.whats_new = None;
            }
            self.composed = None;
            return;
        }
        #[cfg(feature = "standalone-production")]
        if let Some(dialog) = &mut self.standalone_dialog {
            if dialog.press(x, y) {
                self.standalone_dialog = None;
            }
            self.composed = None;
            return;
        }
        if !self.expanded() && y >= DRAWER_TOP {
            return;
        }
        // A press on the readout a value is being typed into leaves it typed into; anywhere
        // else ends the typing, nothing set (decisions.md R52).
        if let Some((r, _)) = self.typing {
            let on_it = match r {
                Typed::Strip(r) => {
                    !self.in_arp((x, y))
                        && matches!(strip::hit(self.rail, x, y), Some(StripTarget::Switch(t) | StripTarget::Readout(t)) if t == r)
                }
                Typed::Arp(i) => {
                    self.in_arp((x, y)) && arp_ui::hit(x, y) == Some(ArpTarget::Readout(i))
                }
            };
            if on_it {
                return;
            }
        }
        // A wheel's gesture still waiting for its rest, or a drag whose release never came
        // (its window lost the pointer): ended before anything else begins.
        self.end_gestures();
        // ULTRA's note showing: the press closes it, read.
        if self.ultra_note {
            self.ultra_note = false;
            self.keep_settings(Change::NoteRead);
            return;
        }
        // A control's menu open: the press is its own (an item, or closing it).
        match self
            .learning
            .press(self.renderer.fonts(), &self.params.midi_map, (x, y))
        {
            Pressed::Nothing => {}
            Pressed::Done => return,
            Pressed::Indicator(shown) => {
                self.keep_settings(Change::Shutter(shown));
                return;
            }
            Pressed::WhatsNew => {
                self.show_release_notes(crate::whats_new::current());
                return;
            }
            Pressed::StandaloneSettings => {
                self.playing.release();
                #[cfg(feature = "standalone-production")]
                crate::standalone_settings::request_open();
                return;
            }
            Pressed::Dials(law) => {
                self.set_frequency_dials(law);
                return;
            }
            Pressed::Look(skin) => {
                self.set_look(skin);
                return;
            }
            Pressed::Lettering(lettering) => {
                self.set_lettering(lettering);
                return;
            }
            Pressed::Agents(on) => {
                self.set_agents(on);
                return;
            }
            Pressed::SizeMenu => {
                let size = self.renderer.text_size() * MENU_TEXT;
                let sizes = self.sizes();
                self.learning.open_size_menu(
                    self.renderer.fonts(),
                    settings_menu_corner(),
                    size,
                    &sizes,
                    self.width,
                );
                return;
            }
            Pressed::Size(width) => {
                // Asked of the host at the next frame, as the grip's are.
                self.gripped = Some(width);
                return;
            }
            Pressed::List(chosen) => {
                self.show_midi_list(chosen);
                return;
            }
            Pressed::FxReplace(slot) => {
                self.fx_view.replace(slot);
                return;
            }
            Pressed::FxRemove(slot) => {
                self.fx_view.remove(slot);
                return;
            }
        }
        // The effects' list open: a press elsewhere puts it away, and is its own.
        if self.fx_view.list_open() && !self.in_fx((x, y)) {
            self.fx_view.close_list();
            return;
        }
        if let Some((dx, dy)) = self.in_drawer((x, y)) {
            match drawer_hit(&self.browser.drawer, dx, dy) {
                Some(DrawerTarget::Update) => self.update.press(),
                Some(DrawerTarget::Key(PresetKey::MidiLearn)) => {
                    self.learning.list = !self.learning.list;
                }
                Some(DrawerTarget::MidiRow(i)) => {
                    self.learning.list_press(&self.params.midi_map, i, None);
                }
                Some(DrawerTarget::MidiAction(i, a)) => {
                    self.learning.list_press(&self.params.midi_map, i, Some(a));
                }
                Some(DrawerTarget::Back) | None => {}
                // Anything of the presets' shows them again in the MIDI list's place.
                Some(t) => {
                    self.learning.list = false;
                    let setter = ParamSetter::new(self.context.as_ref());
                    self.browser.drawer_press(t, dx, &self.params, &setter);
                }
            }
            self.follow_drawer();
            return;
        }
        // The arpeggiator's drawer, over the lower page: its own parts only, and while it slides
        // nothing.
        if self.in_arp((x, y)) {
            if !self.lower.settled() {
                return;
            }
            if let Some(t) = arp_ui::hit(x, y) {
                self.arp_press(t);
            } else if let Some(target) = interact::hit_arp(x, y) {
                self.press_target(target, modifiers);
            }
            return;
        }
        // The effects' drawer and its list (decisions.md R-FX-DRAWER).
        if self.in_fx((x, y)) {
            if self.lower.settled() {
                self.fx_view.press(x, y);
            }
            return;
        }
        if self.playing.visible
            && let Some(target) = keyboard::hit(self.playing.root, x, y, self.playing.standalone)
        {
            match target {
                keyboard::Target::Note(note) => self.playing.press(note),
                keyboard::Target::Down | keyboard::Target::Up => {
                    self.playing.shift(target == keyboard::Target::Up);
                    self.keep_settings(Change::KeyboardRoot(self.playing.root));
                }
                keyboard::Target::Type => {
                    self.playing.release();
                    self.playing.typing = !self.playing.typing;
                    self.playing.wants_keys = self.playing.typing;
                }
                keyboard::Target::Panic => self.panic(),
                keyboard::Target::Back => {}
            }
            self.hover = None;
            self.strip_scene.hover = None;
            return;
        }
        if let Some(t) = strip::hit(self.rail, x, y) {
            self.strip_press(t);
            return;
        }
        let Some(target) = self.hit(x, y) else {
            return;
        };
        self.press_target(target, modifiers);
    }

    /// A press on a control (the panel's, the strip's knobs, the arpeggiator's drawer's), on
    /// POWER or the grip.
    fn press_target(&mut self, target: Target, modifiers: Modifiers) {
        let now = Instant::now();
        let double = self
            .last_press
            .is_some_and(|(t, last)| last == target && now - t < DOUBLE_CLICK);
        self.last_press = Some((now, target));
        match target {
            Target::Control(i) | Target::Legend(i, _) => {
                if !self.operable(i) {
                    return;
                }
                let kind = CONTROLS[i].kind;
                if double && !matches!(kind, Kind::Rocker { .. } | Kind::Toggle { .. }) {
                    if let Some(p) = self.operated(i) {
                        self.set_once(i, f64::from(p.default()));
                    }
                    self.last_press = None;
                    return;
                }
                let value = self.value_of(i);
                if !matches!(kind, Kind::Rocker { .. } | Kind::Toggle { .. })
                    && let Some(p) = self.operated(i)
                {
                    p.begin(&self.setter());
                }
                self.drag = Some(Drag {
                    control: i,
                    from: self.pointer,
                    pressed: target,
                    value,
                    base: value,
                    at: self.pointer,
                    fine: modifiers.contains(Modifiers::SHIFT),
                    moved: false,
                });
            }
            Target::Power => self.set_power(self.params.bypass.value()),
            Target::Grip => {
                self.resizing = Some((self.pointer.0, self.width));
            }
        }
    }

    /// PANIC: every note off, from every source, and the controllers reset (the keyboard's
    /// button, and an agent's).
    fn panic(&mut self) {
        #[cfg(feature = "standalone-production")]
        if let Some(app) = plugin_kit_standalone::AppHandle::current() {
            let token = self.playing.panic_standalone();
            let accepted = app.panic_tagged(token);
            debug_assert!(
                accepted,
                "local panic tokens are nonzero and fit in 63 bits"
            );
        } else {
            self.playing.panic();
        }
        #[cfg(not(feature = "standalone-production"))]
        self.playing.panic();
    }

    /// A right click: the menu of the control under the pointer (MIDI Learn: decisions.md R34),
    /// on the panel or the strip; elsewhere none.
    fn context_menu(&mut self) {
        if self.standalone_dialog_open() || self.whats_new.is_some() {
            return;
        }
        self.playing.release();
        self.end_gestures();
        let (x, y) = self.in_drawing(self.pointer);
        if !self.expanded() && y >= DRAWER_TOP {
            return;
        }
        let size = self.renderer.text_size() * MENU_TEXT;
        let map = &self.params.midi_map;
        // (A drawer dropped from the rail is over the keyboard's top: its own menus.)
        if self.playing.visible && keyboard::inside(x, y) && !self.in_lower((x, y)) {
            self.learning.close_menu();
            return;
        }
        if self.in_drawer((x, y)).is_some() || self.on_rail((x, y)).is_some() {
            self.learning.close_menu();
            return;
        }
        // The arpeggiator's drawer: a knob's menu, as the panel's (its tabs are learned from the
        // presets' drawer's MIDI list).
        if self.in_arp((x, y)) {
            match interact::hit_arp(x, y).filter(|_| self.lower.settled()) {
                Some(target @ Target::Control(i)) => self.learning.open_menu(
                    self.renderer.fonts(),
                    map,
                    Some(Place::Panel(i)),
                    Some(target),
                    (x, y),
                    size,
                ),
                _ => self.learning.close_menu(),
            }
            return;
        }
        // An effects' slot: REPLACE… and REMOVE.
        if self.in_fx((x, y)) {
            match self
                .fx_view
                .menu_for(x, y)
                .filter(|_| self.lower.settled() && !self.fx_view.list_open())
            {
                Some((slot, name)) => {
                    self.learning
                        .open_fx_menu(self.renderer.fonts(), (x, y), size, slot, &name);
                }
                None => self.learning.close_menu(),
            }
            return;
        }
        if let Some(t) = strip::hit(self.rail, x, y) {
            // A tab's menu (the strip's knobs' are the panel's), above it.
            match t.control() {
                Some(c) => {
                    let (x0, y0, x1, _) = strip::span(c);
                    self.learning.open_menu(
                        self.renderer.fonts(),
                        map,
                        Some(Place::Strip(c)),
                        None,
                        ((x0 + x1) / 2.0, y0),
                        size,
                    );
                }
                None => self.learning.close_menu(),
            }
            return;
        }
        let target = self.hit(x, y);
        // QUALITY's toggle: its indicator shown or hidden (decisions.md R48).
        if let Some(Target::Control(i) | Target::Legend(i, _)) = target
            && CONTROLS.get(i).is_some_and(|c| c.param == "quality")
        {
            self.learning.open_quality_menu(
                self.renderer.fonts(),
                (x, y),
                size,
                self.settings.shutter,
            );
            return;
        }
        let place = match target {
            Some(Target::Control(i) | Target::Legend(i, _)) => Some(Place::Panel(i)),
            _ => None,
        };
        self.learning
            .open_menu(self.renderer.fonts(), map, place, target, (x, y), size);
    }

    /// The drawer opened on its MIDI list, `chosen` in view.
    fn show_midi_list(&mut self, chosen: Option<usize>) {
        self.learning.show_list(chosen);
        if !self.browser.open {
            self.browser.show(true, &self.params);
        }
        self.follow_drawer();
    }

    /// The pointer moved.
    fn moved(&mut self, modifiers: Modifiers) {
        if self.standalone_dialog_open() || self.whats_new.is_some() {
            return;
        }
        let (x, y) = self.in_drawing(self.pointer);
        if self.playing.dragging() {
            let target = keyboard::hit(self.playing.root, x, y, self.playing.standalone);
            self.playing.drag(match target {
                Some(keyboard::Target::Note(note)) => Some(note),
                _ => None,
            });
            return;
        }
        self.playing.hover = if self.expanded()
            && self.playing.visible
            && self.in_drawer((x, y)).is_none()
            && !self.in_lower((x, y))
            && !self.fx_view.pressing()
            && !self.learning.over_menu(self.renderer.fonts(), (x, y))
        {
            keyboard::hit(self.playing.root, x, y, self.playing.standalone)
        } else {
            None
        };
        if self.playing.hover.is_some() {
            self.hover = None;
            self.strip_scene.hover = None;
            return;
        }
        if let Some((x0, w0)) = self.resizing {
            // Asked of the host at the next frame: a resize renders the whole panel again.
            let width = clamp_width((f64::from(w0) + self.pointer.0 - x0).round().max(0.0) as u32);
            self.gripped = Some(width);
            return;
        }
        let fine = modifiers.contains(Modifiers::SHIFT);
        let pointer = self.pointer;
        let Some(d) = &mut self.drag else {
            let at = self.in_drawing(pointer);
            let (x, y) = at;
            if !self.expanded() && y >= DRAWER_TOP {
                self.hover = None;
                self.strip_scene.hover = None;
                self.browser.bar.hover = None;
                return;
            }
            if self.learning.menu_open() {
                self.learning.hover(self.renderer.fonts(), at);
            }
            let over_drawer = self.in_drawer(at);
            self.browser.drawer.hover =
                over_drawer.and_then(|(dx, dy)| drawer_hit(&self.browser.drawer, dx, dy));
            if self.in_arp(at) {
                let settled = self.lower.settled();
                self.arp_scene.hover = arp_ui::hit(x, y).filter(|_| settled);
                self.hover = interact::hit_arp(x, y).filter(|_| settled);
                self.strip_scene.hover = None;
                self.browser.bar.hover = None;
                return;
            }
            self.arp_scene.hover = None;
            // The effects' drawer: a slot carried, the part under the pointer.
            if self.in_fx(at) || self.fx_view.pressing() {
                if self.lower.settled() {
                    self.fx_view.moved(x, y);
                }
                self.hover = None;
                self.strip_scene.hover = None;
                self.browser.bar.hover = None;
                return;
            }
            self.fx_view.hover = None;
            let on_strip = if over_drawer.is_some() {
                None
            } else {
                strip::hit(self.rail, x, y)
            };
            self.browser.bar.hover = match on_strip {
                Some(StripTarget::Bar(t)) => Some(t),
                _ => None,
            };
            (self.hover, self.strip_scene.hover) = if over_drawer.is_some() {
                (None, None)
            } else if on_strip.is_some() {
                (None, on_strip)
            } else {
                (self.hit(x, y), None)
            };
            return;
        };
        if (pointer.0 - d.from.0).abs() + (pointer.1 - d.from.1).abs() > 3.0 {
            d.moved = true;
        }
        if !d.moved {
            return;
        }
        if fine != d.fine {
            d.base = d.value;
            d.at = pointer;
            d.fine = fine;
        }
        let kind = CONTROLS[d.control].kind;
        if let Some(v) = interact::drag(
            &kind,
            d.base,
            pointer.0 - d.at.0,
            pointer.1 - d.at.1,
            d.fine,
        ) && v != d.value
        {
            d.value = v;
            let (i, v) = (d.control, v);
            if let Some(p) = self.operated(i) {
                p.set(&self.setter(), v as f32);
            }
        }
    }

    /// Whether a press is being held: a control, an effects' slot or the grip dragged.
    fn dragging(&self) -> bool {
        self.drag.is_some()
            || self.resizing.is_some()
            || self.playing.dragging()
            || self.fx_view.pressing()
    }

    fn released(&mut self) {
        self.playing.release_pointer();
        self.fx_view.released();
        if self.resizing.take().is_some() {
            return;
        }
        if let Some(d) = self.drag.take() {
            self.let_go(d, true);
        }
    }

    /// Every gesture still open ended (a wheel's, a drag's, a slider's; the grip let go):
    /// before another begins, as the editor closes and as its window loses the focus or the
    /// pointer's capture. A gesture never begins inside another (decisions.md R18).
    fn end_gestures(&mut self) {
        self.playing.release_pointer();
        self.fx_view.cancel_press();
        self.typing = None;
        self.end_scroll();
        self.resizing = None;
        if let Some(d) = self.drag.take() {
            self.let_go(d, false);
        }
    }

    /// A drag over, its gesture ended: `released` by the pointer (a press that did not move
    /// then picks what it pressed), else cut short.
    fn let_go(&mut self, d: Drag, released: bool) {
        let kind = CONTROLS[d.control].kind;
        let mut v = d.value;
        if released && !d.moved {
            let picked = match d.pressed {
                Target::Legend(_, k) => Some(k as f64 / 5.0),
                _ => {
                    let (cx, cy) = CONTROLS[d.control].centre();
                    let (x, y) = self.in_drawing(d.from);
                    interact::click(&kind, v, x - cx, y - cy)
                }
            };
            if let Some(p) = picked {
                v = p;
            }
        }
        v = interact::release(&kind, v);
        let Some(p) = self.operated(d.control) else {
            return;
        };
        let s = self.setter();
        if matches!(kind, Kind::Rocker { .. } | Kind::Toggle { .. }) {
            if v != d.value {
                p.begin(&s);
                p.set(&s, v as f32);
                p.end(&s);
            }
        } else {
            if v != d.value {
                p.set(&s, v as f32);
            }
            p.end(&s);
        }
    }

    fn scrolled(&mut self, delta: ScrollDelta, modifiers: Modifiers) {
        if let Some(notes) = &mut self.whats_new {
            let rows = match delta {
                ScrollDelta::Lines { y, .. } => f64::from(y),
                ScrollDelta::Pixels { y, .. } => {
                    f64::from(y) * self.dpr / self.renderer.scale() / 60.0
                }
            };
            self.list_travel += rows;
            let whole = self.list_travel.trunc();
            notes.scroll(-(whole as i32));
            self.list_travel -= whole;
            self.composed = None;
            return;
        }
        if self.standalone_dialog_open() {
            return;
        }
        if self.drag.is_some() {
            return;
        }
        let (x, y) = self.in_drawing(self.pointer);
        if self.playing.visible && keyboard::inside(x, y) && !self.in_lower((x, y)) {
            return;
        }
        if let Some((dx, dy)) = self.in_drawer((x, y)) {
            // The list (or the tags, under the pointer) scrolls a row for each row's travel, the rest kept for the next: a
            // wheel's lines (a slow turn on macOS gives fractions of one), a trackpad's or a
            // Magic Mouse's points as the list's rows at this size (each event a few points,
            // which a whole-notch threshold had dropped). A turn the other way starts again.
            let rows = match delta {
                ScrollDelta::Lines { y, .. } => f64::from(y),
                ScrollDelta::Pixels { y, .. } => {
                    f64::from(y) * self.dpr / self.renderer.scale() / ROW_H
                }
            };
            if rows * self.list_travel < 0.0 {
                self.list_travel = 0.0;
            }
            self.list_travel += rows;
            let whole = self.list_travel.trunc();
            if whole != 0.0 {
                if self.learning.list {
                    self.learning.scroll(-(whole as i32));
                } else if in_tags(dx, dy) {
                    self.browser.scroll_tags(-(whole as i32));
                } else {
                    self.browser.scroll(-(whole as i32));
                }
                self.list_travel -= whole;
            }
            return;
        }
        if !self.expanded() && y >= DRAWER_TOP {
            return;
        }
        // The effects' list scrolls a row for each row's travel, as the presets' list does.
        if self.in_fx((x, y)) {
            if self.fx_view.list_open() {
                let rows = match delta {
                    ScrollDelta::Lines { y, .. } => f64::from(y),
                    ScrollDelta::Pixels { y, .. } => {
                        f64::from(y) * self.dpr / self.renderer.scale() / fx_ui::LINE
                    }
                };
                if rows * self.list_travel < 0.0 {
                    self.list_travel = 0.0;
                }
                self.list_travel += rows;
                let whole = self.list_travel.trunc();
                if whole != 0.0 {
                    self.fx_view.scroll(-(whole as i32));
                    self.list_travel -= whole;
                }
            }
            return;
        }
        let over = if self.in_arp((x, y)) {
            interact::hit_arp(x, y).filter(|_| self.lower.settled())
        } else if strip::hit(self.rail, x, y).is_some() {
            return;
        } else {
            self.hit(x, y)
        };
        let Some(Target::Control(i) | Target::Legend(i, _)) = over else {
            return;
        };
        if !self.operable(i) {
            return;
        }
        let (notches, travel) = match delta {
            ScrollDelta::Lines { y, .. } => (
                if y > 0.0 {
                    1.0
                } else if y < 0.0 {
                    -1.0
                } else {
                    0.0
                },
                0.0,
            ),
            ScrollDelta::Pixels { y, .. } => {
                let travel = self
                    .scroll
                    .as_ref()
                    .filter(|s| s.control == i)
                    .map_or(0.0, |s| s.travel)
                    + f64::from(y);
                if travel.abs() < NOTCH {
                    (0.0, travel)
                } else {
                    (travel.signum(), 0.0)
                }
            }
        };
        if self.scroll.as_ref().is_some_and(|s| s.control != i) {
            self.end_scroll();
        }
        if self.scroll.is_none() {
            let value = self.value_of(i);
            if let Some(p) = self.operated(i) {
                p.begin(&self.setter());
            }
            self.scroll = Some(Scroll {
                control: i,
                value,
                last: Instant::now(),
                travel: 0.0,
            });
        }
        let kind = CONTROLS[i].kind;
        let Some(s) = &mut self.scroll else { return };
        s.last = Instant::now();
        s.travel = travel;
        if notches != 0.0 {
            let v = interact::step(
                &kind,
                s.value,
                notches > 0.0,
                modifiers.contains(Modifiers::SHIFT),
            );
            if v != s.value {
                s.value = v;
                if let Some(p) = self.operated(i) {
                    p.set(&self.setter(), v as f32);
                }
            }
        }
    }

    fn end_scroll(&mut self) {
        let Some(s) = self.scroll.take() else { return };
        let kind = CONTROLS[s.control].kind;
        let v = interact::release(&kind, s.value);
        if let Some(p) = self.operated(s.control) {
            let setter = self.setter();
            if v != s.value {
                p.set(&setter, v as f32);
            }
            p.end(&setter);
        }
    }

    /// The width the grip took the window to since the last frame, asked of the host.
    fn grip(&mut self) {
        if let Some(width) = self.gripped.take() {
            self.resize(width);
        }
    }

    /// Asks the host for a window `width` wide: whether it agreed.
    fn resize(&mut self, width: u32) -> bool {
        self.reconcile_resize();
        if width == self.width {
            return false;
        }
        let old = self.params.editor_width.swap(width, Ordering::Relaxed);
        let agreed = self.ask_resize(width);
        if agreed {
            self.width = width;
            self.resizing_window = true;
        } else {
            self.params.editor_width.store(old, Ordering::Relaxed);
        }
        self.params.editor_resizing.finish(self.resize_view());
        agreed
    }

    /// The scene from the parameters, the controls operated and what the audio reports.
    fn update_scene(&mut self) {
        self.playing.sync();
        if self.playing.active()
            && !self.note_typing_allowed()
            && (self.browser.open
                || self.typing.is_some()
                || self.learning.holds_keys(&self.params.midi_map)
                || self.standalone_dialog_open())
        {
            self.playing.release();
        }
        for (i, c) in CONTROLS.iter().enumerate() {
            self.scene.values[i] = match (&self.drag, &self.scroll) {
                (Some(d), _)
                    if d.control == i
                        && !matches!(c.kind, Kind::Rocker { .. } | Kind::Toggle { .. }) =>
                {
                    d.value
                }
                (_, Some(s)) if s.control == i => s.value,
                _ => self.value_of(i),
            };
        }
        self.scene.power = !self.params.bypass.value();
        self.scene.detune_off = !self.params.doubled.value();
        self.scene.voices_off = voices_inert(&self.params);
        self.scene.overload = Meters::load(&self.meters.overload);
        self.move_opening(Instant::now(), self.params.quality.value());
        self.scene.midi = (
            Meters::load(&self.meters.bend),
            Meters::load(&self.meters.modulation),
        );
        self.scene.tip = self.tip();
        self.strip_scene = strip_scene(
            &self.params,
            self.meters.voices.load(Ordering::Relaxed),
            (&self.levels, self.output),
            std::mem::take(&mut self.strip_scene),
        );
        self.strip_scene.keyboard = self.playing.scene(self.renderer.skin());
        self.browser.bar.keyboard = self.playing.visible;
        self.browser.bar.panel = self.expanded();
        self.browser.bar.arp = self.params.arp_on.value();
        // The effects' (decisions.md R-FX-DRAWER): the chain read when it is free; the FX key lit
        // while one plays.
        self.fx_view.poll(Instant::now());
        self.browser.bar.fx = self.fx_view.active();
        self.fx_scene = self.fx_view.scene();
        self.arp_scene = arp_scene(
            &self.params,
            &self.scene.values,
            std::mem::take(&mut self.arp_scene),
        );
        // A value being typed: what is typed so far, lit, the next digit's place marked while
        // there is room for one.
        if let Some((r, text)) = &self.typing {
            let cells = text.chars().filter(|c| *c != '.').count();
            let room = match r {
                Typed::Arp(0) => cells < 2 && !text.ends_with(['.', 't']),
                Typed::Arp(1) => cells < 1,
                _ => cells < 3,
            };
            let shown = if room {
                format!("{text}_")
            } else {
                text.clone()
            };
            match r {
                Typed::Strip(r) => {
                    if let Some(i) = Readout::ALL.iter().position(|x| x == r) {
                        self.strip_scene.readouts[i] = (shown, true);
                    }
                }
                Typed::Arp(i) => self.arp_scene.readouts[(*i).min(3)] = (shown, true),
            }
        }
    }

    /// The tip: over the control operated, else over the one under the pointer (an effects'
    /// slot's: its effect's whole name and maker).
    fn tip(&self) -> Option<(String, f64, f64)> {
        if self.lower.which == Lower::Fx
            && self.lower.settled()
            && self.drag.is_none()
            && let Some(t) = self.fx_view.tip()
        {
            return Some(t);
        }
        let operated = self
            .drag
            .as_ref()
            .map(|d| d.control)
            .or(self.scroll.as_ref().map(|s| s.control));
        let target = match operated {
            Some(i) => Target::Control(i),
            None => self.hover?,
        };
        match target {
            Target::Control(i) | Target::Legend(i, _) => {
                let c = &CONTROLS[i];
                let p = self.operated(i)?;
                let (x, y) = c.centre();
                let top = art::bounds(&c.kind)[1];
                let label = if c.param == "feedback" && feedback_silent(&self.scene.values) {
                    FEEDBACK_SILENT
                } else {
                    c.label
                };
                Some((
                    format!("{label}: {}", p.text(self.scene.values[i] as f32)),
                    x,
                    y + top,
                ))
            }
            Target::Power => {
                let state = if self.params.bypass.value() {
                    "Off (bypassed)"
                } else {
                    "On"
                };
                Some((
                    format!("POWER: {state}"),
                    art::COL + POWER.0,
                    art::TOP + POWER.1 - 64.0,
                ))
            }
            Target::Grip => {
                // Above the grip, its right end at the window's (R-SIZE).
                let size = self.renderer.text_size();
                let w = self
                    .renderer
                    .fonts()
                    .advance(GRIP_TIP, size, Weight::Regular, 0.0)
                    + 0.8 * size;
                let top = if self.expanded() {
                    art::GRIP[1]
                } else {
                    f64::from(self.physical_height()) / self.renderer.scale() - COMPACT_GRIP
                };
                Some((GRIP_TIP.to_owned(), art::W - 8.0 - w / 2.0, top))
            }
        }
    }

    /// Whether the pointer is over the grip, or dragging it.
    fn on_grip(&self) -> bool {
        self.resizing.is_some() || self.hover == Some(Target::Grip)
    }

    /// Brings the frame up to the parameters and the gestures: whether it changed.
    fn draw(&mut self) -> bool {
        self.reconcile_resize();
        let preference = self.params.editor_panel.load(Ordering::Relaxed);
        let applied = self.params.editor_panel_applied.load(Ordering::Relaxed);
        if preference != applied {
            if !self
                .set_panel_height(preference || self.browser.open || self.standalone_dialog_open())
            {
                self.params.editor_panel.store(applied, Ordering::Relaxed);
            }
            self.params.editor_panel_applied.store(
                self.params.editor_panel.load(Ordering::Relaxed),
                Ordering::Relaxed,
            );
            self.params.editor_resizing.finish(self.resize_view());
        }
        #[cfg(feature = "standalone-production")]
        {
            if let Some(mut dialog) = crate::standalone_settings::take_requested()
                && self.standalone_dialog.is_none()
            {
                self.playing.release();
                self.end_gestures();
                dialog.set_tempo(self.settings.tempo);
                self.standalone_dialog = Some(dialog);
                self.playing.wants_keys = true;
                self.composed = None;
            }
            if let Some(dialog) = &mut self.standalone_dialog
                && dialog.frame()
            {
                self.composed = None;
            }
            // TEMPO changed there: the arpeggiator's at once, and this computer's (R-ARP).
            if let Some(bpm) = self
                .standalone_dialog
                .as_mut()
                .and_then(crate::standalone_settings::Dialog::take_tempo)
            {
                self.keep_settings(Change::Tempo(bpm));
                self.params.set_tempo(f64::from(bpm));
            }
        }
        let requested_height = self.params.editor_panel.load(Ordering::Relaxed)
            || self.browser.open
            || self.standalone_dialog_open();
        if requested_height != self.expanded() {
            self.set_panel_height(requested_height);
        }
        if self
            .scroll
            .as_ref()
            .is_some_and(|s| s.last.elapsed() > SCROLL_REST)
        {
            self.end_scroll();
        }
        self.update_scene();
        self.browser.tick(&self.params);
        self.update.tick();
        self.browser.drawer.update = self.update.scene();
        // MIDI Learn: what the audio thread caught assigned; the ring, the note and the menu;
        // the drawer's list (the presets again once the drawer has shut). A tip gives way to a
        // note or a menu.
        let map = &self.params.midi_map;
        self.learning.poll(map);
        let bottom = if self.expanded() { art::H } else { DRAWER_TOP };
        self.learning
            .fit_menu(self.renderer.fonts(), [0.0, 0.0, art::W, bottom]);
        let (note, ring, strip_ring, menu) = self.learning.scene(map, self.renderer.text_size());
        let note = note.or_else(|| self.ultra_note.then(|| self.ultra_note()));
        (self.scene.note, self.scene.learning, self.scene.menu) = (note, ring, menu);
        self.strip_scene.learning = strip_ring.filter(|_| !self.playing.visible);
        if self.scene.note.is_some() || self.scene.menu.is_some() {
            self.scene.tip = None;
        }
        if !self.browser.open && !self.browser.sliding() {
            self.learning.list = false;
        }
        self.browser.drawer.midi = self
            .learning
            .list
            .then(|| self.learning.list_scene(&self.params.midi_map));
        let notes_height = f64::from(self.physical_height()) / self.renderer.scale();
        if let Some(notes) = &mut self.whats_new {
            notes.fit(self.renderer.fonts(), notes_height, self.settings.lettering);
        }
        let panel = self.renderer.render(&self.scene);
        self.strip_scene.bar.clone_from(&self.browser.bar);
        // (The strip's parts drawn again over the panel's frame where it changed there: the
        // panel's places that reach the strip, not QUALITY's opening above it as well.)
        // (The drops move on with QUALITY's opening, on its clock: the two in one frame.)
        let foot = (art::PANEL_H * self.renderer.scale()).round() as i32;
        let under = self
            .renderer
            .damages()
            .iter()
            .copied()
            .filter(|d| d[3] > foot)
            .reduce(|a, b| {
                [
                    a[0].min(b[0]),
                    a[1].min(b[1]),
                    a[2].max(b[2]),
                    a[3].max(b[3]),
                ]
            });
        let strip = self.strip.render_at(
            &self.strip_scene,
            self.renderer.frame(),
            under,
            self.motion_at.unwrap_or_else(Instant::now),
        );
        // The drawer dropped from the rail over the lower page while it shows: the arpeggiator's
        // (decisions.md R-ARP; its knobs drawn as the panel's renderer draws controls) or the
        // effects' (R-FX-DRAWER).
        self.lower.tick();
        let which = self.lower.which;
        let lower_k = self.lower_top().map_or(0.0, |_| self.lower.reveal());
        let lower_moved = (lower_k, which) != self.lower_shown;
        let lower = lower_k > 0.0
            && (match (which, &mut self.fx) {
                (Lower::Fx, Some(fx)) => fx.render(&self.fx_scene),
                _ => self.arp.render(&self.arp_scene, &mut self.renderer),
            } || self.lower.sliding()
                || lower_moved);
        let lower_shut = lower_k <= 0.0 && lower_moved;
        // The drawer over the strip, dropping down from under the rail, while it shows.
        let k = self.browser.reveal();
        // (Its place changed since the frame last shown is a change too: the slide's last frame
        // stops short of its end, which the frame shown must still reach.)
        let moved = k != self.reveal_shown;
        let drawer = k > 0.0
            && (self.drawer.render(&self.browser.drawer) || self.browser.sliding() || moved);
        let shut = k <= 0.0 && moved;
        if !(panel || strip || drawer || shut || lower || lower_shut || self.composed.is_none()) {
            return false;
        }
        self.reveal_shown = k;
        self.lower_shown = (lower_k, which);
        // The places that changed, the panel's and the strip's, put together again alone and
        // so shown (decisions.md R58); all of the frame while the drawer moves, something
        // floats over everything, or there is no frame of this size to change.
        let height = self.physical_height();
        let kept = self
            .composed
            .as_ref()
            .is_some_and(|c| (c.width(), c.height()) == (self.physical_width, height));
        let lower_moving = lower_moved || lower_shut || self.lower.sliding();
        self.damage = (kept
            && !(drawer
                || shut
                || lower_moving
                || self.renderer.floating()
                || self.standalone_dialog_open()
                || self.whats_new.is_some()))
        .then(|| {
            let mut places = self.renderer.damages().to_vec();
            if let Some([x0, y0, x1, y1]) = self.strip.damage() {
                places.push([x0, foot + y0, x1, foot + y1]);
            }
            let lower_damage = match (which, &self.fx) {
                (Lower::Fx, Some(fx)) => fx.damage(),
                _ => self.arp.damage(),
            };
            if lower && let Some([x0, y0, x1, y1]) = lower_damage {
                let top = lower_ui::top_px(self.renderer.scale());
                places.push([x0, top + y0, x1, top + y1]);
            }
            if !self.expanded() {
                let side = (30.0 * self.renderer.scale()).ceil() as i32;
                places.push([
                    self.physical_width as i32 - side,
                    height as i32 - side,
                    self.physical_width as i32,
                    height as i32,
                ]);
            }
            places
        });
        let expanded = self.expanded();
        let lower_top = self.lower_top();
        let scale = self.renderer.scale();
        compose(
            &mut self.composed,
            &self.renderer,
            self.strip.frame(),
            lower_top.map(|top| lower_frame(which, top, &self.arp, self.fx.as_ref(), scale)),
            (k > 0.0).then(|| (self.drawer.frame(), DRAWER_TOP - (1.0 - k) * DRAWER_H)),
            self.damage.as_deref(),
            expanded,
            (self.physical_width, height),
        );
        #[cfg(feature = "standalone-production")]
        if let Some(dialog) = &self.standalone_dialog
            && let Some(frame) = &mut self.composed
        {
            dialog.draw(frame, self.renderer.scale());
        }
        if let Some(notes) = &self.whats_new
            && let Some(frame) = &mut self.composed
        {
            notes.draw(frame, self.renderer.scale(), self.renderer.fonts());
        }
        true
    }

    /// The pointer's events: whether the event was the panel's.
    fn mouse(&mut self, e: MouseEvent) -> bool {
        self.reconcile_resize();
        match e {
            MouseEvent::CursorMoved {
                position,
                modifiers,
            } => {
                self.pointer = (position.x, position.y);
                self.moved(modifiers);
            }
            MouseEvent::ButtonPressed {
                button: MouseButton::Left,
                modifiers,
            } => self.press(modifiers),
            MouseEvent::ButtonPressed {
                button: MouseButton::Right,
                ..
            } => self.context_menu(),
            MouseEvent::ButtonReleased {
                button: MouseButton::Left,
                ..
            } => self.released(),
            MouseEvent::WheelScrolled { delta, modifiers } => self.scrolled(delta, modifiers),
            MouseEvent::CursorLeft => {
                self.playing.drag(None);
                self.playing.hover = None;
                if self.drag.is_none() {
                    self.hover = None;
                }
            }
            _ => return false,
        }
        true
    }
}

/// The window's frame: the panel's renderer's (the whole drawing), the strip's parts over its
/// strip (`strip`, from the panel's foot down), the arpeggiator's drawer over that from its top
/// (drawing units, its plate alone, cut off at the rail's foot) with its shadow laid over what is
/// under its foot, the presets' drawer over everything from `drawer`'s top
/// (drawing units, cut off at the rail's foot), and over everything what floats (MIDI Learn's
/// ring, a tip, a note, a menu); in the places `only` alone (pixels: left, top, right,
/// bottom) when given. (The places, not the rows they span: QUALITY's opening at the panel's
/// top and the strip's drops at its foot put together the whole frame between them.)
#[allow(clippy::too_many_arguments)]
fn compose(
    composed: &mut Option<ca72_panel::Pixmap>,
    renderer: &Renderer,
    strip: &ca72_panel::Pixmap,
    lower: Option<LowerFrame<'_>>,
    drawer: Option<(&ca72_panel::Pixmap, f64)>,
    only: Option<&[[i32; 4]]>,
    expanded: bool,
    (width, height): (u32, u32),
) {
    let frame = renderer.frame();
    let all = [0, 0, width as i32, height as i32];
    let kept = composed
        .as_ref()
        .is_some_and(|c| (c.width(), c.height()) == (width, height));
    let places = match only {
        Some(places) if kept => places,
        _ => std::slice::from_ref(&all),
    };
    let c = match composed {
        Some(c) if kept => c,
        _ => composed.insert(ca72_panel::Pixmap::new(width, height).expect("a frame")),
    };
    let s = renderer.scale();
    let top = (art::PANEL_H * s).round() as i32;
    let rail = (DRAWER_TOP * s).round() as i32;
    let (left, right) = (
        (lower_ui::LEFT * s).round() as i32,
        (lower_ui::RIGHT * s).round() as i32,
    );
    for &[x0, y0, x1, y1] in places {
        paste(c, frame, 0, [x0, y0, x1, y1]);
        paste(c, strip, top, [x0, y0, x1, y1]);
        if let Some(l) = &lower {
            // (Its frame's rows the drawing's from `top_px` down, open; moved up as it slides.)
            let at = lower_ui::top_px(s) - ((lower_ui::TOP - l.top) * s).round() as i32;
            let place = [x0.max(left), y0.max(rail), x1.min(right), y1];
            paste(
                c,
                l.frame,
                at,
                [
                    place[0],
                    place[1],
                    place[2],
                    place[3].min(at + (l.depth * s).round() as i32),
                ],
            );
            for &(p, px, py) in &l.under {
                shade(c, p, (px, at + py), place);
            }
        }
        if let Some((d, at)) = drawer {
            paste(c, d, (at * s).round() as i32, [x0, y0.max(rail), x1, y1]);
        }
    }
    // Logical height is rounded before the host applies DPI scaling. If that adds a row
    // beyond the artwork, extend its last row rather than leave an unpainted viewport edge.
    if expanded && height > frame.height() {
        let row = frame.height().saturating_sub(1) as usize;
        let stride = width as usize * 4;
        let last = c.data()[row * stride..(row + 1) * stride].to_vec();
        for y in frame.height() as usize..height as usize {
            c.data_mut()[y * stride..(y + 1) * stride].copy_from_slice(&last);
        }
    }
    if !expanded {
        let side = (COMPACT_GRIP * s).ceil() as i32;
        let grip = [all[2] - side, all[3] - side, all[2], all[3]];
        paste(c, frame, 0, grip);
        paste(c, strip, top, grip);
        ca72_panel::render::compact_grip(c, s);
    }
    if places == [all] && renderer.floating() {
        renderer.draw_floating(c);
    }
}

/// A drawer dropped from the rail as [`compose`] lays it: its frame (its plate alone shown,
/// `lower::LEFT` to `RIGHT`), how deep it is, its top now (drawing units), and what hangs under
/// it, each premultiplied with its top left corner in the frame's pixels (its shadow; the effects'
/// list).
struct LowerFrame<'a> {
    frame: &'a ca72_panel::Pixmap,
    depth: f64,
    top: f64,
    under: Vec<(&'a ca72_panel::Pixmap, i32, i32)>,
}

/// The drawer `which` as [`compose`] lays it, its top at `top` (drawing units).
fn lower_frame<'a>(
    which: Lower,
    top: f64,
    arp: &'a ArpRenderer,
    fx: Option<&'a FxRenderer>,
    scale: f64,
) -> LowerFrame<'a> {
    match (which, fx) {
        (Lower::Fx, Some(fx)) => LowerFrame {
            frame: fx.frame(),
            depth: fx_ui::DEPTH,
            top,
            under: fx.under(),
        },
        _ => LowerFrame {
            frame: arp.frame(),
            depth: arp_ui::DEPTH,
            top,
            under: arp
                .shadow()
                .map(|p| (p, 0, (arp_ui::DEPTH * scale).round() as i32))
                .into_iter()
                .collect(),
        },
    }
}

/// `p` (premultiplied, partly clear) laid over the frame `c` with its top left corner at
/// (`left`, `top`), inside `place` (left, top, right, bottom) only: what hangs under a drawer
/// dropped from the rail.
fn shade(
    c: &mut ca72_panel::Pixmap,
    p: &ca72_panel::Pixmap,
    (left, top): (i32, i32),
    place: [i32; 4],
) {
    let (w, h) = (c.width() as i32, c.height() as i32);
    let (pw, ph) = (p.width() as i32, p.height() as i32);
    let (x0, x1) = (place[0].max(0).max(left), place[2].min(w).min(left + pw));
    let (y0, y1) = (place[1].max(top).max(0), place[3].min(top + ph).min(h));
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    for y in y0..y1 {
        let src = (((y - top) * pw + x0 - left) * 4) as usize;
        let dst = ((y * w + x0) * 4) as usize;
        let n = (x1 - x0) as usize * 4;
        let (s, d) = (&p.data()[src..src + n], &mut c.data_mut()[dst..dst + n]);
        for (d, s) in d.chunks_exact_mut(4).zip(s.chunks_exact(4)) {
            let keep = 255 - u16::from(s[3]);
            for j in 0..4 {
                d[j] = (u16::from(s[j]) + (u16::from(d[j]) * keep + 127) / 255).min(255) as u8;
            }
        }
    }
}

/// `p` put in the frame `c` with its top at row `top`, inside `place` (left, top, right,
/// bottom) only.
fn paste(c: &mut ca72_panel::Pixmap, p: &ca72_panel::Pixmap, top: i32, place: [i32; 4]) {
    let (w, h) = (c.width() as i32, c.height() as i32);
    let (pw, ph) = (p.width() as i32, p.height() as i32);
    let (x0, x1) = (place[0].max(0), place[2].min(w).min(pw));
    let (y0, y1) = (place[1].max(top).max(0), place[3].min(top + ph).min(h));
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let n = (x1 - x0) as usize * 4;
    for y in y0..y1 {
        let src = (((y - top) * pw + x0) * 4) as usize;
        let dst = ((y * w + x0) * 4) as usize;
        c.data_mut()[dst..dst + n].copy_from_slice(&p.data()[src..src + n]);
    }
}

/// What the arpeggiator's drawer shows (decisions.md R-ARP), from the parameters and the
/// controls' values as the panel shows them (a knob dragged where the drag has it), and the scene
/// before (its hover is set elsewhere).
fn arp_scene(p: &Ca72Params, values: &[f64; CONTROLS.len()], was: ArpScene) -> ArpScene {
    let knob = |id: &str| ca72_panel::controls::index(id).map_or(0.0, |i| values[i]);
    let rate = crate::arp::RATES[p.arp_rate.value().to_index()].0;
    ArpScene {
        on: p.arp_on.value(),
        latch: p.arp_latch.value(),
        direction: p.arp_direction.value().to_index(),
        readouts: [
            (rate.to_owned(), true),
            (p.arp_octaves.value().to_string(), true),
            (format!("{:.0}", p.arp_gate.value()), true),
            (format!("{:.0}", p.arp_swing.value()), true),
        ],
        knobs: arp_ui::KNOB_PARAMS.map(knob),
        hover: was.hover,
    }
}

/// Whether VOICES does nothing (the owner, 2026-10-11, of a MONO preset's VOICES 4: "I don't
/// think those 4 voices do anything"; decisions.md R64): in MONO (neither POLY nor UNISON)
/// with DOUBLE, whose one pair has no place among others, or scattering with WIDTH at 0, where
/// every one of VOICES's places is the centre (R62). Then its knob is dimmed and not operable,
/// and its readout not lit nor typed into, as DETUNE's in SCATTER; its value stays for when it
/// counts again.
fn voices_inert(p: &Ca72Params) -> bool {
    let c = p.controls();
    !c.poly && !c.unison && (p.doubled.value() || c.spread <= 0.0)
}

/// What the strip shows, from the parameters, the voices sounding (a bit a voice), their levels
/// (0 to 1, by voice) and the output's, and the scene before (its rail, hover and MIDI Learn's
/// ring are set elsewhere).
fn strip_scene(
    p: &Ca72Params,
    sounding: u32,
    (levels, output): (&[f64], f64),
    was: StripScene,
) -> StripScene {
    let c = p.controls();
    let unison = c.unison;
    let poly = c.poly && !unison;
    let mono = !poly && !unison;
    let doubled = p.doubled.value();
    // (MONO with DOUBLE: the one pair, all of WIDTH's way out, whatever VOICES and the
    // placement; without, it scatters over VOICES's places: decisions.md R62.)
    let alone = mono && doubled;
    let voices = c.voices.clamp(2, 10);
    let placement = c.placement;
    let mode = Some(if unison { 2 } else { usize::from(poly) });
    // Three places, the readouts' (the mock-up's): a decimal under ten, a sign taking a place.
    let num3 = |v: f64| {
        if v.abs() < 0.05 {
            "0.0".to_owned()
        } else if v.abs() < 9.95 {
            format!("{v:.1}")
        } else {
            format!("{}", v.round())
        }
    };
    let percent = |v: f64| {
        if v > 0.0 {
            format!("{}", (v * 100.0).round())
        } else {
            "OFF".to_owned()
        }
    };
    let auto = if c.auto_gain && c.drive > 0.0 {
        let curve = crate::drive::Curve(p.drive_curve.saved().db);
        (num3(curve.at(c.drive, crate::engine::DRIVE_TOP)), true)
    } else {
        (String::new(), false)
    };
    let readouts = [
        (format!("{voices}"), !voices_inert(p)),
        (percent(c.entropy), c.entropy > 0.0),
        (percent(c.spread), c.spread > 0.0),
        (
            if doubled {
                crate::params::cents(c.double * crate::engine::DOUBLE_CENTS)
            } else {
                "OFF".to_owned()
            },
            doubled,
        ),
        auto,
        (num3(c.drive), c.drive > 0.0),
        (num3(c.level), true),
        (percent(c.inner), c.inner > 0.0 && c.spread > 0.0),
    ];
    let on = |k: usize| sounding & (1 << k) != 0;
    // Where they sound: in MONO with DOUBLE the one pair, all of WIDTH's way out; else each of
    // VOICES's places by the placement (in MONO the one voice at the place of its last attack,
    // R62), its pair as far out with DOUBLE (decisions.md R45), each in its side's band
    // by INNER: where the engine's gains put it, the kit's (R49).
    let (width, inner) = (c.spread, c.inner);
    let field = if alone {
        Field::Double(vec![(place::pair_at(width, inner, place::ALONE), on(0))])
    } else if doubled {
        Field::Double(
            (0..voices)
                .map(|k| {
                    (
                        place::pair_at(width, inner, placement.pair(k, voices)),
                        on(k),
                    )
                })
                .collect(),
        )
    } else {
        Field::Scatter(
            (0..voices)
                .map(|k| (place::voice_at(width, inner, placement, k, voices), on(k)))
                .collect(),
        )
    };
    // The placement's tab by the plug-in's choice (its tabs are its choices, EVEN and CENTER:
    // not the kit's index, which counts its EDGES); none where it moves nothing.
    let placement_lit = !alone;
    StripScene {
        mode,
        stereo: Some(usize::from(doubled)),
        placement: placement_lit.then_some(p.placement.value().to_index()),
        auto: c.auto_gain,
        readouts,
        field,
        // Each voice's level by its place in the field (the one instrument's, the output's, at
        // whichever place it sounds), in steps of a fortieth (a drop does not move for less).
        levels: if mono {
            vec![output; voices]
        } else {
            levels.iter().take(voices).copied().collect()
        }
        .into_iter()
        .map(|v| (v * 40.0).round() / 40.0)
        .collect(),
        detune: if doubled {
            c.double * crate::engine::DOUBLE_CENTS
        } else {
            0.0
        },
        ..was
    }
}

impl Editing {
    /// A key: whether the drawer took it (else it is the host's).
    fn keyed(&mut self, k: &KeyboardEvent) -> bool {
        self.reconcile_resize();
        if let Some(notes) = &mut self.whats_new {
            if k.state == KeyState::Up {
                self.playing.key(k, false);
            }
            if k.state == KeyState::Down {
                match k.key {
                    Key::Escape | Key::Enter => self.whats_new = None,
                    Key::ArrowDown => notes.scroll(1),
                    Key::ArrowUp => notes.scroll(-1),
                    Key::PageDown => notes.page(1),
                    Key::PageUp => notes.page(-1),
                    Key::Home => notes.home(),
                    Key::End => notes.end(),
                    _ => {}
                }
                self.composed = None;
            }
            return true;
        }
        let allowed = self.note_typing_allowed();
        let typing_root = self.playing.typing_root;
        if self.playing.key(k, allowed) {
            if self.playing.typing_root != typing_root {
                self.keep_settings(Change::TypingRoot(self.playing.typing_root));
                self.composed = None;
            }
            return true;
        }
        #[cfg(feature = "standalone-production")]
        if let Some(dialog) = &mut self.standalone_dialog {
            if dialog.key(k) {
                self.standalone_dialog = None;
            }
            self.composed = None;
            return true;
        }
        // MIDI Learn first (decisions.md R34): Escape closes a control's menu, the drawer's MIDI
        // list takes the keys it shows, and Escape cancels learning.
        if self.typing.is_some() {
            return self.type_key(k);
        }
        let map = std::sync::Arc::clone(&self.params.midi_map);
        let down = k.state == KeyState::Down && !modifier(&k.key);
        if down && k.key == Key::Escape && self.learning.menu_open() {
            self.learning.close_menu();
            return true;
        }
        if down && k.key == Key::Escape && self.fx_view.list_open() {
            self.fx_view.close_list();
            return true;
        }
        if self.browser.open && self.learning.list {
            let mut close = false;
            let taken = self.learning.list_key(k, &map, &mut close);
            if close {
                self.browser.show(false, &self.params);
                self.follow_drawer();
            }
            return taken;
        }
        if down && k.key == Key::Escape && map.armed().is_some() {
            self.learning.cancel(&map);
            return true;
        }
        // A key the drawer acts on may set the parameters (the arrows choose presets): no
        // gesture is left open under it. Shift and the other modifiers alone leave a drag.
        if self.browser.open && down {
            self.end_gestures();
        }
        let setter = ParamSetter::new(self.context.as_ref());
        let taken = self.browser.key(k, &self.params, &setter);
        self.follow_drawer();
        taken
    }
}

impl Drop for Editing {
    // The editor closed mid-drag, or within a wheel's rest: no parameter is left touched. And
    // MIDI Learn ends: a controller the audio thread has already caught for it is assigned,
    // else nothing is (decisions.md R34). It no longer watches the levels (the audio thread
    // stops measuring them when no editor does).
    fn drop(&mut self) {
        self.learning.close(&self.params.midi_map);
        self.end_gestures();
        self.meters.watchers.fetch_sub(1, Ordering::Relaxed);
    }
}

/// How long a frame shown on an X11 window is trusted to stay shown. X11 drops what is drawn
/// to a window not yet on screen (a host maps the editor's window after its first frame:
/// Bitwig on Linux showed it black until the pointer changed something) and, without a
/// compositor, what a covering window hid; baseview reports neither. So on Linux the last
/// frame is shown again at this pace, unchanged (a copy, no rendering; decisions.md R19).
/// macOS keeps a window's pixels; Windows asks for them again (`WindowEvent::Damaged`, R28).
const REPAINT: Option<Duration> = if cfg!(target_os = "linux") {
    Some(Duration::from_millis(250))
} else {
    None
};

/// How often the frame is drawn while the window cannot be seen (covered by other windows,
/// minimised, on another desktop, the host hidden): the scene kept up (the drawer, a resize, MIDI
/// Learn) without drawing the shutter and the levels thirty times a second for nobody. Nothing is
/// shown meanwhile; the window seen again is shown whole at once.
const HIDDEN_DRAW: Duration = Duration::from_millis(250);

/// The tests' counts of the frames drawn and shown by the editors' windows (R-EDITOR-CPU).
#[cfg(test)]
static DRAWN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
#[cfg(test)]
static SHOWN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Whether the frame must be shown again though nothing in it changed: not yet shown since
/// the window opened or resized, or (Linux) shown longer ago than [`REPAINT`].
fn repaint_due(shown: Option<Instant>, now: Instant) -> bool {
    match (shown, REPAINT) {
        (None, _) => true,
        (Some(at), Some(every)) => now.saturating_duration_since(at) >= every,
        (Some(_), None) => false,
    }
}

impl WindowHandler for PanelWindow {
    fn on_frame(&mut self, window: &mut Window<'_>) {
        // The keys the host gave the view since the last frame (Cubase), as the window's own,
        // but those the window had too (decisions.md R53).
        if self.host_keys.overflow.swap(false, Ordering::AcqRel) {
            self.editing.playing.release();
            self.host_keys.take();
        }
        for k in self.host_keys.take() {
            if self.twins.host_new(&k, Instant::now()) {
                self.key(window, &k);
            }
        }
        // A drag whose window lost the pointer's capture (Windows: Alt-Tab mid-drag) never
        // sees its release: it ends here (decisions.md R18).
        if self.editing.dragging() && window::lost_capture(window) {
            self.editing.end_gestures();
            self.editing.playing.release_pointer();
        }
        // What agents ask, carried out as the player's edits are, before the frame is drawn.
        if self.editing.serve_agents() {
            self.follow(window);
            self.keyboard(window);
        }
        // The grip's width, asked of the host once a frame.
        self.editing.grip();
        self.follow(window);
        let now = Instant::now();
        // Unseen, drawn only now and then and not shown; seen again, shown whole.
        // A remote AU view may be visible through its host's proxy while its native backing
        // window is occluded. Keep presenting when that window cannot describe visibility.
        let seen = (cfg!(target_os = "macos")
            && !self.editing.context.native_window_occlusion_reliable())
            || window::seen(window);
        if !seen {
            self.shown = None;
            if self
                .hidden_drawn
                .is_some_and(|at| now.saturating_duration_since(at) < HIDDEN_DRAW)
            {
                self.keyboard(window);
                self.editing.measure_when_done();
                return;
            }
            self.hidden_drawn = Some(now);
        }
        if !self.editing.whats_new_checked {
            self.editing
                .release_notes_when_visible(window::release_notes_visible(
                    window,
                    self.editing.context.native_window_occlusion_reliable(),
                ));
        }
        #[cfg(test)]
        DRAWN.fetch_add(1, Ordering::Relaxed);
        let changed = self.editing.draw();
        // A drawer that has just slid shut has asked the host for the shorter window: the
        // editor's view follows in the same frame, before anything is shown. A frame between
        // them showed, on macOS, the tall view in the short window, its top cut off (the
        // owner, 2026-10-03: a flicker as the drawer finished shutting; decisions.md R19).
        self.follow(window);
        // (MIDI Learn done meanwhile, by a controller rather than here: the keyboard back to the
        // host now, not at the editor's next event; decisions.md R34.)
        self.keyboard(window);
        // (Shown again whole when due: X11 may have dropped any of it.)
        let due = repaint_due(self.shown, now);
        if seen && (changed || due) {
            #[cfg(test)]
            SHOWN.fetch_add(1, Ordering::Relaxed);
            self.present(changed && !due);
            self.shown = Some(now);
        }
        self.editing.measure_when_done();
    }

    fn on_event(&mut self, window: &mut Window<'_>, event: Event) -> EventStatus {
        match event {
            Event::Window(WindowEvent::Resized(info)) => {
                self.resized(&info);
                EventStatus::Captured
            }
            // Windows repainting the window (a host's window painted its background over it):
            // the last frame shown, again at once, unless none has been since it opened or
            // resized (the next frame shows one). Not drawn afresh (decisions.md R28).
            Event::Window(WindowEvent::Damaged) => {
                if self.shown.is_some() {
                    self.present(false);
                }
                EventStatus::Captured
            }
            // No gesture is left open (decisions.md R18), nor a control's menu (R34; learning
            // goes on: the controller to be learned is elsewhere).
            Event::Window(WindowEvent::Focused) => {
                self.editing.playing.focused = true;
                self.keyboard(window);
                EventStatus::Ignored
            }
            Event::Window(WindowEvent::Unfocused | WindowEvent::WillClose) => {
                self.editing.playing.focused = false;
                self.editing.playing.release();
                self.editing.end_gestures();
                self.editing.learning.close_menu();
                self.keyboard(window);
                EventStatus::Ignored
            }
            Event::Mouse(e) => {
                let released = matches!(e, MouseEvent::ButtonReleased { .. });
                self.editing.touched |= matches!(
                    e,
                    MouseEvent::ButtonPressed { .. }
                        | MouseEvent::ButtonReleased { .. }
                        | MouseEvent::WheelScrolled { .. }
                );
                let status = if self.editing.mouse(e) {
                    EventStatus::Captured
                } else {
                    EventStatus::Ignored
                };
                if released {
                    window::release_capture(window);
                }
                // The drawer opening or shutting below the bar: the window follows (the
                // owner, 2026-10-02: on macOS it had stayed drawn at the old size until
                // reopened).
                self.follow(window);
                self.keyboard(window);
                self.cursor(window);
                status
            }
            // The drawer, while open, takes the keys (the host keeps its shortcuts).
            Event::Keyboard(k) => {
                // (One the host gave the view just now too: taken once.)
                if !self.twins.own_new(&k, Instant::now()) {
                    return EventStatus::Captured;
                }
                let taken = self.key(window, &k);
                if taken {
                    EventStatus::Captured
                } else {
                    EventStatus::Ignored
                }
            }
        }
    }
}

/// A modifier key alone.
fn modifier(key: &Key) -> bool {
    matches!(
        key,
        Key::Shift
            | Key::Control
            | Key::Alt
            | Key::AltGraph
            | Key::Meta
            | Key::Super
            | Key::CapsLock
            | Key::Fn
            | Key::FnLock
            | Key::Hyper
            | Key::NumLock
            | Key::ScrollLock
            | Key::Symbol
            | Key::SymbolLock
    )
}

/// The usable part of the screen (less the menu bar, dock or taskbar), in logical pixels
/// (`scale` the window's, where the platform counts physical ones): macOS's main screen,
/// Windows' primary monitor, on X11 the monitor the pointer is on (else the primary).
#[allow(unsafe_code)]
mod screen {
    #[cfg(target_os = "macos")]
    pub fn usable(_scale: Option<f64>) -> Option<(f64, f64)> {
        use cocoa::appkit::NSScreen;
        use cocoa::base::nil;
        // SAFETY: a class method of AppKit's, nil where there is no screen.
        let screen = unsafe { NSScreen::mainScreen(nil) };
        if screen == nil {
            return None;
        }
        // SAFETY: `screen` is an NSScreen.
        let frame = unsafe { NSScreen::visibleFrame(screen) };
        Some((frame.size.width, frame.size.height))
    }

    #[cfg(target_os = "windows")]
    pub fn usable(scale: Option<f64>) -> Option<(f64, f64)> {
        use winapi::shared::windef::RECT;
        use winapi::um::winuser::{SPI_GETWORKAREA, SystemParametersInfoW};
        let mut r = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        // SAFETY: SPI_GETWORKAREA writes one RECT through the pointer.
        let ok =
            unsafe { SystemParametersInfoW(SPI_GETWORKAREA, 0, (&mut r as *mut RECT).cast(), 0) };
        if ok == 0 {
            return None;
        }
        // The work area is in the pixels of the host's thread, which the window counts in
        // unless the host gave a scale (decisions.md R18).
        let k = scale.unwrap_or(1.0);
        Some((
            f64::from(r.right - r.left) / k,
            f64::from(r.bottom - r.top) / k,
        ))
    }

    /// The X11 screen spans every monitor: two side by side made the editor open wider
    /// than the one it is on, its grip off it (decisions.md R18). RandR's monitors (1.5),
    /// the window manager's work area (`_NET_WORKAREA`), through a connection of our own.
    #[cfg(target_os = "linux")]
    pub fn usable(scale: Option<f64>) -> Option<(f64, f64)> {
        use x11rb::connection::Connection;
        use x11rb::protocol::randr::ConnectionExt as _;
        use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _};
        let (conn, n) = x11rb::connect(None).ok()?;
        let screen = conn.setup().roots.get(n)?;
        let root = screen.root;
        let whole = Rect {
            x: 0,
            y: 0,
            w: i32::from(screen.width_in_pixels),
            h: i32::from(screen.height_in_pixels),
        };
        let monitors: Vec<(Rect, bool)> = conn
            .randr_query_version(1, 5)
            .ok()
            .and_then(|c| c.reply().ok())
            .and_then(|_| conn.randr_get_monitors(root, true).ok()?.reply().ok())
            .map(|r| {
                r.monitors
                    .iter()
                    .map(|m| {
                        let r = Rect {
                            x: i32::from(m.x),
                            y: i32::from(m.y),
                            w: i32::from(m.width),
                            h: i32::from(m.height),
                        };
                        (r, m.primary)
                    })
                    .collect()
            })
            .unwrap_or_default();
        let pointer = conn
            .query_pointer(root)
            .ok()
            .and_then(|c| c.reply().ok())
            .map(|p| (i32::from(p.root_x), i32::from(p.root_y)));
        let work = conn
            .intern_atom(true, b"_NET_WORKAREA")
            .ok()
            .and_then(|c| c.reply().ok())
            .filter(|a| a.atom != 0)
            .and_then(|a| {
                let p = conn
                    .get_property(false, root, a.atom, AtomEnum::CARDINAL, 0, 4)
                    .ok()?
                    .reply()
                    .ok()?;
                let v: Vec<u32> = p.value32()?.collect();
                let [x, y, w, h] = v[..] else { return None };
                Some(Rect {
                    x: i32::try_from(x).ok()?,
                    y: i32::try_from(y).ok()?,
                    w: i32::try_from(w).ok()?,
                    h: i32::try_from(h).ok()?,
                })
            });
        let area = monitor_area(whole, &monitors, pointer, work);
        let k = scale.unwrap_or(1.0);
        Some((f64::from(area.w) / k, f64::from(area.h) / k))
    }

    /// A rectangle of the X11 screen, in pixels.
    #[cfg(any(target_os = "linux", test))]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Rect {
        pub x: i32,
        pub y: i32,
        pub w: i32,
        pub h: i32,
    }

    #[cfg(any(target_os = "linux", test))]
    impl Rect {
        fn contains(&self, (x, y): (i32, i32)) -> bool {
            (self.x..self.x + self.w).contains(&x) && (self.y..self.y + self.h).contains(&y)
        }

        fn and(&self, o: &Rect) -> Option<Rect> {
            let (x, y) = (self.x.max(o.x), self.y.max(o.y));
            let (w, h) = (
                (self.x + self.w).min(o.x + o.w) - x,
                (self.y + self.h).min(o.y + o.h) - y,
            );
            (w > 0 && h > 0).then_some(Rect { x, y, w, h })
        }
    }

    /// Where on the X11 screen the editor opens: the monitor the pointer is on (where the
    /// host was just clicked), else the primary, else the first, else the whole screen; less
    /// the window manager's panels where its work area says (one rectangle for all the
    /// monitors, so only its part on this monitor counts).
    #[cfg(any(target_os = "linux", test))]
    pub fn monitor_area(
        whole: Rect,
        monitors: &[(Rect, bool)],
        pointer: Option<(i32, i32)>,
        work: Option<Rect>,
    ) -> Rect {
        let monitor = pointer
            .and_then(|p| monitors.iter().find(|(r, _)| r.contains(p)))
            .or_else(|| monitors.iter().find(|(_, primary)| *primary))
            .or(monitors.first())
            .map_or(whole, |(r, _)| *r);
        work.and_then(|w| monitor.and(&w)).unwrap_or(monitor)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    pub fn usable(_scale: Option<f64>) -> Option<(f64, f64)> {
        None
    }
}

/// The platform's windows: the host's parent window as baseview takes it, softbuffer's
/// surface on baseview's window (the two use different versions of `raw-window-handle`), and
/// what baseview leaves out: the keyboard given back, and the pointer's capture (Windows).
#[allow(unsafe_code)]
pub(crate) mod window {
    use std::num::{NonZeroIsize, NonZeroU32};
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::ptr::NonNull;

    use baseview::WindowHandle;
    use nih_plug::editor::ParentWindowHandle;
    use raw_window_handle::{
        HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
    };
    use raw_window_handle_06 as rwh6;

    /// The host's window the editor opens in.
    pub struct Parent(pub ParentWindowHandle);

    impl Parent {
        /// Whether a window can open in it: the host gave one, and on Linux there is an X11
        /// display (baseview panics in the host's thread without one).
        pub fn can_open(&self) -> bool {
            let given = match self.0 {
                ParentWindowHandle::X11Window(w) => w != 0,
                ParentWindowHandle::AppKitNsView(v) => !v.is_null(),
                ParentWindowHandle::Win32Hwnd(h) => !h.is_null(),
            };
            given && display()
        }
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn display() -> bool {
        use x11::xlib;
        // SAFETY: a connection of our own, closed at once.
        unsafe {
            let d = xlib::XOpenDisplay(std::ptr::null());
            if d.is_null() {
                return false;
            }
            xlib::XCloseDisplay(d);
        }
        true
    }

    #[cfg(not(target_os = "linux"))]
    pub(crate) fn display() -> bool {
        true
    }

    // SAFETY: the handle is the host's window, alive while the editor is open, and given as
    // the platform's own kind.
    unsafe impl HasRawWindowHandle for Parent {
        fn raw_window_handle(&self) -> RawWindowHandle {
            match self.0 {
                ParentWindowHandle::X11Window(window) => {
                    let mut h = raw_window_handle::XcbWindowHandle::empty();
                    h.window = window;
                    RawWindowHandle::Xcb(h)
                }
                ParentWindowHandle::AppKitNsView(ns_view) => {
                    let mut h = raw_window_handle::AppKitWindowHandle::empty();
                    h.ns_view = ns_view;
                    RawWindowHandle::AppKit(h)
                }
                ParentWindowHandle::Win32Hwnd(hwnd) => {
                    let mut h = raw_window_handle::Win32WindowHandle::empty();
                    h.hwnd = hwnd;
                    RawWindowHandle::Win32(h)
                }
            }
        }
    }

    /// The open window (if one could open), closed when dropped, the editor then reset.
    pub struct Handle {
        window: Option<WindowHandle>,
        _closing: super::Closing,
    }

    impl Handle {
        pub(super) fn new(window: Option<WindowHandle>, closing: super::Closing) -> Self {
            Handle {
                window,
                _closing: closing,
            }
        }
    }

    // SAFETY: the host drops the editor's handle on its GUI thread, where it opened it; the
    // raw pointers inside are not used from any other.
    unsafe impl Send for Handle {}

    impl Drop for Handle {
        fn drop(&mut self) {
            if let Some(w) = &mut self.window {
                w.close();
            }
        }
    }

    /// The keyboard as the drawer took it from another window (Windows: that window).
    #[derive(Debug)]
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub struct Keys {
        #[cfg(target_os = "windows")]
        had: winapi::shared::windef::HWND,
    }

    #[cfg(target_os = "windows")]
    fn hwnd(window: &baseview::Window<'_>) -> Option<winapi::shared::windef::HWND> {
        match window.raw_window_handle() {
            RawWindowHandle::Win32(h) if !h.hwnd.is_null() => Some(h.hwnd.cast()),
            _ => None,
        }
    }

    /// The keyboard to the editor's window (baseview's focus): on Windows, the window that
    /// had it if another did. Not on X11, where baseview's focus is `unimplemented!()` (it
    /// panicked the editor's thread, the window gone): there the keys go to the window under
    /// the pointer within the host's focused window, so to the drawer while the pointer is
    /// over the editor.
    pub fn take_keys(window: &mut baseview::Window<'_>) -> Option<Keys> {
        #[cfg(target_os = "windows")]
        {
            use winapi::um::winuser::GetFocus;
            let hwnd = hwnd(window)?;
            // SAFETY: the calling thread's focus, the editor's window being on it.
            let had = unsafe { GetFocus() };
            window.focus();
            (had != hwnd).then_some(Keys { had })
        }
        #[cfg(target_os = "macos")]
        {
            window.focus();
            None
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let _ = window;
            None
        }
    }

    /// The keyboard given back as the drawer shuts, to the window it was taken from (else
    /// the host's window holding the editor's), if the editor's window still has it.
    /// baseview on Windows keeps every key its window is given, the host's shortcuts too.
    pub fn give_keys_back(window: &baseview::Window<'_>, keys: Keys) {
        #[cfg(target_os = "windows")]
        {
            use winapi::um::winuser::{GetFocus, GetParent, IsWindow, SetFocus};
            let Some(hwnd) = hwnd(window) else { return };
            // SAFETY: the calling thread's focus and windows; `keys.had` is checked to be a
            // window still before it is given the focus.
            unsafe {
                if GetFocus() != hwnd {
                    return;
                }
                let to = if !keys.had.is_null() && IsWindow(keys.had) != 0 {
                    keys.had
                } else {
                    GetParent(hwnd)
                };
                SetFocus(to);
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (window, keys);
        }
    }

    /// Whether any of the editor's window can be seen: macOS's occlusion state of the window
    /// holding it (covered whole, minimised, on another Space or its application hidden: not
    /// seen); on Windows, its top window not minimised (not `IsWindowVisible`: in a session
    /// without a desktop on the screen, as an agent's over SSH, every window says it is not
    /// visible though it is drawn and shown, and the editor would show nothing). X11 says
    /// nothing reliable without a compositor: always seen there.
    // (objc 0.2's `msg_send!` asks after a `cargo-clippy` feature this crate does not have.)
    #[cfg_attr(target_os = "macos", allow(unexpected_cfgs))]
    pub fn seen(window: &baseview::Window<'_>) -> bool {
        #[cfg(target_os = "macos")]
        {
            use cocoa::appkit::{NSWindow, NSWindowOcclusionState};
            use cocoa::base::{id, nil};
            use objc::{msg_send, sel, sel_impl};
            let RawWindowHandle::AppKit(h) = window.raw_window_handle() else {
                return true;
            };
            let view = h.ns_view as id;
            if view == nil {
                return true;
            }
            // SAFETY: baseview's view, on the main thread that runs its frames; its window
            // is nil until the host puts it in one.
            let holder: id = unsafe { msg_send![view, window] };
            if holder == nil {
                return true;
            }
            // SAFETY: `holder` is an NSWindow.
            let state = unsafe { holder.occlusionState() };
            state.contains(NSWindowOcclusionState::NSWindowOcclusionStateVisible)
        }
        #[cfg(target_os = "windows")]
        {
            use winapi::um::winuser::{GA_ROOT, GetAncestor, IsIconic};
            let Some(hwnd) = hwnd(window) else {
                return true;
            };
            // SAFETY: the editor's window and its top window, on the thread that owns them.
            unsafe {
                let top = GetAncestor(hwnd, GA_ROOT);
                top.is_null() || IsIconic(top) == 0
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let _ = window;
            true
        }
    }

    /// Stricter than repaint visibility: an unmapped scan/test editor must not consume
    /// What's New. A remote AU's backing window can be occluded while its attached view
    /// is presented by the host, so only that case ignores native occlusion (R-AU-REMOTE-VISIBILITY).
    #[cfg_attr(target_os = "macos", allow(unexpected_cfgs))]
    pub fn release_notes_visible(window: &baseview::Window<'_>, reliable: bool) -> bool {
        #[cfg(target_os = "macos")]
        {
            use cocoa::base::{id, nil};
            use objc::{msg_send, sel, sel_impl};
            let RawWindowHandle::AppKit(h) = window.raw_window_handle() else {
                return false;
            };
            let view = h.ns_view as id;
            if view == nil {
                return false;
            }
            // SAFETY: baseview's live NSView, read on its GUI thread.
            let (holder, hidden): (id, objc::runtime::BOOL) = unsafe {
                (
                    msg_send![view, window],
                    msg_send![view, isHiddenOrHasHiddenAncestor],
                )
            };
            if holder == nil || hidden != objc::runtime::NO {
                return false;
            }
            // SAFETY: the live view's NSWindow, on its GUI thread. BOOL differs between
            // Intel and Apple silicon; Rust's bool is not AppKit's return type on Intel.
            let visible: objc::runtime::BOOL = unsafe { msg_send![holder, isVisible] };
            visible != objc::runtime::NO && (!reliable || seen(window))
        }
        #[cfg(target_os = "windows")]
        {
            use winapi::um::winuser::IsWindowVisible;
            let _ = reliable;
            // SAFETY: baseview's live HWND, read on its GUI thread.
            hwnd(window).is_some_and(|hwnd| unsafe { IsWindowVisible(hwnd) } != 0) && seen(window)
        }
        #[cfg(target_os = "linux")]
        {
            use x11rb::protocol::xproto::{ConnectionExt, MapState};
            let _ = reliable;
            let xid = match window.raw_window_handle() {
                RawWindowHandle::Xcb(h) => h.window,
                RawWindowHandle::Xlib(h) => h.window as u32,
                _ => return false,
            };
            let Ok((connection, _)) = x11rb::connect(None) else {
                return false;
            };
            connection
                .get_window_attributes(xid)
                .ok()
                .and_then(|cookie| cookie.reply().ok())
                .is_some_and(|attributes| attributes.map_state == MapState::VIEWABLE)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            let _ = (window, reliable);
            false
        }
    }

    /// Whether the editor's window has lost the pointer's capture (Windows: baseview
    /// captures it at a press and lets it go at the release, and ignores its loss, at Alt-Tab
    /// mid-drag, when no release comes). Elsewhere a release always comes.
    pub fn lost_capture(window: &baseview::Window<'_>) -> bool {
        #[cfg(target_os = "windows")]
        {
            use winapi::um::winuser::GetCapture;
            // SAFETY: the calling thread's capture.
            hwnd(window).is_some_and(|hwnd| unsafe { GetCapture() } != hwnd)
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = window;
            false
        }
    }

    /// The pointer's capture let go once no button is down (Windows). baseview counts the
    /// presses and releases to let it go; a release lost with the capture leaves the count
    /// one up, and its window then kept the capture after every release, the host's own
    /// clicks coming to the editor.
    pub fn release_capture(window: &baseview::Window<'_>) {
        #[cfg(target_os = "windows")]
        {
            use winapi::um::winuser::{
                GetCapture, GetKeyState, ReleaseCapture, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON,
                VK_XBUTTON1, VK_XBUTTON2,
            };
            let Some(hwnd) = hwnd(window) else { return };
            // SAFETY: the calling thread's capture and the buttons as its messages have them.
            unsafe {
                let down = [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON, VK_XBUTTON1, VK_XBUTTON2]
                    .iter()
                    .any(|&b| GetKeyState(b) < 0);
                if !down && GetCapture() == hwnd {
                    ReleaseCapture();
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = window;
        }
    }

    /// baseview's window as softbuffer takes it.
    #[derive(Clone)]
    pub struct Target {
        display: rwh6::RawDisplayHandle,
        window: rwh6::RawWindowHandle,
    }

    impl rwh6::HasDisplayHandle for Target {
        fn display_handle(&self) -> Result<rwh6::DisplayHandle<'_>, rwh6::HandleError> {
            // SAFETY: the display outlives the window drawn in, which outlives the surface.
            Ok(unsafe { rwh6::DisplayHandle::borrow_raw(self.display) })
        }
    }

    impl rwh6::HasWindowHandle for Target {
        fn window_handle(&self) -> Result<rwh6::WindowHandle<'_>, rwh6::HandleError> {
            // SAFETY: the window outlives its surface (both are the editor window's).
            Ok(unsafe { rwh6::WindowHandle::borrow_raw(self.window) })
        }
    }

    fn target(window: &baseview::Window<'_>) -> Option<Target> {
        let display = match window.raw_display_handle() {
            RawDisplayHandle::AppKit(_) => {
                rwh6::RawDisplayHandle::AppKit(rwh6::AppKitDisplayHandle::new())
            }
            RawDisplayHandle::Xlib(h) => rwh6::RawDisplayHandle::Xlib(
                rwh6::XlibDisplayHandle::new(NonNull::new(h.display), h.screen),
            ),
            RawDisplayHandle::Xcb(h) => rwh6::RawDisplayHandle::Xcb(rwh6::XcbDisplayHandle::new(
                NonNull::new(h.connection),
                h.screen,
            )),
            RawDisplayHandle::Windows(_) => {
                rwh6::RawDisplayHandle::Windows(rwh6::WindowsDisplayHandle::new())
            }
            _ => return None,
        };
        let window = match window.raw_window_handle() {
            RawWindowHandle::AppKit(h) => rwh6::RawWindowHandle::AppKit(
                rwh6::AppKitWindowHandle::new(NonNull::new(h.ns_view)?),
            ),
            RawWindowHandle::Xlib(h) => {
                rwh6::RawWindowHandle::Xlib(rwh6::XlibWindowHandle::new(h.window))
            }
            RawWindowHandle::Xcb(h) => {
                rwh6::RawWindowHandle::Xcb(rwh6::XcbWindowHandle::new(NonZeroU32::new(h.window)?))
            }
            RawWindowHandle::Win32(h) => {
                let mut w = rwh6::Win32WindowHandle::new(NonZeroIsize::new(h.hwnd as isize)?);
                w.hinstance = NonZeroIsize::new(h.hinstance as isize);
                rwh6::RawWindowHandle::Win32(w)
            }
            _ => return None,
        };
        Some(Target { display, window })
    }

    /// softbuffer's call, or nothing if it panicked: its Win32 backend asserts that GDI
    /// gave it a device context and a bitmap, which without a desktop it may not, and a
    /// panic there crossed into the host (pluginval crashed, decisions.md R18).
    fn guarded<T>(f: impl FnOnce() -> T) -> Option<T> {
        catch_unwind(AssertUnwindSafe(f)).ok()
    }

    /// A pixel of a frame (premultiplied RGBA, opaque) as the window shows it, opaque: its
    /// highest byte 0xff. softbuffer asks for 0 there, which every backend it is built with
    /// here ignores (Core Graphics skips it, GDI's copy and KMS's XRGB too) except X11 on a
    /// window with an alpha channel, which baseview makes wherever the screen has a 32-bit
    /// visual: there 0 is transparent, and on a compositing desktop the desktop shows through
    /// the panel, added to its colours (decisions.md R38).
    pub fn shown(p: &[u8]) -> u32 {
        0xff00_0000 | (u32::from(p[0]) << 16) | (u32::from(p[1]) << 8) | u32::from(p[2])
    }

    /// The window beyond the frames, opaque as they are.
    pub const EMPTY: u32 = 0xff3b_2213;

    /// The window's pixels (`w` by `h`, a row after another) from frames shown one under the
    /// other from its top left corner, [`EMPTY`] beyond them: a row at a time, each frame's
    /// row turned as a run of pixels (the compiler makes a short loop of it; a pixel at a time,
    /// with a frame found for each, took several times as long).
    pub fn fill(buffer: &mut [u32], (w, h): (usize, usize), frames: &[&ca72_panel::Pixmap]) {
        let mut frames = frames.iter();
        let mut frame = frames.next();
        let mut top = 0usize;
        for (y, row) in buffer.chunks_exact_mut(w).take(h).enumerate() {
            while let Some(f) = frame
                && y >= top + f.height() as usize
            {
                top += f.height() as usize;
                frame = frames.next();
            }
            let n = match frame {
                Some(f) => {
                    let fw = f.width() as usize;
                    let n = fw.min(w);
                    let from = (y - top) * fw * 4;
                    let src = &f.data()[from..from + n * 4];
                    for (out, p) in row[..n].iter_mut().zip(src.chunks_exact(4)) {
                        *out = shown(p);
                    }
                    n
                }
                None => 0,
            };
            row[n..].fill(EMPTY);
        }
    }

    /// The window's pixels in the places `damage` (left, top, right, bottom) alone, as
    /// [`fill`] makes them there, the rest left as it was: the places, cut to the window, that
    /// were turned.
    pub fn fill_places(
        buffer: &mut [u32],
        (w, h): (usize, usize),
        frames: &[&ca72_panel::Pixmap],
        damage: &[[i32; 4]],
    ) -> Vec<[i32; 4]> {
        let mut turned = Vec::with_capacity(damage.len());
        for &[x0, y0, x1, y1] in damage {
            let cut = |v: i32, most: usize| v.clamp(0, most as i32) as usize;
            let (x0, y0, x1, y1) = (cut(x0, w), cut(y0, h), cut(x1, w), cut(y1, h));
            if x1 <= x0 || y1 <= y0 {
                continue;
            }
            turned.push([x0 as i32, y0 as i32, x1 as i32, y1 as i32]);
            let mut frames = frames.iter();
            let mut frame = frames.next();
            let mut top = 0usize;
            for y in y0..y1 {
                while let Some(f) = frame
                    && y >= top + f.height() as usize
                {
                    top += f.height() as usize;
                    frame = frames.next();
                }
                let row = &mut buffer[y * w + x0..y * w + x1];
                let n = match frame {
                    Some(f) if (f.width() as usize) > x0 => {
                        let fw = f.width() as usize;
                        let n = fw.min(x1) - x0;
                        let from = ((y - top) * fw + x0) * 4;
                        let src = &f.data()[from..from + n * 4];
                        for (out, p) in row[..n].iter_mut().zip(src.chunks_exact(4)) {
                            *out = shown(p);
                        }
                        n
                    }
                    _ => 0,
                };
                row[n..].fill(EMPTY);
            }
        }
        turned
    }

    /// Where the frames are shown; nothing if the platform's window could not be drawn in.
    pub struct Surface {
        inner: Option<(
            softbuffer::Context<Target>,
            softbuffer::Surface<Target, Target>,
        )>,
        /// Whether what the window shows is the frame last given, every pixel of it: not
        /// since the surface was made or resized, nor after a frame that could not be shown.
        whole: bool,
    }

    impl Surface {
        pub fn new(window: &baseview::Window<'_>, size: (u32, u32)) -> Self {
            let inner = target(window).and_then(|t| {
                guarded(|| {
                    let context = softbuffer::Context::new(t.clone()).ok()?;
                    let surface = softbuffer::Surface::new(&context, t).ok()?;
                    Some((context, surface))
                })
                .flatten()
            });
            let mut s = Surface {
                inner,
                whole: false,
            };
            s.resize(size);
            s
        }

        pub fn resize(&mut self, (w, h): (u32, u32)) {
            self.whole = false;
            if let (Some((_, surface)), Some(w), Some(h)) =
                (&mut self.inner, NonZeroU32::new(w), NonZeroU32::new(h))
                && guarded(|| surface.resize(w, h)).is_none()
            {
                self.inner = None;
            }
        }

        /// Shows frames (premultiplied RGBA, opaque), one under the other from the top left
        /// corner, in a window of `size` pixels: where they changed since those last shown, in
        /// the places `damage` (left, top, right, bottom), when it is given, or anywhere.
        pub fn present(
            &mut self,
            size: (u32, u32),
            frames: &[&ca72_panel::Pixmap],
            damage: Option<&[[i32; 4]]>,
        ) {
            let Some((_, surface)) = &mut self.inner else {
                return;
            };
            let damage = damage.filter(|_| self.whole);
            match guarded(|| Self::show(surface, size, frames, damage)) {
                Some(shown) => self.whole = shown,
                None => {
                    self.whole = false;
                    self.inner = None;
                }
            }
        }

        /// Whether the frames were shown, all of them.
        fn show(
            surface: &mut softbuffer::Surface<Target, Target>,
            (w, h): (u32, u32),
            frames: &[&ca72_panel::Pixmap],
            damage: Option<&[[i32; 4]]>,
        ) -> bool {
            let Ok(mut buffer) = surface.buffer_mut() else {
                return false;
            };
            let (w, h) = (w as usize, h as usize);
            if buffer.len() < w * h {
                return false;
            }
            // Where the buffer still holds the frame shown last (age 1: a buffer kept, as
            // Windows' and X11's are, and macOS's in this copy of softbuffer, shown in tiles),
            // only the places changed are turned and sent; else the whole frame (decisions.md
            // R58, R59).
            match damage.filter(|_| buffer.age() == 1) {
                Some(damage) => {
                    let rects: Vec<softbuffer::Rect> =
                        fill_places(&mut buffer[..], (w, h), frames, damage)
                            .into_iter()
                            .filter_map(|[x0, y0, x1, y1]| {
                                Some(softbuffer::Rect {
                                    x: x0 as u32,
                                    y: y0 as u32,
                                    width: NonZeroU32::new((x1 - x0) as u32)?,
                                    height: NonZeroU32::new((y1 - y0) as u32)?,
                                })
                            })
                            .collect();
                    buffer.present_with_damage(&rects).is_ok()
                }
                None => {
                    fill(&mut buffer[..], (w, h), frames);
                    buffer.present().is_ok()
                }
            }
        }
    }
}

/// ULTRA's note's lines (decisions.md R48; the owner approved this text on 2026-10-16,
/// R-THREADS): ULTRA's cost against HI (3.7 to 6.4 times over the factory presets, more with
/// FEEDBACK), and on a computer that splits an ULTRA voice over six threads (`splits`) that one
/// voice plays live there; else what one voice takes.
fn ultra_note_lines(splits: bool) -> Vec<(&'static str, Tone)> {
    let mut lines = vec![
        ("ULTRA · NO COMPROMISES", Tone::Accent),
        (
            "THE CIRCUIT'S MODEL EXACTLY, AT ABOUT SIX TIMES THE WORK OF HI.",
            Tone::Plain,
        ),
    ];
    if splits {
        lines.extend([
            // (Wrapped so that no line is wider than the note's widest before.)
            (
                "ON THIS COMPUTER ONE VOICE IS SPREAD OVER SIX THREADS AND",
                Tone::Plain,
            ),
            (
                "CAN BE PLAYED LIVE. WITH MULTIPLE VOICES OR TOO SMALL OF A BUFFER,",
                Tone::Plain,
            ),
            ("IT MAY DROP OUT.", Tone::Plain),
            (
                "IT IS MEANT FOR OFFLINE RENDERS (EXPORT, BOUNCE, FREEZE) AND SMALL",
                Tone::Plain,
            ),
            ("PROJECTS ON POWERFUL COMPUTERS.", Tone::Plain),
        ]);
    } else {
        lines.extend([
            (
                "ONE VOICE TAKES UP TO A WHOLE CORE OF A FAST COMPUTER.",
                Tone::Plain,
            ),
            (
                "IT IS MEANT FOR OFFLINE RENDERS (EXPORT, BOUNCE, FREEZE) AND SMALL",
                Tone::Plain,
            ),
            (
                "PROJECTS ON POWERFUL COMPUTERS. PLAYED LIVE, IT MAY DROP OUT.",
                Tone::Plain,
            ),
        ]);
    }
    lines.extend([
        (
            "RIGHT-CLICK QUALITY'S SWITCH TO HIDE ITS INDICATOR, OR SHOW IT.",
            Tone::Dim,
        ),
        ("CLICK TO CLOSE", Tone::Dim),
    ]);
    lines
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use baseview::Point;
    use nih_plug::context::PluginApi;
    use nih_plug::wrapper::state::PluginState;

    use super::*;
    use ca72_panel::presets::{self as presets_ui, BarTarget, DrawerTarget, FieldId, PresetKey};
    use ca72_panel::ultra;

    /// What the editor asked of the host.
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Call {
        Begin(ParamPtr),
        Set(ParamPtr, f32),
        End(ParamPtr),
    }

    /// A host that records the editor's gestures (and does not apply them).
    #[derive(Default)]
    struct Host {
        api: Option<PluginApi>,
        calls: Mutex<Vec<Call>>,
        /// It will not resize the window.
        fixed: AtomicBool,
        /// How often it was asked to.
        resizes: AtomicU32,
        deferred: AtomicBool,
    }

    impl Host {
        fn take(&self) -> Vec<Call> {
            std::mem::take(&mut *self.calls.lock().expect("not poisoned"))
        }
        fn push(&self, c: Call) {
            self.calls.lock().expect("not poisoned").push(c);
        }
    }

    #[allow(unsafe_code)]
    impl GuiContext for Host {
        fn plugin_api(&self) -> PluginApi {
            self.api.unwrap_or(PluginApi::Clap)
        }
        fn request_resize(&self) -> bool {
            self.resizes.fetch_add(1, Ordering::Relaxed);
            !self.fixed.load(Ordering::Relaxed)
        }
        fn resize_response_will_follow(&self) -> bool {
            self.deferred.load(Ordering::Relaxed)
        }
        unsafe fn raw_begin_set_parameter(&self, param: ParamPtr) {
            self.push(Call::Begin(param));
        }
        unsafe fn raw_set_parameter_normalized(&self, param: ParamPtr, normalized: f32) {
            self.push(Call::Set(param, normalized));
        }
        unsafe fn raw_end_set_parameter(&self, param: ParamPtr) {
            self.push(Call::End(param));
        }
        fn get_state(&self) -> PluginState {
            panic!("the editor does not read the state")
        }
        fn set_state(&self, _state: PluginState) {}
    }

    #[test]
    fn release_notes_wait_for_a_visible_editor_and_reopen_without_changing_the_sound() {
        use crate::engine::{Engine, Event};
        let dir = tempfile::tempdir().expect("settings folder");
        let path = dir.path().join("CA-72/settings.toml");
        let (mut e, host, params) = editing();
        e.settings_file = Some(path.clone());
        e.update = Update::with(
            crate::update::fake::no_curl,
            crate::update::fake::no_browser,
        );
        e.draw();
        let controls = params.controls();
        let preset = crate::presets::values_now(&params);
        let fields = params.serialize_fields();
        let update = e.update.scene();
        let render = || {
            let mut engine = Engine::new();
            engine.set(&params.controls());
            engine.prepare(48_000.0, 12345);
            engine.event(Event::Note { key: 60, on: true });
            (0..4096)
                .map(|_| {
                    let (l, r) = engine.tick_stereo(0.0);
                    (l.to_bits(), r.to_bits())
                })
                .collect::<Vec<_>>()
        };
        let audio = render();
        assert!(audio.iter().any(|&(l, _)| f32::from_bits(l).abs() > 0.001));
        for _ in 0..3 {
            e.release_notes_when_visible(false);
            e.draw();
        }
        assert!(!e.whats_new_checked && e.whats_new.is_none());
        assert!(!path.exists(), "a hidden editor never consumes the history");
        e.release_notes_when_visible(true);
        assert!(e.whats_new.is_some());
        let saved = std::fs::read(&path).expect("claim kept");
        e.draw();
        assert!(keyed(&mut e, Key::Escape));
        assert!(e.whats_new.is_none());
        e.release_notes_when_visible(true);
        assert!(e.whats_new.is_none(), "dismissal is final for this editor");
        e.strip_press(StripTarget::Bar(BarTarget::Settings));
        assert!(menu_items(&e).contains(&("WHAT'S NEW…".to_owned(), true)));
        menu_item(&mut e, "WHAT'S NEW…");
        assert!(e.whats_new.is_some(), "manual reopening remains available");
        e.scrolled(ScrollDelta::Lines { x: 0.0, y: -2.0 }, Modifiers::empty());
        e.draw();
        assert!(keyed(&mut e, Key::End));
        assert!(keyed(&mut e, Key::Home));
        assert!(keyed(&mut e, Key::Enter));
        e.draw();
        assert!(e.whats_new.is_none());
        assert_eq!(params.controls(), controls);
        assert_eq!(crate::presets::values_now(&params), preset);
        assert_eq!(
            params.serialize_fields(),
            fields,
            "saved project fields unchanged"
        );
        assert_eq!(render(), audio, "audio remains bit-identical");
        assert!(host.take().is_empty(), "no sound parameter gestures");
        assert_eq!(e.update.scene(), update, "no network request");
        assert_eq!(
            std::fs::read(&path).expect("settings"),
            saved,
            "reopening/dismissal write nothing"
        );
        drop(e);
        for api in [PluginApi::Clap, PluginApi::Vst3, PluginApi::Standalone] {
            let (mut restarted, _, _) = editing_for_api(
                Ca72Params::default(),
                Library::at(dir.path().join("library")),
                api,
            );
            restarted.settings_file = Some(path.clone());
            restarted.release_notes_when_visible(true);
            assert!(
                restarted.whats_new.is_none(),
                "shared history after restart: {api}"
            );
        }
    }

    #[test]
    fn release_notes_do_not_interrupt_a_held_typing_note_or_lose_its_release() {
        let (mut e, _, _) = editing_standalone();
        e.playing.typing = true;
        let down = KeyboardEvent {
            key: Key::Character("a".into()),
            ..Default::default()
        };
        assert!(e.keyed(&down));
        assert!(e.playing.notes.pop().expect("attack").on);
        let generation = e.playing.notes.generation();
        e.show_release_notes(crate::whats_new::current());
        e.draw();
        assert_eq!(e.playing.notes.generation(), generation);
        assert!(
            e.playing.notes.pop().is_none(),
            "opening leaves the note sounding"
        );
        let up = KeyboardEvent {
            state: KeyState::Up,
            ..down
        };
        assert!(e.keyed(&up));
        assert!(!e.playing.notes.pop().expect("release through the notes").on);
        assert!(keyed(&mut e, Key::Escape));
    }

    #[test]
    fn release_notes_fit_both_looks_lettering_sizes_and_compact_views() {
        for skin in [Skin::Drawn, Skin::Worn] {
            for lettering in [Lettering::Standard, Lettering::Large] {
                for expanded in [false, true] {
                    let (mut e, host, _) = editing();
                    e.set_look(skin);
                    e.set_lettering(lettering);
                    e.params.editor_panel.store(expanded, Ordering::Relaxed);
                    e.set_panel_height(expanded);
                    e.fit(1.0, MIN_WIDTH, MIN_WIDTH);
                    e.show_release_notes(
                        crate::whats_new::assemble(Some("0.2.3"), crate::update::VERSION)
                            .expect("combined notes"),
                    );
                    e.draw();
                    assert_eq!(e.expanded(), expanded);
                    assert_eq!(
                        e.composed.as_ref().expect("notes frame").height(),
                        e.physical_height()
                    );
                    if let Some(out) = std::env::var_os("CA72_WHATS_NEW_PNG") {
                        let out = std::path::Path::new(&out);
                        std::fs::create_dir_all(out).expect("evidence folder");
                        e.composed
                            .as_ref()
                            .expect("notes frame")
                            .save_png(
                                out.join(format!("notes-{skin:?}-{lettering:?}-{expanded}.png")),
                            )
                            .expect("PNG");
                    }
                    let close = at(
                        &e,
                        (
                            art::W - 300.0,
                            f64::from(e.physical_height()) / e.renderer.scale()
                                - if lettering == Lettering::Large {
                                    52.5
                                } else {
                                    42.0
                                },
                        ),
                    );
                    click(&mut e, close);
                    assert!(e.whats_new.is_none(), "close button works in each view");
                    assert!(host.take().is_empty());
                }
            }
        }
    }

    fn editing() -> (Editing, Arc<Host>, Arc<Ca72Params>) {
        // A library that is not there (nothing of the user's read; these tests write none).
        editing_with(Library::at(
            std::env::temp_dir().join("ca72-editor-tests-no-library"),
        ))
    }

    #[test]
    fn typing_octave_is_independent_of_panel_octaves_and_sound_parameters() {
        let (mut e, host, _) = editing_standalone();
        e.playing.visible = true;
        let event = |state, character: &str| KeyboardEvent {
            state,
            key: Key::Character(character.into()),
            ..Default::default()
        };
        assert!(
            !e.keyed(&event(KeyState::Down, "x")),
            "TYPE starts disabled"
        );
        e.playing.typing = true;
        assert!(e.keyed(&event(KeyState::Down, "a")));
        assert_eq!(e.playing.notes.pop().expect("C3 attack").note, 48);
        let p = at(&e, (keyboard::UP_X, keyboard::HEADER));
        click(&mut e, p);
        assert_eq!((e.playing.root, e.playing.typing_root), (53, 48));
        assert!(
            e.playing.notes.pop().is_none(),
            "range button leaves TYPE held"
        );
        assert!(e.keyed(&event(KeyState::Down, "x")));
        assert_eq!((e.playing.root, e.settings.keyboard_root), (53, 53));
        assert_eq!((e.playing.typing_root, e.settings.typing_root), (60, 60));
        assert!(!e.playing.notes.pop().expect("original pitch released").on);
        assert!(
            host.take().is_empty(),
            "keyboard octaves are outside the sound"
        );
        assert!(e.keyed(&event(KeyState::Up, "x")));
        assert!(e.keyed(&event(KeyState::Up, "a")));
        assert!(e.keyed(&event(KeyState::Down, "a")));
        assert_eq!(e.playing.notes.pop().expect("C4 attack").note, 60);
        e.strip_press(StripTarget::Bar(BarTarget::Name));
        assert!(!e.note_typing_allowed(), "preset search owns letters");
        e.keyed(&event(KeyState::Down, "z"));
        assert_eq!(e.playing.typing_root, 60);
    }

    #[test]
    fn panel_hide_and_view_swap_preserve_type_notes_and_the_current_height() {
        let (mut e, _, _) = editing_standalone();
        e.playing.visible = true;
        e.playing.typing = true;
        let key = KeyboardEvent {
            key: Key::Character("a".into()),
            ..Default::default()
        };
        assert!(e.expanded());
        assert!(e.keyed(&key));
        assert!(e.playing.notes.pop().expect("attack").on);
        let generation = e.playing.notes.generation();
        e.toggle_panel();
        assert!(!e.expanded());
        assert_eq!(e.playing.notes.generation(), generation);
        assert!(e.note_typing_allowed());
        assert!(e.keyed(&key), "TYPE still owns a held key in compact view");
        assert!(
            e.playing.notes.pop().is_none(),
            "held key does not retrigger"
        );
        let compact_height = e.window_height();
        e.strip_press(StripTarget::Bar(BarTarget::Keyboard));
        assert!(!e.playing.visible);
        assert!(e.note_typing_allowed(), "extra controls preserve TYPE");
        assert_eq!(e.playing.notes.generation(), generation);
        assert_eq!(e.window_height(), compact_height);
        e.strip_press(StripTarget::Bar(BarTarget::Keyboard));
        assert!(e.playing.visible);
        assert_eq!(e.window_height(), compact_height);
        e.toggle_panel();
        assert!(e.expanded());
        assert!(e.note_typing_allowed());
        let mut up = key.clone();
        up.state = KeyState::Up;
        assert!(e.keyed(&up));
        assert!(!e.playing.notes.pop().expect("original attack released").on);
        assert!(e.keyed(&key));
        assert!(e.playing.notes.pop().expect("new attack").on);
    }

    #[test]
    fn keyboard_view_blocks_hidden_controls_and_releases_for_takeovers() {
        let (mut e, host, params) = editing_standalone();
        let width = e.width;
        let height = e.window_height();
        e.playing.visible = true;
        assert!(e.playing.visible);
        assert_eq!((e.width, e.window_height()), (width, height));
        for id in [
            "voices", "entropy", "spread", "inner", "double", "drive", "level",
        ] {
            let p = at(&e, centre(id));
            go(&mut e, p);
            press(&mut e);
            wheel(&mut e, 1.0);
            release(&mut e);
        }
        assert!(
            host.calls.lock().unwrap().is_empty(),
            "hidden controls cannot gesture"
        );
        let p = at(&e, (keyboard::TYPING_X, keyboard::HEADER));
        go(&mut e, p);
        press(&mut e);
        release(&mut e);
        assert!(e.note_typing_allowed());
        let key = KeyboardEvent {
            key: Key::Character("a".into()),
            ..Default::default()
        };
        assert!(e.keyed(&key));
        let generation = e.playing.notes.generation();
        e.strip_press(StripTarget::Bar(BarTarget::Name));
        assert!(e.playing.notes.generation() > generation);
        assert!(!e.note_typing_allowed());
        e.browser.show(false, &params);
        e.follow_drawer();
        // A readout's typing and a settings menu take over the keyboard as well.
        let generation = e.playing.notes.generation();
        e.start_typing(Readout::Level);
        assert!(e.playing.notes.generation() > generation);
        assert!(!e.note_typing_allowed());
        e.end_gestures();
        e.strip_press(StripTarget::Bar(BarTarget::Settings));
        assert!(!e.note_typing_allowed());
        e.learning.close_menu();
        e.strip_press(StripTarget::Bar(BarTarget::Keyboard));
        assert!(!e.playing.visible);
        assert_eq!((e.width, e.window_height()), (width, height));
    }

    #[test]
    fn keyboard_pointer_releases_outside_and_editor_drop_invalidates_pending_notes() {
        let (mut e, _, _) = editing();
        e.playing.visible = true;
        let r = keyboard::rect(41, 48).unwrap();
        let p = at(&e, ((r[0] + r[2]) / 2.0, r[3] - 20.0));
        go(&mut e, p);
        press(&mut e);
        assert!(e.playing.dragging());
        assert!(e.playing.notes.pop().expect("mouse attack").on);
        go(&mut e, Point { x: -20.0, y: -20.0 });
        assert!(!e.playing.notes.pop().expect("outside release").on);
        assert!(e.playing.dragging(), "can glissando back inside");
        go(&mut e, p);
        assert!(e.playing.notes.pop().expect("reenter attack").on);
        release(&mut e);
        assert!(!e.playing.notes.pop().expect("release").on);
        go(&mut e, p);
        press(&mut e);
        let notes = Arc::clone(&e.playing.notes);
        let generation = notes.generation();
        drop(e);
        assert!(
            notes.generation() > generation,
            "close releases without relying on a queue slot"
        );
    }

    fn editing_with(library: Library) -> (Editing, Arc<Host>, Arc<Ca72Params>) {
        editing_of(Ca72Params::default(), library)
    }

    /// The editor with POLY on, where VOICES counts (decisions.md R64).
    fn editing_poly() -> (Editing, Arc<Host>, Arc<Ca72Params>) {
        let params = Ca72Params {
            poly: BoolParam::new("Poly", true),
            ..Ca72Params::default()
        };
        editing_of(
            params,
            Library::at(std::env::temp_dir().join("ca72-editor-tests-no-library")),
        )
    }

    /// An editor whose instance is at QUALITY HI (a new one is at LO: R-LO-DEFAULT).
    fn editing_at_hi() -> (Editing, Arc<Host>, Arc<Ca72Params>) {
        let params = Ca72Params {
            quality: EnumParam::new("Quality", QualityMode::Hi),
            ..Ca72Params::default()
        };
        editing_of(
            params,
            Library::at(std::env::temp_dir().join("ca72-editor-tests-no-library")),
        )
    }

    fn editing_of(params: Ca72Params, library: Library) -> (Editing, Arc<Host>, Arc<Ca72Params>) {
        editing_for_api(params, library, PluginApi::Clap)
    }

    fn editing_standalone() -> (Editing, Arc<Host>, Arc<Ca72Params>) {
        editing_for_api(
            Ca72Params::default(),
            Library::at(std::env::temp_dir().join("ca72-editor-tests-no-library")),
            PluginApi::Standalone,
        )
    }

    #[test]
    fn keyboard_type_and_panic_are_only_available_in_standalone() {
        for api in [PluginApi::Clap, PluginApi::Vst3, PluginApi::Standalone] {
            let standalone = api == PluginApi::Standalone;
            let (mut e, _, _) = editing_for_api(
                Ca72Params::default(),
                Library::at(std::env::temp_dir().join("ca72-editor-tests-no-library")),
                api,
            );
            e.playing.visible = true;
            assert_eq!(e.playing.standalone, standalone);
            e.playing.typing = true;
            assert_eq!(e.note_typing_allowed(), standalone);
            let scene = e.playing.scene(Skin::Worn).expect("keyboard");
            assert_eq!(scene.standalone, standalone);
            assert_eq!(scene.typing, standalone);
            e.playing.typing = false;
            let type_button = at(&e, (keyboard::TYPING_X, keyboard::HEADER));
            go(&mut e, type_button);
            press(&mut e);
            release(&mut e);
            assert_eq!(e.playing.typing, standalone, "{api}: TYPE hit target");
            let key = KeyboardEvent {
                key: Key::Character("a".into()),
                ..Default::default()
            };
            assert_eq!(e.keyed(&key), standalone, "{api}: note keys");
            if standalone {
                assert!(e.playing.notes.pop().expect("typed attack").on);
                let up = KeyboardEvent {
                    state: KeyState::Up,
                    ..key.clone()
                };
                assert!(e.keyed(&up));
                assert!(!e.playing.notes.pop().expect("typed release").on);
            } else {
                assert!(e.playing.notes.pop().is_none());
            }
            // A hidden PANIC target must not reset an existing local note.
            assert!(e.playing.key(&key, true));
            assert!(e.playing.notes.pop().expect("local attack").on);
            let generation = e.playing.notes.generation();
            let panic_button = at(&e, (keyboard::PANIC_X, keyboard::HEADER));
            go(&mut e, panic_button);
            press(&mut e);
            release(&mut e);
            assert_eq!(
                e.playing.notes.generation() > generation,
                standalone,
                "{api}: PANIC hit target"
            );
            e.playing.release();
        }
    }

    fn editing_for_api(
        params: Ca72Params,
        library: Library,
        api: PluginApi,
    ) -> (Editing, Arc<Host>, Arc<Ca72Params>) {
        let host = Arc::new(Host {
            api: Some(api),
            ..Default::default()
        });
        let params = Arc::new(params);
        let context: Arc<dyn GuiContext> = host.clone();
        let e = Editing::new(
            context,
            Arc::clone(&params),
            Arc::new(Meters::default()),
            (1720, 1720, 1.0),
            library,
            |_| None,
        );
        (e, host, params)
    }

    /// The window's pixels as the frames were turned a pixel at a time, before (the
    /// reference for [`window::fill`]).
    fn filled_a_pixel_at_a_time(
        buffer: &mut [u32],
        (w, h): (usize, usize),
        frames: &[&ca72_panel::Pixmap],
    ) {
        let mut frames = frames.iter();
        let mut frame = frames.next();
        let mut top = 0usize;
        for y in 0..h {
            while let Some(f) = frame
                && y >= top + f.height() as usize
            {
                top += f.height() as usize;
                frame = frames.next();
            }
            let row = &mut buffer[y * w..(y + 1) * w];
            for (x, out) in row.iter_mut().enumerate() {
                *out = match frame {
                    Some(f) if x < f.width() as usize => {
                        let (fw, fy) = (f.width() as usize, y - top);
                        window::shown(&f.data()[(fy * fw + x) * 4..(fy * fw + x) * 4 + 4])
                    }
                    _ => window::EMPTY,
                };
            }
        }
    }

    /// The window's pixels, a row at a time, are the pixels turned one at a time, to the bit:
    /// frames narrower than the window and shorter, one under the other, the window beyond.
    #[test]
    fn the_window_s_pixels_a_row_at_a_time_are_its_pixels_one_at_a_time() {
        let frame = |w: u32, h: u32, k: u8| {
            let mut p = ca72_panel::Pixmap::new(w, h).unwrap();
            for (i, px) in p.data_mut().chunks_exact_mut(4).enumerate() {
                px.copy_from_slice(&[(i as u8).wrapping_mul(k), k, (i / 7) as u8, 255]);
            }
            p
        };
        let (a, b) = (frame(37, 11, 3), frame(29, 6, 5));
        let (w, h) = (41, 20);
        let (mut new, mut old) = (vec![0u32; w * h], vec![0u32; w * h]);
        window::fill(&mut new, (w, h), &[&a, &b]);
        filled_a_pixel_at_a_time(&mut old, (w, h), &[&a, &b]);
        assert_eq!(new, old);
    }

    /// What turning a Retina window's frame into the screen's pixels costs, a row at a time and
    /// a pixel at a time: printed (`-- --ignored --nocapture`).
    #[test]
    #[ignore = "prints timings"]
    fn the_window_s_pixels_timings() {
        let (w, h) = (3440usize, 1965usize);
        let f = ca72_panel::Pixmap::new(w as u32, h as u32).unwrap();
        let mut buffer = vec![0u32; w * h];
        for (name, way) in [
            (
                "a row at a time",
                window::fill as fn(&mut [u32], (usize, usize), &[&ca72_panel::Pixmap]),
            ),
            ("a pixel at a time", filled_a_pixel_at_a_time),
        ] {
            let t = Instant::now();
            for _ in 0..50 {
                way(&mut buffer, (w, h), &[&f]);
            }
            println!(
                "{name}: {:.2} ms a frame",
                t.elapsed().as_secs_f64() * 1000.0 / 50.0
            );
        }
    }

    /// The window played for a look and a measure (decisions.md R48): 24 seconds in real
    /// time at sixty frames a second, a Retina window (3440 pixels across): four POLY voices
    /// with DOUBLE playing chords at HI, then ULTRA, then LO, falling quiet, then HI again with
    /// DETUNE swept from none to all of it. Each frame drawn and turned into the screen's
    /// pixels as `present` does, timed: the whole frame, as on macOS, and the places changed
    /// alone, as on Windows and X11 (decisions.md R58), the screen so made the same as
    /// the whole frame's at every frame. Every other frame's QUALITY column and STEREO display
    /// kept, written to `$CA72_RUN`, the folder, as `frames.rgba` (`size.txt`: its width and
    /// height), with the times in `times.txt` (`-- --ignored --nocapture`). With
    /// `CA72_NO_FRAMES` set, the times alone (the frames are 2 GB, in memory and on disk).
    #[test]
    #[ignore = "plays the window for a while"]
    fn the_window_played() {
        let Some(out) = std::env::var_os("CA72_RUN").map(std::path::PathBuf::from) else {
            return;
        };
        std::fs::create_dir_all(&out).unwrap();
        let keep_frames = std::env::var_os("CA72_NO_FRAMES").is_none();
        let dir = tempfile::tempdir().unwrap();
        let host = Arc::new(Host::default());
        let params = Arc::new(Ca72Params::default());
        let context: Arc<dyn GuiContext> = host.clone();
        let meters = Arc::new(Meters::default());
        let mut e = Editing::new(
            context,
            Arc::clone(&params),
            Arc::clone(&meters),
            (1720, 3440, 2.0),
            Library::at(dir.path()),
            |_| None,
        );
        // (Set as a host's automation sets them: nih-plug's context for tests.)
        let hostless = nih_plug::context::process::TestProcessContext::<crate::Ca72>::new(
            Arc::clone(&params) as Arc<dyn Params>,
            48_000.0,
            ProcessMode::Realtime,
        );
        let set = |p: ParamPtr, v: f32| {
            hostless.automate(p, v);
        };
        set(params.poly.as_ptr(), 1.0);
        set(params.voices.as_ptr(), 0.25);
        set(params.spread.as_ptr(), 0.8);
        set(params.double.as_ptr(), 0.6);
        set(params.quality.as_ptr(), 0.5);
        let s = 3440.0 / art::W;
        // The two parts kept: QUALITY's column, and STEREO's knobs over its display.
        let column = (
            (art::COL + 2925.0) * s,
            (art::TOP - 20.0) * s,
            183.0 * s,
            400.0 * s,
        );
        let (dx, dy, dw, dh) = strip::DISPLAY;
        let stereo = (
            (dx - 10.0) * s,
            (strip::ROWS[1] - 75.0) * s,
            (dw + 20.0) * s,
            (dy + dh + 10.0 - strip::ROWS[1] + 75.0) * s,
        );
        let px = |r: (f64, f64, f64, f64)| (r.0 as usize, r.1 as usize, r.2 as usize, r.3 as usize);
        let (column, stereo) = (px(column), px(stereo));
        let (fw, fh) = (column.2 + 12 + stereo.2, column.3.max(stereo.3));
        let mut frames: Vec<u8> = Vec::new();
        let (mut screen, mut kept): (Vec<u32>, Vec<u32>) = (Vec::new(), Vec::new());
        // A voice's level, `t` seconds into a chord it plays: a quick attack, a decay to its
        // sustain, held 1.6 beats, released.
        let beat = 60.0 / 112.0;
        let env = |t: f64| {
            let held = 1.6 * beat;
            let a = |t: f64| (t / 0.01).min(1.0) * (0.6 + 0.4 * (-t / 0.3).exp());
            if t < 0.0 {
                0.0
            } else if t < held {
                a(t)
            } else {
                a(held) * (-(t - held) / 0.25).exp()
            }
        };
        let t0 = Instant::now();
        // The time this thread has worked, ms: on Linux its own, the computer's other work left
        // out (with builds running alongside, the wall's clock varied a third from run to run);
        // elsewhere the wall's.
        let cpu = || -> f64 {
            #[cfg(target_os = "linux")]
            if let Some(ns) = std::fs::read_to_string("/proc/thread-self/schedstat")
                .ok()
                .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok())
            {
                return ns / 1e6;
            }
            t0.elapsed().as_secs_f64() * 1000.0
        };
        let ticks = 24 * 60;
        // Each frame's work: drawn and turned whole, drawn and turned in places.
        let (mut whole, mut places) = (Vec::new(), Vec::new());
        let mut presented = 0;
        for k in 0..ticks {
            let at = t0 + Duration::from_micros(k * 16_667);
            if let Some(wait) = at.checked_duration_since(Instant::now()) {
                std::thread::sleep(wait);
            }
            let t = k as f64 / 60.0;
            let quality = if t < 6.0 {
                0.5
            } else if t < 12.0 {
                1.0
            } else if t < 18.0 {
                0.0
            } else {
                0.5
            };
            set(params.quality.as_ptr(), quality);
            if t >= 18.0 {
                set(
                    params.double.as_ptr(),
                    ((t - 18.5) / 5.0).clamp(0.0, 1.0) as f32,
                );
            }
            // Chords every two beats (quiet from 15 to 18 seconds), each voice its own
            // velocity.
            let playing = !(15.0..18.0).contains(&t);
            let chord = (t / (2.0 * beat)).floor();
            let since = t - chord * 2.0 * beat;
            let mut levels = [0.0f32; LEVELS];
            let mut mask = 0u32;
            for (v, level) in levels.iter_mut().enumerate().take(4) {
                let vel = 0.55 + 0.45 * (((chord as usize + v) * 7) % 5) as f64 / 4.0;
                let l = if playing {
                    0.35 * vel * env(since - 0.02 * v as f64)
                } else {
                    0.0
                };
                *level = l as f32;
                if l > 1e-3 {
                    mask |= 1 << v;
                }
            }
            let output = (levels.iter().map(|&l| f64::from(l)).sum::<f64>() * 0.7).min(1.0);
            meters.publish(0.0, (0.5, 0.0), mask);
            meters.publish_levels(&levels, output as f32);
            let started = cpu();
            let changed = e.draw();
            let drawn = cpu() - started;
            let (mut turned, mut in_places) = (0.0, 0.0);
            if changed {
                let f = e.composed.as_ref().unwrap_or_else(|| e.renderer.frame());
                let size = (f.width() as usize, f.height() as usize);
                // (As `present` turns it: the window's own code.)
                let fresh = screen.len() != size.0 * size.1;
                screen.resize(size.0 * size.1, 0);
                kept.resize(size.0 * size.1, 0);
                let started = cpu();
                window::fill(&mut screen, size, &[f]);
                turned = cpu() - started;
                let started = cpu();
                match e.damage.as_deref().filter(|_| !fresh) {
                    Some(d) => {
                        window::fill_places(&mut kept, size, &[f], d);
                    }
                    None => window::fill(&mut kept, size, &[f]),
                }
                in_places = cpu() - started;
                assert!(kept == screen, "the screen turned in places at frame {k}");
                presented += 1;
            }
            whole.push(drawn + turned);
            places.push(drawn + in_places);
            if keep_frames && k % 2 == 0 {
                let f = e.composed.as_ref().unwrap_or_else(|| e.renderer.frame());
                let mut img = vec![0u8; fw * fh * 4];
                for (r, x0) in [(column, 0usize), (stereo, column.2 + 12)] {
                    for y in 0..r.3 {
                        let from = ((r.1 + y) * f.width() as usize + r.0) * 4;
                        let to = (y * fw + x0) * 4;
                        img[to..to + r.2 * 4].copy_from_slice(&f.data()[from..from + r.2 * 4]);
                    }
                }
                frames.extend_from_slice(&img);
            }
        }
        if keep_frames {
            std::fs::write(out.join("frames.rgba"), &frames).unwrap();
            std::fs::write(out.join("size.txt"), format!("{fw} {fh}")).unwrap();
        }
        let said = |times: &[f64]| {
            let total: f64 = times.iter().sum();
            let mean = total / times.len() as f64;
            let mut sorted = times.to_vec();
            sorted.sort_by(f64::total_cmp);
            let p95 = sorted[sorted.len() * 95 / 100];
            format!(
                "{total:.0} ms ({:.1} % of one core), {mean:.2} ms a frame on average, {p95:.2} at the 95th percentile",
                total / 24_000.0 * 100.0
            )
        };
        let report = format!(
            "{ticks} frames in 24 s: {presented} presented; the UI thread's work, each frame turned whole (macOS): {}; turned in the places changed (Windows, X11): {}\n",
            said(&whole),
            said(&places)
        );
        print!("{report}");
        std::fs::write(out.join("times.txt"), report).unwrap();
    }

    /// QUALITY's toggle (decisions.md R47, R48) goes where it is clicked, a gesture of
    /// its own as a rocker's: above its nut ULTRA, below it LO, on it (HI already) nothing.
    #[test]
    fn quality_goes_where_its_toggle_is_clicked() {
        let (mut e, host, params) = editing_at_hi();
        let q = params.quality.as_ptr();
        let (x, y) = centre("quality");
        let up = at(&e, (x, y - 40.0));
        click(&mut e, up);
        assert_eq!(
            host.take(),
            vec![Call::Begin(q), Call::Set(q, 1.0), Call::End(q)]
        );
        let down = at(&e, (x, y + 45.0));
        click(&mut e, down);
        assert_eq!(
            host.take(),
            vec![Call::Begin(q), Call::Set(q, 0.0), Call::End(q)]
        );
        let middle = at(&e, (x, y));
        click(&mut e, middle);
        assert_eq!(host.take(), vec![]);
    }

    /// A host's key (Cubase gives a VST3 plug-in its keys through `onKeyDown()`: decisions.md
    /// R53) as the window's own: VST3's codes named, else the character, the keypad's digits
    /// without one too, the modifiers; none for a key the editor has no use for.
    #[test]
    fn a_hosts_key_is_read_as_the_windows_own() {
        let key = |character, key_code, modifiers| HostKey {
            character,
            key_code,
            modifiers,
            down: true,
        };
        let read = |k| host_key_event(k).map(|e| (e.key, e.modifiers));
        assert_eq!(
            read(key(Some('8'), 0, 0)),
            Some((Key::Character("8".into()), Modifiers::empty()))
        );
        assert_eq!(
            read(key(Some('A'), 0, 1)),
            Some((Key::Character("A".into()), Modifiers::SHIFT))
        );
        assert_eq!(read(key(Some('\r'), 4, 0)).map(|r| r.0), Some(Key::Enter));
        assert_eq!(read(key(None, 19, 0)).map(|r| r.0), Some(Key::Enter));
        assert_eq!(
            read(key(Some('\u{1b}'), 6, 0)).map(|r| r.0),
            Some(Key::Escape)
        );
        assert_eq!(
            read(key(Some('\u{8}'), 1, 0)).map(|r| r.0),
            Some(Key::Backspace)
        );
        assert_eq!(read(key(None, 14, 0)).map(|r| r.0), Some(Key::ArrowDown));
        assert_eq!(
            read(key(None, 29, 0)).map(|r| r.0),
            Some(Key::Character("5".into()))
        );
        assert_eq!(
            read(key(None, 38, 0)).map(|r| r.0),
            Some(Key::Character(".".into()))
        );
        assert_eq!(read(key(None, 44, 0)), None, "F5");
        let up = host_key_event(HostKey {
            down: false,
            ..key(Some('8'), 0, 0)
        })
        .expect("a key");
        assert_eq!(up.state, KeyState::Up);
    }

    /// The host's keys are the editor's only while it holds the keyboard (a value typed, the
    /// drawer, MIDI Learn): then every key is taken, so that none is the host's key command
    /// meanwhile, and those it has a use for are kept for the window; else none is taken.
    #[test]
    fn the_hosts_keys_are_taken_only_while_the_editor_holds_the_keyboard() {
        let keys = HostKeys::default();
        let k = |character, key_code| HostKey {
            character,
            key_code,
            modifiers: 0,
            down: true,
        };
        assert!(!keys.offer(k(Some('8'), 0)));
        assert!(keys.take().is_empty());
        keys.holding.store(true, Ordering::Release);
        assert!(keys.offer(k(Some('8'), 0)));
        assert!(keys.offer(k(None, 44)), "F5: taken, kept for nothing");
        assert!(keys.offer(k(Some('\r'), 4)));
        let taken: Vec<Key> = keys.take().into_iter().map(|e| e.key).collect();
        assert_eq!(taken, vec![Key::Character("8".into()), Key::Enter]);
        assert!(keys.take().is_empty());
    }

    /// A key a host gives both ways (the view's and the window's own) is taken once, whichever
    /// comes first; two presses of one key from one way are two; a twin later than [`TWIN`] is a
    /// key of its own.
    #[test]
    fn a_key_given_both_ways_is_taken_once() {
        let mut twins = KeyTwins::default();
        let ev = |c: &str| KeyboardEvent {
            state: KeyState::Down,
            key: Key::Character(c.into()),
            ..KeyboardEvent::default()
        };
        let t0 = Instant::now();
        let at = |ms: u64| t0 + Duration::from_millis(ms);
        assert!(twins.own_new(&ev("8"), at(0)));
        assert!(!twins.host_new(&ev("8"), at(10)));
        assert!(twins.host_new(&ev("4"), at(20)));
        assert!(!twins.own_new(&ev("4"), at(25)));
        assert!(twins.own_new(&ev("1"), at(30)));
        assert!(twins.own_new(&ev("1"), at(40)));
        assert!(!twins.host_new(&ev("1"), at(45)));
        assert!(!twins.host_new(&ev("1"), at(50)));
        assert!(twins.own_new(&ev("2"), at(60)));
        assert!(twins.host_new(&ev("2"), at(60 + 200)));
    }

    /// A value typed with the host's keys (Cubase) as with the window's own.
    #[test]
    fn a_value_typed_with_the_hosts_keys() {
        let (mut e, host, params) = editing();
        let p = on_strip(&e, strip_at(StripTarget::Readout(Readout::Drive)));
        click(&mut e, p);
        click(&mut e, p);
        for (character, key_code) in [(Some('1'), 0), (None, 26), (Some('\r'), 4)] {
            let k = host_key_event(HostKey {
                character,
                key_code,
                modifiers: 0,
                down: true,
            })
            .expect("a key");
            assert!(e.keyed(&k));
        }
        let d = params.drive.as_ptr();
        assert_eq!(
            host.take(),
            vec![
                Call::Begin(d),
                Call::Set(d, params.drive.preview_normalized(12.0)),
                Call::End(d)
            ]
        );
    }

    /// A value typed into a readout (decisions.md R52): a double click on it, its digits,
    /// Enter: its knob set at once, as the readout reads it, within the knob's travel, a gesture
    /// of its own. Escape, or a press elsewhere, leaves it as it was; a press on the readout
    /// itself goes on typing. A switch's readout double-clicked keeps its amount (its first
    /// press undone) and takes a value too. VOICES within 2 to 10; DETUNE in SCATTER takes none;
    /// AUTO GAIN's reads what is measured and takes none.
    #[test]
    fn a_value_typed_into_a_readout_sets_its_knob() {
        let (mut e, host, params) = editing_poly();
        let double = |e: &mut Editing, t: StripTarget| {
            let p = on_strip(e, strip_at(t));
            click(e, p);
            click(e, p);
        };
        let typed = |e: &mut Editing, text: &str| {
            for c in text.chars() {
                keyed(e, Key::Character(c.to_string()));
            }
        };
        let shown = |e: &mut Editing, r: Readout| {
            e.draw();
            let i = Readout::ALL
                .iter()
                .position(|x| *x == r)
                .expect("a readout");
            e.strip_scene.readouts[i].clone()
        };
        // DRIVE: 8.4 dB.
        double(&mut e, StripTarget::Readout(Readout::Drive));
        assert_eq!(host.take(), vec![]);
        assert_eq!(shown(&mut e, Readout::Drive), ("_".to_owned(), true));
        typed(&mut e, "8.4x");
        assert_eq!(shown(&mut e, Readout::Drive), ("8.4_".to_owned(), true));
        // (A press on it goes on typing.)
        let at = on_strip(&e, strip_at(StripTarget::Readout(Readout::Drive)));
        click(&mut e, at);
        keyed(&mut e, Key::Enter);
        let d = params.drive.as_ptr();
        assert_eq!(
            host.take(),
            vec![
                Call::Begin(d),
                Call::Set(d, params.drive.preview_normalized(8.4)),
                Call::End(d)
            ]
        );
        assert!(e.typing.is_none());
        // LEVEL: Escape, and a press elsewhere, leave it; beyond its travel, its end.
        double(&mut e, StripTarget::Readout(Readout::Level));
        typed(&mut e, "-6");
        keyed(&mut e, Key::Escape);
        double(&mut e, StripTarget::Readout(Readout::Level));
        typed(&mut e, "3");
        let elsewhere = on_strip(&e, (1500.0, 1700.0));
        click(&mut e, elsewhere);
        assert_eq!(host.take(), vec![]);
        assert!(e.typing.is_none());
        let l = params.level.as_ptr();
        for (keys, to) in [
            ("-99", 0.0),
            ("159\u{8}", 1.0),
            ("-9", params.level.preview_normalized(-9.0)),
        ] {
            double(&mut e, StripTarget::Readout(Readout::Level));
            for c in keys.chars() {
                if c == '\u{8}' {
                    keyed(&mut e, Key::Backspace);
                } else {
                    keyed(&mut e, Key::Character(c.to_string()));
                }
            }
            keyed(&mut e, Key::Enter);
            assert_eq!(
                host.take(),
                vec![Call::Begin(l), Call::Set(l, to), Call::End(l)],
                "{keys:?}"
            );
        }
        // ENTROPY's readout, a switch: its first press switched it (the test host applies
        // nothing, so there is nothing to undo here), then a value.
        double(&mut e, StripTarget::Switch(Readout::Entropy));
        let n = params.entropy.as_ptr();
        let first = host.take();
        assert_eq!(first.len(), 3, "the first press's switch: {first:?}");
        typed(&mut e, "25");
        keyed(&mut e, Key::Enter);
        assert_eq!(
            host.take(),
            vec![
                Call::Begin(n),
                Call::Set(n, params.entropy.preview_normalized(25.0)),
                Call::End(n)
            ]
        );
        // VOICES: 12 is 10.
        double(&mut e, StripTarget::Readout(Readout::Voices));
        typed(&mut e, "12");
        keyed(&mut e, Key::Enter);
        let v = params.voices.as_ptr();
        assert_eq!(
            host.take(),
            vec![Call::Begin(v), Call::Set(v, 1.0), Call::End(v)]
        );
        // DETUNE in SCATTER: none; AUTO GAIN's: none.
        double(&mut e, StripTarget::Readout(Readout::Detune));
        assert!(e.typing.is_none());
        assert!(!Readout::Auto.typed());
    }

    /// ENTROPY's new reading and OFF/recall use the entire travel, including below 5 %.
    #[test]
    fn entropy_displays_the_new_amount_and_recalls_it_after_off() {
        let (mut e, host, params) = editing();
        let context = nih_plug::context::process::TestProcessContext::<crate::Ca72>::new(
            Arc::clone(&params) as Arc<dyn Params>,
            48_000.0,
            ProcessMode::Realtime,
        );
        let p = params.entropy.as_ptr();
        let shown = |e: &mut Editing| {
            e.draw();
            e.strip_scene.readouts[Readout::ALL
                .iter()
                .position(|r| *r == Readout::Entropy)
                .unwrap()]
            .clone()
        };
        assert_eq!(shown(&mut e), ("5".to_owned(), true));
        for (amount, text) in [(0.05, "5"), (0.02, "2"), (0.43, "43")] {
            assert!(context.automate(p, amount));
            assert_eq!(shown(&mut e), (text.to_owned(), true));
            e.switch_amount(Readout::Entropy, false);
            assert_eq!(
                host.take(),
                vec![Call::Begin(p), Call::Set(p, 0.0), Call::End(p)]
            );
            assert!(context.automate(p, 0.0));
            assert_eq!(shown(&mut e), ("OFF".to_owned(), false));
            e.switch_amount(Readout::Entropy, true);
            assert_eq!(
                host.take(),
                vec![Call::Begin(p), Call::Set(p, amount), Call::End(p)]
            );
        }
    }

    /// The rail's settings key (decisions.md R51) opens its menu: under LOOK the editor's
    /// looks, the realistic by default, marked and not to be chosen; CLASSIC draws the editor in
    /// the drawn skin and is kept, REALISTIC brings the realistic back.
    #[test]
    fn the_settings_key_chooses_the_look() {
        let (mut e, _, _) = editing();
        e.update = Update::with(
            crate::update::fake::no_curl,
            crate::update::fake::no_browser,
        );
        let update = e.update.scene();
        assert_eq!(
            (e.settings.skin, e.renderer.skin()),
            (Skin::Worn, Skin::Worn)
        );
        let key = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Settings)));
        click(&mut e, key);
        assert_eq!(
            e.learning.menu().map(|m| m.title.as_str()),
            Some(format!("SETTINGS · CA-72 {}", crate::Ca72::VERSION).as_str())
        );
        assert_eq!(e.update.scene(), update, "opening Settings does not check");
        assert_eq!(
            menu_items(&e)[..3],
            [
                ("LOOK".to_owned(), false),
                ("REALISTIC".to_owned(), false),
                ("CLASSIC".to_owned(), true),
            ]
        );
        assert_eq!(e.learning.menu().map(|m| m.marked[0]), Some(1));
        // The menu within the window, under the key.
        let m = e.learning.menu().expect("a menu");
        let (w, h) = m.extent(e.renderer.fonts());
        assert!(m.x >= 0.0 && m.x + w <= art::W && m.y + h <= art::H);
        assert!(m.y > strip::centre(Rail::Plugin, StripTarget::Bar(BarTarget::Settings)).1);
        menu_item(&mut e, "CLASSIC");
        assert_eq!(
            (e.settings.skin, e.renderer.skin()),
            (Skin::Drawn, Skin::Drawn)
        );
        assert!(e.learning.menu().is_none());
        e.draw();
        click(&mut e, key);
        assert_eq!(menu_items(&e)[1], ("REALISTIC".to_owned(), true));
        assert_eq!(e.learning.menu().map(|m| m.marked[0]), Some(2));
        // (The head does nothing: the menu stays open.)
        menu_item(&mut e, "LOOK");
        assert!(e.learning.menu().is_some());
        menu_item(&mut e, "REALISTIC");
        assert_eq!(
            (e.settings.skin, e.renderer.skin()),
            (Skin::Worn, Skin::Worn)
        );
        e.draw();
        e.update.press();
        assert_eq!(e.update.scene().text, "COULD NOT CHECK");
        click(&mut e, key);
        assert_eq!(
            e.learning.menu().map(|m| m.title.as_str()),
            Some(format!("SETTINGS · CA-72 {}", crate::Ca72::VERSION).as_str())
        );
        assert_eq!(e.update.scene().text, "COULD NOT CHECK");
    }

    /// The settings key's LETTERING (R-LETTERING): STANDARD by default, marked and not to be
    /// chosen. LARGE prints the panel's and the strip's lettering a size larger, in the look the
    /// editor is in, the menus' and the tip's with it, and is kept; a look chosen after keeps it;
    /// STANDARD brings the standard back. In the large lettering the settings' menu still fits
    /// the window, the panel shown or hidden.
    #[test]
    fn the_settings_key_chooses_the_lettering() {
        let (mut e, _, _) = editing();
        assert_eq!(
            (e.settings.lettering, e.renderer.lettering()),
            (Lettering::Standard, Lettering::Standard)
        );
        let tip = e.renderer.text_size();
        let key = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Settings)));
        click(&mut e, key);
        assert_eq!(
            menu_items(&e)[3..6],
            [
                ("LETTERING".to_owned(), false),
                ("STANDARD".to_owned(), false),
                ("LARGE".to_owned(), true),
            ]
        );
        assert_eq!(
            e.learning.menu().map(|m| m.marked.clone()),
            Some(vec![1, 4, 7])
        );
        let standard = e.learning.menu().expect("a menu").size;
        menu_item(&mut e, "LARGE");
        assert_eq!(
            (
                e.settings.lettering,
                e.renderer.lettering(),
                e.renderer.skin()
            ),
            (Lettering::Large, Lettering::Large, Skin::Worn)
        );
        assert!((e.renderer.text_size() / tip - 1.25).abs() < 1e-9);
        assert!(e.learning.menu().is_none());
        e.draw();
        click(&mut e, key);
        let m = e.learning.menu().expect("a menu");
        assert!((m.size / standard - 1.25).abs() < 1e-9);
        let (w, h) = m.extent(e.renderer.fonts());
        assert!(m.x >= 0.0 && m.x + w <= art::W && m.y + h <= art::H);
        assert_eq!(menu_items(&e)[5], ("LARGE".to_owned(), false));
        menu_item(&mut e, "CLASSIC");
        assert_eq!(
            (e.renderer.skin(), e.renderer.lettering()),
            (Skin::Drawn, Lettering::Large)
        );
        e.draw();
        // The panel hidden: the menu within the view, above the drawer.
        let panel = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Panel)));
        click(&mut e, panel);
        let key = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Settings)));
        click(&mut e, key);
        e.draw();
        let m = e.learning.menu().expect("a menu");
        let (_, h) = m.extent(e.renderer.fonts());
        assert!(m.y >= 0.0 && m.y + h <= DRAWER_TOP, "{} + {h}", m.y);
        menu_item(&mut e, "STANDARD");
        assert_eq!(
            (
                e.settings.lettering,
                e.renderer.lettering(),
                e.renderer.skin()
            ),
            (Lettering::Standard, Lettering::Standard, Skin::Drawn)
        );
        e.draw();
    }

    /// The settings key's FREQUENCY DIALS (decisions.md R54): SEMITONES by default, each
    /// group's choice marked. HARDWARE turns oscillators 2 and 3's knobs to where the hardware's
    /// dial has their pots, the parameters (the sound) as they were, and the tip still reads
    /// semitones; a pot past a law's stop shows its knob at the stop, and a press that does not
    /// move it leaves it there. A drag sets the pot through the law chosen.
    #[test]
    fn the_settings_key_chooses_how_the_frequency_dials_read() {
        use nih_plug::context::process::TestProcessContext;
        let (mut e, host, params) = editing();
        let key = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Settings)));
        click(&mut e, key);
        assert_eq!(
            menu_items(&e)[6..9],
            [
                ("FREQUENCY DIALS".to_owned(), false),
                ("SEMITONES".to_owned(), false),
                ("HARDWARE".to_owned(), true),
            ]
        );
        assert_eq!(
            e.learning
                .menu()
                .map(|m| (m.heads.clone(), m.marked.clone())),
            Some((vec![0, 3, 6, 9], vec![1, 4, 7]))
        );
        // Oscillator 2 at +5 semitones, oscillator 3's pot at its clockwise end (+8.22).
        let ps: Arc<dyn Params> = params.clone();
        let c = TestProcessContext::<crate::Ca72>::new(ps, 48_000.0, ProcessMode::Realtime);
        let five = Pot::Two.wiper(5.0) as f32;
        c.automate(params.osc2_frequency.as_ptr(), five);
        c.automate(params.osc3_frequency.as_ptr(), 1.0);
        menu_item(&mut e, "SEMITONES");
        assert!(
            e.learning.menu().is_some(),
            "the current choice does nothing"
        );
        let away = at(&e, (0.0, 0.0));
        click(&mut e, away);
        let (two, three) = (
            ca72_panel::controls::index("osc2_frequency").expect("a control"),
            ca72_panel::controls::index("osc3_frequency").expect("a control"),
        );
        e.update_scene();
        assert!((e.scene.values[two] - (5.0 + 7.5) / 15.0).abs() < 1e-6);
        assert_eq!(e.scene.values[three], 1.0, "past the stop: at it");
        host.take();
        click(&mut e, key);
        menu_item(&mut e, "HARDWARE");
        assert_eq!(
            (e.settings.frequency, params.freq_law()),
            (Law::Hardware, Law::Hardware)
        );
        assert!(host.take().is_empty(), "the sound as it was");
        e.update_scene();
        let knob = Pot::Two.knob_at(Law::Hardware, f64::from(five));
        assert!((e.scene.values[two] - knob).abs() < 1e-6);
        assert!(
            e.scene.values[two] < (5.0 + 7.5) / 15.0,
            "the hardware's +5 is wider"
        );
        assert_eq!(e.scene.values[three], 1.0);
        click(&mut e, key);
        assert_eq!(
            e.learning.menu().map(|m| m.marked.clone()),
            Some(vec![1, 4, 8])
        );
        let away = at(&e, (0.0, 0.0));
        click(&mut e, away);
        // The tip in semitones either way.
        let p = e.operated(two).expect("a knob");
        assert_eq!(p.text(p.value()), "+5.00 semitones");
        // A drag sets the pot through the law chosen: the knob turned as far through either.
        let drag_up = |e: &mut Editing| {
            let p2 = at(e, centre("osc2_frequency"));
            go(e, p2);
            e.mouse(MouseEvent::ButtonPressed {
                button: MouseButton::Left,
                modifiers: Modifiers::empty(),
            });
            go(
                e,
                Point {
                    x: p2.x,
                    y: p2.y - 20.0,
                },
            );
            e.mouse(MouseEvent::ButtonReleased {
                button: MouseButton::Left,
                modifiers: Modifiers::empty(),
            });
            let f = params.osc2_frequency.as_ptr();
            host.take()
                .into_iter()
                .filter_map(|c| match c {
                    Call::Set(q, v) if q == f => Some(f64::from(v)),
                    _ => None,
                })
                .last()
                .expect("set")
        };
        let turned = |law: Law, wiper: f64| {
            Pot::Two.knob_at(law, wiper) - Pot::Two.knob_at(law, f64::from(five))
        };
        let hardware = turned(Law::Hardware, drag_up(&mut e));
        e.set_frequency_dials(Law::Semitones);
        // (Not a double click, which would restore the default.)
        e.last_press = None;
        let semitones = turned(Law::Semitones, drag_up(&mut e));
        assert!(
            hardware > 0.01 && (hardware - semitones).abs() < 1e-5,
            "{hardware} {semitones}"
        );
        e.set_frequency_dials(Law::Hardware);
        e.last_press = None;
        // Oscillator 3's knob, pinned at its stop, pressed and let go without moving: as it was.
        let p3 = at(&e, centre("osc3_frequency"));
        go(&mut e, p3);
        e.mouse(MouseEvent::ButtonPressed {
            button: MouseButton::Left,
            modifiers: Modifiers::empty(),
        });
        e.mouse(MouseEvent::ButtonReleased {
            button: MouseButton::Left,
            modifiers: Modifiers::empty(),
        });
        let g = params.osc3_frequency.as_ptr();
        assert!(
            !host
                .take()
                .iter()
                .any(|c| matches!(c, Call::Set(q, _) if *q == g)),
            "a press that did not move sets nothing"
        );
    }

    /// A right click on QUALITY's opening (decisions.md R48) opens its menu: its shutter or
    /// always open, the one it is in in its title and not to be chosen; choosing the other
    /// switches it, and the menu says so the next time.
    #[test]
    fn quality_s_menu_shows_or_hides_its_indicator() {
        let (mut e, _, _) = editing();
        assert!(e.settings.shutter, "shown by default");
        let toggle = at(&e, centre("quality"));
        right_click(&mut e, toggle);
        assert_eq!(
            e.learning.menu().map(|m| m.title.as_str()),
            Some("QUALITY: SET PER PLUGIN INSTANCE")
        );
        assert_eq!(
            menu_items(&e),
            vec![
                ("SHOW ITS INDICATOR".to_owned(), false),
                ("HIDE ITS INDICATOR".to_owned(), true),
                ("NOT LEARNED BY MIDI LEARN".to_owned(), false),
                ("MIDI ASSIGNMENTS\u{2026}".to_owned(), true),
            ]
        );
        menu_item(&mut e, "HIDE ITS INDICATOR");
        assert!(!e.settings.shutter && e.learning.menu().is_none());
        e.draw();
        assert!(e.scene.opening.hidden);
        right_click(&mut e, toggle);
        assert_eq!(menu_items(&e)[0], ("SHOW ITS INDICATOR".to_owned(), true));
        menu_item(&mut e, "SHOW ITS INDICATOR");
        assert!(e.settings.shutter);
        // (Where the indicator is, a right click is the panel's, as anywhere off a control.)
        let above = at(&e, (art::COL + 3023.0, art::TOP + 92.0));
        right_click(&mut e, above);
        assert!(e.learning.menu().is_none());
    }

    /// QUALITY's opening (decisions.md R48) follows QUALITY by the time between frames
    /// (a stalled frame does not jump it): from HI, its lamp goes down over the closing time,
    /// then ULTRA's Tesla lamp comes up over the opening time; up, ULTRA's note shows until a
    /// press reads it, and not again; with the shutter off, its own times.
    #[test]
    fn quality_s_opening_follows_quality_and_ultra_s_note_is_read_once() {
        let (mut e, _, _) = editing_at_hi();
        assert_eq!(
            (e.motion.lamp, e.motion.coil),
            (1.0, 0.0),
            "it opens at HI, up"
        );
        let t0 = Instant::now();
        let frames = |e: &mut Editing, from: Instant, n: u32, q: QualityMode| {
            for k in 0..=n {
                e.move_opening(from + Duration::from_millis(u64::from(k) * 40), q);
            }
        };
        let ultra = QualityMode::Ultra;
        e.move_opening(t0, ultra);
        frames(&mut e, t0, 30, ultra);
        assert!(
            (e.motion.lamp - (1.0 - 1.2 / ultra::CLOSE_S)).abs() < 1e-9,
            "{}",
            e.motion.lamp
        );
        assert_eq!(e.motion.coil, 0.0);
        // (A stall of a second moves it on 50 ms.)
        e.move_opening(t0 + Duration::from_millis(2200), ultra);
        assert!(
            (e.motion.lamp - (1.0 - 1.25 / ultra::CLOSE_S)).abs() < 1e-9,
            "{}",
            e.motion.lamp
        );
        frames(&mut e, t0 + Duration::from_millis(2200), 100, ultra);
        assert_eq!((e.motion.lamp, e.motion.coil), (0.0, 1.0));
        assert_eq!(e.scene.opening.coil, 1.0);
        assert!(e.ultra_note);
        assert_eq!(e.ultra_note().lines[0].0, "ULTRA · NO COMPROMISES");
        press(&mut e);
        assert!(!e.ultra_note && e.settings.ultra_note_read);
        e.move_opening(t0 + Duration::from_secs(8), ultra);
        assert!(!e.ultra_note);
        // Hidden: none of it drawn, LO's hamster there at once (shown again, nothing comes up).
        e.settings.shutter = false;
        let t1 = t0 + Duration::from_secs(10);
        frames(&mut e, t1, 1, QualityMode::Lo);
        assert_eq!((e.motion.coil, e.motion.hamster), (0.0, 1.0));
        assert!(e.scene.opening.hidden);
        e.settings.shutter = true;
        frames(&mut e, t1 + Duration::from_secs(1), 1, QualityMode::Lo);
        assert!(!e.scene.opening.hidden && e.scene.opening.hamster == 1.0);
    }

    /// ULTRA's note shows once a computer: read in another editor since this one opened (a
    /// second instance, another host), it is not shown here as the lamp comes up, and this
    /// editor's own change to another setting writes that key alone (the CA-74's review).
    #[test]
    fn ultra_s_note_read_elsewhere_is_not_shown_again() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("CA-72").join("settings.toml");
        let (mut e, _, _) = editing();
        e.settings_file = Some(path.clone());
        // The other editor reads the note and hides nothing; this one then turns its
        // indicator... on (as it was): the note stays read in the file.
        Settings::keep(&path, Change::NoteRead).unwrap();
        e.keep_settings(Change::Shutter(true));
        assert!(Settings::read(&path).ultra_note_read);
        let t0 = Instant::now();
        for k in 0..150u64 {
            e.move_opening(t0 + Duration::from_millis(40 * k), QualityMode::Ultra);
        }
        assert_eq!(e.motion.coil, 1.0);
        assert!(!e.ultra_note && e.settings.ultra_note_read);
    }

    /// What follows the synth moves thirty times a second at baseview's own frame rate, a frame
    /// every 15 ms (macOS, Windows and X11 alike), which reaches a thirtieth of a second only
    /// every third frame: about 90 moves in three seconds (it had been 66, 22 a second: the CA-74's
    /// review); a stalled frame moves it once and the next on time, not in a burst.
    #[test]
    fn the_opening_moves_thirty_times_a_second_on_baseviews_frames() {
        let (mut e, _, _) = editing();
        let t0 = Instant::now();
        let mut moved = 0;
        for k in 0..200u64 {
            let was = e.motion_at;
            e.move_opening(t0 + Duration::from_millis(15 * k), QualityMode::Hi);
            moved += u32::from(e.motion_at != was);
        }
        assert!((89..=91).contains(&moved), "{moved} moves in 3 s");
        // A half-second stall: one move then, the next a thirtieth later or so.
        let late = t0 + Duration::from_millis(3500);
        let was = e.motion_at;
        e.move_opening(late, QualityMode::Hi);
        assert_ne!(e.motion_at, was);
        let mut burst = 0;
        for k in 1..=2u64 {
            let was = e.motion_at;
            e.move_opening(late + Duration::from_millis(15 * k), QualityMode::Hi);
            burst += u32::from(e.motion_at != was);
        }
        assert!(burst <= 1, "{burst} moves within 30 ms of a stall");
    }

    /// What follows the synth is drawn at most thirty times a second: frames a sixtieth of a
    /// second apart move QUALITY's opening on every other one, the levels taken then.
    #[test]
    fn what_follows_the_synth_moves_thirty_times_a_second() {
        let (mut e, _, _) = editing();
        let t0 = Instant::now();
        e.move_opening(t0, QualityMode::Hi);
        let mut moved = 0;
        for k in 1..=60u64 {
            e.meters.publish_levels(&[0.25; LEVELS], 0.25);
            let was = e.motion_at;
            e.move_opening(t0 + Duration::from_micros(k * 16_667), QualityMode::Hi);
            moved += u32::from(e.motion_at != was);
        }
        assert!((28..=31).contains(&moved), "{moved}");
        assert!(e.levels[0] > 0.5 && e.motion.level > 0.0, "{:?}", e.levels);
    }

    /// An editor watches the levels while it is open: the audio thread measures them only
    /// then.
    #[test]
    fn the_levels_are_watched_only_while_an_editor_is_open() {
        let meters = Arc::new(Meters::default());
        assert!(!meters.watched());
        let host = Arc::new(Host::default());
        let context: Arc<dyn GuiContext> = host.clone();
        let dir = tempfile::tempdir().unwrap();
        let e = Editing::new(
            context,
            Arc::new(Ca72Params::default()),
            Arc::clone(&meters),
            (1720, 1720, 1.0),
            Library::at(dir.path()),
            |_| None,
        );
        assert!(meters.watched());
        drop(e);
        assert!(!meters.watched());
    }

    /// The window's frame put together again in the places that changed is the frame put
    /// together whole: through the voices sounding (the strip's drops), the wheels taken from a
    /// keyboard (the strip's left, the panel's layers), the OVERLOAD lamp (the panel above) and
    /// QUALITY's lamp lit by the synth's level at each tick of its clock (the panel's top
    /// right); and the window's pixels turned in those places alone are its pixels turned whole.
    #[test]
    fn the_frame_put_together_in_places_is_the_frame_put_together_whole() {
        let (mut e, _, _) = editing();
        assert!(e.draw());
        let size = |f: &ca72_panel::Pixmap| (f.width() as usize, f.height() as usize);
        let f = e.composed.as_ref().expect("a frame");
        let mut screen = vec![0u32; size(f).0 * size(f).1];
        window::fill(&mut screen, size(f), &[f]);
        let mut places = 0;
        for i in 0..30u8 {
            let lamp = if i % 8 < 4 { 0.0 } else { 1.0 };
            let wheels = (0.5 + 0.05 * f32::from(i % 5), 0.1 * f32::from(i % 3));
            e.meters.publish(lamp, wheels, u32::from(i % 6 < 3));
            let level = 0.2 + 0.1 * f32::from(i % 7);
            e.meters.publish_levels(&[level; LEVELS], level);
            e.motion_due = None;
            if !e.draw() {
                continue;
            }
            let mut whole = None;
            compose(
                &mut whole,
                &e.renderer,
                e.strip.frame(),
                lower_of(&e),
                None,
                None,
                true,
                (e.physical_width, e.physical_height()),
            );
            assert!(e.composed == whole, "frame {i}: not as put together whole");
            let f = e.composed.as_ref().expect("a frame");
            let mut all = vec![0u32; screen.len()];
            window::fill(&mut all, size(f), &[f]);
            let d = e.damage.as_deref().expect("where it changed");
            places += d.len();
            window::fill_places(&mut screen, size(f), &[f], d);
            assert!(screen == all, "frame {i}: not as turned whole");
        }
        assert!(places > 30, "{places} places changed");
    }

    /// The drawer's slide ends where it was going in the frame shown, though its last moving
    /// frame stops short of there: the strip's rows put together again under it afterwards
    /// (the voices' drops) leave it whole. (In REAPER on Windows, 2026-10-10, the list's rows
    /// over the voices' display had come out a few pixels lower than the rest.)
    #[test]
    fn the_drawer_ends_its_slide_in_the_frame_shown() {
        let dir = tempfile::tempdir().unwrap();
        let (mut e, _, _) = editing_with(Library::at(dir.path()));
        e.draw();
        let p = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Name)));
        click(&mut e, p);
        // A moment of the slide short of its end, then its end.
        e.browser
            .slide_began(Instant::now() - crate::presets::SLIDE.mul_f64(0.5));
        e.draw();
        assert!(e.reveal_shown > 0.0 && e.reveal_shown < 0.99);
        e.browser
            .slide_began(Instant::now() - crate::presets::SLIDE * 2);
        e.draw();
        e.meters.publish(0.0, (0.5, 0.0), 1);
        e.draw();
        let mut whole = None;
        compose(
            &mut whole,
            &e.renderer,
            e.strip.frame(),
            None,
            Some((e.drawer.frame(), DRAWER_TOP)),
            None,
            true,
            (e.physical_width, e.physical_height()),
        );
        assert!(e.composed == whole, "the drawer where its slide ended");
    }

    /// The drawer dropped from the rail as the editor lays it.
    fn lower_of(e: &Editing) -> Option<LowerFrame<'_>> {
        e.lower_top().map(|top| {
            lower_frame(
                e.lower.which,
                top,
                &e.arp,
                e.fx.as_ref(),
                e.renderer.scale(),
            )
        })
    }

    /// The arpeggiator's drawer opened by the rail's ARP key and slid all the way down.
    fn arp_open(e: &mut Editing) {
        e.draw();
        let p = on_strip(e, strip_at(StripTarget::Bar(BarTarget::Arp)));
        click(e, p);
        assert!(e.lower.open);
        e.lower.slide = Some((Instant::now() - crate::presets::SLIDE * 2, true));
        e.draw();
        e.draw();
        assert!(e.lower.settled());
    }

    /// The rail's ARP key drops the arpeggiator's drawer over the lower page, inside the window
    /// (its size kept), and puts it away; it is lit while the arpeggiator is on. Open, the
    /// drawer is what the pointer finds there: the strip's tabs and knobs and the keyboard's
    /// buttons under it do nothing; below it, the page answers as before. The frame shown is its
    /// frame put together whole, the drawer's shadow over the page; and the presets' drawer, opened,
    /// is over it.
    #[test]
    fn the_arp_key_drops_its_drawer_over_the_lower_page() {
        let (mut e, host, _) = editing_standalone();
        e.playing.visible = false;
        let size = (e.width, e.window_height());
        arp_open(&mut e);
        assert_eq!((e.width, e.window_height()), size);
        let mut whole = None;
        compose(
            &mut whole,
            &e.renderer,
            e.strip.frame(),
            lower_of(&e),
            None,
            None,
            true,
            (e.physical_width, e.physical_height()),
        );
        assert!(e.composed == whole, "the drawer as put together whole");
        // Its plate in the frame, row for row.
        let c = e.composed.as_ref().expect("a frame");
        let s = e.renderer.scale();
        let (x, y) = ((2000.0 * s) as usize, (arp_ui::TOP + 200.0) * s);
        let y = y.round() as usize;
        let row = arp_ui::top_px(s) as usize;
        let px = |p: &ca72_panel::Pixmap, x: usize, y: usize| {
            let i = (y * p.width() as usize + x) * 4;
            p.data()[i..i + 4].to_vec()
        };
        assert_eq!(px(c, x, y), px(e.arp.frame(), x, y - row));
        // The strip's controls under it do nothing: AUTO GAIN's tab, VOICES's knob's top.
        for (x, y) in [
            strip::Bank::Auto.tab(0),
            (centre("voices").0, arp_ui::TOP + arp_ui::DEPTH - 20.0),
        ] {
            assert_eq!((arp_ui::hit(x, y), interact::hit_arp(x, y)), (None, None));
            let p = at(&e, (x, y));
            go(&mut e, p);
            press(&mut e);
            go(&mut e, below(p, -40.0));
            release(&mut e);
        }
        assert!(host.take().is_empty(), "nothing under the drawer");
        // The keyboard's header under it, the keyboard shown.
        e.strip_press(StripTarget::Bar(BarTarget::Keyboard));
        e.draw();
        let p = at(&e, (keyboard::TYPING_X, keyboard::HEADER));
        click(&mut e, p);
        assert!(!e.playing.typing, "TYPING under the drawer");
        // A key below it plays.
        let p = at(&e, (1000.0, art::H - 60.0));
        go(&mut e, p);
        press(&mut e);
        assert!(e.playing.dragging(), "a key below the drawer");
        release(&mut e);
        // The ARP key lit while the arpeggiator is on.
        e.update_scene();
        assert!(!e.browser.bar.arp);
        // The presets' drawer over it; ARP again puts the presets away and shows it.
        e.strip_press(StripTarget::Bar(BarTarget::Name));
        assert!(e.browser.open && e.lower.open);
        e.strip_press(StripTarget::Bar(BarTarget::Arp));
        assert!(!e.browser.open && e.lower.open);
        e.strip_press(StripTarget::Bar(BarTarget::Arp));
        assert!(!e.lower.open);
        e.lower.slide = None;
        e.draw();
        assert_eq!(e.lower_top(), None);
        // PANEL hiding the lower panel puts it away at once.
        arp_open(&mut e);
        e.toggle_panel();
        assert!(!e.lower.open && e.lower_top().is_none());
    }

    /// The drawer's tabs and knobs operate the arpeggiator's parameters, each a gesture: ON and
    /// LATCH switched, DIRECTION's choice set; RATE and OCTAVES dragged from position to
    /// position, never between; GATE turned; a readout double-clicked takes a value typed (RATE
    /// a note value, 8t a triplet, 16. dotted; OCTAVES a digit; GATE percent).
    #[test]
    fn the_arp_drawer_operates_its_parameters() {
        let (mut e, host, params) = editing();
        arp_open(&mut e);
        host.take();
        let tab = |e: &mut Editing, b: ArpBank, i: usize| {
            let p = at(e, arp_ui::centre(ArpTarget::Tab(b, i)));
            click(e, p);
        };
        let (on, latch, dir) = (
            params.arp_on.as_ptr(),
            params.arp_latch.as_ptr(),
            params.arp_direction.as_ptr(),
        );
        tab(&mut e, ArpBank::Arp, 0);
        assert_eq!(
            host.take(),
            vec![Call::Begin(on), Call::Set(on, 1.0), Call::End(on)]
        );
        tab(&mut e, ArpBank::Arp, 1);
        assert_eq!(
            host.take(),
            vec![Call::Begin(latch), Call::Set(latch, 1.0), Call::End(latch)]
        );
        tab(&mut e, ArpBank::Direction, 3);
        assert_eq!(
            host.take(),
            vec![Call::Begin(dir), Call::Set(dir, 1.0), Call::End(dir)]
        );
        tab(&mut e, ArpBank::Direction, 0);
        assert!(host.take().is_empty(), "UP already");
        // RATE dragged: positions only.
        let rate = params.arp_rate.as_ptr();
        let p = at(&e, centre("arp_rate"));
        go(&mut e, p);
        press(&mut e);
        for dy in 1..60 {
            go(&mut e, below(p, -f64::from(dy)));
        }
        release(&mut e);
        let calls = host.take();
        well_formed(&calls);
        assert!(calls.len() > 3);
        for c in &calls {
            if let Call::Set(q, v) = *c {
                assert_eq!(q, rate);
                assert!(
                    (v * 12.0 - (v * 12.0).round()).abs() < 1e-5,
                    "between at {v}"
                );
            }
        }
        e.update_scene();
        assert!(
            e.scene
                .tip
                .as_ref()
                .is_some_and(|t| t.0.starts_with("RATE: "))
        );
        // Typed.
        let double = |e: &mut Editing, i: usize| {
            let p = at(e, arp_ui::centre(ArpTarget::Readout(i)));
            click(e, p);
            click(e, p);
        };
        let typed = |e: &mut Editing, text: &str| {
            for c in text.chars() {
                keyed(e, Key::Character(c.to_string()));
            }
            keyed(e, Key::Enter);
        };
        for (i, text, ptr, want) in [
            (
                0,
                "8t",
                rate,
                params.arp_rate.preview_normalized(ArpRate::EighthTriplet),
            ),
            (
                0,
                "16.",
                rate,
                params.arp_rate.preview_normalized(ArpRate::SixteenthDotted),
            ),
            (
                1,
                "3",
                params.arp_octaves.as_ptr(),
                params.arp_octaves.preview_normalized(3),
            ),
            (
                2,
                "25",
                params.arp_gate.as_ptr(),
                params.arp_gate.preview_normalized(25.0),
            ),
        ] {
            double(&mut e, i);
            assert!(e.typing.is_some(), "{text}");
            e.draw();
            assert_eq!(e.arp_scene.readouts[i], ("_".to_owned(), true));
            typed(&mut e, text);
            assert_eq!(
                host.take(),
                vec![Call::Begin(ptr), Call::Set(ptr, want), Call::End(ptr)],
                "{text}"
            );
        }
        // Not a note value: nothing.
        double(&mut e, 0);
        typed(&mut e, "5");
        assert!(host.take().is_empty());
        // A knob turned: the frame shown is the frame put together whole.
        let p = at(&e, centre("arp_gate"));
        go(&mut e, p);
        press(&mut e);
        go(&mut e, below(p, -30.0));
        assert!(e.draw());
        let mut whole = None;
        compose(
            &mut whole,
            &e.renderer,
            e.strip.frame(),
            lower_of(&e),
            None,
            None,
            true,
            (e.physical_width, e.physical_height()),
        );
        assert!(e.composed == whole, "a knob turned in the drawer");
        release(&mut e);
    }

    /// The standalone's editor with a chain for its effects' drawer (`fx_view::fake`): two
    /// effects in it, three to be found.
    fn editing_effects() -> (Editing, Arc<crate::fx_view::fake::Fake>) {
        let (mut e, _, _) = editing_standalone();
        let fake = crate::fx_view::fake::Fake::with(
            &["Chorus", "Hall Reverb"],
            &["Delay", "Plate", "Flanger"],
        );
        e.use_effects(FxView::new(Some(fake.clone())));
        assert!(
            e.playing.visible,
            "the standalone's keyboard shown, under the drawer"
        );
        (e, fake)
    }

    /// What the chain was asked to do, once the drawer's threads have done it (`n` of them).
    fn effects_done(e: &mut Editing, fake: &crate::fx_view::fake::Fake, n: usize) -> Vec<String> {
        let t0 = Instant::now();
        while fake.done.lock().map_or(0, |d| d.len()) < n && t0.elapsed() < Duration::from_secs(5) {
            std::thread::sleep(Duration::from_millis(2));
        }
        // (The chain read again at once: something done there has finished.)
        std::thread::sleep(Duration::from_millis(5));
        e.update_scene();
        fake.take()
    }

    /// The drawer dropped from the rail opened by `key` and slid all the way down.
    fn lower_open(e: &mut Editing, rail: Rail, key: BarTarget) {
        e.draw();
        let p = at(e, strip::centre(rail, StripTarget::Bar(key)));
        click(e, p);
        e.lower.slide = Some((Instant::now() - crate::presets::SLIDE * 2, true));
        e.draw();
        e.draw();
        assert!(e.lower.settled());
    }

    /// The standalone's rail has the FX key after ARP, the name's display a key shorter for it;
    /// the plug-ins' has neither, and no effects' drawer. The FX key drops the effects' drawer
    /// over the lower page inside the window (its size kept), and puts it away; ARP's drawer and
    /// FX's are never open together, each key putting the other's away for its own. The key is lit
    /// while an effect plays; the frame shown is its frame put together whole.
    #[test]
    fn the_fx_key_drops_its_drawer_one_drawer_at_a_time() {
        let (plugin, _, _) = editing();
        assert_eq!((plugin.rail, plugin.fx.is_none()), (Rail::Plugin, true));
        let (standalone, _, _) = editing_standalone();
        assert_eq!(standalone.rail, Rail::Plugin, "no chain: no FX key");
        let (fx_key, arp_key) = (
            strip::centre(Rail::Effects, StripTarget::Bar(BarTarget::Fx)),
            strip::centre(Rail::Effects, StripTarget::Bar(BarTarget::Arp)),
        );
        assert_eq!(
            strip::hit(Rail::Plugin, fx_key.0, fx_key.1),
            Some(StripTarget::Bar(BarTarget::Arp)),
            "the plug-ins' ARP where the standalone's FX is"
        );
        let (mut e, fake) = editing_effects();
        assert_eq!(e.rail, Rail::Effects);
        assert_eq!(
            strip::hit(e.rail, fx_key.0, fx_key.1),
            Some(StripTarget::Bar(BarTarget::Fx))
        );
        assert_eq!(
            strip::hit(e.rail, arp_key.0, arp_key.1),
            Some(StripTarget::Bar(BarTarget::Arp))
        );
        let size = (e.width, e.window_height());
        lower_open(&mut e, Rail::Effects, BarTarget::Fx);
        assert!(e.lower.is_open(Lower::Fx));
        assert_eq!((e.width, e.window_height()), size);
        let mut whole = None;
        compose(
            &mut whole,
            &e.renderer,
            e.strip.frame(),
            lower_of(&e),
            None,
            None,
            true,
            (e.physical_width, e.physical_height()),
        );
        assert!(e.composed == whole, "the drawer as put together whole");
        // ARP puts it away for its own; FX again puts that away for the effects'.
        e.strip_press(StripTarget::Bar(BarTarget::Arp));
        assert!(e.lower.is_open(Lower::Arp) && !e.lower.is_open(Lower::Fx));
        e.strip_press(StripTarget::Bar(BarTarget::Fx));
        assert!(e.lower.is_open(Lower::Fx) && !e.lower.is_open(Lower::Arp));
        e.strip_press(StripTarget::Bar(BarTarget::Fx));
        assert!(!e.lower.open);
        // The key lit while an effect plays.
        e.update_scene();
        assert!(e.browser.bar.fx);
        for s in fake.slots.lock().expect("slots").iter_mut() {
            s.bypass = true;
        }
        e.fx_view.poll(Instant::now() + Duration::from_secs(1));
        e.update_scene();
        assert!(!e.browser.bar.fx, "nothing plays");
        // PANEL hiding the lower panel puts it away at once.
        lower_open(&mut e, Rail::Effects, BarTarget::Fx);
        e.toggle_panel();
        assert!(!e.lower.open && e.lower_top().is_none());
    }

    /// The effects' drawer operates the chain: a name clicked opens its effect's window, a tab
    /// bypasses its effect, a name dragged onto another slot moves it there (shown carried over
    /// it meanwhile); "+ ADD EFFECT" drops the list of the effects found (and SCAN AGAIN), a row
    /// chosen adds its effect; a right click offers REPLACE… (the list again, to put another in
    /// its place) and REMOVE. Under the drawer the page does nothing; the frame shown with the list
    /// open is the frame put together whole.
    #[test]
    fn the_fx_drawer_operates_the_chain() {
        let (mut e, fake) = editing_effects();
        lower_open(&mut e, Rail::Effects, BarTarget::Fx);
        let name = |e: &Editing, i: usize| at(e, fx_ui::centre(fx_ui::FxTarget::Name(i)));
        // A name clicked: its effect's window.
        let p = name(&e, 0);
        click(&mut e, p);
        assert_eq!(effects_done(&mut e, &fake, 1), ["window 0"]);
        // A tab: its effect bypassed.
        let p = at(&e, fx_ui::centre(fx_ui::FxTarget::Tab(1)));
        click(&mut e, p);
        assert_eq!(effects_done(&mut e, &fake, 1), ["bypass 1 true"]);
        assert!(!e.fx_scene.slots[1].lit && e.fx_scene.slots[0].lit);
        // A name dragged onto another's.
        let (from, to) = (name(&e, 0), name(&e, 1));
        go(&mut e, from);
        press(&mut e);
        for k in 1..=10 {
            let t = f64::from(k) / 10.0;
            go(
                &mut e,
                Point {
                    x: from.x + (to.x - from.x) * t,
                    y: from.y,
                },
            );
        }
        e.update_scene();
        assert_eq!(e.fx_scene.drag, Some((0, Some(1))));
        release(&mut e);
        assert_eq!(effects_done(&mut e, &fake, 1), ["move 0 1"]);
        assert_eq!(e.fx_scene.slots[0].text, "Hall Reverb");
        // "+ ADD EFFECT": the list; a row chosen.
        assert_eq!(e.fx_scene.slots[2].text, "+ ADD EFFECT");
        let p = name(&e, 2);
        click(&mut e, p);
        e.update_scene();
        let list = e.fx_scene.list.clone().expect("the list");
        let names: Vec<&str> = list.rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["Delay", "Plate", "Flanger", "SCAN AGAIN"]);
        assert_eq!(list.title, "ADD TO SLOT 3");
        e.draw();
        let mut whole = None;
        compose(
            &mut whole,
            &e.renderer,
            e.strip.frame(),
            lower_of(&e),
            None,
            None,
            true,
            (e.physical_width, e.physical_height()),
        );
        assert!(e.composed == whole, "the list as put together whole");
        // (The wheel over it scrolls it; three effects and SCAN AGAIN all show, so it stays.)
        let p = at(&e, fx_ui::row_centre(&list, 1));
        go(&mut e, p);
        wheel(&mut e, -2.0);
        e.update_scene();
        assert_eq!(e.fx_scene.list.as_ref().map(|l| l.first), Some(0));
        click(&mut e, p);
        assert!(!e.fx_view.list_open());
        assert_eq!(effects_done(&mut e, &fake, 1), ["add Plate"]);
        assert_eq!(e.fx_scene.slots[2].text, "Plate");
        assert_eq!(e.fx_scene.slots[3].text, "+ ADD EFFECT");
        // A right click: REPLACE… and REMOVE.
        let p = name(&e, 0);
        right_click(&mut e, p);
        assert_eq!(
            menu_items(&e),
            [
                ("REPLACE\u{2026}".to_owned(), true),
                ("REMOVE".into(), true)
            ]
        );
        menu_item(&mut e, "REPLACE\u{2026}");
        e.update_scene();
        let list = e.fx_scene.list.clone().expect("the list");
        assert_eq!(list.title, "REPLACE SLOT 1");
        let p = at(&e, fx_ui::row_centre(&list, 2));
        click(&mut e, p);
        assert_eq!(effects_done(&mut e, &fake, 1), ["replace 0 Flanger"]);
        let p = name(&e, 1);
        right_click(&mut e, p);
        menu_item(&mut e, "REMOVE");
        assert_eq!(effects_done(&mut e, &fake, 1), ["remove 1"]);
        assert_eq!(e.fx_scene.slots[1].text, "Plate");
        // The list put away by a press elsewhere, which does nothing else.
        let p = name(&e, 2);
        click(&mut e, p);
        e.update_scene();
        assert!(e.fx_view.list_open());
        let p = at(&e, (200.0, 300.0));
        click(&mut e, p);
        assert!(!e.fx_view.list_open());
        // Under the drawer: AUTO GAIN's tab does nothing.
        let (x, y) = strip::Bank::Auto.tab(0);
        assert!(fx_ui::inside(x, y));
        let before = e.params.auto_gain.value();
        let p = at(&e, (x, y));
        click(&mut e, p);
        assert_eq!(
            e.params.auto_gain.value(),
            before,
            "nothing under the drawer"
        );
        assert!(fake.take().is_empty());
    }

    /// Every gesture well formed: a parameter's begin only while it has none open, its
    /// changes and end only inside one, and none left open.
    fn well_formed(calls: &[Call]) {
        let mut open: Vec<ParamPtr> = Vec::new();
        for (i, c) in calls.iter().enumerate() {
            match *c {
                Call::Begin(p) => {
                    assert!(
                        !open.contains(&p),
                        "a begin inside a gesture at {i}: {calls:?}"
                    );
                    open.push(p);
                }
                Call::Set(p, _) => {
                    assert!(
                        open.contains(&p),
                        "a change outside a gesture at {i}: {calls:?}"
                    )
                }
                Call::End(p) => {
                    let k = open.iter().position(|&q| q == p);
                    let k = k.unwrap_or_else(|| panic!("an end with no begin at {i}: {calls:?}"));
                    open.remove(k);
                }
            }
        }
        assert!(open.is_empty(), "left open: {calls:?}");
    }

    fn press(e: &mut Editing) {
        e.mouse(MouseEvent::ButtonPressed {
            button: MouseButton::Left,
            modifiers: Modifiers::empty(),
        });
    }

    fn release(e: &mut Editing) {
        e.mouse(MouseEvent::ButtonReleased {
            button: MouseButton::Left,
            modifiers: Modifiers::empty(),
        });
    }

    fn wheel(e: &mut Editing, lines: f32) {
        e.mouse(MouseEvent::WheelScrolled {
            delta: ScrollDelta::Lines { x: 0.0, y: lines },
            modifiers: Modifiers::empty(),
        });
    }

    fn below(p: Point, dy: f64) -> Point {
        Point {
            x: p.x,
            y: p.y + dy,
        }
    }

    /// Where a point of the drawing is in the window, in logical pixels.
    fn at(e: &Editing, (x, y): (f64, f64)) -> Point {
        let k = e.renderer.scale() / e.dpr;
        Point { x: x * k, y: y * k }
    }

    fn centre(param: &str) -> (f64, f64) {
        CONTROLS[ca72_panel::controls::index(param).expect("a control")].centre()
    }

    fn go(e: &mut Editing, p: Point) {
        e.mouse(MouseEvent::CursorMoved {
            position: p,
            modifiers: Modifiers::empty(),
        });
    }

    fn click(e: &mut Editing, p: Point) {
        go(e, p);
        e.mouse(MouseEvent::ButtonPressed {
            button: MouseButton::Left,
            modifiers: Modifiers::empty(),
        });
        e.mouse(MouseEvent::ButtonReleased {
            button: MouseButton::Left,
            modifiers: Modifiers::empty(),
        });
    }

    fn right_click(e: &mut Editing, p: Point) {
        go(e, p);
        e.mouse(MouseEvent::ButtonPressed {
            button: MouseButton::Right,
            modifiers: Modifiers::empty(),
        });
        e.mouse(MouseEvent::ButtonReleased {
            button: MouseButton::Right,
            modifiers: Modifiers::empty(),
        });
    }

    /// The open menu's items, each its label and whether it can be chosen.
    fn menu_items(e: &Editing) -> Vec<(String, bool)> {
        e.learning.menu().expect("a menu").items.clone()
    }

    /// The open menu's item of this label clicked.
    fn menu_item(e: &mut Editing, label: &str) {
        let m = e.learning.menu().expect("a menu").clone();
        let k = m
            .items
            .iter()
            .position(|(t, _)| t == label)
            .unwrap_or_else(|| panic!("no {label} in {:?}", m.items));
        let p = at(e, m.item_at(k));
        click(e, p);
    }

    /// A key as the window gives it to the editor.
    fn keyed(e: &mut Editing, k: Key) -> bool {
        e.keyed(&KeyboardEvent {
            state: KeyState::Down,
            key: k,
            ..KeyboardEvent::default()
        })
    }

    fn learnable(id: &str) -> usize {
        crate::learn::index(id).expect("learnable")
    }

    fn note_lines(e: &Editing) -> Vec<String> {
        e.scene
            .note
            .as_ref()
            .map(|n| n.lines.iter().map(|(t, _)| t.clone()).collect())
            .unwrap_or_default()
    }

    /// Every pixel the window is given is opaque, the frames' and the window's beyond them, so
    /// that a window with an alpha channel (baseview's on X11) shows the panel and nothing
    /// through it; the colour is the frame's.
    #[test]
    fn the_window_is_given_opaque_pixels() {
        assert_eq!(window::shown(&[0x12, 0x34, 0x56, 0xff]), 0xff12_3456);
        assert_eq!(window::shown(&[0, 0, 0, 0xff]) >> 24, 0xff);
        assert_eq!(window::EMPTY >> 24, 0xff);
    }

    /// A control's menu (a right click): MIDI LEARN rings it and says it waits, Escape cancels it,
    /// a controller caught is assigned and said; REMOVE MIDI ASSIGNMENT removes it. None of it
    /// is a gesture of a parameter's (the sound is left as it is), and a press outside the menu
    /// only closes it.
    #[test]
    fn a_controls_menu_learns_cancels_and_removes_its_controller() {
        let (mut e, host, params) = editing();
        let map = Arc::clone(&params.midi_map);
        let cutoff = learnable("cutoff");
        let knob = ca72_panel::controls::index("cutoff").expect("a control");
        let p = at(&e, centre("cutoff"));
        right_click(&mut e, p);
        e.draw();
        let m = e.scene.menu.clone().expect("a menu drawn");
        assert_eq!(m.title, "CUTOFF FREQUENCY · NO MIDI CONTROLLER");
        assert_eq!(
            m.items,
            [
                ("MIDI LEARN".to_owned(), true),
                ("REMOVE MIDI ASSIGNMENT".to_owned(), false),
                ("MIDI ASSIGNMENTS…".to_owned(), true),
            ]
        );
        menu_item(&mut e, "MIDI LEARN");
        assert_eq!(map.armed(), Some(cutoff));
        e.draw();
        assert_eq!((e.scene.menu.clone(), e.scene.learning), (None, Some(knob)));
        assert_eq!(note_lines(&e)[0], "MIDI LEARN: CUTOFF FREQUENCY");
        assert_eq!(e.scene.tip, None);
        assert!(e.learning.holds_keys(&map));
        // Escape: cancelled.
        assert!(keyed(&mut e, Key::Escape));
        assert_eq!(map.armed(), None);
        e.draw();
        assert_eq!(e.scene.learning, None);
        // Learned again, and a controller caught by the audio thread: assigned and said.
        let p = at(&e, centre("cutoff"));
        right_click(&mut e, p);
        menu_item(&mut e, "MIDI LEARN");
        assert_eq!(map.incoming(0, 74), crate::learn::Incoming::Caught);
        e.draw();
        assert_eq!(
            map.assignment(cutoff)
                .map(crate::learn::Cc::text)
                .as_deref(),
            Some("CH 1 · CC 74")
        );
        assert_eq!(note_lines(&e), ["CUTOFF FREQUENCY: CH 1 · CC 74"]);
        assert_eq!(e.scene.learning, None);
        // Its menu now names it, and removes it.
        let p = at(&e, centre("cutoff"));
        right_click(&mut e, p);
        assert_eq!(
            e.learning.menu().map(|m| m.title.as_str()),
            Some("CUTOFF FREQUENCY · CH 1 · CC 74")
        );
        assert!(menu_items(&e)[1].1, "REMOVE can be chosen");
        menu_item(&mut e, "REMOVE MIDI ASSIGNMENT");
        assert_eq!(map.assignment(cutoff), None);
        // A press elsewhere with a menu open only closes it.
        let p = at(&e, centre("cutoff"));
        right_click(&mut e, p);
        let p = at(&e, centre("emphasis"));
        click(&mut e, p);
        assert!(!e.learning.menu_open());
        assert_eq!(params.cutoff.value(), 0.0);
        assert!(host.take().is_empty(), "no gesture");
    }

    /// Arming another control moves the learning there; a controller another control had moves
    /// to the one learned, and the note says from which.
    #[test]
    fn learning_another_control_moves_it_and_a_controller_reused_moves() {
        let (mut e, _host, params) = editing();
        let map = Arc::clone(&params.midi_map);
        map.assign(
            learnable("emphasis"),
            crate::learn::Cc { channel: 0, cc: 74 },
        );
        let p = at(&e, centre("cutoff"));
        right_click(&mut e, p);
        menu_item(&mut e, "MIDI LEARN");
        let p = at(&e, centre("glide"));
        right_click(&mut e, p);
        menu_item(&mut e, "MIDI LEARN");
        assert_eq!(map.armed(), Some(learnable("glide")));
        map.incoming(0, 74);
        e.draw();
        assert_eq!(
            note_lines(&e),
            [
                "GLIDE: CH 1 · CC 74",
                "TAKEN FROM EMPHASIS, WHICH HAS NONE NOW"
            ]
        );
        assert_eq!(map.assignment(learnable("emphasis")), None);
        assert_eq!(map.assignment(learnable("cutoff")), None);
    }

    /// Closing the editor ends learning: nothing assigned, unless the audio thread had already
    /// caught a controller for it (that message came, so it is the assignment).
    #[test]
    fn closing_the_editor_cancels_learning() {
        let (mut e, _host, params) = editing();
        let map = Arc::clone(&params.midi_map);
        let p = at(&e, centre("cutoff"));
        right_click(&mut e, p);
        menu_item(&mut e, "MIDI LEARN");
        drop(e);
        assert_eq!(map.armed(), None);
        assert!(map.assignments().iter().all(Option::is_none));
        let (mut e, _host, _) = editing();
        e.params = Arc::clone(&params);
        let p = at(&e, centre("cutoff"));
        right_click(&mut e, p);
        menu_item(&mut e, "MIDI LEARN");
        map.incoming(2, 20);
        drop(e);
        assert_eq!(map.armed(), None);
        assert_eq!(
            map.assignment(learnable("cutoff")),
            Some(crate::learn::Cc { channel: 2, cc: 20 })
        );
    }

    /// The strip's controls have the menu too (a tab's above it, the control ringed on the
    /// strip; a knob's as the panel's knobs'); a wheel and POWER say why they are not learned,
    /// MIDI LEARN not offered.
    #[test]
    fn the_strips_controls_learn_and_the_wheels_and_power_say_why_not() {
        let (mut e, host, params) = editing();
        let map = Arc::clone(&params.midi_map);
        let p = on_strip(&e, strip_at(StripTarget::Tab(Bank::Mode, 1)));
        right_click(&mut e, p);
        let m = e.learning.menu().expect("a menu").clone();
        assert_eq!(m.title, "POLY · NO MIDI CONTROLLER");
        let (_, h) = m.extent(e.renderer.fonts());
        assert!(m.y >= 0.0 && m.y + h <= art::H + 1e-6, "inside the window");
        menu_item(&mut e, "MIDI LEARN");
        e.draw();
        assert_eq!(e.strip_scene.learning, Some(strip::StripControl::Poly));
        assert_eq!(e.scene.learning, None);
        let p = at(&e, centre("spread"));
        right_click(&mut e, p);
        assert_eq!(
            e.learning.menu().map(|m| m.title.as_str()),
            Some("WIDTH · NO MIDI CONTROLLER")
        );
        let p = on_strip(&e, strip_at(StripTarget::Tab(Bank::Mode, 1)));
        right_click(&mut e, p);
        menu_item(&mut e, "CANCEL MIDI LEARN");
        assert_eq!(map.armed(), None);
        // INNER's knob (the strip's last): its menu where it was pressed, MIDI LEARN rings it as
        // the panel's knobs are rung, and its note says so over it.
        let inner = ca72_panel::controls::index("inner").expect("a control");
        let p = at(&e, centre("inner"));
        right_click(&mut e, p);
        let m = e.learning.menu().expect("a menu").clone();
        assert_eq!(m.title, "INNER · NO MIDI CONTROLLER");
        let (_, h) = m.extent(e.renderer.fonts());
        assert!((m.x - centre("inner").0).abs() < 1e-6, "at INNER's knob");
        assert!(m.y >= 0.0 && m.y + h <= art::H + 1e-6, "inside the window");
        menu_item(&mut e, "MIDI LEARN");
        assert_eq!(map.armed(), Some(learnable("inner")));
        e.draw();
        assert_eq!(
            (e.scene.learning, e.strip_scene.learning),
            (Some(inner), None)
        );
        assert_eq!(note_lines(&e)[0], "MIDI LEARN: INNER");
        let p = at(&e, centre("inner"));
        right_click(&mut e, p);
        menu_item(&mut e, "CANCEL MIDI LEARN");
        assert_eq!(map.armed(), None);
        for (param, title) in [
            ("pitch_wheel", "PITCH: MIDI PITCH BEND MOVES IT"),
            (
                "mod_wheel",
                "MODULATION: THE MODULATION WHEEL (CC 1) MOVES IT",
            ),
            ("quality", "QUALITY: SET PER PLUGIN INSTANCE"),
        ] {
            let p = at(&e, centre(param));
            right_click(&mut e, p);
            assert_eq!(e.learning.menu().map(|m| m.title.as_str()), Some(title));
            // (QUALITY's has its indicator's two first: `quality_s_menu_shows_or_hides_its_
            // indicator`.)
            let items = menu_items(&e);
            assert_eq!(
                items[items.len() - 2..],
                [
                    ("NOT LEARNED BY MIDI LEARN".to_owned(), false),
                    ("MIDI ASSIGNMENTS…".to_owned(), true),
                ]
            );
            assert_eq!(items.len(), if param == "quality" { 4 } else { 2 });
            // (What cannot be chosen leaves the menu open.)
            menu_item(&mut e, "NOT LEARNED BY MIDI LEARN");
            assert!(e.learning.menu_open());
        }
        let p = at(&e, (art::COL + POWER.0, art::TOP + POWER.1));
        right_click(&mut e, p);
        assert_eq!(
            e.learning.menu().map(|m| m.title.as_str()),
            Some("POWER: THE HOST'S BYPASS")
        );
        assert!(keyed(&mut e, Key::Escape));
        assert!(!e.learning.menu_open());
        assert!(host.take().is_empty(), "no gesture");
    }

    /// The drawer's MIDI list, from its MIDI button or a control's MIDI ASSIGNMENTS…: every
    /// learnable control and its controller, operated from the keyboard (the arrows choose,
    /// Enter learns or cancels, Delete removes, a letter finds a control, Escape cancels then
    /// closes) and by the pointer (a row's LEARN).
    #[test]
    fn the_drawers_midi_list_is_operated_from_the_keyboard() {
        let (mut e, host, params) = editing();
        let map = Arc::clone(&params.midi_map);
        open(&mut e);
        let p = in_drawer(&e, presets_ui::midi_centre());
        click(&mut e, p);
        assert!(e.learning.list);
        e.draw();
        let list = e.browser.drawer.midi.clone().expect("the list shown");
        assert_eq!(list.rows.len(), crate::learn::LEARNABLE.len());
        assert_eq!(list.rows[0].name, "TUNE");
        assert!(keyed(&mut e, Key::ArrowDown));
        assert!(keyed(&mut e, Key::ArrowDown));
        assert_eq!(e.learning.chosen(), learnable("glide"));
        assert!(keyed(&mut e, Key::Enter));
        assert_eq!(map.armed(), Some(learnable("glide")));
        e.draw();
        assert!(
            e.browser
                .drawer
                .midi
                .as_ref()
                .is_some_and(|l| l.rows[2].waiting)
        );
        assert!(keyed(&mut e, Key::Escape), "Escape cancels first");
        assert_eq!(map.armed(), None);
        assert!(e.browser.open);
        map.assign(learnable("glide"), crate::learn::Cc { channel: 0, cc: 5 });
        e.draw();
        assert_eq!(
            e.browser
                .drawer
                .midi
                .as_ref()
                .map(|l| l.rows[2].assignment.as_str()),
            Some("CH 1 · CC 5")
        );
        assert!(keyed(&mut e, Key::Delete));
        assert_eq!(map.assignment(learnable("glide")), None);
        assert!(keyed(&mut e, Key::Character("c".into())));
        assert_eq!(e.learning.chosen(), learnable("cutoff"));
        e.draw();
        let list = e.browser.drawer.midi.clone().expect("the list");
        assert!(list.first <= list.chosen && list.chosen < list.first + presets_ui::ROWS_SHOWN);
        // LEARN on a row, by the pointer.
        let p = presets_ui::midi_action_centre(&list, list.chosen, presets_ui::MidiAction::Learn)
            .expect("the chosen row's LEARN");
        let p = in_drawer(&e, p);
        click(&mut e, p);
        assert_eq!(map.armed(), Some(learnable("cutoff")));
        assert!(keyed(&mut e, Key::Escape));
        assert!(keyed(&mut e, Key::Escape), "then Escape closes");
        assert!(!e.browser.open);
        slid(&mut e);
        assert_eq!(e.browser.drawer.midi, None, "the presets next time");
        // From a control's menu: the list, that control chosen.
        let p = at(&e, centre("emphasis"));
        right_click(&mut e, p);
        menu_item(&mut e, "MIDI ASSIGNMENTS…");
        assert!(e.browser.open && e.learning.list);
        assert_eq!(e.learning.chosen(), learnable("emphasis"));
        assert!(host.take().is_empty(), "no gesture");
    }

    /// A right click mid-drag ends the drag's gesture before the menu opens.
    #[test]
    fn a_right_click_ends_a_drag_first() {
        let (mut e, host, _params) = editing();
        let p = at(&e, centre("cutoff"));
        go(&mut e, p);
        press(&mut e);
        go(&mut e, below(p, -40.0));
        right_click(&mut e, p);
        assert!(e.learning.menu_open());
        well_formed(&host.take());
    }

    #[test]
    fn a_drag_is_one_gesture_of_its_knob() {
        let (mut e, host, params) = editing();
        let p = at(&e, centre("cutoff"));
        go(&mut e, p);
        e.mouse(MouseEvent::ButtonPressed {
            button: MouseButton::Left,
            modifiers: Modifiers::empty(),
        });
        go(
            &mut e,
            Point {
                x: p.x,
                y: p.y - 2.0,
            },
        );
        go(
            &mut e,
            Point {
                x: p.x,
                y: p.y - 60.0,
            },
        );
        go(
            &mut e,
            Point {
                x: p.x,
                y: p.y - 300.0,
            },
        );
        e.mouse(MouseEvent::ButtonReleased {
            button: MouseButton::Left,
            modifiers: Modifiers::empty(),
        });
        let cutoff = params.cutoff.as_ptr();
        assert_eq!(
            host.take(),
            vec![
                Call::Begin(cutoff),
                Call::Set(cutoff, 0.75),
                Call::Set(cutoff, 1.0),
                Call::End(cutoff)
            ]
        );
        // A knob of the strip's, INNER's, the same (from its 0).
        let p = at(&e, centre("inner"));
        go(&mut e, p);
        press(&mut e);
        go(&mut e, below(p, -2.0));
        go(&mut e, below(p, -60.0));
        go(&mut e, below(p, -300.0));
        release(&mut e);
        let inner = params.inner.as_ptr();
        assert_eq!(
            host.take(),
            vec![
                Call::Begin(inner),
                Call::Set(inner, 0.25),
                Call::Set(inner, 1.0),
                Call::End(inner)
            ]
        );
    }

    #[test]
    fn clicks_flip_switches_pick_legends_and_turn_power() {
        let (mut e, host, params) = editing();
        {
            let p = at(&e, centre("osc1_on"));
            click(&mut e, p);
        }
        let on = params.osc1_on.as_ptr();
        assert_eq!(
            host.take(),
            vec![Call::Begin(on), Call::Set(on, 0.0), Call::End(on)]
        );

        let i = ca72_panel::controls::index("osc1_range").expect("a control");
        let Kind::Selector(set) = CONTROLS[i].kind else {
            panic!("a selector")
        };
        let (cx, cy) = CONTROLS[i].centre();
        let (lx, ly) = e.renderer.layout().legend_at(set, 0);
        {
            let p = at(&e, (cx + lx, cy + ly));
            click(&mut e, p);
        }
        let range = params.osc1_range.as_ptr();
        assert_eq!(
            host.take(),
            vec![Call::Begin(range), Call::Set(range, 0.0), Call::End(range)]
        );

        {
            let p = at(&e, (art::COL + POWER.0, art::TOP + POWER.1));
            click(&mut e, p);
        }
        let bypass = params.bypass.as_ptr();
        assert_eq!(
            host.take(),
            vec![
                Call::Begin(bypass),
                Call::Set(bypass, 1.0),
                Call::End(bypass)
            ]
        );
    }

    #[test]
    fn a_double_click_restores_the_default() {
        let (mut e, host, params) = editing();
        let p = at(&e, centre("osc2_frequency"));
        click(&mut e, p);
        host.take();
        click(&mut e, p);
        let f = params.osc2_frequency.as_ptr();
        assert_eq!(
            host.take(),
            vec![Call::Begin(f), Call::Set(f, 0.5), Call::End(f)]
        );
    }

    #[test]
    fn the_pitch_wheel_settles_in_its_detent() {
        let (mut e, host, params) = editing();
        let p = at(&e, centre("pitch_wheel"));
        go(&mut e, p);
        e.mouse(MouseEvent::ButtonPressed {
            button: MouseButton::Left,
            modifiers: Modifiers::empty(),
        });
        go(
            &mut e,
            Point {
                x: p.x,
                y: p.y - 5.0,
            },
        );
        e.mouse(MouseEvent::ButtonReleased {
            button: MouseButton::Left,
            modifiers: Modifiers::empty(),
        });
        let w = params.pitch_wheel.as_ptr();
        let calls = host.take();
        assert_eq!(calls.first(), Some(&Call::Begin(w)));
        assert_eq!(
            &calls[calls.len() - 2..],
            &[Call::Set(w, 0.5), Call::End(w)]
        );
    }

    #[test]
    fn the_mouse_wheel_turns_a_knob_until_it_rests() {
        let (mut e, host, params) = editing();
        {
            let p = at(&e, centre("emphasis"));
            go(&mut e, p);
        }
        for _ in 0..2 {
            e.mouse(MouseEvent::WheelScrolled {
                delta: ScrollDelta::Lines { x: 0.0, y: 1.0 },
                modifiers: Modifiers::empty(),
            });
        }
        let emphasis = params.emphasis.as_ptr();
        assert_eq!(
            host.take(),
            vec![
                Call::Begin(emphasis),
                Call::Set(emphasis, 0.02),
                Call::Set(emphasis, 0.04)
            ]
        );
        if let Some(s) = &mut e.scroll {
            s.last -= SCROLL_REST * 2;
        }
        e.draw();
        assert_eq!(host.take(), vec![Call::End(emphasis)]);
    }

    /// A wheel's gesture waits for the wheel to rest, and a press ends it before beginning
    /// its own, on that control or another: never a begin inside a gesture, which CLAP
    /// forbids (decisions.md R18).
    #[test]
    fn a_wheels_gesture_ends_before_another_begins() {
        let (mut e, host, params) = editing_poly();
        let cutoff = params.cutoff.as_ptr();
        let p = at(&e, centre("cutoff"));
        go(&mut e, p);
        wheel(&mut e, 1.0);
        // CUTOFF grabbed within the wheel's rest.
        press(&mut e);
        go(&mut e, below(p, -60.0));
        release(&mut e);
        let calls = host.take();
        well_formed(&calls);
        assert_eq!(
            (calls[0], calls[2], calls[3], calls[calls.len() - 1]),
            (
                Call::Begin(cutoff),
                Call::End(cutoff),
                Call::Begin(cutoff),
                Call::End(cutoff)
            ),
            "{calls:?}"
        );
        // A wheel on one knob, then a switch clicked; then VOICES stepped by the wheel.
        let p = at(&e, centre("emphasis"));
        go(&mut e, p);
        wheel(&mut e, 1.0);
        let p = at(&e, centre("osc1_on"));
        click(&mut e, p);
        let p = at(&e, centre("emphasis"));
        go(&mut e, p);
        wheel(&mut e, 1.0);
        let p = at(&e, centre("voices"));
        go(&mut e, p);
        wheel(&mut e, 1.0);
        e.end_gestures();
        let calls = host.take();
        well_formed(&calls);
        assert_eq!(
            calls.iter().filter(|c| matches!(c, Call::Begin(_))).count(),
            4
        );
    }

    /// The editor closed within a wheel's rest, or mid-drag: every gesture ends.
    #[test]
    fn closing_the_editor_ends_every_gesture() {
        let (mut e, host, params) = editing();
        let p = at(&e, centre("emphasis"));
        go(&mut e, p);
        wheel(&mut e, 1.0);
        drop(e);
        let calls = host.take();
        well_formed(&calls);
        assert_eq!(calls.last(), Some(&Call::End(params.emphasis.as_ptr())));

        let (mut e, host, _params) = editing();
        let p = at(&e, centre("cutoff"));
        go(&mut e, p);
        press(&mut e);
        go(&mut e, below(p, -60.0));
        drop(e);
        well_formed(&host.take());

        let (mut e, host, _params) = editing();
        let p = at(&e, centre("spread"));
        go(&mut e, p);
        press(&mut e);
        go(&mut e, below(p, -40.0));
        drop(e);
        well_formed(&host.take());
    }

    /// A drag whose release never came (on Windows its window lost the pointer's capture,
    /// at Alt-Tab) ends as the next press comes, and as the window loses the focus or the
    /// capture, picking nothing.
    #[test]
    fn a_drag_whose_release_was_lost_ends_at_the_next_press() {
        let (mut e, host, params) = editing();
        let (cutoff, emphasis) = (params.cutoff.as_ptr(), params.emphasis.as_ptr());
        let p = at(&e, centre("cutoff"));
        go(&mut e, p);
        press(&mut e);
        go(&mut e, below(p, -60.0));
        // No release: EMPHASIS pressed next.
        let p = at(&e, centre("emphasis"));
        go(&mut e, p);
        press(&mut e);
        release(&mut e);
        let calls = host.take();
        well_formed(&calls);
        let at_end = calls.iter().position(|c| *c == Call::End(cutoff));
        let at_begin = calls.iter().position(|c| *c == Call::Begin(emphasis));
        assert!(at_end < at_begin && at_end.is_some(), "{calls:?}");
        // RANGE pressed (a click would step it), cut short by the window: nothing picked.
        let range = params.osc1_range.as_ptr();
        let p = at(&e, centre("osc1_range"));
        go(&mut e, p);
        press(&mut e);
        assert!(e.dragging());
        e.end_gestures();
        assert!(!e.dragging());
        release(&mut e);
        assert_eq!(host.take(), vec![Call::Begin(range), Call::End(range)]);
    }

    /// The name plate does nothing (the owner, 2026-10-03: it no longer turns over): a click
    /// on it sets nothing, redraws nothing and shows no tip.
    #[test]
    fn the_name_plate_does_nothing() {
        let (mut e, host, _params) = editing();
        e.draw();
        let p = at(&e, (art::PLATE_X + 20.0, art::PLATE_Y + 20.0));
        click(&mut e, p);
        assert_eq!(host.take(), vec![]);
        assert!(!e.draw(), "redrawn");
        assert_eq!(e.scene.tip, None);
    }

    #[test]
    fn a_tip_names_the_control_and_its_value() {
        let (mut e, _host, _params) = editing();
        {
            let p = at(&e, centre("osc1_volume"));
            go(&mut e, p);
        }
        e.update_scene();
        assert_eq!(
            e.scene.tip.as_ref().map(|t| t.0.as_str()),
            Some("OSCILLATOR-1 VOLUME: 8.00")
        );
        // A strip knob's, INNER's, in its unit: turned from 0 to 40 %.
        let p = at(&e, centre("inner"));
        go(&mut e, p);
        press(&mut e);
        go(&mut e, below(p, -96.0));
        e.update_scene();
        assert_eq!(
            e.scene.tip.as_ref().map(|t| t.0.as_str()),
            Some("INNER: 40 %")
        );
        release(&mut e);
    }

    #[test]
    fn every_control_operates_a_parameter() {
        let p = Ca72Params::default();
        for c in CONTROLS.iter() {
            assert!(param(&p, c.param).is_some(), "{}", c.param);
        }
    }

    #[test]
    fn the_editor_keeps_the_panels_proportions() {
        // The panel's 512 and the strip's 470 under it (A6, the name board gone: the drawing
        // 3108 by 1775).
        assert_eq!(height_for(1720), 982);
        assert_eq!(clamp_width(10), MIN_WIDTH);
        assert_eq!(clamp_width(99_999), MAX_WIDTH);
    }

    #[test]
    fn it_opens_at_four_fifths_of_the_screen_until_resized() {
        // A 14-inch MacBook Pro's screen less the menu bar; a 34-inch ultrawide's.
        assert_eq!(fitted_width(Some((1512.0, 949.0))), 1210);
        // The height's share sets it on a wide screen (the panel and the strip under it).
        assert_eq!(fitted_width(Some((3440.0, 1400.0))), 1961);
        assert_eq!(fitted_width(Some((3840.0, 800.0))), 1121);
        assert_eq!(height_for(1121), 640);
        assert_eq!(fitted_width(None), DEFAULT_WIDTH);
        assert_eq!(
            Ca72Params::default().editor_width.load(Ordering::Relaxed),
            0
        );
    }

    /// X11's screen spans every monitor: the first size comes from the monitor the pointer
    /// is on (two 2560 by 1440 side by side made the editor 3435 wide on one, its grip off
    /// it), less the panels the work area leaves out.
    #[test]
    fn on_x11_it_opens_to_fit_the_monitor_it_is_on() {
        use screen::{Rect, monitor_area};
        let r = |x, y, w, h| Rect { x, y, w, h };
        let whole = r(0, 0, 5120, 1440);
        let (left, right) = (r(0, 0, 2560, 1440), r(2560, 0, 2560, 1440));
        let monitors = [(left, true), (right, false)];
        assert_eq!(fitted_width(Some((5120.0, 1440.0))), 2017);
        assert_eq!(
            monitor_area(whole, &monitors, Some((3000, 700)), None),
            right
        );
        assert_eq!(fitted_width(Some((2560.0, 1440.0))), 2017);
        // The pointer elsewhere: the primary; no RandR: the whole screen, as before.
        assert_eq!(monitor_area(whole, &monitors, None, None), left);
        assert_eq!(monitor_area(whole, &[], Some((3000, 700)), None), whole);
        // A panel along the screen's top; a work area that misses the monitor is ignored.
        let work = Some(r(0, 32, 5120, 1408));
        assert_eq!(
            monitor_area(whole, &monitors, Some((3000, 700)), work),
            r(2560, 32, 2560, 1408)
        );
        assert_eq!(
            monitor_area(whole, &monitors, Some((3000, 700)), Some(left)),
            right
        );
    }

    /// On Windows and Linux the window takes the host's scale or none, never the system's:
    /// the host frames it at nih-plug's size times its own scale (1 when it gave none).
    #[test]
    fn the_window_scales_as_the_host_says() {
        if cfg!(target_os = "macos") {
            assert_eq!(scale_policy(None), WindowScalePolicy::SystemScaleFactor);
        } else {
            assert_eq!(scale_policy(None), WindowScalePolicy::ScaleFactor(1.0));
            assert_eq!(scale_policy(Some(1.5)), WindowScalePolicy::ScaleFactor(1.5));
        }
    }

    /// The window is the drawing's proportions, its width as saved, the drawer open or shut
    /// (it drops down over the strip: A6).
    #[test]
    fn the_editor_opens_at_the_drawings_proportions() {
        let params = Arc::new(Ca72Params::default());
        params.editor_width.store(1720, Ordering::Relaxed);
        let editor = Ca72Editor::new(Arc::clone(&params), Arc::new(Meters::default()));
        let size = (1720, height_for(1720));
        assert_eq!(editor.size(), size);
        assert_eq!(editor.opening(), size);
        assert_eq!(size.1, (1720.0 * art::H / art::W).round() as u32);
    }

    /// A point of the strip (the drawing's units) in the window.
    fn on_strip(e: &Editing, p: (f64, f64)) -> Point {
        at(e, p)
    }

    /// Where a strip target is, found where the strip's hit test finds it.
    fn strip_at(target: StripTarget) -> (f64, f64) {
        let p = strip::centre(Rail::Plugin, target);
        assert_eq!(
            strip::hit(Rail::Plugin, p.0, p.1),
            Some(target),
            "{target:?} not found"
        );
        p
    }

    /// The strip's tabs and switches (A6): MODE's MONO, POLY and UNISON set POLY and UNISON;
    /// SCATTER | DOUBLE turns DETUNE off and back on at its amount (half way if it never was);
    /// the placement's EVEN and CENTER are its parameter's two choices; AUTO GAIN's tab toggles
    /// it; ENTROPY's, WIDTH's and INNER's readouts turn their amounts off and on; VOICES is a
    /// knob, a drag one gesture. Each click a gesture of its own for each parameter it changes,
    /// none for one already so.
    #[test]
    fn the_strip_operates_its_parameters() {
        let (mut e, host, params) = editing();
        let tab = |e: &mut Editing, b: Bank, i: usize| {
            let p = on_strip(e, strip_at(StripTarget::Tab(b, i)));
            click(e, p);
        };
        let (poly, unison) = (params.poly.as_ptr(), params.unison.as_ptr());
        tab(&mut e, Bank::Mode, 1);
        assert_eq!(
            host.take(),
            vec![Call::Begin(poly), Call::Set(poly, 1.0), Call::End(poly)]
        );
        tab(&mut e, Bank::Mode, 2);
        assert_eq!(
            host.take(),
            vec![
                Call::Begin(unison),
                Call::Set(unison, 1.0),
                Call::End(unison)
            ]
        );
        // (The test host applies nothing: POLY and UNISON are still off, so MONO is already.)
        tab(&mut e, Bank::Mode, 0);
        assert_eq!(host.take(), vec![], "MONO already");
        let doubled = params.doubled.as_ptr();
        tab(&mut e, Bank::Stereo, 1);
        assert_eq!(
            host.take(),
            vec![
                Call::Begin(doubled),
                Call::Set(doubled, 1.0),
                Call::End(doubled)
            ]
        );
        tab(&mut e, Bank::Stereo, 0);
        assert_eq!(
            host.take(),
            vec![],
            "SCATTER already (DOUBLE still off here)"
        );
        let placement = params.placement.as_ptr();
        tab(&mut e, Bank::Placement, 0);
        assert_eq!(host.take(), vec![], "EVEN already");
        tab(&mut e, Bank::Placement, 1);
        assert_eq!(
            host.take(),
            vec![
                Call::Begin(placement),
                Call::Set(placement, 1.0),
                Call::End(placement)
            ]
        );
        assert_eq!(
            params.placement.preview_plain(1.0),
            Scatter::Centre,
            "the second tab CENTER"
        );
        let auto = params.auto_gain.as_ptr();
        tab(&mut e, Bank::Auto, 0);
        assert_eq!(
            host.take(),
            vec![Call::Begin(auto), Call::Set(auto, 0.0), Call::End(auto)]
        );
        // ENTROPY starts at the old minimum, now 5 %: its first click turns it off.
        // WIDTH and INNER start off: their first click recalls half way (they never were).
        for (r, p, to) in [
            (Readout::Entropy, params.entropy.as_ptr(), 0.0),
            (Readout::Width, params.spread.as_ptr(), 0.5),
            (Readout::Inner, params.inner.as_ptr(), 0.5),
        ] {
            let at = on_strip(&e, strip_at(StripTarget::Switch(r)));
            click(&mut e, at);
            assert_eq!(
                host.take(),
                vec![Call::Begin(p), Call::Set(p, to), Call::End(p)]
            );
        }
        // VOICES's knob dragged up: nothing at these defaults, where it does nothing (MONO,
        // SCATTER, WIDTH 0: R64); with POLY on, one gesture of VOICES.
        let drag_voices = |e: &mut Editing| {
            let p = at(e, centre("voices"));
            go(e, p);
            press(e);
            go(e, below(p, -120.0));
            release(e);
        };
        drag_voices(&mut e);
        assert_eq!(host.take(), vec![], "VOICES dragged where it does nothing");
        let (mut poly_e, poly_host, poly_params) = editing_poly();
        let voices = poly_params.voices.as_ptr();
        drag_voices(&mut poly_e);
        let calls = poly_host.take();
        well_formed(&calls);
        assert_eq!(calls.first(), Some(&Call::Begin(voices)));
        assert_eq!(calls.last(), Some(&Call::End(voices)));
        // The strip draws what the parameters hold.
        e.update_scene();
        assert!(e.draw());
    }

    /// The strip's readouts that are switches turn their amounts off, keeping each, and back on
    /// at it: WIDTH's and INNER's each their own (INNER's switch the same as WIDTH's).
    #[test]
    fn the_strips_switches_turn_their_amounts_off_and_back_on_at_their_last() {
        let amount = |name: &str, percent: f32| {
            FloatParam::new(
                name,
                percent,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
        };
        let (mut e, host, _) = editing();
        e.params = Arc::new(Ca72Params {
            spread: amount("Width", 70.0),
            inner: amount("Inner", 40.0),
            ..Ca72Params::default()
        });
        let switch = |e: &mut Editing, r: Readout| {
            let p = on_strip(e, strip_at(StripTarget::Switch(r)));
            click(e, p);
        };
        // Off, each kept.
        for (r, p) in [
            (Readout::Width, e.params.spread.as_ptr()),
            (Readout::Inner, e.params.inner.as_ptr()),
        ] {
            switch(&mut e, r);
            assert_eq!(
                host.take(),
                vec![Call::Begin(p), Call::Set(p, 0.0), Call::End(p)]
            );
        }
        // (The test host applies nothing: the parameters as the host would hold them now.)
        e.params = Arc::new(Ca72Params::default());
        // Back on, each at its own.
        for (r, p, was) in [
            (Readout::Inner, e.params.inner.as_ptr(), 0.4),
            (Readout::Width, e.params.spread.as_ptr(), 0.7),
        ] {
            switch(&mut e, r);
            assert_eq!(
                host.take(),
                vec![Call::Begin(p), Call::Set(p, was), Call::End(p)]
            );
        }
    }

    /// The strip lights the placement's tab by the plug-in's choice, EVEN the first and CENTER
    /// the second (not by the kit's index, which counts its EDGES), in MONO too, whose one
    /// voice scatters by it (R62); none in MONO with DOUBLE, whose pair has no place
    /// among others.
    #[test]
    fn the_strip_lights_the_placements_tab() {
        let lit = |poly: bool, doubled: bool, placement: Scatter| {
            let p = Ca72Params {
                poly: BoolParam::new("Poly", poly),
                doubled: BoolParam::new("Double", doubled),
                placement: EnumParam::new("Scatter Placement", placement),
                ..Ca72Params::default()
            };
            strip_scene(&p, 0, (&[], 0.0), StripScene::default()).placement
        };
        assert_eq!(lit(true, false, Scatter::Even), Some(0));
        assert_eq!(lit(true, false, Scatter::Centre), Some(1));
        assert_eq!(lit(false, false, Scatter::Centre), Some(1));
        assert_eq!(lit(false, true, Scatter::Centre), None);
        assert_eq!(Bank::Placement.words(), ["EVEN", "CENTER"]);
    }

    /// In MONO without DOUBLE the strip shows VOICES's places as the engine's gains have them,
    /// the one voice lit at the place its last attack took (the engine's bit for it), at the
    /// output's level, with VOICES, WIDTH and INNER lit (R62); with DOUBLE, the one pair,
    /// and VOICES not lit.
    #[test]
    fn mono_shows_its_places_and_the_one_it_sounds_at() {
        let amount = |name: &str, percent: f32| {
            FloatParam::new(
                name,
                percent,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
        };
        let scene = |doubled: bool| {
            let p = Ca72Params {
                spread: amount("Width", 100.0),
                inner: amount("Inner", 40.0),
                doubled: BoolParam::new("Double", doubled),
                ..Ca72Params::default()
            };
            strip_scene(&p, 1 << 2, (&[], 0.5), StripScene::default())
        };
        let s = scene(false);
        let Field::Scatter(places) = &s.field else {
            panic!("{:?}", s.field);
        };
        let lit: Vec<usize> = (0..places.len()).filter(|&k| places[k].1).collect();
        assert_eq!((places.len(), lit), (4, vec![2]));
        for (k, &(x, _)) in places.iter().enumerate() {
            let want = place::voice_at(1.0, 0.4, place::Placement::Even, k, 4);
            assert_eq!(x, want, "place {k}");
        }
        assert_eq!(s.levels[2], 0.5);
        let shown = |s: &StripScene, r: Readout| {
            s.readouts[Readout::ALL.iter().position(|&x| x == r).unwrap()].1
        };
        for r in [Readout::Voices, Readout::Width, Readout::Inner] {
            assert!(shown(&s, r), "{r:?} not lit");
        }
        let d = scene(true);
        assert!(matches!(&d.field, Field::Double(pairs) if pairs.len() == 1));
        assert!(!shown(&d, Readout::Voices), "VOICES lit with DOUBLE");
        assert!(shown(&d, Readout::Width));
    }

    /// VOICES where it does nothing (R64): in MONO with DOUBLE, or scattering at WIDTH 0;
    /// it counts again with POLY, with UNISON, or with WIDTH up in MONO without DOUBLE (R62).
    /// The editor then draws its knob dimmed, takes no press, drag nor typed value for it, and
    /// leaves its readout unlit; as for DETUNE in SCATTER, the other knobs answer.
    #[test]
    fn voices_is_dimmed_where_it_does_nothing() {
        let params = |poly: bool, unison: bool, doubled: bool, width: f32| Ca72Params {
            poly: BoolParam::new("Poly", poly),
            unison: BoolParam::new("Unison", unison),
            doubled: BoolParam::new("Double", doubled),
            spread: FloatParam::new(
                "Width",
                width,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            ),
            ..Ca72Params::default()
        };
        for (poly, unison, doubled, width, inert) in [
            (false, false, true, 100.0, true),
            (false, false, true, 0.0, true),
            (false, false, false, 0.0, true),
            (false, false, false, 100.0, false),
            (true, false, false, 0.0, false),
            (true, false, true, 0.0, false),
            (false, true, true, 0.0, false),
            (false, true, false, 100.0, false),
        ] {
            let p = params(poly, unison, doubled, width);
            assert_eq!(
                voices_inert(&p),
                inert,
                "POLY {poly}, UNISON {unison}, DOUBLE {doubled}, WIDTH {width}"
            );
            let s = strip_scene(&p, 0, (&[], 0.0), StripScene::default());
            assert_eq!(s.readouts[0].1, !inert, "the readout lit");
        }
        // The editor, at the defaults (MONO, SCATTER, WIDTH 0).
        let (mut e, host, params) = editing();
        assert!(voices_inert(&params));
        e.draw();
        assert!(e.scene.voices_off);
        let voices = ca72_panel::controls::index("voices").expect("a control");
        let cutoff = ca72_panel::controls::index("cutoff").expect("a control");
        assert!(!e.operable(voices) && e.operable(cutoff));
        host.take();
        let pt = at(&e, centre("voices"));
        click(&mut e, pt);
        go(&mut e, pt);
        wheel(&mut e, 1.0);
        e.end_gestures();
        assert!(host.take().is_empty(), "VOICES took a press or the wheel");
        e.start_typing(Readout::Voices);
        assert!(e.typing.is_none());
        e.start_typing(Readout::Drive);
        assert!(e.typing.is_some());
    }

    #[test]
    fn the_grip_saves_the_width_it_was_given() {
        let (mut e, _host, params) = editing();
        assert!(e.resize(1500));
        assert_eq!(
            (e.width, params.editor_width.load(Ordering::Relaxed)),
            (1500, 1500)
        );
        assert!(!e.resize(1500));
        // Every frame then drawn at that size (on macOS the window applies it itself: its
        // baseview reports no resize of its own making).
        e.fit(1.0, 1500, 1500);
        e.draw();
        let widths = [e.renderer.frame().width(), e.strip.frame().width()];
        assert_eq!(widths, [1500; 2]);
        // (The panel's renderer draws the whole drawing, the strip's parts over its foot.)
        let tall = e.renderer.frame().height();
        assert!(
            tall.abs_diff(height_for(1500)) <= 2,
            "{tall} for {}",
            height_for(1500)
        );
    }

    /// The settings key's WINDOW: SIZE… (R-SIZE) opens the SIZE menu where the settings' was:
    /// shares of the size the editor opens at on a 1920 by 1080 screen, then FIT TO SCREEN; the
    /// size the window is marked and not to be chosen. A size chosen is asked of the host as
    /// the grip's are, and saved with the session. Where the screen is known, sizes too large
    /// for it cannot be chosen, and FIT TO SCREEN is the size the editor opens at on it.
    #[test]
    fn the_settings_key_s_size_menu_resizes_the_window() {
        let (mut e, host, params) = editing();
        let key = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Settings)));
        click(&mut e, key);
        let items = menu_items(&e);
        assert_eq!(
            items[9..11],
            [
                ("WINDOW".to_owned(), false),
                ("SIZE\u{2026}".to_owned(), true)
            ]
        );
        menu_item(&mut e, "SIZE\u{2026}");
        let m = e.learning.menu().expect("the SIZE menu");
        assert_eq!(m.title, "SIZE");
        let labels: Vec<_> = m.items.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(
            labels,
            ["75%", "100%", "125%", "150%", "200%", "FIT TO SCREEN"]
        );
        // (At 1720 wide, none of them: none marked; the screen not known, all fit.)
        assert!(m.marked.is_empty() && m.items.iter().all(|(_, can)| *can));
        let (w, h) = m.extent(e.renderer.fonts());
        assert!(m.x >= 0.0 && m.x + w <= art::W && m.y + h <= art::H);
        menu_item(&mut e, "100%");
        assert!(e.learning.menu().is_none());
        // (Asked of the host at the next frame, as the grip's are.)
        e.grip();
        e.draw();
        assert_eq!(
            (e.width, params.editor_width.load(Ordering::Relaxed)),
            (DEFAULT_WIDTH, DEFAULT_WIDTH)
        );
        assert_eq!(host.resizes.load(Ordering::Relaxed), 1);
        e.fit(1.0, DEFAULT_WIDTH, height_for(DEFAULT_WIDTH));
        e.draw();
        // A screen 1512 by 945 (a 14-inch MacBook Pro's, less its menu bar).
        e.screen = |_| Some((1512.0, 945.0));
        let sizes = e.sizes();
        assert_eq!(
            sizes
                .iter()
                .map(|&(_, w, fits)| (w, fits))
                .collect::<Vec<_>>(),
            [
                (1035, true),
                (1380, true),
                (1725, false),
                (2070, false),
                (2760, false),
                (fitted_width(Some((1512.0, 945.0))), true),
            ]
        );
        let key = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Settings)));
        click(&mut e, key);
        menu_item(&mut e, "SIZE\u{2026}");
        assert_eq!(
            e.learning
                .menu()
                .map(|m| (m.marked.clone(), m.items[1].1, m.items[2].1)),
            Some((vec![1], false, false)),
            "100% marked; 125% too large for the screen"
        );
        menu_item(&mut e, "FIT TO SCREEN");
        e.grip();
        e.draw();
        assert_eq!(e.width, fitted_width(Some((1512.0, 945.0))));
    }

    /// The grip (R-SIZE): over it, the tip says what it does, within the window, in the full
    /// view and the compact; and its lines are plain to see.
    #[test]
    fn the_grip_says_what_it_does() {
        let (mut e, _, _) = editing();
        e.draw();
        let p = at(&e, (art::W - 20.0, art::H - 20.0));
        go(&mut e, p);
        assert!(e.on_grip());
        let (t, x, _) = e.tip().expect("a tip");
        assert_eq!(t, GRIP_TIP);
        let size = e.renderer.text_size();
        let w = e.renderer.fonts().advance(&t, size, Weight::Regular, 0.0) + 0.8 * size;
        assert!(x + w / 2.0 <= art::W && x - w / 2.0 >= 0.0);
        e.draw();
        assert!(e.scene.tip.is_some());
        let p = at(&e, (art::W / 2.0, art::H / 2.0));
        go(&mut e, p);
        assert!(!e.on_grip());
        // The compact view's, in the rail's corner.
        let panel = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Panel)));
        click(&mut e, panel);
        e.draw();
        let p = at(&e, (art::W - 20.0, DRAWER_TOP - 20.0));
        go(&mut e, p);
        assert!(e.on_grip());
        assert_eq!(e.tip().map(|(t, ..)| t), Some(GRIP_TIP.to_owned()));
        // The drawing's grip: on the strip's face inside its trim, brighter than the face.
        let mut r = Renderer::new(0.5, 1.0);
        r.render(&Scene::default());
        let f = r.frame();
        let px = |x: f64, y: f64| {
            f.pixel((x * 0.5) as u32, (y * 0.5) as u32)
                .expect("in it")
                .red()
        };
        assert!(px(art::W - 47.0, art::H - 25.0) > px(art::W - 150.0, art::H - 40.0) + 60);
    }

    /// The large lettering's settings' menu in the smallest window, the panel hidden: made
    /// smaller to fit the view rather than running off it (R-LETTERING).
    #[test]
    fn a_menu_too_tall_for_the_view_is_made_smaller() {
        let (mut e, _, _) = editing();
        e.set_lettering(Lettering::Large);
        assert!(e.resize(MIN_WIDTH));
        e.fit(1.0, MIN_WIDTH, height_for(MIN_WIDTH));
        e.draw();
        let panel = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Panel)));
        click(&mut e, panel);
        e.draw();
        let key = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Settings)));
        click(&mut e, key);
        e.draw();
        let m = e.learning.menu().expect("a menu");
        let (w, h) = m.extent(e.renderer.fonts());
        assert!(m.y >= 0.0 && m.y + h <= DRAWER_TOP + 1e-6, "{} + {h}", m.y);
        assert!(m.x >= 0.0 && m.x + w <= art::W);
        assert!(m.size < e.renderer.text_size() * MENU_TEXT, "made smaller");
    }

    #[test]
    fn the_panel_key_snaps_and_presets_temporarily_reveal_it() {
        let (mut e, host, params) = editing();
        e.draw();
        let panel = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Panel)));
        let full = e.window_height();
        let values = e.scene.values;
        click(&mut e, panel);
        assert!(!params.editor_panel.load(Ordering::Relaxed));
        assert_eq!(e.window_height(), height_for_panel(e.width, false));
        assert!(e.window_height() < full && e.resizing_window);
        e.draw();
        assert_eq!(host.resizes.load(Ordering::Relaxed), 1);
        assert_eq!(e.composed.as_ref().unwrap().height(), e.window_height());
        assert_eq!(e.hit(100.0, DRAWER_TOP + 20.0), None);
        assert_eq!(e.hit(art::W - 10.0, DRAWER_TOP - 10.0), Some(Target::Grip));

        open(&mut e);
        assert_eq!(e.window_height(), full);
        assert!(!params.editor_panel.load(Ordering::Relaxed), "temporary");
        assert_eq!(host.resizes.load(Ordering::Relaxed), 2);
        assert!(key(&mut e, Key::Escape));
        e.follow_drawer();
        assert_eq!(e.window_height(), height_for_panel(e.width, false));
        e.draw();
        assert_eq!(host.resizes.load(Ordering::Relaxed), 3);
        click(&mut e, panel);
        assert_eq!(e.window_height(), full);
        assert!(params.editor_panel.load(Ordering::Relaxed));
        assert_eq!(host.resizes.load(Ordering::Relaxed), 4);
        e.draw();
        assert_eq!(e.scene.values, values, "showing the panel changes no sound");
        assert!(host.take().is_empty(), "no parameter gestures");

        click(&mut e, panel);
        params.editor_width.store(e.width, Ordering::Relaxed);
        let editor = Ca72Editor::new(Arc::clone(&params), Arc::clone(&e.meters));
        assert_eq!(
            editor.opening(),
            (e.width, height_for_panel(e.width, false))
        );
    }

    #[test]
    fn a_refused_panel_resize_keeps_the_current_view() {
        let (mut e, host, params) = editing();
        e.draw();
        assert!(e.browser.bar.panel);
        let panel = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Panel)));
        host.fixed.store(true, Ordering::Relaxed);
        click(&mut e, panel);
        assert!(params.editor_panel.load(Ordering::Relaxed));
        assert!(e.expanded());
        assert!(!e.resizing_window);
        e.draw();
        assert!(e.browser.bar.panel, "refused collapse keeps PANEL active");
        host.fixed.store(false, Ordering::Relaxed);
        click(&mut e, panel);
        assert!(!e.expanded());
        e.draw();
        assert!(!e.browser.bar.panel, "collapsed PANEL is neutral");
        host.fixed.store(true, Ordering::Relaxed);
        click(&mut e, panel);
        e.draw();
        assert!(!e.browser.bar.panel, "refused expansion stays neutral");
        let name = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Name)));
        click(&mut e, name);
        assert!(!e.browser.open && !e.expanded());
        assert!(!params.editor_panel.load(Ordering::Relaxed));
    }

    #[test]
    fn the_compact_frame_drawn_in_parts_is_the_frame_drawn_whole() {
        let (mut e, _, params) = editing();
        let hostless = nih_plug::context::process::TestProcessContext::<crate::Ca72>::new(
            Arc::clone(&params) as Arc<dyn Params>,
            48_000.0,
            ProcessMode::Realtime,
        );
        let panel = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Panel)));
        click(&mut e, panel);
        for i in 0..12 {
            hostless.automate(params.cutoff.as_ptr(), i as f32 / 12.0);
            e.draw();
            let mut whole = None;
            compose(
                &mut whole,
                &e.renderer,
                e.strip.frame(),
                None,
                None,
                None,
                false,
                (e.physical_width, e.physical_height()),
            );
            assert!(e.composed == whole, "compact frame {i}");
        }
        let grip = at(&e, (art::W - 10.0, DRAWER_TOP - 10.0));
        go(&mut e, grip);
        e.mouse(MouseEvent::ButtonPressed {
            button: MouseButton::Left,
            modifiers: Modifiers::empty(),
        });
        assert!(e.resizing.is_some(), "the compact grip is operable");
        e.end_gestures();
    }

    #[test]
    fn compact_settings_stay_inside_the_view_and_are_operable() {
        let (mut e, _, _) = editing();
        let panel = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Panel)));
        click(&mut e, panel);
        let settings = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Settings)));
        click(&mut e, settings);
        e.draw();
        let menu = e.learning.menu().unwrap();
        let (_, h) = menu.extent(e.renderer.fonts());
        assert!(menu.y >= 0.0 && menu.y + h <= DRAWER_TOP);
        menu_item(&mut e, "CLASSIC");
        assert_eq!(e.renderer.skin(), Skin::Drawn);
        assert!(!e.expanded());
    }

    #[test]
    fn recalled_panel_visibility_reaches_an_open_editor_and_keeps_presets_visible() {
        let (mut e, _, params) = editing();
        params.editor_width.store(e.width, Ordering::Relaxed);
        let editor = Ca72Editor::new(Arc::clone(&params), Arc::clone(&e.meters));
        editor.open.store(true, Ordering::Relaxed);
        params.editor_panel.store(false, Ordering::Relaxed);
        editor.param_values_changed();
        assert_eq!(editor.size().1, height_for_panel(e.width, false));
        e.draw();
        assert!(!e.expanded() && e.resizing_window);
        params.editor_panel.store(true, Ordering::Relaxed);
        assert_eq!(editor.size().1, height_for(e.width));
        e.draw();
        assert!(e.expanded());
        open(&mut e);
        params.editor_panel.store(false, Ordering::Relaxed);
        assert_eq!(editor.size().1, height_for(e.width), "presets still shown");
        e.draw();
        assert!(e.expanded() && e.browser.open);
        assert!(key(&mut e, Key::Escape));
        e.follow_drawer();
        assert_eq!(editor.size().1, height_for_panel(e.width, false));
    }

    #[test]
    fn refused_expansion_preserves_type_notes_when_restoring_compact() {
        let (mut e, host, params) = editing_standalone();
        e.toggle_panel();
        e.draw();
        assert!(!e.expanded());
        host.deferred.store(true, Ordering::Relaxed);
        e.playing.visible = true;
        e.playing.typing = true;
        e.toggle_panel();
        let (request, _) = params.editor_resizing.requested().expect("expansion");
        let key = KeyboardEvent {
            key: Key::Character("a".into()),
            ..Default::default()
        };
        assert!(e.keyed(&key));
        assert!(e.playing.notes.pop().expect("attack").on);
        let generation = e.playing.notes.generation();
        params.editor_resizing.answer(request, false);
        e.draw();
        assert!(!e.expanded());
        assert_eq!(e.playing.notes.generation(), generation);
        assert!(e.playing.wants_keys);
        assert!(e.keyed(&key));
        assert!(
            e.playing.notes.pop().is_none(),
            "held note does not retrigger"
        );
        let mut up = key;
        up.state = KeyState::Up;
        assert!(e.keyed(&up));
        assert!(!e.playing.notes.pop().expect("original note released").on);
    }

    #[test]
    fn delayed_refusal_restores_the_view_and_ignores_superseded_requests() {
        let (mut e, host, params) = editing();
        host.deferred.store(true, Ordering::Relaxed);
        e.draw();
        let original_width = params.editor_width.load(Ordering::Relaxed);
        e.toggle_panel();
        let (first, _) = params.editor_resizing.requested().expect("a request");
        e.draw();
        e.toggle_panel();
        e.draw();
        e.toggle_panel();
        let (last, _) = params.editor_resizing.requested().expect("a request");
        params.editor_resizing.answer(first, false);
        e.draw();
        assert!(
            !e.expanded(),
            "an obsolete refusal cannot undo the last toggle"
        );
        params.editor_resizing.answer(last, false);
        e.draw();
        assert!(e.expanded());
        assert!(params.editor_panel.load(Ordering::Relaxed));
        assert_eq!(params.editor_width.load(Ordering::Relaxed), original_width);
    }

    #[test]
    fn delayed_width_refusal_keeps_the_restored_frame_through_native_follow() {
        for dpr in [1.0, 1.25, 2.0] {
            let (mut e, host, params) = editing();
            let original = e.width;
            let physical = (f64::from(original) * dpr).round() as u32;
            e.fit(dpr, original, physical);
            e.draw();
            host.deferred.store(true, Ordering::Relaxed);
            e.resize(original + 100);
            let wider = e.width;
            e.fit(dpr, wider, (f64::from(wider) * dpr).round() as u32);
            e.draw();
            let (request, _) = params.editor_resizing.requested().expect("width request");
            params.editor_resizing.answer(request, false);
            e.draw();
            assert_eq!(e.width, original);
            let composed = e.composed.clone();
            assert_eq!(composed.as_ref().expect("restored frame").width(), physical);
            // on_frame follows this restoration before presenting the composed controls.
            e.fit(dpr, original, physical);
            assert!(
                e.composed == composed,
                "native follow preserves restored controls"
            );
        }
    }

    #[test]
    fn a_refused_shrink_keeps_presets_and_the_compact_preference() {
        let (mut e, host, params) = editing();
        e.toggle_panel();
        e.draw();
        host.deferred.store(true, Ordering::Relaxed);
        open(&mut e);
        e.draw();
        let (reveal, _) = params.editor_resizing.requested().expect("reveal");
        params.editor_resizing.answer(reveal, true);
        e.draw();
        assert!(e.browser.open);
        e.browser.show(false, &params);
        e.follow_drawer();
        e.draw();
        let (shrink, _) = params.editor_resizing.requested().expect("shrink");
        params.editor_resizing.answer(shrink, false);
        e.draw();
        assert!(e.browser.open, "the host still owns the full presets view");
        assert!(e.expanded());
        assert!(
            !params.editor_panel.load(Ordering::Relaxed),
            "retain the compact preference"
        );
    }

    #[test]
    fn refusal_keeps_presets_that_opened_without_changing_height() {
        let (mut e, host, _) = editing();
        open(&mut e);
        e.draw();
        assert!(e.browser.open);
        host.fixed.store(true, Ordering::Relaxed);
        e.toggle_panel();
        e.draw();
        assert!(
            e.browser.open,
            "a refused toggle must preserve the existing presets view"
        );
        e.resize(e.width + 100);
        e.draw();
        assert!(
            e.browser.open,
            "a refused width must preserve the existing presets view"
        );
    }

    #[test]
    fn recalled_visibility_during_a_queued_width_does_not_retry_refusal() {
        let (mut e, host, params) = editing();
        host.deferred.store(true, Ordering::Relaxed);
        e.resize(e.width + 100);
        e.draw();
        let (width, _) = params.editor_resizing.requested().expect("width request");
        params.editor_panel.store(false, Ordering::Relaxed);
        e.reconcile_resize();
        params.editor_resizing.answer(width, true);
        e.draw();
        let (compact, _) = params.editor_resizing.requested().expect("height request");
        let requests = host.resizes.load(Ordering::Relaxed);
        params.editor_resizing.answer(compact, false);
        e.draw();
        assert_eq!(
            host.resizes.load(Ordering::Relaxed),
            requests,
            "refusal must not immediately retry the collapse"
        );
        assert_eq!(
            params.editor_panel.load(Ordering::Relaxed),
            params.editor_panel_applied.load(Ordering::Relaxed)
        );
        let requests = host.resizes.load(Ordering::Relaxed);
        e.draw();
        assert_eq!(
            host.resizes.load(Ordering::Relaxed),
            requests,
            "a refusal must not become a retry loop"
        );
    }

    #[test]
    fn frames_fit_the_rounded_physical_viewport_at_fractional_scales() {
        for width in [861, 862, 1119, 1121] {
            for dpr in [1.0, 1.25, 1.5, 2.0] {
                let (mut e, _, _) = editing();
                let physical_width = (f64::from(width) * dpr).round() as u32;
                e.fit(dpr, width, physical_width);
                for expanded in [true, false] {
                    if expanded != e.expanded() {
                        e.toggle_panel();
                    }
                    e.draw();
                    let frame = e.composed.as_ref().expect("a frame");
                    assert_eq!(
                        (frame.width(), frame.height()),
                        (physical_width, e.physical_height()),
                        "width {width}, scale {dpr}, expanded {expanded}"
                    );
                    if !expanded {
                        let bottom = f64::from(e.physical_height()) / dpr;
                        let (x, y) =
                            e.in_drawing((f64::from(width) - 1.0 / dpr, bottom - 1.0 / dpr));
                        assert_eq!(e.hit(x, y), Some(Target::Grip));
                    }
                }
            }
        }
    }

    #[test]
    fn a_height_only_resize_keeps_the_frame_just_composed() {
        let (mut e, _, params) = editing();
        for expanded in [false, true] {
            params.editor_panel.store(expanded, Ordering::Relaxed);
            e.draw();
            let composed = e.composed.clone();
            assert!(e.resizing_window && composed.is_some());
            // on_frame follows a resize asked during draw before presenting that frame.
            let physical = WindowInfo::from_logical_size(
                Size::new(f64::from(e.width), f64::from(e.window_height())),
                e.dpr,
            )
            .physical_size();
            e.fit(e.dpr, e.width, physical.width);
            assert!(e.composed == composed, "keep the rail and compact grip");
        }
    }

    /// The grip's drag asks the host for a size once a frame at most, the last the pointer
    /// took it to (each resize renders the whole panel again).
    #[test]
    fn the_grip_resizes_once_a_frame() {
        let (mut e, host, params) = editing();
        let grip = at(&e, (art::W - 27.0, art::H - 27.0));
        go(&mut e, grip);
        press(&mut e);
        assert!(e.resizing.is_some());
        for dx in 1..=20 {
            let p = Point {
                x: grip.x + f64::from(dx) * 5.0,
                y: grip.y,
            };
            go(&mut e, p);
        }
        assert_eq!(
            host.resizes.load(Ordering::Relaxed),
            0,
            "none between frames"
        );
        e.grip();
        assert_eq!(host.resizes.load(Ordering::Relaxed), 1);
        assert_eq!(
            (e.width, params.editor_width.load(Ordering::Relaxed)),
            (1820, 1820)
        );
        e.grip();
        assert_eq!(
            host.resizes.load(Ordering::Relaxed),
            1,
            "nothing moved since"
        );
        release(&mut e);
    }

    // ---- The presets (decisions.md R10).

    /// A point of the drawer (its units) in the window, the drawer open.
    fn in_drawer(e: &Editing, (x, y): (f64, f64)) -> Point {
        let (top, _, _) = e.drawer_span().expect("the drawer open");
        at(e, (x, top + y))
    }

    fn key(e: &mut Editing, k: Key) -> bool {
        let setter = ParamSetter::new(e.context.as_ref());
        e.browser.key(
            &KeyboardEvent {
                state: KeyState::Down,
                key: k,
                ..KeyboardEvent::default()
            },
            &e.params,
            &setter,
        )
    }

    fn typed(e: &mut Editing, s: &str) {
        for c in s.chars() {
            key(e, Key::Character(c.to_string()));
        }
    }

    /// The drawer, opened from the bar's name and slid open.
    fn open(e: &mut Editing) {
        let p = on_strip(e, strip_at(StripTarget::Bar(BarTarget::Name)));
        click(e, p);
        assert!(e.browser.open);
        std::thread::sleep(crate::presets::SLIDE + Duration::from_millis(20));
        e.draw();
        assert_eq!(e.browser.reveal(), 1.0);
    }

    /// The drawer's slide over, and drawn.
    fn slid(e: &mut Editing) {
        std::thread::sleep(crate::presets::SLIDE + Duration::from_millis(20));
        e.draw();
    }

    /// A library of the factory's presets and twelve of the user's (`Pad 01` to `Pad 12`), so
    /// that the list runs on below the rows shown whatever the factory's count.
    fn long_library(dir: &std::path::Path) -> Library {
        let lib = Library::at(dir);
        for i in 1..=12 {
            lib.save(&format!("Pad {i:02}"), vec![], None).unwrap();
        }
        lib
    }

    fn row_of(e: &Editing, name: &str) -> usize {
        e.browser
            .drawer
            .rows
            .iter()
            .position(|r| r.name == name)
            .unwrap_or_else(|| panic!("{name} not listed"))
    }

    fn preset(p: &Ca72Params) -> String {
        p.preset.read().unwrap().clone()
    }

    /// A trackpad's or a Magic Mouse's scrolling (a few points an event) and a wheel turned
    /// slowly (fractions of a line) scroll the list a row for each row's travel; a turn the
    /// other way starts again (the owner, 2026-10-02: neither the wheel nor touch scrolled it).
    #[test]
    fn small_scrolls_add_up_to_rows() {
        let dir = tempfile::tempdir().unwrap();
        let (mut e, _host, _params) = editing_with(long_library(dir.path()));
        e.draw();
        open(&mut e);
        let list = in_drawer(
            &e,
            presets_ui::row_centre(&e.browser.drawer, 0).expect("shown"),
        );
        go(&mut e, list);
        let wheel = |e: &mut Editing, delta: ScrollDelta| {
            e.mouse(MouseEvent::WheelScrolled {
                delta,
                modifiers: Modifiers::empty(),
            });
        };
        // A row's travel in points at this size, in 4-point events: one row down.
        let row_points = ROW_H * e.renderer.scale() / e.dpr;
        let events = (row_points / 4.0).ceil() as usize;
        assert!(events > 2, "{row_points} points a row");
        for _ in 0..events {
            wheel(&mut e, ScrollDelta::Pixels { x: 0.0, y: -4.0 });
        }
        assert_eq!(e.browser.drawer.first, 1, "a row's travel of a trackpad");
        // A wheel turned slowly: three tenths of a line at a time.
        for _ in 0..4 {
            wheel(&mut e, ScrollDelta::Lines { x: 0.0, y: -0.3 });
        }
        assert_eq!(e.browser.drawer.first, 2, "1.2 lines of a slow wheel");
        // Back up: the 0.2 left over is dropped, so one line up is one row.
        wheel(&mut e, ScrollDelta::Lines { x: 0.0, y: 1.0 });
        assert_eq!(e.browser.drawer.first, 1);
    }

    #[test]
    fn the_rail_drops_the_drawer_over_the_strip_and_a_row_sets_the_plugin() {
        let dir = tempfile::tempdir().unwrap();
        let (mut e, host, params) = editing_with(long_library(dir.path()));
        e.draw();
        let shut = e.window_height();
        assert_eq!(shut, height_for(1720));
        open(&mut e);
        // Over the strip from under the rail, the window as it was, nothing asked of the host.
        assert_eq!(e.window_height(), shut);
        assert!(!e.resizing_window);
        assert_eq!(host.resizes.load(Ordering::Relaxed), 0);
        assert_eq!(e.drawer_span().unwrap(), (DRAWER_TOP, DRAWER_TOP, art::H));
        assert!(!e.browser.bar.below && e.browser.bar.open);
        assert_eq!(
            e.browser.drawer.rows.len(),
            crate::library::factory().len() + 12
        );
        // A row sets every parameter it names (or its default), the preset remembered.
        host.take();
        let i = row_of(&e, "Warp Field");
        // Below the rows shown: the wheel scrolls the list to it.
        let list = in_drawer(
            &e,
            presets_ui::row_centre(&e.browser.drawer, 0).expect("shown"),
        );
        go(&mut e, list);
        while presets_ui::row_centre(&e.browser.drawer, i).is_none() {
            let first = e.browser.drawer.first;
            e.mouse(MouseEvent::WheelScrolled {
                delta: ScrollDelta::Lines { x: 0.0, y: -1.0 },
                modifiers: Modifiers::empty(),
            });
            assert_eq!(
                e.browser.drawer.first,
                first + 1,
                "the wheel scrolls the list"
            );
        }
        let (x, y) = presets_ui::row_centre(&e.browser.drawer, i).expect("shown");
        let pt = in_drawer(&e, (x, y));
        click(&mut e, pt);
        assert_eq!(preset(&params), "Warp Field");
        let calls = host.take();
        let set = |p: ParamPtr| {
            calls
                .iter()
                .any(|c| matches!(c, Call::Set(q, _) if *q == p))
        };
        assert!(
            set(params.osc2_on.as_ptr())
                && set(params.cutoff.as_ptr())
                && set(params.mod_wheel.as_ptr())
        );
        assert!(!set(params.pitch_wheel.as_ptr()) && !set(params.bypass.as_ptr()));
        // Each a gesture of its own.
        let begins = calls.iter().filter(|c| matches!(c, Call::Begin(_))).count();
        let ends = calls.iter().filter(|c| matches!(c, Call::End(_))).count();
        assert_eq!(begins, ends);
        e.draw();
        assert_eq!(e.browser.bar.name, "Warp Field");
        assert!(e.browser.drawer.rows[i].current);
        // A double click on a row chooses it and closes the drawer.
        e.browser.scroll(-100);
        let j = row_of(&e, "Brass Choir");
        let pt = in_drawer(
            &e,
            presets_ui::row_centre(&e.browser.drawer, j).expect("shown"),
        );
        click(&mut e, pt);
        click(&mut e, pt);
        assert_eq!(preset(&params), "Brass Choir");
        assert!(!e.browser.open);
        slid(&mut e);
        assert_eq!(e.window_height(), shut);
        // Escape closes it too.
        open(&mut e);
        assert!(key(&mut e, Key::Escape));
        slid(&mut e);
        assert!(!e.browser.open);
        assert!(
            !key(&mut e, Key::Character("a".into())),
            "closed: the host's keys"
        );
    }

    /// The editor's width, fitted to the screen as it opens, stays, and is measured again
    /// when it next opens.
    #[test]
    fn the_width_stays_until_the_editor_opens_again() {
        let ed = Ca72Editor::new(Arc::new(Ca72Params::default()), Arc::new(Meters::default()));
        let (w, h) = ed.opening();
        ed.fitted.store(w, Ordering::Relaxed);
        assert_eq!(ed.size(), (w, h));
        drop(ed.closing());
        assert_eq!(
            ed.fitted.load(Ordering::Relaxed),
            0,
            "measured again next time"
        );
    }

    /// The drawer drops down from under the rail over the strip's sections and slides back up
    /// under it as it shuts: the panel and the rail stay as they are, the window its size, the
    /// host never asked to resize it (A6; it had grown the window below the strip). A row there
    /// sets the plug-in.
    #[test]
    fn the_drawer_drops_over_the_strip_and_leaves_the_panel() {
        let dir = tempfile::tempdir().unwrap();
        let (mut e, host, _params) = editing_with(Library::at(dir.path()));
        e.draw();
        open(&mut e);
        let c = e.composed.as_ref().expect("the window's frame");
        let frame = e.renderer.frame();
        let viewport_width = e.physical_width as usize;
        let row = |p: &ca72_panel::Pixmap, y: u32| {
            let w = p.width() as usize;
            let left = y as usize * w * 4;
            p.data()[left..left + viewport_width * 4].to_vec()
        };
        let s = e.renderer.scale();
        let panel = ((art::PANEL_H - 40.0) * s) as u32;
        let low = ((art::H - 100.0) * s) as u32;
        assert_eq!(row(c, panel), row(frame, panel), "the panel as it is");
        assert_ne!(row(c, low), row(frame, low), "the drawer over the strip");
        let open_row = row(c, low);
        assert_eq!(host.resizes.load(Ordering::Relaxed), 0);
        // A row there sets the plug-in: one on the list's first page, not the current preset.
        let i = e.browser.drawer.first + 1;
        let name = e.browser.drawer.rows[i].name.clone();
        assert!(!e.browser.drawer.rows[i].current);
        let pt = in_drawer(
            &e,
            presets_ui::row_centre(&e.browser.drawer, i).expect("shown"),
        );
        click(&mut e, pt);
        e.draw();
        assert_eq!(e.browser.bar.name, name);
        // Shut: it slides back up (held at the slide's start), the window as it is.
        assert!(key(&mut e, Key::Escape));
        e.follow_drawer();
        e.browser
            .slide_began(Instant::now() + Duration::from_secs(3600));
        e.draw();
        assert!(!e.browser.open && e.browser.sliding());
        assert_eq!(e.window_height(), height_for(1720));
        e.browser
            .slide_began(Instant::now() - crate::presets::SLIDE * 2);
        e.draw();
        let c = e.composed.as_ref().expect("the window's frame");
        assert_ne!(row(c, low), open_row, "the strip again");
        assert_eq!(host.resizes.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn typing_searches_and_the_arrows_step_through_the_list() {
        let dir = tempfile::tempdir().unwrap();
        let (mut e, _host, params) = editing_with(Library::at(dir.path()));
        open(&mut e);
        typed(&mut e, "GLIDE");
        let names: Vec<&str> = e
            .browser
            .drawer
            .rows
            .iter()
            .map(|r| r.name.as_str())
            .collect();
        assert!(names.contains(&"Singing Drift") && names.contains(&"Horn Bloom"));
        assert!(!names.contains(&"Felt Bass"), "{names:?}");
        key(&mut e, Key::Backspace);
        assert_eq!(e.browser.drawer.search.text, "GLID");
        key(&mut e, Key::ArrowDown);
        let first = e.browser.drawer.rows[0].name.clone();
        assert_eq!(preset(&params), first);
        key(&mut e, Key::ArrowDown);
        assert_eq!(preset(&params), e.browser.drawer.rows[1].name);
        key(&mut e, Key::ArrowUp);
        assert_eq!(preset(&params), first);
        // The host's shortcuts stay the host's.
        let setter = ParamSetter::new(e.context.as_ref());
        assert!(!e.browser.key(
            &KeyboardEvent {
                state: KeyState::Down,
                key: Key::Character("s".into()),
                modifiers: Modifiers::CONTROL,
                ..KeyboardEvent::default()
            },
            &e.params,
            &setter,
        ));
        // A tag chip filters; the bar's arrows step too.
        key(&mut e, Key::Escape);
        open(&mut e);
        let bass = e
            .browser
            .drawer
            .chips
            .iter()
            .position(|(t, _)| t == "prog")
            .expect("a chip");
        let setter = ParamSetter::new(e.context.as_ref());
        e.browser
            .drawer_press(DrawerTarget::Chip(bass), 0.0, &e.params, &setter);
        let names: Vec<&str> = e
            .browser
            .drawer
            .rows
            .iter()
            .map(|r| r.name.as_str())
            .collect();
        assert_eq!(names, ["Singing Drift"]);
        let pt = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Next)));
        click(&mut e, pt);
        assert_eq!(preset(&params), "Singing Drift");
    }

    /// An arrow choosing a preset ends a wheel's gesture still open on a knob it sets.
    #[test]
    fn a_key_choosing_a_preset_ends_a_gesture_first() {
        let dir = tempfile::tempdir().unwrap();
        let (mut e, host, _params) = editing_with(Library::at(dir.path()));
        e.draw();
        open(&mut e);
        let p = at(&e, centre("cutoff"));
        go(&mut e, p);
        wheel(&mut e, 1.0);
        assert!(e.keyed(&KeyboardEvent {
            state: KeyState::Down,
            key: Key::ArrowDown,
            ..KeyboardEvent::default()
        }));
        let calls = host.take();
        well_formed(&calls);
        assert!(calls.iter().filter(|c| matches!(c, Call::Begin(_))).count() > 2);
    }

    #[test]
    fn saved_renamed_tagged_deleted_and_restored_from_the_drawer() {
        let dir = tempfile::tempdir().unwrap();
        let (mut e, _host, params) = editing_with(Library::at(dir.path()));
        // SAVE… opens the drawer at the name; Tab to the tags; Enter saves.
        e.draw();
        let pt = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Save)));
        click(&mut e, pt);
        assert_eq!(e.browser.drawer.focus, Some(FieldId::SaveName));
        std::thread::sleep(crate::presets::SLIDE + Duration::from_millis(20));
        e.draw();
        typed(&mut e, "My Bass");
        key(&mut e, Key::Tab);
        typed(&mut e, "bass, test");
        key(&mut e, Key::Enter);
        assert!(dir.path().join("My Bass.toml").exists());
        assert_eq!(preset(&params), "My Bass");
        let lib = Library::at(dir.path());
        let mine = lib.find("My Bass").unwrap();
        assert_eq!(mine.sound.tags, ["bass", "test"]);
        // Its values: every parameter but the PITCH wheel and the bypass, plain.
        let ids: Vec<&str> = mine.sound.values.iter().map(|(k, _)| k.as_str()).collect();
        assert!(ids.contains(&"cutoff") && ids.contains(&"osc1_range") && ids.contains(&"voices"));
        assert!(!ids.contains(&"pitch_wheel") && !ids.contains(&"bypass"));
        assert!(mine.sound.values.contains(&("osc1_range".to_owned(), 3.0)));
        // SAVE AS: the preset's name to begin with; a factory preset's name, it says it replaces
        // it; Escape takes the save back.
        let key_at = |e: &Editing, k: PresetKey| in_drawer(e, presets_ui::key_centre(k));
        let pt = key_at(&e, PresetKey::SaveAs);
        click(&mut e, pt);
        assert_eq!(e.browser.drawer.save_name.text, "My Bass");
        assert_eq!(e.browser.drawer.focus, Some(FieldId::SaveName));
        for _ in 0.."My Bass".len() {
            key(&mut e, Key::Backspace);
        }
        typed(&mut e, "Felt Bass");
        assert!(e.browser.drawer.replace);
        assert!(e.browser.drawer.hint.contains("factory"));
        key(&mut e, Key::Escape);
        assert_eq!(e.browser.drawer.editing, None);
        assert!(e.browser.open);
        // RENAME, for the preset set: renamed in place (the plug-in keeps it).
        let pt = key_at(&e, PresetKey::Rename);
        click(&mut e, pt);
        assert_eq!(e.browser.drawer.focus, Some(FieldId::Edit));
        for _ in 0.."My Bass".len() {
            key(&mut e, Key::Backspace);
        }
        typed(&mut e, "Your Bass");
        key(&mut e, Key::Enter);
        assert!(lib.find("Your Bass").is_some() && lib.find("My Bass").is_none());
        assert_eq!(preset(&params), "Your Bass");
        // TAGS: its tags edited.
        let pt = key_at(&e, PresetKey::Tags);
        click(&mut e, pt);
        key(&mut e, Key::End);
        typed(&mut e, ", deep");
        key(&mut e, Key::Enter);
        assert_eq!(
            lib.find("Your Bass").unwrap().sound.tags,
            ["bass", "test", "deep"]
        );
        // DELETE: deleted once asked (DELETE again).
        let pt = key_at(&e, PresetKey::Delete);
        click(&mut e, pt);
        assert!(lib.find("Your Bass").is_some(), "deleted before asked");
        e.draw();
        click(&mut e, pt);
        assert!(lib.find("Your Bass").is_none());
        e.draw();
        assert!(!e.browser.bar.found, "the bar says the preset is gone");
        // A factory preset deleted (Enter once asked), then restored; the star makes a
        // favourite.
        let i = row_of(&e, "Singing Drift");
        let setter = ParamSetter::new(e.context.as_ref());
        e.browser
            .drawer_press(DrawerTarget::Row(i), 0.0, &e.params, &setter);
        e.browser.drawer_press(
            DrawerTarget::Key(PresetKey::Delete),
            0.0,
            &e.params,
            &setter,
        );
        key(&mut e, Key::Enter);
        assert!(
            e.browser
                .drawer
                .rows
                .iter()
                .all(|r| r.name != "Singing Drift")
        );
        let setter = ParamSetter::new(e.context.as_ref());
        e.browser.drawer_press(
            DrawerTarget::Key(PresetKey::Restore),
            0.0,
            &e.params,
            &setter,
        );
        let i = row_of(&e, "Singing Drift");
        e.browser
            .drawer_press(DrawerTarget::Star(i), 0.0, &e.params, &setter);
        assert!(lib.find("Singing Drift").unwrap().favorite);
    }

    #[test]
    fn a_frame_is_shown_again_on_linux_only() {
        let now = Instant::now();
        assert!(repaint_due(None, now), "the first frame is always shown");
        assert!(!repaint_due(Some(now), now));
        let later = now + Duration::from_secs(1);
        assert_eq!(repaint_due(Some(now), later), cfg!(target_os = "linux"));
    }

    /// The window as it opens (the panel and the strip under it), written to
    /// `$CA72_EDITOR_PNG` for a look: the README's `docs/panel.png`.
    #[test]
    #[ignore = "writes an image for a look"]
    fn the_window_png() {
        let Some(out) = std::env::var_os("CA72_EDITOR_PNG") else {
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let (mut e, _host, _params) = editing_with(Library::at(dir.path()));
        window_frame(&mut e).save_png(out).unwrap();
    }

    /// The large lettering (R-LETTERING) beside the standard, for a look: in
    /// `$CA72_LETTERING_PNG`, the folder, the window as it opens on a 1920 by 1080 screen (1380
    /// wide) in each look and lettering, `<look>-<lettering>.png`; and in the realistic look with
    /// the settings' menu open, `settings-<lettering>.png`, CUTOFF FREQUENCY's,
    /// `knob-<lettering>.png`, and the SIZE menu, `size-<lettering>.png` (R-SIZE); and the
    /// pointer over the grip, its tip shown, `grip-<lettering>.png`.
    #[test]
    #[ignore = "writes images for a look"]
    fn the_lettering_pngs() {
        let Some(out) = std::env::var_os("CA72_LETTERING_PNG").map(std::path::PathBuf::from) else {
            return;
        };
        std::fs::create_dir_all(&out).unwrap();
        for (name, lettering) in [
            ("standard", Lettering::Standard),
            ("large", Lettering::Large),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let host = Arc::new(Host {
                api: Some(PluginApi::Clap),
                ..Default::default()
            });
            let context: Arc<dyn GuiContext> = host.clone();
            let mut e = Editing::new(
                context,
                Arc::new(Ca72Params::default()),
                Arc::new(Meters::default()),
                (1380, 1380, 1.0),
                Library::at(dir.path()),
                |_| None,
            );
            e.set_lettering(lettering);
            for (look, skin) in [("classic", Skin::Drawn), ("realistic", Skin::Worn)] {
                e.set_look(skin);
                window_frame(&mut e)
                    .save_png(out.join(format!("{look}-{name}.png")))
                    .unwrap();
            }
            let key = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Settings)));
            click(&mut e, key);
            window_frame(&mut e)
                .save_png(out.join(format!("settings-{name}.png")))
                .unwrap();
            let away = at(&e, (0.0, 0.0));
            click(&mut e, away);
            let knob = at(&e, centre("cutoff"));
            right_click(&mut e, knob);
            window_frame(&mut e)
                .save_png(out.join(format!("knob-{name}.png")))
                .unwrap();
            let away = at(&e, (0.0, 0.0));
            click(&mut e, away);
            click(&mut e, key);
            menu_item(&mut e, "SIZE\u{2026}");
            window_frame(&mut e)
                .save_png(out.join(format!("size-{name}.png")))
                .unwrap();
            click(&mut e, away);
            let grip = at(&e, (art::W - 50.0, art::H - 30.0));
            go(&mut e, grip);
            window_frame(&mut e)
                .save_png(out.join(format!("grip-{name}.png")))
                .unwrap();
        }
    }

    /// VOICES dimmed where it does nothing and lit where it counts (decisions.md R64), in
    /// each look, for a look: `voices-<look>-<state>.png` in `$CA72_VOICES_PNG`, the folder.
    #[test]
    #[ignore = "writes images for a look"]
    fn the_window_voices_pngs() {
        let Some(out) = std::env::var_os("CA72_VOICES_PNG").map(std::path::PathBuf::from) else {
            return;
        };
        let amount = |name: &str, v: f32| {
            FloatParam::new(
                name,
                v,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
        };
        let states = [
            ("mono-width-0", false, false, 0.0),
            ("mono-double", false, true, 60.0),
            ("mono-width-100", false, false, 100.0),
            ("poly", true, false, 60.0),
        ];
        for (look, skin) in [("realistic", Skin::Worn), ("classic", Skin::Drawn)] {
            for (state, poly, doubled, width) in states {
                let params = Ca72Params {
                    poly: BoolParam::new("Poly", poly),
                    doubled: BoolParam::new("Double", doubled),
                    spread: amount("Width", width),
                    ..Ca72Params::default()
                };
                let dir = tempfile::tempdir().unwrap();
                let (mut e, _host, _params) = editing_of(params, Library::at(dir.path()));
                // (Not `set_look`, which keeps the look in this computer's settings.)
                e.renderer = Renderer::with_skin(skin, e.renderer.scale(), e.dpr);
                e.composed = None;
                window_frame(&mut e)
                    .save_png(out.join(format!("voices-{look}-{state}.png")))
                    .unwrap();
            }
        }
    }

    /// The window's frame as shown (the panel's renderer's whole drawing, the strip's parts over
    /// its foot, the drawer and what floats over them: `compose`), for a look.
    fn window_frame(e: &mut Editing) -> &ca72_panel::Pixmap {
        e.draw();
        e.composed.as_ref().expect("the window's frame")
    }

    /// QUALITY's opening (decisions.md R48) for looking at, written to `$CA72_ULTRA_PNG`,
    /// the folder: the panel's last column close up (two pixels a unit) at QUALITY's three
    /// positions, each's thing up at rest and as the synth plays, `quality-<lo|hi|ultra>-<level>
    /// .png`; each coming up, `shutter-<setting>-<p>.png`; none of it, `hidden.png`; and the
    /// window at ULTRA with its note, `note.png`, and the opening's menu, `menu.png`.
    #[test]
    #[ignore = "writes images for a look"]
    fn the_ultra_pngs() {
        let Some(out) = std::env::var_os("CA72_ULTRA_PNG").map(std::path::PathBuf::from) else {
            return;
        };
        std::fs::create_dir_all(&out).unwrap();
        let k = 2.0;
        let mut r = Renderer::with_skin(Skin::Worn, k, 1.0);
        let crop = |r: &Renderer, name: &str| {
            let f = r.frame();
            let (x0, y0) = (
                ((art::COL + 2930.0) * k) as u32,
                ((art::TOP - 20.0) * k) as u32,
            );
            let (w, h) = (((3108.0 - 2930.0) * k) as u32, (640.0 * k) as u32);
            let mut c = ca72_panel::Pixmap::new(w, h).unwrap();
            for y in 0..h {
                let from = ((y0 + y) * f.width() + x0) as usize * 4;
                let to = (y * w) as usize * 4;
                c.data_mut()[to..to + w as usize * 4]
                    .copy_from_slice(&f.data()[from..from + w as usize * 4]);
            }
            c.save_png(out.join(name)).unwrap();
        };
        let q = ca72_panel::controls::index("quality").unwrap();
        let mut scene = Scene {
            values: [0.5; CONTROLS.len()],
            ..Scene::default()
        };
        use ca72_panel::ultra::Opening;
        // QUALITY's three positions, each's thing up: at rest, and at the synth's levels.
        for (name, v) in [("lo", 0.0), ("hi", 0.5), ("ultra", 1.0)] {
            scene.values[q] = v;
            for level in [0.0, 0.5, 1.0] {
                let mut o = Opening {
                    level,
                    spark: 1.3,
                    run: if level > 0.0 { 1.0 } else { 0.0 },
                    asleep: if level > 0.0 { 0.0 } else { 1.0 },
                    stride: 3,
                    turn: 20.0,
                    ..Opening::default()
                };
                match name {
                    "lo" => o.hamster = 1.0,
                    "hi" => o.lamp = 1.0,
                    _ => o.coil = 1.0,
                }
                scene.opening = o;
                r.render(&scene);
                crop(&r, &format!("quality-{name}-{level}.png"));
            }
        }
        // Each coming up through the shutter.
        for (name, v) in [("lo", 0.0), ("hi", 0.5), ("ultra", 1.0)] {
            scene.values[q] = v;
            for p in [0.0, 0.16, 0.32, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
                let mut o = Opening {
                    level: 0.6,
                    spark: 2.1,
                    run: p,
                    stride: (p * 7.0) as u8,
                    ..Opening::default()
                };
                match name {
                    "lo" => o.hamster = p,
                    "hi" => o.lamp = p,
                    _ => o.coil = p,
                }
                scene.opening = o;
                r.render(&scene);
                crop(&r, &format!("shutter-{name}-{p}.png"));
            }
        }
        // None of it: the panel blank above QUALITY.
        scene.opening = Opening {
            hidden: true,
            coil: 1.0,
            ..Opening::default()
        };
        r.render(&scene);
        crop(&r, "hidden.png");
        let dir = tempfile::tempdir().unwrap();
        let (mut e, _host, _params) = editing_with(Library::at(dir.path()));
        e.draw();
        e.scene.values[q] = 1.0;
        e.scene.opening = Opening {
            coil: 1.0,
            ..Opening::default()
        };
        e.scene.note = Some(e.ultra_note());
        e.renderer.render(&e.scene);
        e.renderer.frame().save_png(out.join("note.png")).unwrap();
        // Both of its texts: a computer that splits an ULTRA voice, and one that does not.
        for (splits, name) in [(true, "note-split.png"), (false, "note-whole.png")] {
            e.scene.note = Some(e.ultra_note_for(splits));
            e.renderer.render(&e.scene);
            e.renderer.frame().save_png(out.join(name)).unwrap();
        }
        // QUALITY's menu, its toggle right-clicked.
        e.ultra_note = false;
        e.settings.ultra_note_read = true;
        let size = e.renderer.text_size() * MENU_TEXT;
        let at = (art::COL + 3023.0, art::TOP + 266.0);
        e.learning
            .open_quality_menu(e.renderer.fonts(), at, size, e.settings.shutter);
        e.scene.note = None;
        e.scene.menu = e.learning.menu().cloned();
        e.renderer.render(&e.scene);
        e.renderer.frame().save_png(out.join("menu.png")).unwrap();
    }

    /// ULTRA's note as this computer plays it (decisions.md R-THREADS): where it splits an
    /// ULTRA voice over six threads, that one voice plays live; where it does not, what one
    /// voice takes. Either fits where the old one did, left of the lamp at the window's opening
    /// size: no line wider than the old note's widest, its box within the panel.
    #[test]
    fn ultra_s_note_says_what_this_computer_plays() {
        let (e, _host, _params) = editing();
        let split = e.ultra_note_for(true);
        let whole = e.ultra_note_for(false);
        let text = |n: &Note| -> Vec<String> { n.lines.iter().map(|(t, _)| t.clone()).collect() };
        assert_eq!(
            text(&e.ultra_note()),
            text(&e.ultra_note_for(crate::engine::split_voices() > 0)),
            "not as this computer plays it"
        );
        for (n, says) in [
            (
                &split,
                "ON THIS COMPUTER ONE VOICE IS SPREAD OVER SIX THREADS AND",
            ),
            (
                &whole,
                "ONE VOICE TAKES UP TO A WHOLE CORE OF A FAST COMPUTER.",
            ),
        ] {
            assert_eq!(
                n.lines[0],
                ("ULTRA · NO COMPROMISES".to_owned(), Tone::Accent)
            );
            assert!(
                text(n)
                    .iter()
                    .any(|t| t.contains("ABOUT SIX TIMES THE WORK OF HI"))
            );
            assert!(text(n).iter().any(|t| t == says), "{:?}", text(n));
            let dim: Vec<_> = n.lines.iter().rev().take(2).collect();
            assert_eq!(dim[0], &("CLICK TO CLOSE".to_owned(), Tone::Dim));
            assert_eq!(
                dim[1].0,
                "RIGHT-CLICK QUALITY'S SWITCH TO HIDE ITS INDICATOR, OR SHOW IT."
            );
        }
        assert!(
            text(&split)
                .join(" ")
                .contains("WITH MULTIPLE VOICES OR TOO SMALL OF A BUFFER")
        );
        assert!(
            text(&whole)
                .iter()
                .any(|t| t.ends_with("PLAYED LIVE, IT MAY DROP OUT."))
        );
        // The old note's widest line, as wide as any may be.
        let fonts = e.renderer.fonts();
        let wide = |t: &str| fonts.advance(t, split.size, Weight::Regular, 0.0);
        let old = [
            "THE CIRCUIT'S MODEL EXACTLY, AT ABOUT SEVEN TIMES THE WORK OF HI:",
            "ONE VOICE TAKES ABOUT A WHOLE CORE OF A FAST COMPUTER.",
            "IT IS MEANT FOR OFFLINE RENDERS (EXPORT, BOUNCE, FREEZE) AND SMALL",
            "PROJECTS ON POWERFUL COMPUTERS. PLAYED LIVE, IT MAY DROP OUT.",
            "RIGHT-CLICK QUALITY'S SWITCH TO HIDE ITS INDICATOR, OR SHOW IT.",
        ]
        .map(wide)
        .into_iter()
        .fold(0.0, f64::max);
        for n in [&split, &whole] {
            for (t, _) in &n.lines {
                assert!(wide(t) <= old + 1e-9, "{t}: {} against {old}", wide(t));
            }
            // Its box: left of the lamp's opening, within the panel (not pushed in by the
            // panel's edge), its foot within the panel's height.
            let (w, h) = n.extent(fonts);
            let (left, right) = (n.x - w / 2.0, n.x + w / 2.0);
            let top = n.y - n.size * 0.4 - h;
            assert!(
                left >= 0.0 && right < art::COL + 3023.0 - 53.6,
                "{left}..{right}"
            );
            assert!(top >= 0.0 && n.y <= art::PANEL_H, "{top}..{}", n.y);
        }
    }

    /// MIDI Learn drawn (decisions.md R34), for looking at: CUTOFF being learned (ringed, its
    /// note) with MAIN OUTPUT VOLUME's menu open, then POLY being learned after a reserved
    /// controller was moved, then the strip's INNER being learned with its menu open, each as
    /// `learn-<n>.png` in `$CA72_LEARN_PNG`, the folder.
    #[test]
    fn the_window_learning_png() {
        let Some(out) = std::env::var_os("CA72_LEARN_PNG") else {
            return;
        };
        let out = std::path::PathBuf::from(out);
        let dir = tempfile::tempdir().unwrap();
        let shot = |e: &mut Editing, name: &str| {
            window_frame(e).save_png(out.join(name)).unwrap();
        };
        let (mut e, _host, params) = editing_with(Library::at(dir.path()));
        params
            .midi_map
            .assign(learnable("volume"), crate::learn::Cc { channel: 0, cc: 7 });
        params.midi_map.arm(learnable("cutoff"));
        let p = at(&e, centre("volume"));
        right_click(&mut e, p);
        shot(&mut e, "learn-1.png");
        e.learning.close_menu();
        params.midi_map.arm(learnable("poly"));
        params.midi_map.refuse(1);
        shot(&mut e, "learn-2.png");
        params.midi_map.arm(learnable("inner"));
        let p = at(&e, centre("inner"));
        right_click(&mut e, p);
        shot(&mut e, "learn-3.png");
    }

    /// The settings key's menu with FREQUENCY DIALS (decisions.md R54), and oscillator 2's
    /// knob at +7 semitones with its tip, through each law: `settings-<n>.png` in
    /// `$CA72_SETTINGS_PNG`, the folder.
    #[test]
    fn the_window_settings_png() {
        use nih_plug::context::process::TestProcessContext;
        let Some(out) = std::env::var_os("CA72_SETTINGS_PNG") else {
            return;
        };
        let out = std::path::PathBuf::from(out);
        let dir = tempfile::tempdir().unwrap();
        let (mut e, _host, params) = editing_with(Library::at(dir.path()));
        let ps: Arc<dyn Params> = params.clone();
        let c = TestProcessContext::<crate::Ca72>::new(ps, 48_000.0, ProcessMode::Realtime);
        c.automate(params.osc2_frequency.as_ptr(), Pot::Two.wiper(7.0) as f32);
        let key = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Settings)));
        click(&mut e, key);
        window_frame(&mut e)
            .save_png(out.join("settings-1.png"))
            .unwrap();
        e.learning.close_menu();
        for (n, law) in [(2, Law::Semitones), (3, Law::Hardware)] {
            e.set_frequency_dials(law);
            let p = at(&e, centre("osc2_frequency"));
            go(&mut e, p);
            window_frame(&mut e)
                .save_png(out.join(format!("settings-{n}.png")))
                .unwrap();
        }
    }

    /// SETTINGS offers AGENTS in the standalone alone (decisions.md R-MCP): ALLOW and OFF, the
    /// one in force marked; off until chosen. `agents-<n>.png` in `$CA72_AGENTS_PNG`.
    #[test]
    fn agents_are_allowed_in_the_standalones_settings_menu() {
        let out = std::env::var_os("CA72_AGENTS_PNG").map(std::path::PathBuf::from);
        let (mut e, _host, _) = editing();
        let key = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Settings)));
        click(&mut e, key);
        assert!(
            !menu_items(&e).iter().any(|(t, _)| t == "AGENTS"),
            "not in a host"
        );
        e.learning.close_menu();
        let (mut e, _host, _) = editing_standalone();
        assert!(!e.settings.agents && !e.agents_allowed());
        click(&mut e, key);
        let items = menu_items(&e);
        let at = items
            .iter()
            .position(|(t, _)| t == "AGENTS")
            .expect("AGENTS");
        assert_eq!(
            items[at + 1..at + 3],
            [("ALLOW".to_owned(), true), ("OFF".to_owned(), false)]
        );
        if let Some(out) = &out {
            window_frame(&mut e)
                .save_png(out.join("agents-1.png"))
                .unwrap();
        }
        menu_item(&mut e, "ALLOW");
        assert!(e.settings.agents && e.agents_allowed());
        click(&mut e, key);
        let items = menu_items(&e);
        let at = items
            .iter()
            .position(|(t, _)| t == "AGENTS")
            .expect("AGENTS");
        assert_eq!(
            items[at + 1..at + 3],
            [("ALLOW".to_owned(), false), ("OFF".to_owned(), true)]
        );
        if let Some(out) = &out {
            window_frame(&mut e)
                .save_png(out.join("agents-2.png"))
                .unwrap();
        }
        menu_item(&mut e, "OFF");
        assert!(!e.agents_allowed());
    }

    /// The drawer's update check (decisions.md R27): its button the editor's, not the
    /// presets'; a check that could not be made offers the releases' page. No parameter is
    /// touched and the drawer stays open.
    #[test]
    fn the_drawer_checks_for_updates() {
        use crate::update::{RELEASES, VERSION, fake};
        let dir = tempfile::tempdir().unwrap();
        let (mut e, host, _params) = editing_with(Library::at(dir.path()));
        e.update = Update::with(fake::no_curl, fake::browser);
        e.draw();
        open(&mut e);
        let s = &e.browser.drawer.update;
        assert_eq!(
            (s.text.as_str(), s.button.as_str()),
            (format!("CA-72 {VERSION}").as_str(), "CHECK FOR UPDATES")
        );
        let button = in_drawer(&e, presets_ui::update_centre());
        go(&mut e, button);
        assert_eq!(e.browser.drawer.hover, Some(DrawerTarget::Update));
        click(&mut e, button);
        e.draw();
        assert_eq!(e.browser.drawer.update.text, "COULD NOT CHECK");
        click(&mut e, button);
        assert_eq!(fake::opened(), vec![RELEASES.to_owned()]);
        assert!(e.browser.open);
        assert_eq!(host.take(), vec![]);
    }

    /// The compact view, its controls shown, and presets temporarily revealing it, using
    /// the editor's actual frames for the visual preview.
    #[test]
    #[ignore = "writes images for a look"]
    fn the_lower_panel_png() {
        let Some(out) = std::env::var_os("CA72_PANEL_PNG") else {
            return;
        };
        let out = std::path::PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let (mut e, _, _) = editing_with(Library::at(dir.path()));
        window_frame(&mut e).save_png(out.join("full.png")).unwrap();
        let panel = on_strip(&e, strip_at(StripTarget::Bar(BarTarget::Panel)));
        click(&mut e, panel);
        window_frame(&mut e)
            .save_png(out.join("short.png"))
            .unwrap();
        open(&mut e);
        window_frame(&mut e)
            .save_png(out.join("presets.png"))
            .unwrap();
    }

    /// The window's frames stacked (the panel with the drawer over it, the strip, the bar),
    /// written to `$CA72_EDITOR_PNG` for a look.
    #[test]
    #[ignore = "writes an image for a look"]
    fn the_window_with_the_drawer_open_png() {
        let Some(out) = std::env::var_os("CA72_EDITOR_PNG") else {
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let (mut e, _host, _params) = editing_with(Library::at(dir.path()));
        e.draw();
        open(&mut e);
        let i = row_of(&e, "Brass Choir");
        let pt = in_drawer(
            &e,
            presets_ui::row_centre(&e.browser.drawer, i).expect("shown"),
        );
        click(&mut e, pt);
        let pt = in_drawer(
            &e,
            presets_ui::row_centre(&e.browser.drawer, i + 1).expect("shown"),
        );
        go(&mut e, pt);
        window_frame(&mut e).save_png(out).unwrap();
    }
}

/// The editor in a real window on Windows, in a host that resizes windows (decisions.md R26).
/// Opening the presets' drawer, shutting it and dragging the grip ask the host for another size
/// from within the editor's event handler. The host may resize the editor's window right there,
/// from within `IPlugFrame::resizeView()` (here `resizes_editor`), or the editor's own resize may
/// run while it still handles the press (the drawer taking the keyboard sent its window messages
/// from within the handler). Either way baseview gave the busy handler a `WM_SIZE`: a panic inside
/// its window procedure, which aborted the host's process (0.1.0, in Sonar). These tests aborted
/// the test process then. And Windows repainting the editor's window, in a host whose window
/// paints over it (decisions.md R28). The windows are the test's own, off the screen (or, where
/// the editor's pixels are read, on it but invisible), and the pointer is never over them; the
/// presets come from an empty folder of the test's.
#[cfg(all(test, target_os = "windows"))]
#[allow(unsafe_code)]
mod windows_window_tests {
    use std::path::PathBuf;
    use std::ptr::null_mut;
    use std::sync::MutexGuard;

    use ca72_panel::presets::BarTarget;
    use nih_plug::context::PluginApi;
    use nih_plug::editor::ParentWindowHandle;
    use nih_plug::wrapper::state::PluginState;
    use winapi::shared::minwindef::{LPARAM, WPARAM};
    use winapi::shared::windef::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, HBRUSH, HWND, RECT};
    use winapi::um::wingdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC,
        DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDIBits, GetPixel, SRCCOPY, SelectObject,
    };
    use winapi::um::winuser::{
        COLOR_BTNFACE, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GW_CHILD,
        GetClientRect, GetDC, GetWindow, HWND_BOTTOM, LWA_ALPHA, MK_LBUTTON, MSG, PM_REMOVE,
        PW_RENDERFULLCONTENT, PeekMessageW, PostMessageW, PrintWindow, RDW_ALLCHILDREN, RDW_ERASE,
        RDW_INVALIDATE, RDW_NOCHILDREN, RDW_UPDATENOW, RedrawWindow, RegisterClassW, ReleaseDC,
        SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SetLayeredWindowAttributes,
        SetThreadDpiAwarenessContext, SetWindowPos, TranslateMessage, WM_LBUTTONDOWN, WM_LBUTTONUP,
        WM_MOUSEMOVE, WNDCLASSW, WS_CLIPCHILDREN, WS_EX_LAYERED, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP, WS_VISIBLE,
    };
    use winapi::um::winuser::{
        DLGC_WANTALLKEYS, MK_RBUTTON, SendMessageW, WM_GETDLGCODE, WM_RBUTTONDOWN, WM_RBUTTONUP,
    };

    use super::*;
    use crate::opening::DOZE_AFTER;

    /// The host's window. Off the screen, where nothing drawn in it shows, its painting kept
    /// off the editor's window (`WS_CLIPCHILDREN`). Or as a host's dialog (REAPER's FX window):
    /// painting its background over the editor's window whenever it is repainted, and on the
    /// screen, so that the editor's pixels can be read; but at the bottom of the windows, an
    /// alpha of 1 in 255, and letting the pointer through to whatever is under it. The host
    /// repaints it as it resizes it, from within the editor's request.
    #[derive(Clone, Copy, PartialEq)]
    enum HostWindow {
        OffScreen,
        Dialog,
    }

    /// A host that grows its own window (the editor's parent) to the size the editor asks, and
    /// with `resizes_editor` the editor's window as well, from within the request; with
    /// `repaints`, it then repaints both windows there and then.
    struct WindowHost {
        editor: Arc<Ca72Editor>,
        parent: usize,
        resizes_editor: bool,
        repaints: bool,
        resizes: AtomicU32,
        /// The gestures the editor began.
        gestures: AtomicU32,
        deferred: AtomicBool,
        refused: AtomicBool,
        pending: Mutex<Vec<(u64, (u32, u32))>>,
    }

    impl GuiContext for WindowHost {
        fn plugin_api(&self) -> PluginApi {
            PluginApi::Vst3
        }
        fn request_resize(&self) -> bool {
            self.resizes.fetch_add(1, Ordering::Relaxed);
            let (request, size) = self.editor.resize_request();
            if self.deferred.load(Ordering::Relaxed) {
                self.pending
                    .lock()
                    .expect("resize requests")
                    .push((request, size));
                return true;
            }
            self.answer(request, size, !self.refused.load(Ordering::Relaxed))
        }
        fn resize_response_will_follow(&self) -> bool {
            true
        }
        unsafe fn raw_begin_set_parameter(&self, _param: ParamPtr) {
            self.gestures.fetch_add(1, Ordering::Relaxed);
        }
        unsafe fn raw_set_parameter_normalized(&self, _param: ParamPtr, _normalized: f32) {}
        unsafe fn raw_end_set_parameter(&self, _param: ParamPtr) {}
        fn get_state(&self) -> PluginState {
            panic!("the editor does not read the state")
        }
        fn set_state(&self, _state: PluginState) {}
    }

    impl WindowHost {
        fn answer(&self, request: u64, (w, h): (u32, u32), accepted: bool) -> bool {
            if !accepted {
                self.editor.resize_result(request, false);
                return false;
            }
            let (w, h) = (w as i32, h as i32);
            let parent = self.parent as HWND;
            let flags = SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE;
            // SAFETY: the test's own windows, on the thread that made them (the editor asks from
            // its event handler).
            unsafe {
                SetWindowPos(parent, null_mut(), 0, 0, w, h, flags);
                if self.resizes_editor {
                    SetWindowPos(GetWindow(parent, GW_CHILD), null_mut(), 0, 0, w, h, flags);
                }
                if self.repaints {
                    let flags = RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_ALLCHILDREN;
                    RedrawWindow(parent, null_mut(), null_mut(), flags);
                }
            }
            self.editor.resize_result(request, true);
            true
        }
    }

    // Native capture and focus are shared by the desktop, including across test threads.
    static DESKTOP: Mutex<()> = Mutex::new(());

    /// The editor's window open in the host's, at the narrowest width and a pixel a point.
    struct Opened {
        parent: HWND,
        editor: Arc<Ca72Editor>,
        host: Arc<WindowHost>,
        /// What the audio reports, as the plug-in's processing publishes it.
        meters: Arc<Meters>,
        handle: baseview::WindowHandle,
        _presets: tempfile::TempDir,
        // Last to drop: cleanup and all owned window state precede the next fixture.
        _desktop: MutexGuard<'static, ()>,
    }

    impl Opened {
        fn new(resizes_editor: bool, host_kind: HostWindow) -> Self {
            let desktop = DESKTOP
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            // The windows in pixels whatever the screen's scale (baseview sets the process's
            // awareness only once its first window is open, and the tests share the process).
            // SAFETY: this thread's own setting.
            unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
            let presets = tempfile::tempdir().expect("a temporary folder");
            let params = Arc::new(Ca72Params::default());
            params.editor_width.store(MIN_WIDTH, Ordering::Relaxed);
            let meters = Arc::new(Meters::default());
            let editor = Arc::new(Ca72Editor::new(Arc::clone(&params), Arc::clone(&meters)));
            let (w, h) = editor.opening();
            editor.open.store(true, Ordering::Release);
            let parent = host_window(w, h, host_kind);
            let host = Arc::new(WindowHost {
                editor: Arc::clone(&editor),
                parent: parent as usize,
                resizes_editor,
                repaints: host_kind == HostWindow::Dialog,
                resizes: AtomicU32::new(0),
                gestures: AtomicU32::new(0),
                deferred: AtomicBool::new(false),
                refused: AtomicBool::new(false),
                pending: Mutex::new(Vec::new()),
            });
            let context: Arc<dyn GuiContext> = host.clone();
            let window_meters = Arc::clone(&meters);
            let dir: PathBuf = presets.path().to_path_buf();
            let handle = Window::open_parented(
                &window::Parent(ParentWindowHandle::Win32Hwnd(parent.cast())),
                WindowOpenOptions {
                    title: String::from("CA-72"),
                    size: Size::new(f64::from(w), f64::from(h)),
                    scale: WindowScalePolicy::ScaleFactor(1.0),
                },
                move |window: &mut Window<'_>| {
                    PanelWindow::new(
                        window,
                        context,
                        params,
                        window_meters,
                        w,
                        1.0,
                        Library::at(dir),
                        Arc::new(HostKeys::default()),
                    )
                },
            );
            pump(Duration::from_millis(300));
            Opened {
                parent,
                editor,
                host,
                meters,
                handle,
                _presets: presets,
                _desktop: desktop,
            }
        }

        /// The editor's window.
        fn window(&self) -> HWND {
            // SAFETY: the test's own window.
            unsafe { GetWindow(self.parent, GW_CHILD) }
        }

        /// Its size, in pixels (logical ones: its scale is 1).
        fn size(&self) -> (u32, u32) {
            let mut r = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            // SAFETY: the test's own window and a rectangle of this function's.
            unsafe { GetClientRect(self.window(), &mut r) };
            ((r.right - r.left) as u32, (r.bottom - r.top) as u32)
        }

        /// Drawing units to the window's pixels.
        fn at(&self, (x, y): (f64, f64)) -> (f64, f64) {
            let k = f64::from(self.size().0) / art::W;
            (x * k, y * k)
        }

        /// The pointer moved to `at` (pixels), pressed, moved `dx` across in steps, released.
        fn drag(&self, at: (f64, f64), dx: f64) {
            let lp = |(x, y): (f64, f64)| {
                (((y.round() as u32) << 16) | (x.round() as u32 & 0xFFFF)) as LPARAM
            };
            let w = self.window();
            // SAFETY: messages posted to the test's own window.
            unsafe {
                PostMessageW(w, WM_MOUSEMOVE, 0, lp(at));
                PostMessageW(w, WM_LBUTTONDOWN, MK_LBUTTON as WPARAM, lp(at));
            }
            pump(Duration::from_millis(50));
            for i in 1..=4 {
                let to = (at.0 + dx * f64::from(i) / 4.0, at.1);
                // SAFETY: as above.
                unsafe { PostMessageW(w, WM_MOUSEMOVE, MK_LBUTTON as WPARAM, lp(to)) };
                pump(Duration::from_millis(50));
            }
            // SAFETY: as above.
            unsafe { PostMessageW(w, WM_LBUTTONUP, 0, lp((at.0 + dx, at.1))) };
            pump(Duration::from_millis(400));
        }

        fn click_bar(&self, target: BarTarget) {
            let (x, y) = strip::centre(Rail::Plugin, StripTarget::Bar(target));
            self.drag(self.at((x, y)), 0.0);
        }

        /// A click on the preset's name, in the strip under the panel.
        fn click_name(&self) {
            self.click_bar(BarTarget::Name);
        }

        fn close(self) {
            drop(self);
        }
    }

    impl Drop for Opened {
        fn drop(&mut self) {
            // Also clean up after an assertion fails, on the thread that owns the HWNDs.
            self.handle.close();
            pump(Duration::from_millis(100));
            // SAFETY: the test's own parent; its child has processed its close request.
            unsafe { DestroyWindow(self.parent) };
        }
    }

    /// The host's window, `w` by `h`, of `kind`, with no button on the taskbar and not brought
    /// to the front, so that it does not take the developer's pointer or keys.
    fn host_window(w: u32, h: u32, kind: HostWindow) -> HWND {
        let (name, background, clips, see_through, at) = match kind {
            HostWindow::OffScreen => ("CA72TestHost\0", null_mut(), WS_CLIPCHILDREN, 0, -20000),
            HostWindow::Dialog => (
                "CA72TestDialog\0",
                (COLOR_BTNFACE + 1) as usize as HBRUSH,
                0,
                WS_EX_LAYERED | WS_EX_TRANSPARENT,
                0,
            ),
        };
        let class: Vec<u16> = name.encode_utf16().collect();
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(DefWindowProcW),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: null_mut(),
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: background,
            lpszMenuName: null_mut(),
            lpszClassName: class.as_ptr(),
        };
        // SAFETY: a window class of the test's (registered by the first test to get here; the
        // others' calls fail harmlessly) and a window of it, on this thread.
        let hwnd = unsafe {
            RegisterClassW(&wc);
            CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | see_through,
                class.as_ptr(),
                class.as_ptr(),
                WS_POPUP | WS_VISIBLE | clips,
                at,
                at,
                w as i32,
                h as i32,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
            )
        };
        assert!(!hwnd.is_null(), "a window for the test");
        if kind == HostWindow::Dialog {
            let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE;
            // SAFETY: the test's own window.
            unsafe {
                SetLayeredWindowAttributes(hwnd, 0, 1, LWA_ALPHA);
                SetWindowPos(hwnd, HWND_BOTTOM, 0, 0, 0, 0, flags);
            }
        }
        hwnd
    }

    /// The editor's window's pixels, 64 by 64 across it, as drawn there (`GetPixel` reads
    /// the window's own, whatever covers it on the screen, but nothing off the screen).
    fn pixels(o: &Opened) -> Vec<u32> {
        let (w, h) = o.size();
        let at = |i: u32, n: u32| (n * (2 * i + 1) / 128) as i32;
        let window = o.window();
        // SAFETY: the test's own window, and its device context, released.
        unsafe {
            let dc = GetDC(window);
            let p = (0..64)
                .flat_map(|j| (0..64).map(move |i| (at(i, w), at(j, h))))
                .map(|(x, y)| GetPixel(dc, x, y))
                .collect();
            ReleaseDC(window, dc);
            p
        }
    }

    /// Every pixel of the editor's window, as drawn there (as `pixels`, but all of them).
    fn every_pixel(o: &Opened) -> Vec<u32> {
        let (w, h) = o.size();
        let mut out = vec![0u32; (w * h) as usize];
        let window = o.window();
        // SAFETY: the test's own window and its device context, released; a device context
        // and a bitmap of this function's, deleted; `out` holds the bitmap's pixels.
        unsafe {
            let src = GetDC(window);
            let dc = CreateCompatibleDC(src);
            let bitmap = CreateCompatibleBitmap(src, w as i32, h as i32);
            let old = SelectObject(dc, bitmap.cast());
            BitBlt(dc, 0, 0, w as i32, h as i32, src, 0, 0, SRCCOPY);
            SelectObject(dc, old);
            let mut info: BITMAPINFO = std::mem::zeroed();
            info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            info.bmiHeader.biWidth = w as i32;
            info.bmiHeader.biHeight = -(h as i32);
            info.bmiHeader.biPlanes = 1;
            info.bmiHeader.biBitCount = 32;
            info.bmiHeader.biCompression = BI_RGB;
            GetDIBits(
                dc,
                bitmap,
                0,
                h,
                out.as_mut_ptr().cast(),
                &mut info,
                DIB_RGB_COLORS,
            );
            DeleteObject(bitmap.cast());
            DeleteDC(dc);
            ReleaseDC(window, src);
        }
        out
    }

    fn colours(pixels: &[u32]) -> usize {
        let mut c = pixels.to_vec();
        c.sort_unstable();
        c.dedup();
        c.len()
    }

    /// `window` captured as a screenshot tool does (`PrintWindow`), into a bitmap thrown away.
    fn print(window: HWND) {
        let mut r = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        // SAFETY: the test's own window; a device context and a bitmap of this function's,
        // deleted, and the screen's, released.
        unsafe {
            GetClientRect(window, &mut r);
            let screen = GetDC(null_mut());
            let dc = CreateCompatibleDC(screen);
            let bitmap = CreateCompatibleBitmap(screen, r.right, r.bottom);
            let old = SelectObject(dc, bitmap.cast());
            PrintWindow(window, dc, PW_RENDERFULLCONTENT);
            SelectObject(dc, old);
            DeleteObject(bitmap.cast());
            DeleteDC(dc);
            ReleaseDC(null_mut(), screen);
        }
    }

    /// This thread's messages, for `d`.
    fn pump(d: Duration) {
        let until = Instant::now() + d;
        // SAFETY: a message structure of this function's, for this thread's messages.
        let mut msg: MSG = unsafe { std::mem::zeroed() };
        while Instant::now() < until {
            // SAFETY: as above.
            while unsafe { PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) } != 0 {
                // SAFETY: a message just taken from this thread's queue.
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    /// The editor's window's size once it is `want`, or after 5 s, its messages handled meanwhile.
    fn settled(o: &Opened, want: (u32, u32)) -> (u32, u32) {
        let until = Instant::now() + Duration::from_secs(5);
        while o.size() != want && Instant::now() < until {
            pump(Duration::from_millis(20));
        }
        o.size()
    }

    /// The presets' drawer opens and shuts over the strip, the window as it is: the host never
    /// asked to resize it (A6; the drawer had grown the window below the strip), where the host
    /// grows its own window and where it resizes the editor's as well.
    fn the_drawer_opens_and_shuts(resizes_editor: bool) {
        let o = Opened::new(resizes_editor, HostWindow::OffScreen);
        let short = (MIN_WIDTH, height_for(MIN_WIDTH));
        assert_eq!(settled(&o, short), short);
        o.click_name();
        pump(Duration::from_millis(600));
        assert_eq!(o.size(), short, "the window as it was, the drawer open");
        o.click_name();
        pump(Duration::from_millis(600));
        assert_eq!(o.size(), short, "the window as it was, the drawer shut");
        assert_eq!(o.host.resizes.load(Ordering::Relaxed), 0, "the host asked");
        o.close();
    }

    #[test]
    fn the_drawer_opens_and_shuts_where_the_host_grows_its_own_window() {
        the_drawer_opens_and_shuts(false);
    }

    #[test]
    fn the_drawer_opens_and_shuts_where_the_host_resizes_the_editors_window() {
        the_drawer_opens_and_shuts(true);
    }

    /// Repeated height changes travel through actual native windows, including a host that
    /// resizes the child synchronously from inside the request. Presets temporarily reveal
    /// the lower panel, then restore the compact view, whose grip still changes the width.
    fn panel_sizes(resizes_editor: bool) {
        let o = Opened::new(resizes_editor, HostWindow::OffScreen);
        let full = (MIN_WIDTH, height_for_panel(MIN_WIDTH, true));
        let compact = (MIN_WIDTH, height_for_panel(MIN_WIDTH, false));
        for _ in 0..8 {
            o.click_bar(BarTarget::Panel);
            assert_eq!(settled(&o, compact), compact);
            assert_eq!(o.editor.size(), compact);
            o.click_bar(BarTarget::Panel);
            assert_eq!(settled(&o, full), full);
            assert_eq!(o.editor.size(), full);
        }
        assert_eq!(o.host.resizes.load(Ordering::Relaxed), 16);
        o.click_bar(BarTarget::Panel);
        assert_eq!(settled(&o, compact), compact);
        o.click_name();
        assert_eq!(settled(&o, full), full, "presets reveal the hidden panel");
        o.click_name();
        assert_eq!(
            settled(&o, compact),
            compact,
            "presets restore the compact view"
        );
        let grip = o.at((art::W - 12.0, DRAWER_TOP - 12.0));
        o.drag(grip, 120.0);
        let asked = o.editor.size();
        assert!(asked.0 > MIN_WIDTH, "the compact grip changes the width");
        assert_eq!(asked.1, height_for_panel(asked.0, false));
        assert_eq!(settled(&o, asked), asked);
        o.click_bar(BarTarget::Panel);
        let expanded = (asked.0, height_for_panel(asked.0, true));
        assert_eq!(settled(&o, expanded), expanded, "expanding keeps the width");
        o.close();
    }

    #[test]
    fn panel_sizes_where_the_host_grows_its_own_window() {
        panel_sizes(false);
    }

    #[test]
    fn panel_sizes_where_the_host_resizes_the_editors_window() {
        panel_sizes(true);
    }

    #[test]
    fn refused_and_delayed_panel_sizes_restore_the_native_window() {
        let o = Opened::new(true, HostWindow::OffScreen);
        let full = (MIN_WIDTH, height_for_panel(MIN_WIDTH, true));
        let compact = (MIN_WIDTH, height_for_panel(MIN_WIDTH, false));
        o.host.refused.store(true, Ordering::Relaxed);
        o.click_bar(BarTarget::Panel);
        assert_eq!(o.size(), full);
        o.host.refused.store(false, Ordering::Relaxed);
        o.host.deferred.store(true, Ordering::Relaxed);
        for _ in 0..3 {
            o.click_bar(BarTarget::Panel);
        }
        let requests = std::mem::take(&mut *o.host.pending.lock().expect("requests"));
        assert_eq!(requests.len(), 3);
        o.host.answer(requests[0].0, requests[0].1, false);
        pump(Duration::from_millis(200));
        assert_eq!(
            o.size(),
            compact,
            "an obsolete refusal does not change the latest view"
        );
        o.host.answer(requests[1].0, requests[1].1, true);
        o.host.answer(requests[2].0, requests[2].1, false);
        pump(Duration::from_millis(500));
        assert_eq!(o.size(), full, "the last host-confirmed view is restored");
        assert_eq!(o.editor.size(), full);
        o.host.deferred.store(false, Ordering::Relaxed);
        o.click_bar(BarTarget::Panel);
        assert_eq!(settled(&o, compact), compact);
        o.close();
    }

    #[test]
    fn changing_panel_height_keeps_the_native_frame_after_repaint() {
        let o = Opened::new(true, HostWindow::Dialog);
        for expanded in [false, true, false, true] {
            o.click_bar(BarTarget::Panel);
            let asked = (MIN_WIDTH, height_for_panel(MIN_WIDTH, expanded));
            assert_eq!(settled(&o, asked), asked);
            pump(Duration::from_millis(400));
            let before = every_pixel(&o);
            assert!(colours(&before) >= 100, "the panel is drawn");
            let flags = RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_ALLCHILDREN;
            // SAFETY: the test's own windows on their creating thread.
            unsafe { RedrawWindow(o.parent, null_mut(), null_mut(), flags) };
            pump(Duration::from_millis(200));
            assert_eq!(
                every_pixel(&o),
                before,
                "repainting preserves the complete frame"
            );
        }
        o.close();
    }

    #[test]
    fn delayed_width_refusal_preserves_the_native_controls_after_repaint() {
        let o = Opened::new(true, HostWindow::Dialog);
        let original = o.size();
        o.host.deferred.store(true, Ordering::Relaxed);
        let grip = o.at((art::W - 12.0, art::H - 12.0));
        o.drag(grip, 120.0);
        assert!(o.size().0 > original.0);
        let requests = std::mem::take(&mut *o.host.pending.lock().expect("requests"));
        let &(request, size) = requests.last().expect("width request");
        o.host.answer(request, size, false);
        assert_eq!(settled(&o, original), original);
        pump(Duration::from_millis(300));
        let before = every_pixel(&o);
        assert!(colours(&before) >= 100, "the restored controls are drawn");
        let flags = RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_ALLCHILDREN;
        // SAFETY: the test's own windows on their creating thread.
        unsafe { RedrawWindow(o.parent, null_mut(), null_mut(), flags) };
        pump(Duration::from_millis(200));
        assert!(
            every_pixel(&o) == before,
            "repainting preserves the restored controls"
        );
        o.close();
    }

    /// A press with no move of the pointer before it (the pointer set where it is, or the
    /// window come under it) lands where it is: POLY's tab, pressed so, is chosen. baseview gave
    /// the press without its place, and the editor took it where the pointer had last been
    /// seen (in Cubase, a first click that did nothing; baseview's PATCHES.md, change 5).
    #[test]
    fn a_press_with_no_move_before_it_lands_where_it_is() {
        let o = Opened::new(false, HostWindow::OffScreen);
        let (x, y) = o.at(strip::centre(
            Rail::Plugin,
            StripTarget::Tab(strip::Bank::Mode, 1),
        ));
        let at = (((y.round() as u32) << 16) | (x.round() as u32 & 0xFFFF)) as LPARAM;
        let w = o.window();
        // SAFETY: messages posted to the test's own window.
        unsafe { PostMessageW(w, WM_LBUTTONDOWN, MK_LBUTTON as WPARAM, at) };
        pump(Duration::from_millis(60));
        // SAFETY: as above.
        unsafe { PostMessageW(w, WM_LBUTTONUP, 0, at) };
        pump(Duration::from_millis(300));
        assert!(
            o.host.gestures.load(Ordering::Relaxed) >= 1,
            "POLY's tab not chosen"
        );
        o.close();
    }

    /// The window shown where it changed (decisions.md R58, R59: on Windows the
    /// changed places alone are turned and sent to the window) is the window shown whole: the
    /// synth playing (QUALITY's opening following it, the voices' drops), a knob turned, the
    /// presets' drawer sliding open and shut, then all of it still; every pixel of the window
    /// then as when Windows has the editor repaint all of it. Still is the same frame at every
    /// look for longer than anything on the panel holds still before it moves again: at LO, a
    /// new instance's QUALITY (R-LO-DEFAULT), the hamster's wheel stops and he stands
    /// [`DOZE_AFTER`] before he dozes off (two frames alike 400 ms apart, there, were called
    /// still and he curled up after: R-STILL-LONGER).
    #[test]
    fn the_window_shown_in_places_is_the_window_shown_whole() {
        let o = Opened::new(false, HostWindow::Dialog);
        let play = |levels: f32, ms: u64| {
            let until = Instant::now() + Duration::from_millis(ms);
            let mut k = 0u32;
            while Instant::now() < until {
                let mut l = [0.0f32; LEVELS];
                for (v, l) in l.iter_mut().enumerate().take(4) {
                    *l = levels * (0.6 + 0.4 * ((k as usize + v) % 3) as f32 / 2.0);
                }
                let mask = if levels > 0.0 { 0b1111 } else { 0 };
                o.meters.publish(0.0, (0.5, 0.0), mask);
                o.meters.publish_levels(&l, levels);
                pump(Duration::from_millis(30));
                k += 1;
            }
        };
        play(0.6, 1500);
        let cutoff = &CONTROLS[ca72_panel::controls::index("cutoff").expect("a control")];
        o.drag(o.at(cutoff.centre()), 50.0);
        play(0.8, 800);
        o.click_name();
        play(0.5, 700);
        o.click_name();
        play(0.0, 300);
        // Still: the same frame at every look, 400 ms apart, for a second past the longest
        // anything holds still.
        let span = Duration::from_secs_f64(DOZE_AFTER + 1.0);
        let until = Instant::now() + Duration::from_secs(20);
        let mut still = every_pixel(&o);
        let mut since = Instant::now();
        while since.elapsed() < span {
            pump(Duration::from_millis(400));
            let now = every_pixel(&o);
            if now != still {
                assert!(Instant::now() < until, "the window never still");
                (still, since) = (now, Instant::now());
            }
        }
        assert!(colours(&still) >= 100, "the panel shown");
        // Windows asks for all of it again: the editor shows its whole frame.
        let flags = RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_ALLCHILDREN;
        // SAFETY: the test's own windows.
        unsafe { RedrawWindow(o.parent, null_mut(), null_mut(), flags) };
        pump(Duration::from_millis(200));
        let whole = every_pixel(&o);
        let apart = still.iter().zip(&whole).filter(|(a, b)| a != b).count();
        assert!(
            apart == 0,
            "{apart} pixels shown otherwise than in the whole frame"
        );
        o.close();
    }

    /// An editor whose window cannot be seen (its host's window minimised) draws its frame a
    /// few times a second instead of at the screen's rate, and shows none of it; seen again,
    /// it shows its whole frame (decisions.md R-EDITOR-CPU).
    #[test]
    fn an_unseen_window_draws_seldom_and_is_whole_when_seen_again() {
        use winapi::um::winuser::{SW_MINIMIZE, SW_RESTORE, ShowWindow};
        let o = Opened::new(false, HostWindow::Dialog);
        let play = |levels: f32, ms: u64| {
            let until = Instant::now() + Duration::from_millis(ms);
            let mut k = 0u32;
            while Instant::now() < until {
                let mut l = [0.0f32; LEVELS];
                for (v, l) in l.iter_mut().enumerate().take(4) {
                    *l = levels * (0.6 + 0.4 * ((k as usize + v) % 3) as f32 / 2.0);
                }
                o.meters.publish(0.0, (0.5, 0.0), 0b1111);
                o.meters.publish_levels(&l, levels);
                pump(Duration::from_millis(30));
                k += 1;
            }
        };
        let counts = || (DRAWN.load(Ordering::Relaxed), SHOWN.load(Ordering::Relaxed));
        play(0.6, 800);
        let (d0, s0) = counts();
        play(0.6, 2000);
        let (d1, s1) = counts();
        // SAFETY: the test's own window.
        unsafe { ShowWindow(o.parent, SW_MINIMIZE) };
        play(0.6, 400);
        let (d2, s2) = counts();
        play(0.6, 2000);
        let (d3, s3) = counts();
        let (seen, unseen) = (d1 - d0, d3 - d2);
        println!(
            "2 s seen: {seen} frames drawn, {} shown; 2 s unseen: {unseen} drawn, {} shown",
            s1 - s0,
            s3 - s2
        );
        assert!(seen >= 40, "{seen} frames drawn in 2 s while seen");
        assert!(unseen <= 12, "{unseen} frames drawn in 2 s while unseen");
        assert_eq!(s3 - s2, 0, "frames shown while unseen");
        // SAFETY: the test's own window.
        unsafe { ShowWindow(o.parent, SW_RESTORE) };
        play(0.0, 300);
        // Still (as `the_window_shown_in_places_is_the_window_shown_whole` waits for it), then
        // the window shown whole by Windows' own repaint: the same pixels.
        let span = Duration::from_secs_f64(crate::opening::DOZE_AFTER + 1.0);
        let until = Instant::now() + Duration::from_secs(20);
        let mut still = every_pixel(&o);
        let mut since = Instant::now();
        while since.elapsed() < span {
            pump(Duration::from_millis(400));
            let now = every_pixel(&o);
            if now != still {
                assert!(Instant::now() < until, "the window never still");
                (still, since) = (now, Instant::now());
            }
        }
        assert!(colours(&still) >= 100, "the panel shown again");
        let flags = RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_ALLCHILDREN;
        // SAFETY: the test's own windows.
        unsafe { RedrawWindow(o.parent, null_mut(), null_mut(), flags) };
        pump(Duration::from_millis(200));
        let whole = every_pixel(&o);
        let apart = still.iter().zip(&whole).filter(|(a, b)| a != b).count();
        assert_eq!(
            apart, 0,
            "{apart} pixels shown otherwise than in the whole frame"
        );
        o.close();
    }

    /// The grip asks for its width at the editor's next frame; the host resizes the editor's
    /// window from within that.
    #[test]
    fn the_grip_resizes_where_the_host_resizes_the_editors_window() {
        let o = Opened::new(true, HostWindow::OffScreen);
        let grip = o.at((
            (art::GRIP[0] + art::GRIP[2]) / 2.0,
            (art::GRIP[1] + art::GRIP[3]) / 2.0,
        ));
        o.drag(grip, 120.0);
        assert!(
            o.host.resizes.load(Ordering::Relaxed) >= 1,
            "the host asked"
        );
        let asked = o.editor.size();
        assert_eq!(
            settled(&o, asked),
            asked,
            "the window as the editor last asked"
        );
        o.close();
    }

    /// Windows repainting the editor's window, in a host whose window paints its background
    /// over it as REAPER's FX window does: the editor shows its frame again, though nothing in it
    /// changed. 0.1.0 left the host's background there until something in the panel changed.
    #[test]
    fn the_panel_is_shown_again_as_windows_repaints_its_window() {
        let o = Opened::new(false, HostWindow::Dialog);
        let panel = pixels(&o);
        assert!(
            colours(&panel) >= 100,
            "the panel shown ({} colours)",
            colours(&panel)
        );
        let repaint = |flags| {
            // SAFETY: the test's own window.
            unsafe { RedrawWindow(o.parent, null_mut(), null_mut(), flags) };
            pump(Duration::from_millis(100));
            pixels(&o)
        };
        // The host's window alone: its background over the editor's, which Windows does not
        // ask to repaint. So this host's painting reaches the editor's pixels here.
        let shown = repaint(RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_NOCHILDREN);
        assert_eq!(colours(&shown), 1, "the host's background");
        // Both windows, as a host repainting its own does; then a capture of the host's.
        let shown = repaint(RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_ALLCHILDREN);
        assert!(
            shown == panel,
            "the panel again within 100 ms of the host's window repainted ({} colours)",
            colours(&shown)
        );
        print(o.parent);
        pump(Duration::from_millis(100));
        let shown = pixels(&o);
        assert!(
            shown == panel,
            "the panel again within 100 ms of the host's window captured ({} colours)",
            colours(&shown)
        );
        // The drawer open over the strip, the window as it is: shown again as Windows repaints
        // it.
        o.click_name();
        pump(Duration::from_millis(600));
        let drawer = pixels(&o);
        assert!(drawer != panel, "the drawer shown");
        assert!(
            colours(&drawer) >= 100,
            "the panel and the drawer shown ({} colours)",
            colours(&drawer)
        );
        let shown = repaint(RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW | RDW_ALLCHILDREN);
        assert!(
            shown == drawer,
            "the panel and the drawer again ({} colours)",
            colours(&shown)
        );
        o.close();
    }

    /// While the editor holds the keyboard (the presets' drawer open, a control's menu), its
    /// window asks a host's dialog for every key: in REAPER's FX window, a dialog, an arrow moved
    /// REAPER's focus to one of its own buttons rather than reaching the drawer's list
    /// (decisions.md R34). And only then: otherwise the dialog keeps its keys.
    #[test]
    fn the_editor_asks_a_dialog_for_every_key_only_while_it_holds_them() {
        let o = Opened::new(false, HostWindow::OffScreen);
        let asks = || {
            // SAFETY: a message sent to the test's own window.
            let code = unsafe { SendMessageW(o.window(), WM_GETDLGCODE, 0, 0) };
            code & DLGC_WANTALLKEYS != 0
        };
        assert!(!asks(), "nothing held");
        o.click_name();
        assert!(asks(), "the presets' drawer open");
        o.click_name();
        pump(Duration::from_millis(400));
        assert!(!asks(), "the drawer shut");
        // A right click on CUTOFF FREQUENCY: its menu.
        let cutoff = &CONTROLS[ca72_panel::controls::index("cutoff").expect("a control")];
        let (x, y) = o.at(cutoff.centre());
        let at = (((y.round() as u32) << 16) | (x.round() as u32 & 0xFFFF)) as LPARAM;
        let w = o.window();
        // SAFETY: messages posted to the test's own window.
        unsafe {
            PostMessageW(w, WM_MOUSEMOVE, 0, at);
            PostMessageW(w, WM_RBUTTONDOWN, MK_RBUTTON as WPARAM, at);
        }
        pump(Duration::from_millis(50));
        // SAFETY: as above.
        unsafe { PostMessageW(w, WM_RBUTTONUP, 0, at) };
        pump(Duration::from_millis(300));
        assert!(asks(), "a control's menu open");
        // A click on the wood above the panel: the menu shut.
        o.drag(o.at((art::W / 2.0, 20.0)), 0.0);
        assert!(!asks(), "the menu shut");
        o.close();
    }
}
