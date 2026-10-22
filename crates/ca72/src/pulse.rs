//! Board 1's Q11/Q12 rectangular shaper and the front-panel width divider (dwg 1448).
//! Thresholds are DC bifurcations of the transistor/resistor circuit, derived at setup;
//! these are not waveform samples or a fit to the hardware's harmonic amplitudes.

use crate::mna::{Circuit, GND, NoConvergence, Part};
use crate::{contour::Q2N3392, vca::Q2N4058};

/// The active component profile is a source constant, so changing it changes every
/// source-derived snapshot fingerprint. It cannot be activated by a dependency feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PulseProfile {
    Factory,
    ReferenceStandard,
}
pub const ACTIVE_PROFILE: PulseProfile = PulseProfile::Factory;
pub const ACTIVE_PROFILE_ID: &str = match ACTIVE_PROFILE {
    PulseProfile::Factory => "factory-drawn-pulse",
    PulseProfile::ReferenceStandard => "provisional-reference-standard-pulse-374k-41k2",
};

/// An unloaded voltage source and its Thevenin resistance at board pin 22B.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WidthSource {
    pub volts: f64,
    pub ohms: f64,
}

impl WidthSource {
    pub const fn ideal(volts: f64) -> Self {
        Self { volts, ohms: 0.0 }
    }
    /// R034=1.5K, R033=1K, R032=7.5K from ground to -10V, for all three oscillators.
    /// The square position grounds pin22B; the other two retain the divider's loading.
    pub fn continuous(volts: f64) -> Self {
        let volts = volts.clamp(-3.45, 0.0);
        let p = -volts / 10.0;
        Self {
            volts,
            ohms: 10e3 * p * (1.0 - p),
        }
    }
    pub const FIXED: [Self; 3] = [
        Self::ideal(0.0),
        Self {
            volts: -1.5,
            ohms: 1275.0,
        },
        Self {
            volts: -2.5,
            ohms: 1875.0,
        },
    ];
}

/// The as-drawn comparator parts. A measured-reference component set must be selected
/// explicitly; the factory circuit remains available to the lab and model tests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PulseCircuit {
    pub r1: f64,
    pub r40: f64,
    pub r44: f64,
    pub r39: f64,
    pub r35: f64,
    pub r2: f64,
    pub r13: f64,
    pub r22: f64,
}

