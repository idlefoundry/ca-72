CA-72 LFO stress listening tests

Sixteen deliberately difficult presets, with three versions of each:
  before: 1374b0f's behaviour, LFO targets held for 32 samples, no volume ramp.
  after: 29ddf24 and 1c85e11, this branch's behaviour.
  reference-every-sample: the same LFOs evaluated every sample, for comparison.

48 kHz, stereo, 24-bit PCM, eight seconds each. Each preset's three versions
use exactly the same gain. Their loudest peak is -14 dBFS. Notes start at
0.25 s, stop by 7.4 s, and leave a release. No loudness matching between
different presets. Compare versions at the same playback volume.

Start with 03 (filter zipper), 06 (SOFT volume clicks), then 05 (intentional
SHARP clicks) and 16 (the slower control). The references retain intended
waveform edges, modulation, note attacks and release; they are not silent
or necessarily free of roughness. A raw waveform null is not a useful
noise measurement where stepped pitch makes oscillator phases drift.

What to listen for, during the notes (0.6 to 7.3 s unless stated):
01 Tri 40Hz OSC2 FREQ: steady high fizz around the sweeping oscillator,
   roughly 1.5 kHz from its partials; distinguish it from intended beating.
02 Saw 50Hz TUNE chord: a rough halo around the rapid pitch sweeps; the
   saw resets themselves remain in the per-sample reference.
03 Tri 30Hz CUTOFF EMPH 10: fizz around the filter's whistle throughout the
   held note. This is the strongest predicted 32-sample zipper example.
04 Saw 50Hz CUTOFF EMPH 9 chord: the same under a chord; sharp-sounding
   filter motion alone does not establish an unwanted click.
05 Square SHARP VOLUME: clicks twice a 6 Hz cycle (every 83 ms) are intended
   by SHARP. Compare with 06 and the reference; the fix preserves SHARP.
06 Square SOFT VOLUME: the same 6 Hz tremolo with a 2 ms soft edge. Listen
   for a short cluster of clicks at each edge before, a smooth dip after.
07 Saw SHARP CUTOFF EMPH 9: a hard reset every 0.2 s is intentional.
08 Saw SOFT CUTOFF EMPH 9: compare the same resets with SOFT, and listen
   for added fizz during the sweep, rather than judging the whistle alone.
09 Four LFOs clamp CUTOFF: four sources repeatedly reach the limits of the
   cutoff's travel. Listen for extra ticks at the flattened sweep's entry
   and exit, against the reference's intentional stops and resonances.
10 POLY MONO off RETRIG fast: overlapping fast notes and periodic chords,
   with independent LFOs and stolen voices. Listen for extra ticks on note
   starts, voice changes and the gaps; some volume edges are SHARP by design.
11 RISE and ENV: a note each second from 0.25 s. Listen for steps as the
   modulation fades in and at the one-cycle envelope's end, not just attacks.
12 Timed 1-32 at 200 BPM: rapid timed filter chops with a second pitch sweep.
   The chops are intentional; listen for extra fizz between them.
13 PWM 45Hz WAVEFORM: continuous pulse-width motion towards its floor.
   Listen for a thin whistle or harsh grain over the intended bright PWM.
14 FEEDBACK and DETUNE: dense feedback with detune swept 20 times a second.
   Listen for extra grain; pitch drift makes a direct waveform null misleading.
15 CPU worst 10 voices: doubled ten-note chords every half second, four
   independent LFOs, 32 assignments. Listen for missing notes or gaps.
   These are complete offline renders: they cannot demonstrate live CPU drops.
16 Slow 2Hz CUTOFF EMPH 10: 03 at 2 Hz. Compare the slow whistle for extra
   fine fizz, against the far more demanding 30 Hz version.

Trying the presets in CA-72 V0.3 on the Mac

The files in presets/ use the normal preset format. They have NOT been
installed in the shared preset library used by CA-72 0.2.8. There is no
single-file import action in this build. To keep the libraries separate,
run the existing V0.3 standalone from Terminal with a private library:

  IDLE_FOUNDRY_SHARED="$HOME/Downloads/CA-72-LFO-stress/isolated-library" \
    "$HOME/Applications/CA-72 V0.3.app/Contents/MacOS/ca72-standalone"

The isolated-library/CA-72/Presets folder supplied here contains only copies
of these sixteen presets. In V0.3, open PRESETS, search LFO TEST (or the
LFO Tests tag), and choose one. Set QUALITY to HI and MAIN OUTPUT carefully:
the live presets keep their original level; only the WAVs are attenuated.
For 12, a running host at 200 BPM gives the intended timed rates; the
standalone's own clock may differ. V0.3 installed on the Mac may predate the
two fixes: use the WAVs for the exact before/after comparison. Launch from
Terminal as above each time; launching normally uses the shared library.

Reproducing the measurements from this branch (Linux)

  export CARGO_TARGET_DIR=$PWD/target CARGO_BUILD_JOBS=6
  export IDLE_FOUNDRY_SHARED=$(mktemp -d /tmp/c72.XXXX)
  export CA72_LFO_OUT=/path/to/output CA72_LFO_COMPARE=1 CA72_QUALITY=HI
  CA72_LFO_LEGACY=1 CA72_LFO_LABEL=before cargo test --release \
    -p ca72-plugin --features lfo-lab --test lfo_stress zipper \
    -- --ignored --nocapture --test-threads 1
  CA72_LFO_LEGACY=0 CA72_LFO_LABEL=after cargo test --release \
    -p ca72-plugin --features lfo-lab --test lfo_stress zipper \
    -- --ignored --nocapture --test-threads 1
  python3 docs/lfo-stress/measure-wavs.py "$CA72_LFO_OUT"

metrics-before.csv and metrics-after.csv contain the original measurement
columns, including the aligned-reference floors. wav-metrics.csv resolves
the high-band envelope pulse at 1.5, 3 and 4.5 kHz against the per-sample
reference. These ratios are not audio noise levels or hearing thresholds.
The summary explains the limits and the listening-priority thresholds.

CPU and live-deadline result (Linux, HI, four workers)

The 8-sample update improves the 03/12/15 pulse measurements, but the extreme
15 is not safe in real time on the reference machine: before loses 968-1048
voice runs with 84-128 late blocks; after loses 2819-3368 with 818-873 late
blocks in two 8 s runs. These counts are not unique notes, and each unheard
run covers a shorter piece at 8 samples, so they are not a missing-audio
duration ratio. Late host blocks use the same 256-frame unit in both cases.
The offline WAVs wait for every voice and do not reproduce this deadline
loss. The volume ramp clears 06's click signature at no clear CPU increase; the finer step
adds about 12 % single-thread work to 15. Recommendation for review: keep
the volume ramp; revise the 8-sample choice before release. Both fixes stay
on the branch for the owner's decision. cpu-cost.csv and paced.csv give the
numbers; measurements/ contains the original logs. Human listening and Mac
preset loading were not performed.

The delivered folder's SUMMARY.md contains every preset's figures, the
listening-priority thresholds, verification and recommendations. Spectral
statistics use a mono average; the WAVs preserve stereo.
