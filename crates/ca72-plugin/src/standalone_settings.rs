//! Standalone Audio & MIDI settings, drawn inside the existing editor. All
//! device work is submitted to the shared host's control thread.

use std::fmt::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::{Duration, Instant};

use ca72_panel::{Pixmap, art, fonts::Fonts};
use keyboard_types::{Key, KeyState, KeyboardEvent, Modifiers};
use plugin_kit_standalone::{AppHandle, Config, DeviceId, Phase, Snapshot, negotiate};

static REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn available() -> bool {
    AppHandle::current().is_some()
}
pub fn request_open() {
    if available() {
        REQUESTED.store(true, Ordering::Release);
    }
}
pub fn take_requested() -> Option<Dialog> {
    let requested = REQUESTED.swap(false, Ordering::AcqRel);
    let app = AppHandle::current()?;
    static ALERTS: OnceLock<Mutex<Alerts>> = OnceLock::new();
    let mut alerts = ALERTS
        .get_or_init(|| Mutex::new(Alerts::default()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !alerts
        .host
        .upgrade()
        .is_some_and(|host| Arc::ptr_eq(&host, &app))
    {
        *alerts = Alerts {
            host: Arc::downgrade(&app),
            ..Alerts::default()
        };
    }
    let mut notify = false;
    if Instant::now() >= alerts.next_poll {
        alerts.next_poll = Instant::now() + Duration::from_millis(200);
        let snapshot = app.snapshot();
        let signature = snapshot.error.or(snapshot.warning);
        notify = signature.is_some() && signature != alerts.last_issue;
        alerts.last_issue = signature;
    }
    (requested || notify).then(|| Dialog::new(app))
}

struct Alerts {
    host: Weak<AppHandle>,
    next_poll: Instant,
    last_issue: Option<String>,
}
impl Default for Alerts {
    fn default() -> Self {
        Self {
            host: Weak::new(),
            next_poll: Instant::now(),
            last_issue: None,
        }
    }
}

const X: f64 = 360.0;
const Y: f64 = 155.0;
const WIDTH: f64 = 2380.0;
const HEIGHT: f64 = 1480.0;
const ROW_Y: f64 = 390.0;
const ROW_H: f64 = 90.0;
const VALUE_X: f64 = X + 560.0;
const NEXT_X: f64 = X + WIDTH - 100.0;
const BUTTON_Y: f64 = Y + HEIGHT - 125.0;

#[derive(Clone, Debug)]
enum Choice {
    Output(Option<DeviceId>),
    OutputChannels([u16; 2]),
    Input(Option<DeviceId>),
    InputChannels([u16; 2]),
    Rate(u32),
    Buffer(u32),
    Midi(Option<String>),
    Channel(Option<u8>),
}
#[derive(Debug)]
struct SelectionMenu {
    row: usize,
    highlighted: usize,
    top: usize,
    options: Vec<(String, Choice)>,
}
impl SelectionMenu {
    fn visible(&self) -> usize {
        self.options.len().saturating_sub(self.top).min(10)
    }
    fn rectangle(&self) -> [f64; 4] {
        let height = self.visible() as f64 * 64.0 + 72.0;
        let top = (ROW_Y + self.row as f64 * ROW_H + ROW_H)
            .min(BUTTON_Y - height - 15.0)
            .max(Y + 190.0);
        [VALUE_X - 25.0, top, X + WIDTH - VALUE_X - 10.0, height]
    }
    fn move_by(&mut self, offset: i32) {
        self.highlighted =
            (self.highlighted as i32 + offset).clamp(0, self.options.len() as i32 - 1) as usize;
        if self.highlighted < self.top {
            self.top = self.highlighted;
        }
        if self.highlighted >= self.top + 10 {
            self.top = self.highlighted - 9;
        }
    }
}
fn device_label(id: &DeviceId) -> String {
    format!(
        "{} ({} out / {} in)",
        id.name, id.output_channels, id.input_channels
    )
}

/// UI model, including an unapplied draft. Cancel simply discards this model.
#[derive(Debug)]
pub struct Dialog {
    app: Arc<AppHandle>,
    snapshot: Snapshot,
    draft: Config,
    focus: usize,
    menu: Option<SelectionMenu>,
    local_error: Option<String>,
    next_poll: Instant,
    stamp: String,
    changed: bool,
}

impl Dialog {
    fn new(app: Arc<AppHandle>) -> Self {
        let snapshot = app.snapshot();
        Self {
            draft: snapshot.config.clone(),
            snapshot,
            app,
            focus: 0,
            menu: None,
            local_error: None,
            next_poll: Instant::now(),
            stamp: String::new(),
            changed: true,
        }
    }

    pub fn frame(&mut self) -> bool {
        if Instant::now() >= self.next_poll {
            self.next_poll = Instant::now() + Duration::from_millis(200);
            let snapshot = self.app.snapshot();
            let stamp = format!("{snapshot:?}");
            if stamp != self.stamp {
                self.stamp = stamp;
                self.snapshot = snapshot;
                self.changed = true;
            }
        }
        std::mem::take(&mut self.changed)
    }

    fn output(&self) -> Option<&DeviceId> {
        self.draft
            .output
            .as_ref()
            .or(self.snapshot.devices.default_output.as_ref())
    }
    fn channel_choices(&self, input: bool) -> Vec<[u16; 2]> {
        let channels = if input {
            self.draft.input.as_ref().map_or(0, |id| id.input_channels)
        } else {
            self.output().map_or(0, |id| id.output_channels)
        };
        // Every mono channel and every adjacent stereo pair is selectable.
        let mut choices: Vec<_> = (0..channels).map(|channel| [channel, channel]).collect();
        choices.extend((0..channels.saturating_sub(1)).map(|channel| [channel, channel + 1]));
        choices
    }
    fn rates(&self) -> Vec<u32> {
        let mut rates = vec![0];
        if let Some(id) = self.output()
            && let Some(device) = self
                .snapshot
                .devices
                .audio
                .iter()
                .find(|device| device.id == *id)
        {
            rates.extend(device.sample_rates.iter().copied());
        }
        if !rates.contains(&self.draft.sample_rate) {
            rates.push(self.draft.sample_rate);
        }
        rates.sort_unstable();
        rates.dedup();
        rates
    }
    fn buffers(&self) -> Vec<u32> {
        let mut buffers = vec![32, 64, 128, 256, 512, 1024, 2048, 4096, 8192];
        if let Some(id) = self.output()
            && let Some(device) = self
                .snapshot
                .devices
                .audio
                .iter()
                .find(|device| device.id == *id)
        {
            for capability in &device.output_configs {
                if let Some([minimum, maximum]) = capability.buffer_range {
                    buffers.extend([minimum.max(16), maximum.min(8192)]);
                }
            }
        }
        buffers.push(self.draft.buffer_frames);
        buffers.retain(|frames| (16..=8192).contains(frames));
        buffers.sort_unstable();
        buffers.dedup();
        buffers
    }
    fn choices(&self) -> Vec<Option<DeviceId>> {
        let mut choices = vec![None];
        choices.extend(
            self.snapshot
                .devices
                .audio
                .iter()
                .filter(|device| device.id.output_channels > 0)
                .map(|device| Some(device.id.clone())),
        );
        if !choices.contains(&self.draft.output) {
            choices.push(self.draft.output.clone());
        }
        choices
    }
    fn input_choices(&self) -> Vec<Option<DeviceId>> {
        let mut choices = vec![None];
        choices.extend(
            self.snapshot
                .devices
                .audio
                .iter()
                .filter(|device| device.id.input_channels > 0)
                .map(|device| Some(device.id.clone())),
        );
        if !choices.contains(&self.draft.input) {
            choices.push(self.draft.input.clone());
        }
        choices
    }
    fn options(&self, row: usize) -> Vec<(String, Choice)> {
        match row {
            0 => self
                .choices()
                .into_iter()
                .map(|id| {
                    (
                        id.as_ref()
                            .map_or_else(|| "System default".into(), device_label),
                        Choice::Output(id),
                    )
                })
                .collect(),
            1 => self
                .channel_choices(false)
                .into_iter()
                .map(|value| (channels(value), Choice::OutputChannels(value)))
                .collect(),
            2 => self
                .input_choices()
                .into_iter()
                .map(|id| {
                    (
                        id.as_ref().map_or_else(|| "DISABLED".into(), device_label),
                        Choice::Input(id),
                    )
                })
                .collect(),
            3 => self
                .channel_choices(true)
                .into_iter()
                .map(|value| (channels(value), Choice::InputChannels(value)))
                .collect(),
            4 => self
                .rates()
                .into_iter()
                .map(|rate| {
                    (
                        if rate == 0 {
                            "Device default".into()
                        } else {
                            format!("{rate} Hz")
                        },
                        Choice::Rate(rate),
                    )
                })
                .collect(),
            5 => self
                .buffers()
                .into_iter()
                .map(|frames| (format!("{frames} frames"), Choice::Buffer(frames)))
                .collect(),
            6 => std::iter::once(("Disable hardware MIDI inputs".into(), Choice::Midi(None)))
                .chain(
                    self.snapshot
                        .devices
                        .midi_inputs
                        .iter()
                        .map(|name| (name.clone(), Choice::Midi(Some(name.clone())))),
                )
                .collect(),
            7 => std::iter::once(("ALL".into(), Choice::Channel(None)))
                .chain(
                    (1..=16).map(|channel| (channel.to_string(), Choice::Channel(Some(channel)))),
                )
                .collect(),
            _ => Vec::new(),
        }
    }
    fn selected(&self, choice: &Choice) -> bool {
        match choice {
            Choice::Output(id) => self.draft.output == *id,
            Choice::OutputChannels(channels) => self.draft.output_channels == *channels,
            Choice::Input(id) => self.draft.input == *id,
            Choice::InputChannels(channels) => self.draft.input_channels == *channels,
            Choice::Rate(rate) => self.draft.sample_rate == *rate,
            Choice::Buffer(frames) => self.draft.buffer_frames == *frames,
            Choice::Midi(None) => self.draft.midi_inputs.is_empty(),
            Choice::Midi(Some(name)) => self.draft.midi_inputs.contains(name),
            Choice::Channel(channel) => self.draft.midi_channel == *channel,
        }
    }
    fn choose(&mut self, choice: Choice) {
        match choice {
            Choice::Output(id) => {
                self.draft.output = id;
                self.draft.input = None;
                self.draft.output_channels =
                    if self.output().is_some_and(|id| id.output_channels > 1) {
                        [0, 1]
                    } else {
                        [0, 0]
                    };
            }
            Choice::OutputChannels(channels) => self.draft.output_channels = channels,
            Choice::Input(id) => {
                self.draft.input = id;
                self.draft.input_channels = [0, 0];
            }
            Choice::InputChannels(channels) => self.draft.input_channels = channels,
            Choice::Rate(rate) => self.draft.sample_rate = rate,
            Choice::Buffer(frames) => self.draft.buffer_frames = frames,
            Choice::Midi(None) => self.draft.midi_inputs.clear(),
            Choice::Midi(Some(name)) => {
                if self.draft.midi_inputs.contains(&name) {
                    self.draft.midi_inputs.retain(|selected| selected != &name);
                } else {
                    self.draft.midi_inputs.push(name);
                }
            }
            Choice::Channel(channel) => self.draft.midi_channel = channel,
        }
        self.local_error = None;
        self.changed = true;
    }
    fn open_menu(&mut self, row: usize) {
        let options = self.options(row);
        if options.is_empty() {
            return;
        }
        let highlighted = options
            .iter()
            .position(|(_, choice)| self.selected(choice))
            .unwrap_or(0);
        self.menu = Some(SelectionMenu {
            row,
            top: highlighted.saturating_sub(9),
            highlighted,
            options,
        });
        self.focus = row;
        self.changed = true;
    }
    fn cycle(&mut self, row: usize, direction: i32) {
        self.local_error = None;
        match row {
            0 => {
                self.draft.output = stepped(&self.choices(), &self.draft.output, direction)
                    .cloned()
                    .flatten();
                self.draft.input = None;
                let channels = self.output().map_or(0, |id| id.output_channels);
                self.draft.output_channels = if channels > 1 { [0, 1] } else { [0, 0] };
            }
            1 => {
                if let Some(channels) = stepped(
                    &self.channel_choices(false),
                    &self.draft.output_channels,
                    direction,
                ) {
                    self.draft.output_channels = *channels;
                }
            }
            2 => {
                let choices = self.input_choices();
                self.draft.input = stepped(&choices, &self.draft.input, direction)
                    .cloned()
                    .flatten();
            }
            3 => {
                if let Some(channels) = stepped(
                    &self.channel_choices(true),
                    &self.draft.input_channels,
                    direction,
                ) {
                    self.draft.input_channels = *channels;
                }
            }
            4 => {
                if let Some(rate) = stepped(&self.rates(), &self.draft.sample_rate, direction) {
                    self.draft.sample_rate = *rate;
                }
            }
            5 => {
                if let Some(buffer) = stepped(&self.buffers(), &self.draft.buffer_frames, direction)
                {
                    self.draft.buffer_frames = *buffer;
                }
            }
            6 => self.open_menu(6),
            7 => {
                let channels: Vec<_> = std::iter::once(None).chain((1..=16).map(Some)).collect();
                self.draft.midi_channel = stepped(&channels, &self.draft.midi_channel, direction)
                    .copied()
                    .flatten();
            }
            _ => {}
        }
        self.focus = row;
        self.changed = true;
    }
    fn validation(&self) -> Result<(), String> {
        negotiate(&self.draft, &self.snapshot.devices).map(|_| ())
    }
    fn apply(&mut self) {
        self.local_error = self
            .validation()
            .and_then(|()| self.app.apply(self.draft.clone()))
            .err();
        self.changed = true;
    }

    /// Pointer coordinates are the panel's drawing units. All presses in this
    /// overlay are consumed by the editor, including outside the dialog's box.
    pub fn press(&mut self, x: f64, y: f64) -> bool {
        if let Some(menu) = &self.menu {
            let [left, top, width, height] = menu.rectangle();
            if (left..left + width).contains(&x) && (top..top + height).contains(&y) {
                let index = ((y - top) / 64.0) as usize;
                if index < menu.visible() {
                    let chosen = menu.options[menu.top + index].1.clone();
                    let multiple = matches!(chosen, Choice::Midi(_));
                    self.choose(chosen);
                    if !multiple {
                        self.menu = None;
                    }
                } else if let Some(menu) = &mut self.menu {
                    let direction = if x < left + width / 2.0 { -10 } else { 10 };
                    menu.move_by(direction);
                    self.changed = true;
                }
            } else {
                self.menu = None;
                self.changed = true;
            }
            return false;
        }
        if (BUTTON_Y..BUTTON_Y + 75.0).contains(&y) {
            if (X + 45.0..X + 330.0).contains(&x) {
                self.local_error = self.app.refresh().err();
                self.changed = true;
            }
            if (X + WIDTH - 710.0..X + WIDTH - 405.0).contains(&x) {
                return true;
            }
            if (X + WIDTH - 365.0..X + WIDTH - 50.0).contains(&x) {
                self.apply();
            }
            return false;
        }
        if (ROW_Y..ROW_Y + ROW_H * 8.0).contains(&y) && (X..X + WIDTH).contains(&x) {
            let row = ((y - ROW_Y) / ROW_H) as usize;
            self.focus = row;
            if x >= VALUE_X - 30.0 {
                self.open_menu(row);
            } else {
                self.changed = true;
            }
        }
        false
    }

    pub fn key(&mut self, event: &KeyboardEvent) -> bool {
        if event.state != KeyState::Down {
            return false;
        }
        if let Some(menu) = &mut self.menu {
            match event.key {
                Key::Escape => self.menu = None,
                Key::ArrowUp => menu.move_by(-1),
                Key::ArrowDown => menu.move_by(1),
                Key::PageUp => menu.move_by(-10),
                Key::PageDown => menu.move_by(10),
                Key::Home => {
                    menu.highlighted = 0;
                    menu.top = 0;
                }
                Key::End => {
                    menu.highlighted = menu.options.len() - 1;
                    menu.top = menu.highlighted.saturating_sub(9);
                }
                Key::Enter => {
                    let chosen = menu.options[menu.highlighted].1.clone();
                    let multiple = matches!(chosen, Choice::Midi(_));
                    self.choose(chosen);
                    if !multiple {
                        self.menu = None;
                    }
                }
                Key::Character(ref text) if text == " " => {
                    let chosen = menu.options[menu.highlighted].1.clone();
                    let multiple = matches!(chosen, Choice::Midi(_));
                    self.choose(chosen);
                    if !multiple {
                        self.menu = None;
                    }
                }
                Key::Tab => self.menu = None,
                _ => {}
            }
            self.changed = true;
            return false;
        }
        match event.key {
            Key::Escape => return true,
            Key::Tab => {
                self.focus = (self.focus
                    + if event.modifiers.contains(Modifiers::SHIFT) {
                        10
                    } else {
                        1
                    })
                    % 11;
                self.changed = true;
            }
            Key::ArrowDown => {
                self.focus = (self.focus + 1) % 11;
                self.changed = true;
            }
            Key::ArrowUp => {
                self.focus = (self.focus + 10) % 11;
                self.changed = true;
            }
            Key::ArrowLeft => self.cycle(self.focus, -1),
            Key::ArrowRight => self.cycle(self.focus, 1),
            Key::Enter => match self.focus {
                8 => {
                    self.local_error = self.app.refresh().err();
                    self.changed = true;
                }
                9 => return true,
                10 => self.apply(),
                _ => self.open_menu(self.focus),
            },
            Key::Character(ref text) if text == " " => self.open_menu(self.focus),
            _ => {}
        }
        false
    }

    pub fn draw(&self, onto: &mut Pixmap, scale: f64) {
        let fonts = Fonts::new();
        let svg = self.svg();
        let options = resvg::usvg::Options {
            fontdb: fonts.database(),
            ..Default::default()
        };
        let Ok(tree) = resvg::usvg::Tree::from_str(&svg, &options) else {
            return;
        };
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(scale as f32, scale as f32),
            &mut onto.as_mut(),
        );
    }

    /// Kept public for static visual proof without opening any device/window.
    pub fn svg(&self) -> String {
        let mut svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}"><rect width="100%" height="100%" fill="#08090b" opacity=".72"/><rect x="{X}" y="{Y}" width="{WIDTH}" height="{HEIGHT}" rx="20" fill="#1f2228" stroke="#4a4640" stroke-width="4"/><g font-family="URW Gothic" fill="#ebe6d8">"##,
            art::W,
            art::H,
            art::W,
            art::H
        );
        text(
            &mut svg,
            X + 55.0,
            Y + 85.0,
            52.0,
            "AUDIO & MIDI",
            "#f0a030",
        );
        text(
            &mut svg,
            X + 55.0,
            Y + 153.0,
            31.0,
            &format!(
                "{} · Click a value to choose; Tab selects a setting; Enter opens its list",
                self.snapshot.devices.backend
            ),
            "#a6a5a1",
        );
        let output = self.draft.output.as_ref().map_or_else(
            || {
                format!(
                    "System default ({})",
                    self.snapshot
                        .devices
                        .default_output
                        .as_ref()
                        .map_or("unavailable", |id| id.name.as_str())
                )
            },
            |id| id.name.clone(),
        );
        let input = self
            .draft
            .input
            .as_ref()
            .map_or_else(|| "DISABLED".to_owned(), |id| id.name.clone());
        let rate = if self.draft.sample_rate == 0 {
            "Device default".to_owned()
        } else {
            format!("{} Hz", self.draft.sample_rate)
        };
        let midi = if self.draft.midi_inputs.is_empty() {
            "No hardware MIDI inputs selected".to_owned()
        } else {
            format!(
                "{} selected · {}",
                self.draft.midi_inputs.len(),
                self.draft.midi_inputs.join(", ")
            )
        };
        let rows = [
            ("OUTPUT DEVICE", output),
            ("OUTPUT CHANNELS", channels(self.draft.output_channels)),
            ("EXTERNAL INPUT", input),
            (
                "INPUT CHANNELS",
                if self.draft.input.is_some() {
                    channels(self.draft.input_channels)
                } else {
                    "Input is disabled".to_owned()
                },
            ),
            ("SAMPLE RATE", rate),
            (
                "BUFFER REQUEST",
                format!("{} frames", self.draft.buffer_frames),
            ),
            ("MIDI INPUTS", midi),
            (
                "MIDI CHANNEL",
                self.draft
                    .midi_channel
                    .map_or_else(|| "ALL".to_owned(), |channel| channel.to_string()),
            ),
        ];
        for (row, (label, value)) in rows.iter().enumerate() {
            let y = ROW_Y + row as f64 * ROW_H;
            let fill = if self.focus == row {
                "#3a3022"
            } else {
                "#15171b"
            };
            let _ = write!(
                svg,
                r##"<rect x="{}" y="{y}" width="{}" height="{}" rx="8" fill="{fill}"/>"##,
                X + 35.0,
                WIDTH - 70.0,
                ROW_H - 12.0
            );
            text(&mut svg, X + 65.0, y + 51.0, 30.0, label, "#a6a5a1");
            text(
                &mut svg,
                VALUE_X,
                y + 51.0,
                33.0,
                &ellipsize(value, 75),
                "#ebe6d8",
            );
            let _ = write!(
                svg,
                r##"<path d="M {} {} l 24 0 l -12 16 z" fill="#f0a030"/>"##,
                NEXT_X - 12.0,
                y + 31.0
            );
        }
        let status_y = ROW_Y + ROW_H * 8.0 + 50.0;
        let phase = match self.snapshot.phase {
            Phase::Stopped => "STOPPED",
            Phase::Starting => "STARTING",
            Phase::Running => "RUNNING",
            Phase::Recovering => "RECOVERING",
            Phase::Error => "ERROR",
        };
        let actual = self.snapshot.buffer_duration_ms().map_or_else(
            || "Audio has not started".to_owned(),
            |ms| {
                format!(
                    "{} Hz · actual callback {} frames · buffer duration {ms:.2} ms",
                    self.snapshot.actual_sample_rate, self.snapshot.actual_buffer_frames
                )
            },
        );
        text(
            &mut svg,
            X + 55.0,
            status_y,
            34.0,
            &format!("{phase}   {actual}"),
            "#f0a030",
        );
        let issue = self
            .local_error
            .clone()
            .or_else(|| self.snapshot.error.clone())
            .or_else(|| self.validation().err())
            .or_else(|| self.snapshot.warning.clone());
        if let Some(issue) = issue {
            text(
                &mut svg,
                X + 55.0,
                status_y + 67.0,
                30.0,
                &ellipsize(&issue, 124),
                "#e5b94d",
            );
        }
        text(
            &mut svg,
            X + 55.0,
            status_y + 135.0,
            27.0,
            "External input is adapted to the output clock. Changing settings briefly interrupts sound.",
            "#a6a5a1",
        );
        for (button, (x, width, label, enabled)) in [
            (X + 45.0, 285.0, "REFRESH", true),
            (X + WIDTH - 710.0, 305.0, "CANCEL", true),
            (X + WIDTH - 365.0, 315.0, "APPLY", self.validation().is_ok()),
        ]
        .into_iter()
        .enumerate()
        {
            let _ = write!(
                svg,
                r##"<rect x="{x}" y="{BUTTON_Y}" width="{width}" height="75" rx="8" fill="{}" stroke="{}" stroke-width="3"/>"##,
                if enabled { "#3a3022" } else { "#15171b" },
                if self.focus == button + 8 {
                    "#f0a030"
                } else {
                    "#4a4640"
                }
            );
            text(
                &mut svg,
                x + 44.0,
                BUTTON_Y + 51.0,
                35.0,
                label,
                if enabled { "#f0a030" } else { "#707174" },
            );
        }
        if let Some(menu) = &self.menu {
            let [left, top, width, height] = menu.rectangle();
            let _ = write!(
                svg,
                r##"<rect x="{left}" y="{top}" width="{width}" height="{height}" rx="8" fill="#1f2228" stroke="#82715c" stroke-width="4"/>"##
            );
            for (visible, (label, choice)) in menu
                .options
                .iter()
                .skip(menu.top)
                .take(menu.visible())
                .enumerate()
            {
                let y = top + visible as f64 * 64.0;
                if menu.top + visible == menu.highlighted {
                    let _ = write!(
                        svg,
                        r##"<rect x="{}" y="{y}" width="{}" height="64" fill="#3a3022"/>"##,
                        left + 4.0,
                        width - 8.0
                    );
                }
                if self.selected(choice) {
                    let _ = write!(
                        svg,
                        r##"<path d="M {} {} l 11 11 l 22 -25" fill="none" stroke="#f0a030" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>"##,
                        left + 28.0,
                        y + 31.0
                    );
                }
                text(
                    &mut svg,
                    left + 85.0,
                    y + 45.0,
                    32.0,
                    &ellipsize(label, 95),
                    "#ebe6d8",
                );
            }
            let footer_y = top + menu.visible() as f64 * 64.0 + 44.0;
            let footer = if menu.options.len() > 10 {
                "↑ Previous page                                Next page ↓"
            } else if menu.row == 6 {
                "Click to select inputs; click outside to finish"
            } else {
                "↑ ↓ move   ·   Enter selects   ·   Escape closes"
            };
            text(&mut svg, left + 30.0, footer_y, 27.0, footer, "#a6a5a1");
        }
        svg.push_str("</g></svg>");
        svg
    }
}

