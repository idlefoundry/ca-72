//! Exact values at the knobs, in the panel's printed units (R-KNOB-ENTRY).

use ca72_panel::controls::{Dial, Legends, TIME_MARKS};
use ca72_panel::value_entry::Field;
use nih_plug::prelude::Param;

use super::{CONTROLS, Editing, Key, KeyState, KeyboardEvent, Kind, Modifiers, Readout, art};

#[derive(Debug)]
pub(super) struct Entry {
    control: usize,
    text: String,
    selected: bool,
    edited: bool,
}

pub(super) fn shortcut(modifiers: Modifiers) -> bool {
    modifiers.contains(if cfg!(target_os = "macos") {
        Modifiers::META
    } else {
        Modifiers::CONTROL
    })
}

fn readout(control: usize) -> Option<Readout> {
    Some(match CONTROLS[control].param {
        "voices" => Readout::Voices,
        "entropy" => Readout::Entropy,
        "spread" => Readout::Width,
        "inner" => Readout::Inner,
        "double" => Readout::Detune,
        "drive" => Readout::Drive,
        "level" => Readout::Level,
        _ => return None,
    })
}

// Printed times in ms at their angles (art::marks). Between them the printed scale is
// straight; below the first mark it meets zero at the stop, past the last it continues at
// that slope to the stop. These are dial readings, not measured contour durations.
const TIMES: [(f64, f64); 7] = [
    (-150.0, 0.0),
    TIME_MARKS[0],
    TIME_MARKS[1],
    TIME_MARKS[2],
    TIME_MARKS[3],
    TIME_MARKS[4],
    TIME_MARKS[5],
];

fn time_at(knob: f64) -> f64 {
    let angle = knob.clamp(0.0, 1.0) * 300.0 - 150.0;
    let pair = TIMES
        .windows(2)
        .find(|p| angle <= p[1].0)
        .unwrap_or(&TIMES[5..]);
    let [(a, ta), (b, tb)] = pair else {
        unreachable!()
    };
    ta + (tb - ta) * (angle - a) / (b - a)
}

fn time_knob(ms: f64) -> f64 {
    let pair = TIMES
        .windows(2)
        .find(|p| ms <= p[1].1)
        .unwrap_or(&TIMES[5..]);
    let [(a, ta), (b, tb)] = pair else {
        unreachable!()
    };
    ((a + (b - a) * (ms - ta) / (tb - ta) + 150.0) / 300.0).clamp(0.0, 1.0)
}

const FOOTAGES: [f64; 6] = [0.0, 32.0, 16.0, 8.0, 4.0, 2.0];

fn unit(control: usize) -> &'static str {
    if let Some(r) = readout(control) {
        return match r {
            Readout::Entropy | Readout::Width | Readout::Inner => "%",
            Readout::Detune => "cents",
            Readout::Drive | Readout::Level => "dB",
            _ => "voices",
        };
    }
    match CONTROLS[control].kind {
        Kind::Knob {
            dial: Dial::Time, ..
        } => "ms",
        Kind::Knob {
            dial: Dial::Tune, ..
        } => "semitones",
        Kind::Knob {
            dial: Dial::Freq, ..
        } => "dial",
        Kind::Selector(Legends::Ranges) => "ft · 0 = LO",
        Kind::Selector(_) => "position 1–6",
        _ => "dial",
    }
}

fn plain(control: usize, knob: f64) -> f64 {
    match CONTROLS[control].kind {
        Kind::Knob { dial, .. } => match dial {
            Dial::Tune => knob * 5.0 - 2.5,
            Dial::Cutoff => knob * 10.0 - 5.0,
            Dial::Freq => knob * 15.0 - 7.5,
            Dial::Time => time_at(knob),
            _ => knob * 10.0,
        },
        Kind::Selector(Legends::Ranges) => FOOTAGES[(knob * 5.0).round() as usize],
        Kind::Selector(_) => (knob * 5.0).round() + 1.0,
        _ => unreachable!("only knobs and selectors take values"),
    }
}

