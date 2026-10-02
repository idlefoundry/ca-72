//! The editor: the front panel (`ca72-panel`) in a window of its own (baseview), its frames
//! shown with softbuffer, and under it the plug-in's own controls (`ca72_panel::strip`: POLY,
//! VOICES with what plays in real time here, ANALOG and SPREAD). Every control operates its
//! parameter through the host, a drag one gesture (one undo step in hosts that keep them);
//! the POWER switch is the bypass; the OVERLOAD lamp and the wheels show what the audio
//! thread reports. A click on the name plate turns it over; the grip at the panel's bottom
//! right corner resizes the window. Under the strip the presets' bar, and their drawer over
//! the panel (`crate::presets`, decisions.md R10), which takes the keyboard while open.

use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};

use baseview::{
    Event, EventStatus, MouseButton, MouseEvent, ScrollDelta, Size, Window, WindowEvent,
    WindowHandler, WindowInfo, WindowOpenOptions, WindowScalePolicy,
};
use ca72_panel::art::{self, PLATE_TURN, POWER};
use ca72_panel::controls::{FEEDBACK_SILENT, feedback_silent};
use ca72_panel::presets::{
    BAR_H, BarRenderer, DRAWER_H, DRAWER_TOP, DrawerRenderer, bar_hit, drawer_hit, overlay,
};
use ca72_panel::strip::{self, Amount, STRIP_H, StripRenderer, StripScene, StripTarget};
use ca72_panel::{CONTROLS, Kind, Renderer, Scene, Target, interact};
use keyboard_types::Modifiers;
use nih_plug::prelude::*;

use crate::library::Library;
use crate::params::Ca72Params;
use crate::presets::Browser;

/// The share of the usable screen the editor opens at until it is resized.
const SCREEN_SHARE: f64 = 0.8;
/// The editor's width at first where the screen's size is unknown, in logical pixels.
pub const DEFAULT_WIDTH: u32 = 1380;
/// How small and large the grip makes it.
const MIN_WIDTH: u32 = 860;
const MAX_WIDTH: u32 = 3438;

/// Two presses within this long are a double click.
const DOUBLE_CLICK: Duration = Duration::from_millis(400);
/// A mouse wheel's turns are one gesture until it rests this long.
const SCROLL_REST: Duration = Duration::from_millis(400);
/// A trackpad's travel that makes a notch, in pixels.
const NOTCH: f64 = 40.0;

/// The editor's height at `width`: the panel in its proportions, and the strip and the
/// presets' bar under it.
pub fn height_for(width: u32) -> u32 {
    (f64::from(width) * (art::H + STRIP_H + BAR_H) / art::W).round() as u32
}

fn clamp_width(width: u32) -> u32 {
    width.clamp(MIN_WIDTH, MAX_WIDTH)
}

/// The width the editor opens at before it is resized: its share of the usable screen
/// (logical pixels), its height kept within the same share.
fn fitted_width(screen: Option<(f64, f64)>) -> u32 {
    screen.map_or(DEFAULT_WIDTH, |(w, h)| {
        let w = (w * SCREEN_SHARE).min(h * SCREEN_SHARE * art::W / (art::H + STRIP_H + BAR_H));
        clamp_width(w.round() as u32)
    })
}

/// What the audio thread reports to the editor after each block, and what the plug-in
/// measured of this machine when it was activated.
#[derive(Debug)]
pub struct Meters {
    overload: AtomicU32,
    bend: AtomicU32,
    modulation: AtomicU32,
    /// How many POLY voices play in real time here (u32::MAX: not measured), without and
    /// with FEEDBACK, at the host's largest block, frames.
    fits: AtomicU32,
    fits_feedback: AtomicU32,
    block: AtomicU32,
}

impl Default for Meters {
    fn default() -> Self {
        Meters {
            overload: AtomicU32::new(0.0f32.to_bits()),
            bend: AtomicU32::new(0.5f32.to_bits()),
            modulation: AtomicU32::new(0.0f32.to_bits()),
            fits: AtomicU32::new(u32::MAX),
            fits_feedback: AtomicU32::new(u32::MAX),
            block: AtomicU32::new(0),
        }
    }
}

impl Meters {
    /// How many POLY voices this machine plays in real time in blocks of `block` frames
    /// (`crate::engine::realtime_voices`).
    pub fn measured(&self, fits: u32, fits_feedback: u32, block: u32) {
        self.block.store(block, Ordering::Relaxed);
        self.fits.store(fits, Ordering::Relaxed);
        self.fits_feedback.store(fits_feedback, Ordering::Relaxed);
    }

    /// What [`Meters::measured`] last gave, without or with FEEDBACK: the voices and the
    /// block.
    pub fn fits(&self, feedback: bool) -> Option<(u32, u32)> {
        let n = if feedback {
            &self.fits_feedback
        } else {
            &self.fits
        }
        .load(Ordering::Relaxed);
        (n != u32::MAX).then(|| (n, self.block.load(Ordering::Relaxed)))
    }

    /// The OVERLOAD lamp (0..1) and a MIDI keyboard's wheels as the panel draws them.
    pub fn publish(&self, overload: f32, (bend, modulation): (f32, f32)) {
        self.overload.store(overload.to_bits(), Ordering::Relaxed);
        self.bend.store(bend.to_bits(), Ordering::Relaxed);
        self.modulation
            .store(modulation.to_bits(), Ordering::Relaxed);
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
        "pitch_wheel" => &p.pitch_wheel,
        "mod_wheel" => &p.mod_wheel,
        _ => return None,
    })
}