fn channels(channels: [u16; 2]) -> String {
    if channels[0] == channels[1] {
        format!("Mono {}", u32::from(channels[0]) + 1)
    } else {
        format!(
            "Stereo {} / {}",
            u32::from(channels[0]) + 1,
            u32::from(channels[1]) + 1
        )
    }
}
fn stepped<'a, T: PartialEq>(choices: &'a [T], current: &T, direction: i32) -> Option<&'a T> {
    if choices.is_empty() {
        return None;
    }
    let index = choices
        .iter()
        .position(|choice| choice == current)
        .unwrap_or(0) as i32;
    choices.get((index + direction).rem_euclid(choices.len() as i32) as usize)
}
fn ellipsize(text: &str, maximum: usize) -> String {
    if text.chars().count() <= maximum {
        text.to_owned()
    } else {
        text.chars()
            .take(maximum.saturating_sub(1))
            .chain(std::iter::once('…'))
            .collect()
    }
}
fn text(svg: &mut String, x: f64, y: f64, size: f64, value: &str, color: &str) {
    let value: String = value
        .chars()
        .filter_map(|c| match c {
            '&' => Some("&amp;".to_owned()),
            '<' => Some("&lt;".to_owned()),
            '>' => Some("&gt;".to_owned()),
            '\u{0}'..='\u{1f}' | '\u{fffe}' | '\u{ffff}' => None,
            c => Some(c.to_string()),
        })
        .collect();
    let _ = write!(
        svg,
        r#"<text x="{x}" y="{y}" font-size="{size}" fill="{color}">{value}</text>"#
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use plugin_kit_standalone::{
        AppOptions, AudioBlock, AudioProcessor, DeviceInfo, ProcessConfig, StreamCapability,
    };
    struct Noop;
    impl AudioProcessor for Noop {
        fn prepare(&mut self, _config: ProcessConfig) -> Result<(), String> {
            Ok(())
        }
        fn process(&mut self, _block: AudioBlock<'_>) {}
        fn deactivate(&mut self) {}
    }
    #[test]
    fn settings_choices_wrap_and_channel_names_are_explicit() {
        assert_eq!(
            stepped(&[None, Some(1), Some(16)], &None, -1),
            Some(&Some(16))
        );
        assert_eq!(stepped(&[32, 64, 128], &128, 1), Some(&32));
        assert_eq!(stepped::<u32>(&[], &0, 1), None);
        assert_eq!(channels([0, 0]), "Mono 1");
        assert_eq!(channels([2, 3]), "Stereo 3 / 4");
        assert_eq!(
            channels([u16::MAX; 2]),
            "Mono 65536",
            "invalid persisted channels must remain editable"
        );
        assert_eq!(ellipsize("12345", 3), "12…");
    }

    #[test]
    fn dialog_uses_capabilities_and_cancel_preserves_running_config() {
        let directory = tempfile::tempdir().expect("test state folder");
        let mut runtime = plugin_kit_standalone::spawn(
            Box::new(Noop),
            None,
            AppOptions {
                app_id: "ca72-settings-test".into(),
                state_dir: directory.path().to_owned(),
                device_access: false,
            },
        )
        .expect("test host without devices");
        runtime.wait_ready(Duration::from_secs(2)).expect("ready");
        let mut dialog = Dialog::new(runtime.handle());
        let id = DeviceId {
            name: "Virtual & interface".into(),
            input_channels: 2,
            output_channels: 4,
        };
        let capability = StreamCapability {
            channels: 4,
            min_sample_rate: 44_100,
            max_sample_rate: 48_000,
            buffer_range: Some([64, 1024]),
        };
        dialog.snapshot.devices.backend = "Test backend".into();
        dialog.snapshot.devices.default_output = Some(id.clone());
        dialog.snapshot.devices.default_sample_rate = 48_000;
        dialog.snapshot.devices.audio = vec![DeviceInfo {
            id: id.clone(),
            input_configs: vec![StreamCapability {
                channels: 2,
                ..capability.clone()
            }],
            output_configs: vec![capability],
            sample_rates: vec![44_100, 48_000],
            default_sample_rate: 48_000,
        }];
        dialog.snapshot.devices.midi_inputs =
            vec!["Keyboard < one >".into(), "Keyboard two".into()];
        assert!(
            dialog.draft.input.is_none(),
            "external input starts disabled"
        );
        assert!(dialog.validation().is_ok());
        assert_eq!(dialog.channel_choices(false).len(), 7);
        assert!(!dialog.press(VALUE_X + 20.0, ROW_Y + 20.0));
        let menu = dialog.menu.as_ref().expect("explicit output dropdown");
        assert!(menu.options[1].0.contains("Virtual & interface"));
        let [left, top, _, _] = menu.rectangle();
        dialog.press(left + 100.0, top + 64.0 + 20.0);
        assert!(dialog.menu.is_none());
        assert_eq!(dialog.draft.output, Some(id.clone()));
        dialog.cycle(2, 1);
        assert_eq!(dialog.draft.input, Some(id));
        dialog.choose(Choice::Midi(Some("Keyboard two".into())));
        assert_eq!(dialog.draft.midi_inputs, ["Keyboard two"]);
        dialog.open_menu(6);
        let [left, top, _, _] = dialog
            .menu
            .as_ref()
            .expect("multi MIDI dropdown")
            .rectangle();
        dialog.press(left + 100.0, top + 64.0 + 20.0);
        assert!(
            dialog.menu.is_some(),
            "multiple inputs can be selected in one list"
        );
        assert_eq!(dialog.draft.midi_inputs.len(), 2);
        dialog.press(X, Y);
        assert!(dialog.menu.is_none());
        dialog.cycle(7, -1);
        assert_eq!(dialog.draft.midi_channel, Some(16));
        assert!(dialog.press(X + WIDTH - 600.0, BUTTON_Y + 20.0));
        assert_eq!(
            runtime.handle().snapshot().config,
            Config::initial(),
            "cancel never submitted the draft"
        );
        assert!(dialog.svg().contains("Virtual &amp; interface"));
        let scale = 1380.0 / art::W;
        let mut pixmap = Pixmap::new(1380, (art::H * scale).round() as u32).expect("pixels");
        dialog.draw(&mut pixmap, scale);
        assert!(
            pixmap
                .data()
                .chunks_exact(4)
                .any(|pixel| pixel[0] > 120 && pixel[1] > 50),
            "dialog has rendered lettering"
        );
        if let Some(path) = std::env::var_os("CA72_STANDALONE_SETTINGS_PROOF") {
            pixmap.save_png(&path).expect("saved settings proof");
            dialog.open_menu(0);
            let mut menu_pixmap =
                Pixmap::new(1380, (art::H * scale).round() as u32).expect("menu pixels");
            dialog.draw(&mut menu_pixmap, scale);
            let menu_path =
                std::path::PathBuf::from(path).with_file_name("settings-dropdown-proof.png");
            menu_pixmap
                .save_png(menu_path)
                .expect("saved dropdown proof");
        }
        dialog.menu = None;
        dialog.open_menu(7);
        let key = |key| KeyboardEvent {
            state: KeyState::Down,
            key,
            ..KeyboardEvent::default()
        };
        dialog.key(&key(Key::Home));
        dialog.key(&key(Key::ArrowDown));
        dialog.key(&key(Key::Enter));
        assert_eq!(dialog.draft.midi_channel, Some(1));
        assert!(dialog.menu.is_none());
        runtime
            .shutdown()
            .expect("bounded shutdown without callbacks");
    }

    #[test]
    fn session_warnings_open_the_dialog_once_until_the_condition_changes() {
        let directory = tempfile::tempdir().expect("test state folder");
        std::fs::write(directory.path().join("session.json"), "{broken")
            .expect("invalid session fixture");
        let mut runtime = plugin_kit_standalone::spawn(
            Box::new(Noop),
            None,
            AppOptions {
                app_id: "ca72-settings-alert-test".into(),
                state_dir: directory.path().to_owned(),
                device_access: false,
            },
        )
        .expect("test host without devices");
        runtime.wait_ready(Duration::from_secs(2)).expect("ready");
        AppHandle::install_current(&runtime.handle());
        assert!(
            take_requested().is_some(),
            "a malformed session must be visible without opening settings manually"
        );
        assert!(
            take_requested().is_none(),
            "same warning must not reopen a dismissed dialog"
        );
        std::thread::sleep(Duration::from_millis(210));
        assert!(take_requested().is_none());
        request_open();
        assert!(
            take_requested().is_some(),
            "manual settings remain available after dismissing warning"
        );
        let error = runtime
            .shutdown()
            .expect_err("preserved malformed session prevents final save");
        assert!(error.contains("preserved"));
        assert!(
            runtime
                .handle()
                .snapshot()
                .warning
                .as_deref()
                .is_some_and(|warning| warning.contains("preserved"))
        );
        assert_eq!(
            std::fs::read_to_string(directory.path().join("session.json")).expect("preserved"),
            "{broken"
        );
    }
}