fn normalized(control: usize, text: &str) -> Option<f64> {
    let time = matches!(
        CONTROLS[control].kind,
        Kind::Knob {
            dial: Dial::Time,
            ..
        }
    );
    let text = text.trim();
    let (text, scale) = if time {
        if let Some(t) = text.strip_suffix("ms") {
            (t.trim(), 1.0)
        } else if let Some(t) = text.strip_suffix('s') {
            (t.trim(), 1000.0)
        } else {
            (text, 1.0)
        }
    } else {
        (text, 1.0)
    };
    let v = text.parse::<f64>().ok()?;
    if !v.is_finite() {
        return None;
    }
    Some(
        match CONTROLS[control].kind {
            Kind::Knob { dial, .. } => match dial {
                Dial::Tune => (v + 2.5) / 5.0,
                Dial::Cutoff => (v + 5.0) / 10.0,
                Dial::Freq => (v + 7.5) / 15.0,
                Dial::Time => time_knob(v * scale),
                _ => v / 10.0,
            },
            Kind::Selector(Legends::Ranges) => {
                let position = if v <= 0.0 {
                    0
                } else {
                    (1..6).min_by(|&a, &b| {
                        (FOOTAGES[a] - v.clamp(2.0, 32.0))
                            .abs()
                            .total_cmp(&(FOOTAGES[b] - v.clamp(2.0, 32.0)).abs())
                    })?
                };
                position as f64 / 5.0
            }
            Kind::Selector(_) => (v.round().clamp(1.0, 6.0) - 1.0) / 5.0,
            _ => return None,
        }
        .clamp(0.0, 1.0),
    )
}