/// The editor, as the plug-in hands it to the host.
pub struct Ca72Editor {
    params: Arc<Ca72Params>,
    meters: Arc<Meters>,
    /// The host's scale (f32 bits; 0 none given: the system's, as on macOS).
    scale: AtomicU32,
    open: Arc<AtomicBool>,
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
        Ca72Editor {
            params,
            meters,
            scale: AtomicU32::new(0),
            open: Arc::new(AtomicBool::new(false)),
        }
    }

    fn host_scale(&self) -> Option<f32> {
        let s = f32::from_bits(self.scale.load(Ordering::Relaxed));
        (s > 0.0 && s.is_finite()).then_some(s)
    }
}

impl Editor for Ca72Editor {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        context: Arc<dyn GuiContext>,
    ) -> Box<dyn Any + Send> {
        let (width, height) = self.size();
        let scale = self.host_scale();
        let params = Arc::clone(&self.params);
        let meters = Arc::clone(&self.meters);
        let window = Window::open_parented(
            &window::Parent(parent),
            WindowOpenOptions {
                title: String::from("CA-72"),
                size: Size::new(f64::from(width), f64::from(height)),
                scale: scale.map_or(WindowScalePolicy::SystemScaleFactor, |s| {
                    WindowScalePolicy::ScaleFactor(f64::from(s))
                }),
            },
            move |window: &mut Window<'_>| {
                PanelWindow::new(
                    window,
                    context,
                    params,
                    meters,
                    width,
                    f64::from(scale.unwrap_or(1.0)),
                )
            },
        );
        self.open.store(true, Ordering::Release);
        Box::new(window::Handle::new(window, Arc::clone(&self.open)))
    }

    fn size(&self) -> (u32, u32) {
        let width = match self.params.editor_width.load(Ordering::Relaxed) {
            0 => fitted_width(screen::usable(self.host_scale().map(f64::from))),
            w => clamp_width(w),
        };
        (width, height_for(width))
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
    shown: bool,
    editing: Editing,
}

/// The panel being operated: what the pointer does to the parameters, and the scene drawn.
struct Editing {
    context: Arc<dyn GuiContext>,
    params: Arc<Ca72Params>,
    meters: Arc<Meters>,
    /// Physical pixels a logical pixel, and the window's width in logical pixels.
    dpr: f64,
    width: u32,
    renderer: Renderer,
    scene: Scene,
    /// The strip under the panel: its renderer and scene, an amount's slider being dragged,
    /// and each amount's last value but 0 (its switch turns it back on there).
    strip: StripRenderer,
    strip_scene: StripScene,
    sliding: Option<Amount>,
    last: [f32; 2],
    /// The presets: the bar's and the drawer's renderers, what they show and do, and the
    /// panel's frame with the drawer over it (while it shows).
    bar: BarRenderer,
    drawer: DrawerRenderer,
    pub(crate) browser: Browser,
    composed: Option<ca72_panel::Pixmap>,
    /// The pointer, in logical pixels.
    pointer: (f64, f64),
    hover: Option<Target>,
    drag: Option<Drag>,
    scroll: Option<Scroll>,
    last_press: Option<(Instant, Target)>,
    /// The name plate's name, and when it began turning over.
    named: usize,
    turning: Option<Instant>,
    /// The grip dragged: the pointer's x and the width at the press.
    resizing: Option<(f64, u32)>,
}

impl PanelWindow {
    fn new(
        window: &mut Window<'_>,
        context: Arc<dyn GuiContext>,
        params: Arc<Ca72Params>,
        meters: Arc<Meters>,
        width: u32,
        dpr: f64,
    ) -> Self {
        let physical = (
            (f64::from(width) * dpr).round() as u32,
            (f64::from(height_for(width)) * dpr).round() as u32,
        );
        let surface = window::Surface::new(window, physical);
        PanelWindow {
            surface,
            physical,
            shown: false,
            editing: Editing::new(
                context,
                params,
                meters,
                width,
                physical.0,
                dpr,
                Library::shared(),
            ),
        }
    }

    /// The window now `info`'s size: the surface and every frame drawn to it.
    fn resized(&mut self, info: &WindowInfo) {
        let size = info.physical_size();
        let physical = (size.width.max(1), size.height.max(1));
        if physical == self.physical && info.scale() == self.editing.dpr && self.shown {
            return;
        }
        self.physical = physical;
        self.editing.fit(
            info.scale(),
            info.logical_size().width.round() as u32,
            physical.0,
        );
        self.surface.resize(self.physical);
        self.shown = false;
    }

    fn present(&mut self) {
        let e = &self.editing;
        let panel = e.composed.as_ref().unwrap_or_else(|| e.renderer.frame());
        self.surface
            .present(self.physical, &[panel, e.strip.frame(), e.bar.frame()]);
    }
}

impl Editing {
    fn new(
        context: Arc<dyn GuiContext>,
        params: Arc<Ca72Params>,
        meters: Arc<Meters>,
        width: u32,
        physical_width: u32,
        dpr: f64,
        library: Library,
    ) -> Self {
        Editing {
            context,
            params,
            meters,
            dpr,
            width,
            renderer: Renderer::new(f64::from(physical_width) / art::W, dpr),
            scene: Scene::default(),
            strip: StripRenderer::new(f64::from(physical_width) / art::W),
            strip_scene: StripScene::default(),
            sliding: None,
            last: [0.5; 2],
            bar: BarRenderer::new(f64::from(physical_width) / art::W),
            drawer: DrawerRenderer::new(f64::from(physical_width) / art::W),
            browser: Browser::new(library),
            composed: None,
            pointer: (0.0, 0.0),
            hover: None,
            drag: None,
            scroll: None,
            last_press: None,
            named: 0,
            turning: None,
            resizing: None,
        }
    }

