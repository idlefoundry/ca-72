//! The editor: the front panel (`ca72-panel`) in a window of its own (baseview), its frames
//! shown with softbuffer. Every control operates its parameter through the host, a drag
//! one gesture (one undo step in hosts that keep them); the POWER switch is the bypass; the
//! OVERLOAD lamp and the wheels show what the audio thread reports. A click on the name
//! plate turns it over; the grip at the bottom right corner resizes the window.

use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};

use baseview::{
    Event, EventStatus, MouseButton, MouseEvent, ScrollDelta, Size, Window, WindowEvent,
    WindowHandler, WindowOpenOptions, WindowScalePolicy,
};
use ca72_panel::art::{self, PLATE_TURN, POWER};
use ca72_panel::{CONTROLS, Kind, Renderer, Scene, Target, interact};
use keyboard_types::Modifiers;
use nih_plug::prelude::*;

use crate::params::Ca72Params;

/// The editor's width at first, in logical pixels: half a panel unit a pixel.
pub const DEFAULT_WIDTH: u32 = 1720;
/// How small and large the grip makes it.
const MIN_WIDTH: u32 = 860;
const MAX_WIDTH: u32 = 3438;

/// Two presses within this long are a double click.
const DOUBLE_CLICK: Duration = Duration::from_millis(400);
/// A mouse wheel's turns are one gesture until it rests this long.
const SCROLL_REST: Duration = Duration::from_millis(400);
/// A trackpad's travel that makes a notch, in pixels.
const NOTCH: f64 = 40.0;

/// The editor's height at `width`, in the panel's proportions.
pub fn height_for(width: u32) -> u32 {
    (f64::from(width) * art::H / art::W).round() as u32
}

fn clamp_width(width: u32) -> u32 {
    width.clamp(MIN_WIDTH, MAX_WIDTH)
}

/// What the audio thread reports to the editor after each block.
#[derive(Debug)]
pub struct Meters {
    overload: AtomicU32,
    bend: AtomicU32,
    modulation: AtomicU32,
}

impl Default for Meters {
    fn default() -> Self {
        Meters {
            overload: AtomicU32::new(0.0f32.to_bits()),
            bend: AtomicU32::new(0.5f32.to_bits()),
            modulation: AtomicU32::new(0.0f32.to_bits()),
        }
    }
}

impl Meters {
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
                    f64::from(scale.unwrap_or(1.0)),
                )
            },
        );
        self.open.store(true, Ordering::Release);
        Box::new(window::Handle::new(window, Arc::clone(&self.open)))
    }

    fn size(&self) -> (u32, u32) {
        let width = clamp_width(self.params.editor_width.load(Ordering::Relaxed));
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
    /// Physical pixels a logical pixel.
    dpr: f64,
    renderer: Renderer,
    scene: Scene,
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
        dpr: f64,
    ) -> Self {
        let width = clamp_width(params.editor_width.load(Ordering::Relaxed));
        let physical = (
            (f64::from(width) * dpr).round() as u32,
            (f64::from(height_for(width)) * dpr).round() as u32,
        );
        let surface = window::Surface::new(window, physical);
        PanelWindow {
            surface,
            physical,
            shown: false,
            editing: Editing::new(context, params, meters, physical.0, dpr),
        }
    }

    fn present(&mut self) {
        let frame = self.editing.renderer.frame();
        self.surface
            .present(self.physical, frame.width(), frame.height(), frame.data());
    }
}

impl Editing {
    fn new(
        context: Arc<dyn GuiContext>,
        params: Arc<Ca72Params>,
        meters: Arc<Meters>,
        physical_width: u32,
        dpr: f64,
    ) -> Self {
        Editing {
            context,
            params,
            meters,
            dpr,
            renderer: Renderer::new(f64::from(physical_width) / art::W, dpr),
            scene: Scene::default(),
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

    fn setter(&self) -> ParamSetter<'_> {
        ParamSetter::new(self.context.as_ref())
    }

    /// The pointer's place in the drawing.
    fn in_drawing(&self, (x, y): (f64, f64)) -> (f64, f64) {
        let k = self.dpr / self.renderer.scale();
        (x * k, y * k)
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

    fn press(&mut self, modifiers: Modifiers) {
        let (x, y) = self.in_drawing(self.pointer);
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
                self.resizing = Some((
                    self.pointer.0,
                    clamp_width(self.params.editor_width.load(Ordering::Relaxed)),
                ));
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
        let Some(d) = &mut self.drag else {
            let (x, y) = self.in_drawing(pointer);
            self.hover = interact::hit(self.renderer.layout(), x, y);
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
        if self.drag.is_some() {
            return;
        }
        let (x, y) = self.in_drawing(self.pointer);
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
        let old = self.params.editor_width.swap(width, Ordering::Relaxed);
        if old == width {
            return false;
        }
        let agreed = self.context.request_resize();
        if !agreed {
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
                Some((
                    format!("{}: {}", c.label, p.text(self.scene.values[i] as f32)),
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
        self.renderer.render(&self.scene)
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
                let size = info.physical_size();
                self.physical = (size.width.max(1), size.height.max(1));
                let e = &mut self.editing;
                e.dpr = info.scale();
                e.renderer
                    .rescale(f64::from(self.physical.0) / art::W, e.dpr);
                self.surface.resize(self.physical);
                self.shown = false;
                EventStatus::Captured
            }
            Event::Mouse(e) => match self.editing.mouse(e) {
                None => EventStatus::Ignored,
                Some(resize) => {
                    if let Some(width) = resize {
                        window.resize(Size::new(f64::from(width), f64::from(height_for(width))));
                    }
                    EventStatus::Captured
                }
            },
            _ => EventStatus::Ignored,
        }
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

        /// Shows a frame (premultiplied RGBA, opaque, `fw` by `fh`) in a window of `size`
        /// pixels, from its top left corner.
        pub fn present(&mut self, (w, h): (u32, u32), fw: u32, fh: u32, rgba: &[u8]) {
            let Some((_, surface)) = &mut self.inner else {
                return;
            };
            let Ok(mut buffer) = surface.buffer_mut() else {
                return;
            };
            let (w, h, fw) = (w as usize, h as usize, fw as usize);
            if buffer.len() < w * h {
                return;
            }
            for y in 0..h {
                let row = &mut buffer[y * w..(y + 1) * w];
                for (x, out) in row.iter_mut().enumerate() {
                    *out = if x < fw && y < fh as usize {
                        let p = &rgba[(y * fw + x) * 4..(y * fw + x) * 4 + 4];
                        (u32::from(p[0]) << 16) | (u32::from(p[1]) << 8) | u32::from(p[2])
                    } else {
                        0x003b_2213
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
        let host = Arc::new(Host::default());
        let params = Arc::new(Ca72Params::default());
        let context: Arc<dyn GuiContext> = host.clone();
        let e = Editing::new(
            context,
            Arc::clone(&params),
            Arc::new(Meters::default()),
            DEFAULT_WIDTH,
            1.0,
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
        assert_eq!(height_for(DEFAULT_WIDTH), 529);
        assert_eq!(clamp_width(10), MIN_WIDTH);
        assert_eq!(clamp_width(99_999), MAX_WIDTH);
    }
}