impl Editing {
    pub(super) fn start_knob_entry(&mut self, control: usize) -> bool {
        if !self.operable(control)
            || !matches!(
                CONTROLS[control].kind,
                Kind::Knob { .. } | Kind::Selector(_)
            )
        {
            return false;
        }
        let p = &self.params;
        let value = match readout(control) {
            Some(Readout::Voices) => f64::from(p.voices.unmodulated_plain_value()),
            Some(Readout::Entropy) => f64::from(p.entropy.unmodulated_plain_value()),
            Some(Readout::Width) => f64::from(p.spread.unmodulated_plain_value()),
            Some(Readout::Inner) => f64::from(p.inner.unmodulated_plain_value()),
            Some(Readout::Detune) => {
                f64::from(p.double.unmodulated_plain_value()) * crate::engine::DOUBLE_CENTS / 100.0
            }
            Some(Readout::Drive) => f64::from(p.drive.unmodulated_plain_value()),
            Some(Readout::Level) => f64::from(p.level.unmodulated_plain_value()),
            _ => plain(control, self.value_of(control)),
        };
        let value = if value.abs() < 0.00005 { 0.0 } else { value };
        let text = format!("{value:.4}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_owned();
        self.playing.release();
        self.last_press = None;
        self.last_strip_press = None;
        self.knob_entry = Some(Entry {
            control,
            text,
            selected: true,
            edited: false,
        });
        self.typing_keys = true;
        true
    }

    pub(super) fn entry_field(&self, entry: &Entry) -> Field {
        Field {
            bounds: [0.0; 4],
            size: self.renderer.text_size() * 1.15,
            text: entry.text.clone(),
            unit: if CONTROLS[entry.control].param.ends_with("_frequency")
                && self.params.freq_law() == ca72::freq_dial::Law::Semitones
            {
                "semitones"
            } else {
                unit(entry.control)
            },
            selected: entry.selected,
        }
        .placed(
            self.renderer.fonts(),
            CONTROLS[entry.control].centre(),
            [
                0.0,
                0.0,
                art::W,
                f64::from(self.physical_height()) / self.renderer.scale(),
            ],
        )
    }

    pub(super) fn knob_key(&mut self, k: &KeyboardEvent) -> bool {
        if k.state != KeyState::Down {
            return true;
        }
        let Some(entry) = &mut self.knob_entry else {
            return false;
        };
        match &k.key {
            Key::Enter => {
                let entry = self.knob_entry.take().expect("an entry");
                if !entry.edited || !self.operable(entry.control) {
                    return true;
                }
                if let Some(r) = readout(entry.control) {
                    self.set_typed(r, &entry.text);
                } else if let Some(v) = normalized(entry.control, &entry.text) {
                    self.set_once(entry.control, v);
                }
            }
            Key::Escape => self.knob_entry = None,
            Key::Character(c) if shortcut(k.modifiers) && c.eq_ignore_ascii_case("a") => {
                entry.selected = true
            }
            Key::Character(c)
                if !k
                    .modifiers
                    .intersects(Modifiers::CONTROL | Modifiers::META | Modifiers::ALT) =>
            {
                entry.edited = true;
                if entry.selected {
                    entry.text.clear();
                    entry.selected = false;
                }
                for ch in c.chars() {
                    if !ch.is_ascii() || ch.is_control() || entry.text.len() >= 32 {
                        entry.text = "Invalid value".to_owned();
                        break;
                    }
                    entry.text.push(ch);
                }
            }
            Key::Backspace | Key::Delete => {
                entry.edited = true;
                if entry.selected {
                    entry.text.clear();
                    entry.selected = false;
                } else {
                    entry.text.pop();
                }
            }
            _ => {}
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn control(id: &str) -> usize {
        ca72_panel::controls::index(id).expect("a control")
    }

    #[test]
    fn panel_dials_parse_their_numbers_and_clamp_at_the_stops() {
        for (id, text, expected) in [
            ("volume", "7.25", 0.725),
            ("mod_mix", "3.5", 0.35),
            ("tune", "-2", 0.1),
            ("cutoff", "4", 0.9),
            ("osc2_frequency", "-5", 1.0 / 6.0),
            ("osc3_frequency", "7.5", 1.0),
            ("filter_attack", "200", 0.2),
            ("filter_decay", "600 ms", 0.4),
            ("loudness_attack", "1 s", 0.6),
            ("loudness_decay", "5000", 217.0 / 300.0),
            ("filter_attack", "10 s", 258.0 / 300.0),
            ("filter_attack", "1e300 s", 1.0),
            ("osc1_range", "32", 0.2),
            ("osc1_range", "8", 0.6),
            ("osc1_range", "0", 0.0),
            ("osc1_range", "999", 0.2),
            ("osc1_range", "1", 1.0),
            ("osc1_waveform", "3.6", 0.6),
            ("osc3_waveform", "999", 1.0),
        ] {
            let got = normalized(control(id), text).expect("a number");
            assert!((got - expected).abs() < 1e-12, "{id} {text}: {got}");
        }
        for id in [
            "volume",
            "mod_mix",
            "tune",
            "cutoff",
            "osc2_frequency",
            "filter_attack",
            "osc3_waveform",
        ] {
            assert_eq!(normalized(control(id), "-1e30"), Some(0.0), "{id}");
            assert_eq!(normalized(control(id), "1e30"), Some(1.0), "{id}");
            for text in ["", "x", "2x", "--1", "1..2", "NaN", "inf", "-inf", "1e999"] {
                assert_eq!(normalized(control(id), text), None, "{id} {text}");
            }
        }
    }

    #[test]
    fn the_time_scale_returns_every_printed_mark_and_both_stops() {
        for (angle, ms) in TIMES {
            let knob = (angle + 150.0) / 300.0;
            assert!((time_at(knob) - ms).abs() < 1e-9);
            assert!((time_knob(ms) - knob).abs() < 1e-12);
        }
        for k in 0..=1000 {
            let knob = f64::from(k) / 1000.0;
            assert!((time_knob(time_at(knob)) - knob).abs() < 1e-12);
        }
        assert_eq!(time_knob(-1000.0), 0.0);
        assert_eq!(time_knob(1e20), 1.0);
    }
}
