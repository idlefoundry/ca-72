# Fixed pulse source: divider correctness and provisional component comparison

The front-panel waveform selector is a finite source. Drawing 1448 shows ground —
R034 1.5K — wide tap — R033 1K — narrow tap — R032 7.5K — −10 V (oscillator 1
reference designators; the other oscillators repeat it). The exact primary scan is
`fjp/opschem300/minimoog-schematics-12.gif` in the source manifest, SHA256
`c7af4ebfd0e50e15b4ca880acb61722b892c23ef6d78040bbf0674e8a7d508de`.
Thus the fixed unloaded sources are (0 V, 0 Ω), (−1.5 V, 1275 Ω), and
(−2.5 V, 1875 Ω). The old ideal-voltage bench bypassed those resistances.

`PulseCircuit` transcribes Q11/Q12 and their actual resistor network with the shared
transistor models. At setup it follows the stable DC branches to derive the falling
and rising buffer thresholds. A 0.05 V grid is prepared and shared per supported
component set before audio starts. Fixed and CONTINUOUS use the same physical source:
a 10K pot across −10 V has Vth=−10p and Rth=10K·p·(1−p), exactly the drawn tap
impedances. This gives a continuous circuit extension, including the fixed knots.
Audio processing and waveform changes only interpolate already prepared values.

Edge delays versus buffer slew rate come from full oscillator ngspice transients,
separately for the factory and provisional component sets. This is a reduction of
transistor switching time; it contains no waveform samples or hardware harmonic
correction. Direct ideal-source VCO benches remain explicit and retain their old
reduction. The production voice selects the physical source. The physical source
also raises the factory CONTINUOUS endpoint's pulse duty from the old ideal-source
estimate of about 6.4% toward 8%; its voltage endpoint remains −3.45 V.

Factory components remain the default in the divider-correction commit; the candidate has a distinct source commit/fingerprint. A separate source commit changes `pulse::ACTIVE_PROFILE` to `ReferenceStandard`, which explicitly
selects R1=41.2K and R44=374K; R40=43K and the other parts stay drawn. R1 and R40's
original BOM tolerance is ±5% (Table 7-5, page7-4, manifest scan704); R44's is ±5%
(page7-5, scan705). These standard nominal values are a provisional equivalent
mechanism, not proof of the Behringer's installed resistors. A diagnostic inferred
41.24K/372.2K pair is not promoted or labeled a preferred-value pair.

Selection used historical Session C loaded MIX Square and Narrow at the outer
~84/359Hz pitches. Wide and middle pitches were excluded from final selection;
the nominal measurements and early exploratory middle-pitch trials were already
seen, so this is prospective excluded-control validation, not an unseen dataset.
The inferred pair still misses Wide by roughly 0.0008–0.001 duty under the same
harmonic geometry estimator. The rounded standard pair retains explicit residuals.
The hardware component calibration is not accepted as an exact match. Current MAIN
recordings are combined-path validation and must not be called bare source truth.

At 174.64Hz, the drawn divider changes factory SPICE Wide/Narrow midpoint duty
from 0.300567/0.179039 to 0.303966/0.189008. Adding the actual VOLUME10 pot+33K bus
load shifts duty only about +0.00006. The missing divider is a correctness fix even
though it initially moves farther from hardware. Mixer feedback and buffer loading
are measured limitations: the realtime output loading is still primarily a static
Thevenin reduction, and comparator component changes cannot claim bit identity of
other physical outputs. Measured saw H1 change was below 0.00001dB, while weak
triangle partial changes reached ~0.45dB in exploratory parts trials.

The LO source laws are reduced again from the full voice MIX at 220Hz, VOLUME8,
with the matching component set. They use common fixed/CONTINUOUS knot laws.
LO still uses a frequency-independent duty approximation: the full model changes
by about 0.0004–0.0007 duty between 110/220/440Hz. It is not source-reference proof.

## Reproduction and checks

Run ngspice under the project's shared circuit/timing locks and slot limit:

```
CA72_REQUIRE_NGSPICE=1 cargo test --release -p ca72-lab --test pulse_realtime --test vco_realtime
cargo run --release -p ca72-lab --example pulse_reference -- /tmp/pulse-reference
python scripts/calibration/pulse_reference.py circuits/boards/reference/pulse-timing.json --output /tmp/pulse_timing.rs
cargo run --release -p ca72-lab --example pulse_light_laws > /tmp/pulse-light.json
python scripts/calibration/pulse_light_laws.py /tmp/pulse-light.json
cargo test --release -p ca72 --test continuous_wave --test rt_alloc
# Repeat the same contract tests at the separate candidate source commit.
```

The circuit threshold test compares the setup reduction with independent full-board
SPICE DC sweeps for both component sets and all fixed widths. The continuous suite
covers exact knots in every quality, behavior just outside the snap deadband, the
floor and moving widths. Allocation checks count first-use selector changes,
CONTINUOUS modulation and quality changes after constructor preparation. Native
excluded-simulation-pitch harmonic audits are retained with the private session
artifacts; near-null error is reported separately from the strongest-harmonic budget.
No original capture, fitted preset, answer key, preserved executable, or live app is
changed by this source experiment.