impl Default for PulseCircuit {
    fn default() -> Self {
        Self {
            r1: 43e3,
            r40: 43e3,
            r44: 390e3,
            r39: 100e3,
            r35: 100e3,
            r2: 910e3,
            r13: 6.2e3,
            r22: 4.7e3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PulseThresholds {
    pub fall_buf: f64,
    pub rise_buf: f64,
}

impl PulseCircuit {
    /// Provisional equivalent components selected from historical Square+Narrow MIX,
    /// not measured installed parts. Wide remains a documented excluded-control residual.
    pub fn reference() -> Self {
        Self {
            r1: 41200.0,
            r44: 374000.0,
            ..Self::default()
        }
    }
    pub fn instrument() -> Self {
        match ACTIVE_PROFILE {
            PulseProfile::Factory => Self::default(),
            PulseProfile::ReferenceStandard => Self::reference(),
        }
    }

    fn network(self, width: WidthSource, buf: f64) -> Result<Circuit, NoConvergence> {
        // Held: GND, -10V, buffer, ideal divider voltage. Solved: Q11 base/collector,
        // Q12 base/collector, RECT. The width Thevenin resistor combines with R1.
        let mut c = Circuit::new(9, 4);
        c.set(1, -10.0);
        c.set(2, buf);
        c.set(3, width.volts);
        let initial = if buf > 0.0 {
            [(4, 1.0), (5, -10.0), (6, -10.0), (7, 0.0), (8, 0.0)]
        } else {
            [(4, -0.65), (5, -0.02), (6, -9.3), (7, -9.96), (8, -4.295)]
        };
        for (n, v) in initial {
            c.set(n, v);
        }
        for (a, b, r) in [
            (3, 4, self.r1 + width.ohms),
            (2, 4, self.r40),
            (4, 1, self.r44),
            (5, 6, self.r39),
            (6, 1, self.r35),
            (7, 4, self.r2),
            (7, 8, self.r13),
            (0, 8, self.r22),
        ] {
            c.add(Part::Resistor { a, b, r });
        }
        c.add_bjt(5, 4, GND, &Q2N4058, 25.0, true);
        c.add_bjt(7, 6, 1, &Q2N3392, 25.0, false);
        c.dc()?;
        Ok(c)
    }

    /// Follow each stable DC branch toward its transition, refining the buffer threshold
    /// to 1 microvolt. Continuing from the previous solution retains Q11/Q12 hysteresis.
    pub fn thresholds(self, width: WidthSource) -> Result<PulseThresholds, NoConvergence> {
        let edge = |from: f64, to: f64, falling: bool| -> Result<f64, NoConvergence> {
            let mut c = self.network(width, from)?;
            let n = 1200;
            for i in 1..=n {
                let v = from + (to - from) * i as f64 / n as f64;
                let mut next = c.clone();
                next.set(2, v);
                let ok = next.dc().is_ok();
                let switched = ok
                    && if falling {
                        next.v(8) < -2.0
                    } else {
                        next.v(8) > -2.0
                    };
                if switched || !ok {
                    let mut a = from + (to - from) * (i - 1) as f64 / n as f64;
                    let mut b = v;
                    for _ in 0..18 {
                        let mid = (a + b) * 0.5;
                        let mut trial = c.clone();
                        trial.set(2, mid);
                        let ok = trial.dc().is_ok();
                        if ok
                            && if falling {
                                trial.v(8) < -2.0
                            } else {
                                trial.v(8) > -2.0
                            }
                        {
                            b = mid;
                        } else if ok {
                            a = mid;
                            c = trial;
                        } else {
                            b = mid;
                        }
                    }
                    let mut other = self.network(width, to)?;
                    other.set(2, b);
                    other.dc()?;
                    if if falling {
                        other.v(8) >= -2.0
                    } else {
                        other.v(8) <= -2.0
                    } {
                        return Err(NoConvergence {
                            iterations: 0,
                            residual: f64::INFINITY,
                        });
                    }
                    return Ok((a + b) * 0.5);
                }
                c = next;
            }
            // A width whose comparator never resets is beyond this fixed selector's range.
            Err(NoConvergence {
                iterations: 0,
                residual: f64::INFINITY,
            })?
        };
        Ok(PulseThresholds {
            fall_buf: edge(4.2, -4.4, true)?,
            rise_buf: edge(-4.4, 4.2, false)?,
        })
    }
}

/// Setup-time reduction of the fixed selector's comparator. Its physical 10K wiper
/// DC transfer (with all fixed taps exact) is shared; no solve occurs on a
/// waveform change or in audio processing.
#[derive(Debug)]
pub(crate) struct PulseModel {
    pub thresholds: [PulseThresholds; 70],
    pub timing: Timing,
}
#[derive(Debug, Clone, Copy)]
pub(crate) struct Timing {
    pub log_slope: [[f64; 7]; 3],
    pub fall_delay: [[f64; 7]; 3],
    pub rise_delay: [f64; 3],
}
impl PulseModel {
    pub fn shared(circuit: PulseCircuit) -> Result<std::sync::Arc<Self>, NoConvergence> {
        static MODELS: crate::shared::Shared<
            PulseCircuit,
            Result<std::sync::Arc<PulseModel>, NoConvergence>,
        > = crate::shared::Shared::new();
        let cached = MODELS.get(circuit, || {
            let timing = timing(circuit).ok_or(NoConvergence {
                iterations: 0,
                residual: f64::INFINITY,
            })?;
            let mut thresholds = [PulseThresholds {
                fall_buf: 0.0,
                rise_buf: 0.0,
            }; 70];
            for (i, t) in thresholds.iter_mut().enumerate() {
                *t = circuit.thresholds(WidthSource::continuous(-0.05 * i as f64))?;
            }
            Ok(std::sync::Arc::new(Self { thresholds, timing }))
        });
        match &*cached {
            Ok(model) => Ok(model.clone()),
            Err(e) => Err(*e),
        }
    }
    pub fn thresholds(&self, width: f64) -> PulseThresholds {
        let x = (-width / 0.05).clamp(0.0, 69.0);
        let i = (x.floor() as usize).min(68);
        let a = x - i as f64;
        let p = self.thresholds[i];
        let q = self.thresholds[i + 1];
        PulseThresholds {
            fall_buf: p.fall_buf + a * (q.fall_buf - p.fall_buf),
            rise_buf: p.rise_buf + a * (q.rise_buf - p.rise_buf),
        }
    }
    fn width_interpolation(width: f64) -> (usize, f64) {
        if width >= -1.5 {
            (0, -width / 1.5)
        } else {
            (1, (-width - 1.5) / 1.0)
        }
    }
    pub fn falling_delay(&self, width: f64, slope: f64) -> f64 {
        let x = crate::ulp::log(slope.max(1e-9));
        let at = |p: usize| {
            let g = &self.timing.log_slope[p];
            let y = &self.timing.fall_delay[p];
            if x <= g[0] {
                return y[0];
            }
            for i in 1..7 {
                if x <= g[i] {
                    let a = (x - g[i - 1]) / (g[i] - g[i - 1]);
                    return y[i - 1] + a * (y[i] - y[i - 1]);
                }
            }
            y[6]
        };
        let (i, a) = Self::width_interpolation(width);
        let p = at(i);
        let q = at(i + 1);
        p + a * (q - p)
    }
    pub fn rising_delay(&self, width: f64) -> f64 {
        let (i, a) = Self::width_interpolation(width);
        let p = self.timing.rise_delay[i];
        let q = self.timing.rise_delay[i + 1];
        p + a * (q - p)
    }
}
#[path = "pulse_timing.rs"]
mod timing_data;
use timing_data::timing;