    /// The renderers at the window's scale: `dpr` physical pixels a logical one, `width`
    /// logical pixels wide, `physical_width` physical ones.
    fn fit(&mut self, dpr: f64, width: u32, physical_width: u32) {
        self.dpr = dpr;
        self.width = width;
        let scale = f64::from(physical_width) / art::W;
        self.renderer.rescale(scale, dpr);
        self.strip.rescale(scale);
        self.bar.rescale(scale);
        self.drawer.rescale(scale);
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

    /// The drawer's top now (drawing units), while it shows.
    fn drawer_top(&self) -> Option<f64> {
        let k = self.browser.reveal();
        (k > 0.0).then_some(DRAWER_TOP + (1.0 - k) * DRAWER_H)
    }

    /// The pointer's place in the drawer (drawer units), if it is over it.
    fn in_drawer(&self, (x, y): (f64, f64)) -> Option<(f64, f64)> {
        let top = self.drawer_top()?;
        (y >= top && y < art::H).then_some((x, y - top))
    }

    /// The pointer's place on the presets' bar (bar units), if it is over it.
    fn on_bar(&self, (x, y): (f64, f64)) -> Option<(f64, f64)> {
        (y >= art::H + STRIP_H).then_some((x, y - art::H - STRIP_H))
    }

    fn operated(&self, control: usize) -> Option<&dyn Operated> {
        param(&self.params, CONTROLS.get(control)?.param)
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

    fn set_power(&self, on: bool) {
        let s = self.setter();
        s.begin_set_parameter(&self.params.bypass);
        s.set_parameter(&self.params.bypass, !on);
        s.end_set_parameter(&self.params.bypass);
    }

    /// An amount's parameter (ANALOG, SPREAD).
    fn amount(&self, a: Amount) -> &FloatParam {
        match a {
            Amount::Analog => &self.params.analog,
            Amount::Spread => &self.params.spread,
        }
    }

    /// VOICES stepped by `by`, a gesture of its own.
    fn step_voices(&self, by: i32) {
        let s = self.setter();
        let p = &self.params.voices;
        s.begin_set_parameter(p);
        s.set_parameter(p, p.value() + by);
        s.end_set_parameter(p);
    }

    /// A press on the strip under the panel, at (`x`, `y`) in its units.
    fn strip_press(&mut self, x: f64, y: f64) {
        let Some(target) = strip::hit(x, y) else {
            return;
        };
        if let StripTarget::Switch(a) = target {
            // Off: 0, the amount kept; on: back to it (half way if it never was).
            let now = self.amount(a).unmodulated_normalized_value();
            let to = if now > 0.0 {
                self.last[a as usize] = now;
                0.0
            } else {
                self.last[a as usize]
            };
            let (s, p) = (self.setter(), self.amount(a));
            s.begin_set_parameter(p);
            s.set_parameter_normalized(p, to);
            s.end_set_parameter(p);
            return;
        }
        let s = self.setter();
        match target {
            StripTarget::Poly => {
                let p = &self.params.poly;
                s.begin_set_parameter(p);
                s.set_parameter(p, !p.value());
                s.end_set_parameter(p);
            }
            StripTarget::Fewer => self.step_voices(-1),
            StripTarget::More => self.step_voices(1),
            StripTarget::Slider(a) => {
                let p = self.amount(a);
                s.begin_set_parameter(p);
                s.set_parameter_normalized(p, strip::slider_value(a, x) as f32);
                self.sliding = Some(a);
            }
            StripTarget::Switch(_) => {}
        }
    }

    fn press(&mut self, modifiers: Modifiers) {
        let (x, y) = self.in_drawing(self.pointer);
        if let Some((dx, dy)) = self.in_drawer((x, y)) {
            if let Some(t) = drawer_hit(self.drawer.fonts(), &self.browser.drawer, dx, dy) {
                let setter = ParamSetter::new(self.context.as_ref());
                self.browser
                    .drawer_press(t, dx, &self.drawer, &self.params, &setter);
            }
            return;
        }
        if let Some((bx, by)) = self.on_bar((x, y)) {
            if let Some(t) = bar_hit(bx, by) {
                let setter = ParamSetter::new(self.context.as_ref());
                self.browser.bar_press(t, &self.params, &setter);
            }
            return;
        }
        if y >= art::H {
            self.strip_press(x, y - art::H);
            return;
        }
        let Some(target) = interact::hit(self.renderer.layout(), x, y) else {
            return;
        };
        let now = Instant::now();
        let double = self
            .last_press
            .is_some_and(|(t, last)| last == target && now - t < DOUBLE_CLICK);
        self.last_press = Some((now, target));
        match target {
            Target::Control(i) | Target::Legend(i, _) => {
                let kind = CONTROLS[i].kind;
                if double && !matches!(kind, Kind::Rocker { .. }) {
                    if let Some(p) = self.operated(i) {
                        self.set_once(i, f64::from(p.default()));
                    }
                    self.last_press = None;
                    return;
                }
                let value = self.value_of(i);
                if !matches!(kind, Kind::Rocker { .. })
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
            Target::Plate => {
                if self.turning.is_none() {
                    self.turning = Some(now);
                }
            }
            Target::Grip => {
                self.resizing = Some((self.pointer.0, self.width));
            }
        }
    }

    /// The pointer moved: a width the window is to be resized to, if the grip is dragged
    /// and the host agrees.
    fn moved(&mut self, modifiers: Modifiers) -> Option<u32> {
        if let Some((x0, w0)) = self.resizing {
            let width = clamp_width((f64::from(w0) + self.pointer.0 - x0).round().max(0.0) as u32);
            return self.resize(width).then_some(width);
        }
        let fine = modifiers.contains(Modifiers::SHIFT);
        let pointer = self.pointer;
        if let Some(a) = self.sliding {
            let (x, _) = self.in_drawing(pointer);
            let p = self.amount(a);
            self.setter()
                .set_parameter_normalized(p, strip::slider_value(a, x) as f32);
            return None;
        }
        let Some(d) = &mut self.drag else {
            let at = self.in_drawing(pointer);
            let (x, y) = at;
            let over_drawer = self.in_drawer(at);
            self.browser.drawer.hover = over_drawer
                .and_then(|(dx, dy)| drawer_hit(self.drawer.fonts(), &self.browser.drawer, dx, dy));
            self.browser.bar.hover = self.on_bar(at).and_then(|(bx, by)| bar_hit(bx, by));
            (self.hover, self.strip_scene.hover) =
                if over_drawer.is_some() || self.on_bar(at).is_some() {
                    (None, None)
                } else if y >= art::H {
                    (None, strip::hit(x, y - art::H))
                } else {
                    (interact::hit(self.renderer.layout(), x, y), None)
                };
            return None;
        };
        if (pointer.0 - d.from.0).abs() + (pointer.1 - d.from.1).abs() > 3.0 {
            d.moved = true;
        }
        if !d.moved {
            return None;
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
        None
    }

    fn released(&mut self) {
        if self.resizing.take().is_some() {
            return;
        }
        if let Some(a) = self.sliding.take() {
            self.setter().end_set_parameter(self.amount(a));
            return;
        }
        let Some(d) = self.drag.take() else {
            return;
        };
        let kind = CONTROLS[d.control].kind;
        let mut v = d.value;
        if !d.moved {
            let picked = match d.pressed {
                Target::Legend(_, k) => Some(k as f64 / 5.0),
                _ => {
                    let (cx, _) = CONTROLS[d.control].centre();
                    interact::click(&kind, v, self.in_drawing(d.from).0 - cx)
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
        if matches!(kind, Kind::Rocker { .. }) {
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
        if self.drag.is_some() || self.sliding.is_some() {
            return;
        }
        let (x, y) = self.in_drawing(self.pointer);
        if self.in_drawer((x, y)).is_some() {
            // The list scrolls, a row a notch.
            let dy = match delta {
                ScrollDelta::Lines { y, .. } => f64::from(y),
                ScrollDelta::Pixels { y, .. } => f64::from(y) / NOTCH,
            };
            if dy.abs() >= 0.5 {
                self.browser.scroll(if dy > 0.0 { -1 } else { 1 });
            }
            return;
        }
        if self.on_bar((x, y)).is_some() {
            return;
        }
        if y >= art::H {
            // VOICES steps with the wheel, a notch a voice.
            if let (Some(StripTarget::Fewer | StripTarget::More), ScrollDelta::Lines { y: dy, .. }) =
                (strip::hit(x, y - art::H), delta)
                && dy != 0.0
            {
                self.step_voices(if dy > 0.0 { 1 } else { -1 });
            }
            return;
        }
        let Some(Target::Control(i) | Target::Legend(i, _)) =
            interact::hit(self.renderer.layout(), x, y)
        else {
            return;
        };
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

    /// Asks the host for a window `width` wide: whether it agreed.
    fn resize(&mut self, width: u32) -> bool {
        if width == self.width {
            return false;
        }
        let old = self.params.editor_width.swap(width, Ordering::Relaxed);
        let agreed = self.context.request_resize();
        if agreed {
            self.width = width;
        } else {
            self.params.editor_width.store(old, Ordering::Relaxed);
        }
        agreed
    }

    /// The scene from the parameters, the controls operated and what the audio reports.
    fn update_scene(&mut self) {
        for (i, c) in CONTROLS.iter().enumerate() {
            self.scene.values[i] = match (&self.drag, &self.scroll) {
                (Some(d), _) if d.control == i && !matches!(c.kind, Kind::Rocker { .. }) => d.value,
                (_, Some(s)) if s.control == i => s.value,
                _ => self.value_of(i),
            };
        }
        self.scene.power = !self.params.bypass.value();
        self.scene.overload = Meters::load(&self.meters.overload);
        self.scene.midi = (
            Meters::load(&self.meters.bend),
            Meters::load(&self.meters.modulation),
        );
        self.scene.plate = match self.turning {
            None => (self.named, 0.0),
            Some(t0) => {
                let t = t0.elapsed().as_secs_f64();
                if t >= PLATE_TURN {
                    self.turning = None;
                    self.named = 1 - self.named;
                    (self.named, 0.0)
                } else {
                    let (angle, swapped) = art::plate_turn(t);
                    (if swapped { 1 - self.named } else { self.named }, angle)
                }
            }
        };
        self.scene.tip = self.tip();
        let p = &self.params;
        self.strip_scene.poly = p.poly.value();
        self.strip_scene.voices = u32::try_from(p.voices.value()).unwrap_or(4);
        self.strip_scene.fits = self.meters.fits(self.params.feedback.value() > 0.0);
        self.strip_scene.analog = f64::from(p.analog.unmodulated_normalized_value());
        self.strip_scene.spread = f64::from(p.spread.unmodulated_normalized_value());
    }

    /// The tip: over the control operated, else over the one under the pointer.
    fn tip(&self) -> Option<(String, f64, f64)> {
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
            Target::Plate | Target::Grip => None,
        }
    }

    /// Brings the frame up to the parameters and the gestures: whether it changed.
    fn draw(&mut self) -> bool {
        if self
            .scroll
            .as_ref()
            .is_some_and(|s| s.last.elapsed() > SCROLL_REST)
        {
            self.end_scroll();
        }
        self.update_scene();
        self.browser.tick(&self.params);
        let panel = self.renderer.render(&self.scene);
        let strip = self.strip.render(&self.strip_scene);
        let bar = self.bar.render(&self.browser.bar);
        // The drawer over the panel, while it shows (and moves).
        let k = self.browser.reveal();
        let mut drawer = false;
        if k > 0.0 {
            drawer = self.drawer.render(&self.browser.drawer);
            let sliding = self.browser.sliding();
            if panel || drawer || sliding || self.composed.is_none() {
                let frame = self.renderer.frame();
                let c = self.composed.get_or_insert_with(|| frame.clone());
                if (c.width(), c.height()) == (frame.width(), frame.height()) {
                    c.data_mut().copy_from_slice(frame.data());
                } else {
                    *c = frame.clone();
                }
                let top = ((DRAWER_TOP + (1.0 - k) * DRAWER_H) * self.renderer.scale()).round();
                overlay(c, self.drawer.frame(), top as i32);
                drawer = true;
            }
        } else if self.composed.take().is_some() {
            drawer = true;
        }
        panel || strip || bar || drawer
    }

    /// The pointer's events: a width the window is to be resized to, if the grip took it
    /// there; `None` for an event not the panel's.
    fn mouse(&mut self, e: MouseEvent) -> Option<Option<u32>> {
        match e {
            MouseEvent::CursorMoved {
                position,
                modifiers,
            } => {
                self.pointer = (position.x, position.y);
                return Some(self.moved(modifiers));
            }
            MouseEvent::ButtonPressed {
                button: MouseButton::Left,
                modifiers,
            } => self.press(modifiers),
            MouseEvent::ButtonReleased {
                button: MouseButton::Left,
                ..
            } => self.released(),
            MouseEvent::WheelScrolled { delta, modifiers } => self.scrolled(delta, modifiers),
            MouseEvent::CursorLeft => {
                if self.drag.is_none() {
                    self.hover = None;
                }
            }
            _ => return None,
        }
        Some(None)
    }
}

impl WindowHandler for PanelWindow {
    fn on_frame(&mut self, _window: &mut Window<'_>) {
        if self.editing.draw() || !self.shown {
            self.present();
            self.shown = true;
        }
    }

    fn on_event(&mut self, window: &mut Window<'_>, event: Event) -> EventStatus {
        match event {
            Event::Window(WindowEvent::Resized(info)) => {
                self.resized(&info);
                EventStatus::Captured
            }
            Event::Mouse(e) => {
                let status = match self.editing.mouse(e) {
                    None => EventStatus::Ignored,
                    Some(resize) => {
                        if let Some(width) = resize {
                            let size = Size::new(f64::from(width), f64::from(height_for(width)));
                            window.resize(size);
                            // On macOS baseview reports no resize of its own making (only a
                            // change of scale): the frames follow the new size here, else the
                            // host's window shrinks around a picture drawn at the old one (the
                            // owner, 2026-10-02). Elsewhere the report that follows finds the
                            // size already taken.
                            self.resized(&WindowInfo::from_logical_size(size, self.editing.dpr));
                        }
                        EventStatus::Captured
                    }
                };
                // A field of the drawer took the caret: the keyboard comes here.
                if std::mem::take(&mut self.editing.browser.wants_keys) {
                    window.focus();
                }
                status
            }
            // The drawer, while open, takes the keys (the host keeps its shortcuts).
            Event::Keyboard(k) => {
                let e = &mut self.editing;
                let setter = ParamSetter::new(e.context.as_ref());
                if e.browser.key(&k, &e.params, &setter) {
                    EventStatus::Captured
                } else {
                    EventStatus::Ignored
                }
            }
            _ => EventStatus::Ignored,
        }
    }
}

/// The usable part of the screen (less the menu bar, dock or taskbar), in logical pixels
/// (`scale` the host's, where the platform counts physical ones): macOS's main screen,
/// Windows' primary monitor, X11's screen (all its monitors together).
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
        use winapi::um::winuser::{GetDpiForSystem, SPI_GETWORKAREA, SystemParametersInfoW};
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
        // SAFETY: no arguments.
        let k = scale.unwrap_or_else(|| f64::from(unsafe { GetDpiForSystem() }) / 96.0);
        Some((
            f64::from(r.right - r.left) / k,
            f64::from(r.bottom - r.top) / k,
        ))
    }

    #[cfg(target_os = "linux")]
    pub fn usable(scale: Option<f64>) -> Option<(f64, f64)> {
        use x11::xlib;
        // SAFETY: a connection of our own, closed before it is left.
        unsafe {
            let d = xlib::XOpenDisplay(std::ptr::null());
            if d.is_null() {
                return None;
            }
            let s = xlib::XDefaultScreen(d);
            let (w, h) = (xlib::XDisplayWidth(d, s), xlib::XDisplayHeight(d, s));
            xlib::XCloseDisplay(d);
            let k = scale.unwrap_or(1.0);
            Some((f64::from(w) / k, f64::from(h) / k))
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    pub fn usable(_scale: Option<f64>) -> Option<(f64, f64)> {
        None
    }
}

/// The platform's windows: the host's parent window as baseview takes it, and softbuffer's
/// surface on baseview's window (the two use different versions of `raw-window-handle`).
#[allow(unsafe_code)]
mod window {
    use std::num::{NonZeroIsize, NonZeroU32};
    use std::ptr::NonNull;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use baseview::WindowHandle;
    use nih_plug::editor::ParentWindowHandle;
    use raw_window_handle::{
        HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
    };
    use raw_window_handle_06 as rwh6;

    /// The host's window the editor opens in.
    pub struct Parent(pub ParentWindowHandle);

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

    /// The open window, closed when dropped.
    pub struct Handle {
        window: WindowHandle,
        open: Arc<AtomicBool>,
    }

    impl Handle {
        pub fn new(window: WindowHandle, open: Arc<AtomicBool>) -> Self {
            Handle { window, open }
        }
    }

    // SAFETY: the host drops the editor's handle on its GUI thread, where it opened it; the
    // raw pointers inside are not used from any other.
    unsafe impl Send for Handle {}

    impl Drop for Handle {
        fn drop(&mut self) {
            self.open.store(false, Ordering::Release);
            self.window.close();
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

    /// Where the frames are shown; nothing if the platform's window could not be drawn in.
    pub struct Surface {
        inner: Option<(
            softbuffer::Context<Target>,
            softbuffer::Surface<Target, Target>,
        )>,
    }

    impl Surface {
        pub fn new(window: &baseview::Window<'_>, size: (u32, u32)) -> Self {
            let inner = target(window).and_then(|t| {
                let context = softbuffer::Context::new(t.clone()).ok()?;
                let surface = softbuffer::Surface::new(&context, t).ok()?;
                Some((context, surface))
            });
            let mut s = Surface { inner };
            s.resize(size);
            s
        }

        pub fn resize(&mut self, (w, h): (u32, u32)) {
            if let (Some((_, surface)), Some(w), Some(h)) =
                (&mut self.inner, NonZeroU32::new(w), NonZeroU32::new(h))
            {
                let _ = surface.resize(w, h);
            }
        }

        /// Shows frames (premultiplied RGBA, opaque), one under the other from the top left
        /// corner, in a window of `size` pixels.
        pub fn present(&mut self, (w, h): (u32, u32), frames: &[&ca72_panel::Pixmap]) {
            let Some((_, surface)) = &mut self.inner else {
                return;
            };
            let Ok(mut buffer) = surface.buffer_mut() else {
                return;
            };
            let (w, h) = (w as usize, h as usize);
            if buffer.len() < w * h {
                return;
            }
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
                            let p = &f.data()[(fy * fw + x) * 4..(fy * fw + x) * 4 + 4];
                            (u32::from(p[0]) << 16) | (u32::from(p[1]) << 8) | u32::from(p[2])
                        }
                        _ => 0x003b_2213,
                    };
                }
            }
            let _ = buffer.present();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use baseview::Point;
    use nih_plug::context::PluginApi;
    use nih_plug::wrapper::state::PluginState;

    use super::*;
    use ca72_panel::presets::{self as presets_ui, BarTarget, DrawerTarget, FieldId, RowAction};
    use keyboard_types::{Key, KeyState, KeyboardEvent};

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
        calls: Mutex<Vec<Call>>,
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
            PluginApi::Clap
        }
        fn request_resize(&self) -> bool {
            true
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

    fn editing() -> (Editing, Arc<Host>, Arc<Ca72Params>) {
        // A library that is not there (nothing of the user's read; these tests write none).
        editing_with(Library::at(
            std::env::temp_dir().join("ca72-editor-tests-no-library"),
        ))
    }

    fn editing_with(library: Library) -> (Editing, Arc<Host>, Arc<Ca72Params>) {
        let host = Arc::new(Host::default());
        let params = Arc::new(Ca72Params::default());
        let context: Arc<dyn GuiContext> = host.clone();
        let e = Editing::new(
            context,
            Arc::clone(&params),
            Arc::new(Meters::default()),
            1720,
            1720,
            1.0,
            library,
        );
        (e, host, params)
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

    #[test]
    fn the_plate_turns_over_and_back() {
        let (mut e, _host, _params) = editing();
        let p = at(&e, (art::PLATE_X + 20.0, art::PLATE_Y + 20.0));
        click(&mut e, p);
        assert!(e.turning.is_some());
        e.turning = e
            .turning
            .map(|t| t - Duration::from_secs_f64(PLATE_TURN * 0.6));
        e.update_scene();
        assert_eq!(e.scene.plate.0, 1, "past half way it shows the other name");
        e.turning = e.turning.map(|t| t - Duration::from_secs_f64(PLATE_TURN));
        e.update_scene();
        assert_eq!((e.named, e.scene.plate, e.turning), (1, (1, 0.0), None));
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
        // The panel's 529, the strip's 48 and the presets' bar's 40 under it.
        assert_eq!(height_for(1720), 617);
        assert_eq!(clamp_width(10), MIN_WIDTH);
        assert_eq!(clamp_width(99_999), MAX_WIDTH);
    }

    #[test]
    fn it_opens_at_four_fifths_of_the_screen_until_resized() {
        // A 14-inch MacBook Pro's screen less the menu bar; a 34-inch ultrawide's.
        assert_eq!(fitted_width(Some((1512.0, 949.0))), 1210);
        assert_eq!(fitted_width(Some((3440.0, 1400.0))), 2752);
        // Short and wide: the height's share sets it (the panel, the strip and the bar).
        assert_eq!(fitted_width(Some((3840.0, 800.0))), 1785);
        assert_eq!(height_for(1785), 640);
        assert_eq!(fitted_width(None), DEFAULT_WIDTH);
        assert_eq!(
            Ca72Params::default().editor_width.load(Ordering::Relaxed),
            0
        );
    }

    /// A point of the strip under the panel (its units) in the window.
    fn on_strip(e: &Editing, (x, y): (f64, f64)) -> Point {
        at(e, (x, art::H + y))
    }

    /// Where a strip target is, found by scanning its middle row.
    fn strip_at(target: StripTarget) -> (f64, f64) {
        let y = STRIP_H / 2.0;
        let xs: Vec<f64> = (0..3438)
            .map(f64::from)
            .filter(|&x| strip::hit(x, y) == Some(target))
            .collect();
        assert!(!xs.is_empty(), "{target:?} not found");
        (xs[xs.len() / 2], y)
    }

    /// POLY, VOICES, ANALOG and SPREAD from the strip under the panel: each its own
    /// parameter, a click one gesture, a slider's drag one gesture; an amount's switch turns
    /// it off and back on at its amount.
    #[test]
    fn the_strip_operates_poly_voices_analog_and_spread() {
        let (mut e, host, params) = editing();
        let p = on_strip(&e, strip_at(StripTarget::Poly));
        click(&mut e, p);
        let poly = params.poly.as_ptr();
        assert_eq!(
            host.take(),
            vec![Call::Begin(poly), Call::Set(poly, 1.0), Call::End(poly)]
        );
        let voices = params.voices.as_ptr();
        let p = on_strip(&e, strip_at(StripTarget::More));
        click(&mut e, p);
        // (4 to 5 of 2..10.)
        assert_eq!(
            host.take(),
            vec![
                Call::Begin(voices),
                Call::Set(voices, 0.375),
                Call::End(voices)
            ]
        );
        let p = on_strip(&e, strip_at(StripTarget::Fewer));
        e.mouse(MouseEvent::CursorMoved {
            position: p,
            modifiers: Modifiers::empty(),
        });
        e.mouse(MouseEvent::WheelScrolled {
            delta: ScrollDelta::Lines { x: 0.0, y: -1.0 },
            modifiers: Modifiers::empty(),
        });
        assert_eq!(
            host.take(),
            vec![
                Call::Begin(voices),
                Call::Set(voices, 0.125),
                Call::End(voices)
            ]
        );
        // ANALOG's slider: pressed at its middle, dragged to its end, one gesture.
        let analog = params.analog.as_ptr();
        let (x, y) = strip_at(StripTarget::Slider(Amount::Analog));
        let (from, to) = (on_strip(&e, (x, y)), on_strip(&e, (x + 2000.0, y)));
        go(&mut e, from);
        e.mouse(MouseEvent::ButtonPressed {
            button: MouseButton::Left,
            modifiers: Modifiers::empty(),
        });
        go(&mut e, to);
        e.mouse(MouseEvent::ButtonReleased {
            button: MouseButton::Left,
            modifiers: Modifiers::empty(),
        });
        let calls = host.take();
        assert_eq!(calls.first(), Some(&Call::Begin(analog)));
        assert_eq!(
            &calls[calls.len() - 2..],
            &[Call::Set(analog, 1.0), Call::End(analog)]
        );
        assert!(
            calls.iter().all(|c| match c {
                Call::Begin(p) | Call::Set(p, _) | Call::End(p) => *p == analog,
            }),
            "{calls:?}"
        );
        // SPREAD's switch: on from 0 at half way (it never was), off again keeping it.
        let spread = params.spread.as_ptr();
        let p = on_strip(&e, strip_at(StripTarget::Switch(Amount::Spread)));
        click(&mut e, p);
        assert_eq!(
            host.take(),
            vec![
                Call::Begin(spread),
                Call::Set(spread, 0.5),
                Call::End(spread)
            ]
        );
        // The strip draws what the parameters hold and what plays in real time here.
        e.meters.measured(6, 2, 512);
        e.update_scene();
        assert_eq!(e.strip_scene.fits, Some((6, 512)));
        assert!(e.draw());
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
        let widths = [
            e.renderer.frame().width(),
            e.strip.frame().width(),
            e.bar.frame().width(),
        ];
        assert_eq!(widths, [1500; 3]);
        let tall: u32 = [e.renderer.frame(), e.strip.frame(), e.bar.frame()]
            .iter()
            .map(|f| f.height())
            .sum();
        assert!(
            tall.abs_diff(height_for(1500)) <= 2,
            "{tall} for {}",
            height_for(1500)
        );
    }

    // ---- The presets (decisions.md R10).

    /// A point of the presets' bar (its units) in the window.
    fn on_bar(e: &Editing, (x, y): (f64, f64)) -> Point {
        at(e, (x, art::H + STRIP_H + y))
    }

    /// A point of the drawer (its units) in the window, the drawer open.
    fn in_drawer(e: &Editing, (x, y): (f64, f64)) -> Point {
        at(e, (x, DRAWER_TOP + y))
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
        click(e, on_bar(e, presets_ui::bar_centre(BarTarget::Name)));
        assert!(e.browser.open);
        std::thread::sleep(crate::presets::SLIDE + Duration::from_millis(20));
        e.draw();
        assert_eq!(e.browser.reveal(), 1.0);
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

    #[test]
    fn the_bar_opens_the_drawer_over_the_panel_and_a_row_sets_the_plugin() {
        let dir = tempfile::tempdir().unwrap();
        let (mut e, host, params) = editing_with(Library::at(dir.path()));
        e.draw();
        assert!(e.composed.is_none());
        open(&mut e);
        // Drawn over the panel's lower part: the frame shown differs from the panel's there.
        let c = e.composed.as_ref().expect("the drawer over the panel");
        let frame = e.renderer.frame();
        let row = |p: &ca72_panel::Pixmap, y: u32| {
            let w = p.width() as usize;
            p.data()[(y as usize * w) * 4..((y as usize + 1) * w) * 4].to_vec()
        };
        let low = (frame.height() as f64 * 0.9) as u32;
        let high = (frame.height() as f64 * 0.1) as u32;
        assert_ne!(row(c, low), row(frame, low));
        assert_eq!(row(c, high), row(frame, high));
        assert_eq!(e.browser.drawer.rows.len(), crate::library::factory().len());
        // A row sets every parameter it names (or its default), the preset remembered.
        host.take();
        let i = row_of(&e, "Undertow Growl");
        // Below the rows shown: the wheel scrolls the list to it.
        let mid = in_drawer(&e, presets_ui::field_centre(FieldId::SaveName, None));
        let list = Point {
            x: mid.x,
            y: mid.y - 120.0,
        };
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
        assert_eq!(preset(&params), "Undertow Growl");
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
        assert_eq!(e.browser.bar.name, "Undertow Growl");
        assert!(e.browser.drawer.rows[i].current);
        // Escape closes it.
        assert!(key(&mut e, Key::Escape));
        assert!(!e.browser.open);
        assert!(
            !key(&mut e, Key::Character("a".into())),
            "closed: the host's keys"
        );
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
        assert!(names.contains(&"Lead") && names.contains(&"Hollow Glider"));
        assert!(!names.contains(&"Bass"), "{names:?}");
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
            .position(|(t, _)| t == "whistle")
            .expect("a chip");
        let setter = ParamSetter::new(e.context.as_ref());
        e.browser
            .drawer_press(DrawerTarget::Chip(bass), 0.0, &e.drawer, &e.params, &setter);
        let names: Vec<&str> = e
            .browser
            .drawer
            .rows
            .iter()
            .map(|r| r.name.as_str())
            .collect();
        assert_eq!(names, ["Cruising Whistle"]);
        let pt = on_bar(&e, presets_ui::bar_centre(BarTarget::Next));
        click(&mut e, pt);
        assert_eq!(preset(&params), "Cruising Whistle");
    }

    #[test]
    fn saved_renamed_tagged_deleted_and_restored_from_the_drawer() {
        let dir = tempfile::tempdir().unwrap();
        let (mut e, _host, params) = editing_with(Library::at(dir.path()));
        // SAVE… opens the drawer at the name; Tab to the tags; Enter saves.
        e.draw();
        let pt = on_bar(&e, presets_ui::bar_centre(BarTarget::Save));
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
        // Saving as a factory preset's name says it replaces it.
        let setter = ParamSetter::new(e.context.as_ref());
        e.browser.drawer_press(
            DrawerTarget::Field(FieldId::SaveName),
            0.0,
            &e.drawer,
            &e.params,
            &setter,
        );
        typed(&mut e, "Bass");
        assert!(e.browser.drawer.replace);
        assert!(e.browser.drawer.hint.contains("factory"));
        // Renamed in place (the plug-in keeps it), its tags edited.
        let i = row_of(&e, "My Bass");
        let (x, y) =
            presets_ui::action_centre(&e.browser.drawer, i, RowAction::Rename).expect("shown");
        let pt = in_drawer(&e, (x, y));
        click(&mut e, pt);
        assert_eq!(e.browser.drawer.focus, Some(FieldId::Edit));
        for _ in 0.."My Bass".len() {
            key(&mut e, Key::Backspace);
        }
        typed(&mut e, "Your Bass");
        key(&mut e, Key::Enter);
        assert!(lib.find("Your Bass").is_some() && lib.find("My Bass").is_none());
        assert_eq!(preset(&params), "Your Bass");
        let i = row_of(&e, "Your Bass");
        let (x, y) =
            presets_ui::action_centre(&e.browser.drawer, i, RowAction::Tags).expect("shown");
        let pt = in_drawer(&e, (x, y));
        click(&mut e, pt);
        key(&mut e, Key::End);
        typed(&mut e, ", deep");
        key(&mut e, Key::Enter);
        assert_eq!(
            lib.find("Your Bass").unwrap().sound.tags,
            ["bass", "test", "deep"]
        );
        // Deleted after asking.
        let i = row_of(&e, "Your Bass");
        let (x, y) =
            presets_ui::action_centre(&e.browser.drawer, i, RowAction::Delete).expect("shown");
        let pt = in_drawer(&e, (x, y));
        click(&mut e, pt);
        assert!(lib.find("Your Bass").is_some(), "deleted before asked");
        let (x, y) =
            presets_ui::action_centre(&e.browser.drawer, i, RowAction::Confirm).expect("asked");
        let pt = in_drawer(&e, (x, y));
        click(&mut e, pt);
        assert!(lib.find("Your Bass").is_none());
        e.draw();
        assert!(!e.browser.bar.found, "the bar says the preset is gone");
        // A factory preset deleted, then restored; the star makes a favourite.
        let i = row_of(&e, "Lead");
        let setter = ParamSetter::new(e.context.as_ref());
        for a in [RowAction::Delete, RowAction::Confirm] {
            e.browser.drawer_press(
                DrawerTarget::Action(i, a),
                0.0,
                &e.drawer,
                &e.params,
                &setter,
            );
        }
        assert!(e.browser.drawer.rows.iter().all(|r| r.name != "Lead"));
        e.browser
            .drawer_press(DrawerTarget::Restore, 0.0, &e.drawer, &e.params, &setter);
        let i = row_of(&e, "Lead");
        e.browser
            .drawer_press(DrawerTarget::Star(i), 0.0, &e.drawer, &e.params, &setter);
        assert!(lib.find("Lead").unwrap().favorite);
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
        let i = row_of(&e, "Brass Tutti");
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
        e.draw();
        let frames = [e.composed.as_ref().unwrap(), e.strip.frame(), e.bar.frame()];
        let w = frames[0].width();
        let h: u32 = frames.iter().map(|f| f.height()).sum();
        let mut all = ca72_panel::Pixmap::new(w, h).unwrap();
        let mut y = 0usize;
        for f in frames {
            let n = (f.width() * f.height() * 4) as usize;
            let start = y * w as usize * 4;
            all.data_mut()[start..start + n].copy_from_slice(f.data());
            y += f.height() as usize;
        }
        all.save_png(out).unwrap();
    }
}
