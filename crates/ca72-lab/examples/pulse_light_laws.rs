use ca72::{
    pulse::PulseCircuit,
    voice::{Panel, Quality, Voice, Waveform},
};
use serde_json::json;
fn main() {
    let rate = 48000.;
    let proto = Voice::prototype(rate);
    let mut rows = vec![];
    for (tag, pc) in [
        ("factory", PulseCircuit::default()),
        ("reference", PulseCircuit::reference()),
    ] {
        for midi in [45, 57, 69] {
            for wave in [
                Waveform::Sawtooth,
                Waveform::Square,
                Waveform::WideRectangle,
                Waveform::NarrowRectangle,
            ] {
                let mut v = proto.clone();
                v.prepare_pulse(pc)
                    .expect("pulse circuit reference measurement");
                v.panel = Panel::default();
                v.panel.quality = Quality::Potato;
                for n in 0..3 {
                    v.panel.osc[n].on = n == 0;
                    v.panel.osc[n].volume = 0.8;
                }
                v.panel.osc[0].waveform = wave;
                v.note(midi, true);
                let mut z = vec![];
                let mut periods = vec![];
                let mut last = 0.;
                let mut count = 0;
                for i in 0..48000 {
                    v.tick();
                    let (r, c) = v.probe_resets(0);
                    if i >= 12000 {
                        z.push(v.probe().1);
                        if c > count && last > 0. {
                            periods.push(r - last);
                        }
                        last = r;
                        count = c;
                    }
                }
                let hz = rate / (periods.iter().sum::<f64>() / periods.len() as f64);
                let win: Vec<f64> = (0..z.len())
                    .map(|i| {
                        0.5 - 0.5
                            * (2. * std::f64::consts::PI * i as f64 / (z.len() - 1) as f64).cos()
                    })
                    .collect();
                let sum = win.iter().sum::<f64>();
                let mut hs = vec![];
                for k in 1..=60 {
                    let (mut re, mut im) = (0., 0.);
                    for (i, (v, w)) in z.iter().zip(&win).enumerate() {
                        let ph = 2. * std::f64::consts::PI * k as f64 * hz * i as f64 / rate;
                        re += v * w * ph.cos();
                        im += v * w * ph.sin();
                    }
                    hs.push(2. * re.hypot(im) / sum);
                }
                rows.push(json!({"tag":tag,"midi":midi,"wave":format!("{wave:?}"),"hz":hz,"bus_harmonics":hs}));
            }
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&rows).expect("pulse circuit reference measurement")
    );
}
