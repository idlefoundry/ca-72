//! Issue #67's steady high notes, through the same engine as `preset_render`, and the
//! circuit oscillator alone. No audio device or shared library is opened. The companion
//! `scripts/measurements/aliasing.py` builds this unchanged in three detached worktrees.

use ca72::tuning::{self, Range, Tuning, osc1_inputs};
use ca72::vco::Vco;
use ca72::voice::Waveform;
use ca72_plugin::engine::{Controls, Engine, Event};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

const N: usize = 1 << 17;
const WAVES: [(&str, Waveform, f64); 6] = [
    ("triangle", Waveform::Triangle, 0.0),
    ("shark", Waveform::SharkTooth, 0.0),
    ("saw", Waveform::Sawtooth, 0.0),
    ("square", Waveform::Square, 0.0),
    ("wide", Waveform::WideRectangle, -1.5),
    ("narrow", Waveform::NarrowRectangle, -2.5),
];

fn write(path: &Path, samples: &[f64]) -> std::io::Result<()> {
    let mut f = BufWriter::new(File::create(path)?);
    for &x in samples {
        assert!(x.is_finite(), "non-finite sample in {}", path.display());
        f.write_all(&x.to_le_bytes())?;
    }
    f.flush()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 1 || args[0].starts_with('-') {
        return Err("usage: aliasing_render <empty-output-directory>".into());
    }
    let dir = Path::new(&args[0]);
    fs::create_dir_all(dir)?;
    if fs::read_dir(dir)?.next().is_some() {
        return Err("output directory must be empty".into());
    }
    let mut manifest = String::from("stage\tmode\trate\tmidi\twave\tf0_reset_hz\tfile\n");
    for rate in [44_100, 48_000, 96_000] {
        let sr = f64::from(rate);
        let mut tuned = Vco::new(sr, 4);
        let (t, _) = tuning::folkman_1973(&mut tuned, sr, Tuning::default());
        for mode in ["LO", "HI", "ULTRA"] {
            for midi in [84u8, 96, 108] {
                for (wave, waveform, width) in WAVES {
                    let mut c = Controls {
                        lock: true,
                        potato: mode == "LO",
                        ultra: mode == "ULTRA",
                        auto_gain: false,
                        ..Controls::default()
                    };
                    c.panel.osc[0].waveform = waveform;
                    c.panel.osc[0].range = Range::R8;
                    c.panel.osc[0].volume = 0.8;
                    c.panel.cutoff = 1.0;
                    c.panel.emphasis = 0.0;
                    c.panel.contour_amount = 0.0;
                    c.panel.keyboard_control_1 = false;
                    c.panel.keyboard_control_2 = false;
                    c.panel.loudness_contour.attack = 0.0;
                    c.panel.loudness_contour.decay = 0.0;
                    c.panel.loudness_contour.sustain = 1.0;
                    let mut e = Engine::new();
                    e.set(&c);
                    e.prepare(sr, 1);
                    // Ordinary offline calls, no worker deadlines or real-time scheduling.
                    let warm = rate as usize;
                    let mut out = Vec::with_capacity(N);
                    let mut at = 0;
                    while at < warm + N {
                        if at == 0 {
                            e.event(Event::Note {
                                key: midi,
                                on: true,
                            });
                        }
                        let len = 256.min(warm + N - at);
                        let (ext, mut l, mut r) = ([0.0; 256], [0.0; 256], [0.0; 256]);
                        e.render(&ext[..len], &mut l[..len], &mut r[..len]);
                        e.end_block(len);
                        for (k, (&l, &r)) in l[..len].iter().zip(&r).enumerate() {
                            if at + k >= warm {
                                out.push(0.5 * (f64::from(l) + f64::from(r)));
                            }
                        }
                        at += len;
                    }
                    let file = format!("engine-{mode}-{rate}-{midi}-{wave}.f64");
                    write(&dir.join(&file), &out)?;
                    manifest.push_str(&format!(
                        "engine\t{mode}\t{rate}\t{midi}\t{wave}\t0\t{file}\n"
                    ));
                    if mode != "LO" {
                        let mut v = Vco::new(sr, if mode == "ULTRA" { 4 } else { 1 });
                        v.expo.r11 = t.r11;
                        v.expo.a8 = t.a8;
                        let current = v.expo.input_current(&osc1_inputs(
                            &t,
                            f64::from(midi) - 41.0,
                            Range::R8,
                        ));
                        for _ in 0..warm {
                            v.tick(current, width);
                        }
                        let first = v.last_reset();
                        out.clear();
                        for _ in 0..N {
                            let o = v.tick(current, width);
                            let y = match waveform {
                                Waveform::Triangle => o.tri,
                                Waveform::Sawtooth => o.saw,
                                Waveform::SharkTooth => {
                                    // The switch's R030/R031 mixture, unloaded (waveform_source).
                                    let ga = 1.0 / (4300.0 * 4700.0 / 9000.0 + 47e3);
                                    let gb = 1.0 / (750.0 * 8200.0 / 8950.0 + 10e3);
                                    (o.saw * ga + o.tri * gb) / (ga + gb)
                                }
                                _ => o.rect,
                            };
                            out.push(y);
                        }
                        let last = v.last_reset();
                        let f0 = sr * (last.1 - first.1) as f64 / (last.0 - first.0);
                        let file = format!("oscillator-{mode}-{rate}-{midi}-{wave}.f64");
                        write(&dir.join(&file), &out)?;
                        manifest.push_str(&format!(
                            "oscillator\t{mode}\t{rate}\t{midi}\t{wave}\t{f0:.12}\t{file}\n"
                        ));
                    }
                }
                eprintln!("{rate} Hz {mode} MIDI {midi}: six waveforms rendered");
            }
        }
    }
    // A manifest appears only after every render has been written successfully.
    fs::write(dir.join("renders.tsv"), manifest)?;
    Ok(())
}
