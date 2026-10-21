# Local reference-fitting experiments

`ca72_experiment` makes a bounded, inspectable experiment from an actual local recording.
It combines offline rendering with reusable named-region measurements and optional
preservation of approved expression. It does not transcribe a recording, separate sources,
choose a winner or change the live instrument. The analogue DSP is unchanged.

## Inputs

The usual `ca72_render` source is accepted: one of `init:true`, `snapshot`, `snapshot_file`,
`preset` or `preset_file`. With no source the selected live instance is captured once.
`init:true` uses fresh factory parameter defaults without an app or a personal library.
Common `values` and `modulations` apply before each independent candidate's overrides.
All existing render conditions and dry-output requirements apply.

Additional arguments:

- `reference`, optional `reference_start`: original local WAV and crop start.
- `approved`, optional `approved_start`: approved expression baseline, cropped independently.
- `regions`: 1–32 unique named `{id,start,seconds,fundamentals_hz?,modulation_hz?}` windows,
  relative to each selected clip. Total region duration including overlaps is at most120s.
- `candidates`: 1–8 `{id,values?,modulations?}` variations. IDs use1–40ASCIIletters/digits,
  underscores or hyphens. The common request is bounded to120renderedseconds/12millionframes.
- `preservation_limits`: optional nonnegative `envelope_rmse_db`, `envelope_p95_db`,
  `am_depth_db`, `fm_depth_cents` and `pitch_cents`. These are explicit engineering budgets.

Reference, approved and rendered selections must have identical rates and frame counts.
Every candidate's settings/routing, all regions and limits validate before creating output.
The reference regions are decoded/measured once, and approved regions once when present.
Candidate rendering is sequential, detached and uses the normal synth process.

## Measurements and limitations

Named regions are also available on `ca72_analyze` and `ca72_compare`. Compare optionally
accepts approved audio and the same preservation limits. The legacy whole-clip outputs
remain available. Full region evidence is saved in `metrics.json`; compact summaries are
returned through MCP.

Each region measures integrated H1–H8 bands, one-twelfth-octave bands from20Hz to the lower
of20kHz/Nyquist, upper6–20kHz power when available, and spectral peaks. Supplied fundamentals
are refined separately in each file within±4%. Regions need at least1024samples, four
periods of each supplied fundamental and bins within its search range. Physical resolution
and zero-padding bin spacing are reported separately. Insufficient resolution is refused.

Harmonic RMS error has uniform weights and a−60dB floor. Spectral RMS error has a−80dB floor.
Both use total fundamental-band power for normalization, or50–1000Hz when no fundamentals
are supplied. Chord bands can overlap and describe combined energy; this is not voice
separation. Strong candidate peaks without a similarly strong nearby reference peak are
flagged. Peak flags are diagnostics: a tuning change or intended harmonic change can also
produce one. There is no universal or perceptual similarity score.

For a single supplied fundamental and known `modulation_hz`, approximate AM/FM depths use
a50msHann complex demodulator, DC removal, AC-based channel selection and a complete
sinusoidal least-squares fit including constant/linear trend. Window and finite-difference
responses are compensated. At least three cycles must resolve; window response below0.8
is unresolved (roughly rates above11Hz). This avoids reporting attenuated high-rate motion
as a smaller real depth. Null is not absent movement. Use the existing sideband analysis
for faster modulation; its own resolution limits still apply.

Preservation uses50msAC-RMS frames with10ms hops and one common gain fitted to approved
frames audible above−45dB of peak AC power. The union of audible approved/candidate frames
includes added tails. Whole-clip and each named region are checked separately, along with
AM/FM-depth differences and supplied-fundamental pitch shifts where requested. AM/FM
limits without any modulation region, or pitch limits without fundamentals, are rejected.
A requested unresolved measurement yields null rather than a pass. No limits means no
pass/fail verdict. The budgets are not a listening approval.

## Outputs and continuation

Every candidate has its normal render manifest/WAV, exact `snapshot.json`, full regional
`metrics.json`, and level-matched audition WAV. `report.html` provides the reference,
approved baseline and candidates at a common listening level and headroom cap, with
separate metrics and preservation verdicts. `experiment.json` is written last as the
completion marker and records request, sources, source fingerprint and analysis format.
A runtime failure can leave partial candidate artifacts without this marker.

Continue a candidate by passing its returned `snapshot_path` as `snapshot_file` to another
render or experiment. `ca72_preset_export` also accepts that file, avoiding large state
copies through the agent context. Snapshot files are bounded to1MiB and require the same
engine fingerprint; ordinary preset files retain normal serialization and portability.
Export and reload the actual preset before presenting final listening evidence. The batch
report itself demonstrates candidates before serialization, not final preset delivery.

## Acceptance checks

Synthetic analysis tests exercise malformed/duplicate/out-of-range regions, silence,
unresolved low fundamentals, independent pitch refinement, stereo phase cancellation,
chord balance, added tones outsideH1–H8, calibrated AM/FM, DC-heavy channel selection,
common-gain invariance, isolated short-note regressions, candidate-only tails and unresolved
requested budgets. Production stdio tests exercise all-candidate preflight, batch bounds,
cache counts, exact single-render equivalence, snapshot-file continuation/export and
region-aware preservation. Existing MCP audio regressions remain in the suite.

Real-recording exercises and final review/check results are recorded in
[mcp-acoustic-validation.md](mcp-acoustic-validation.md). These tools do not change the
approved listening presets simply to make a new diagnostic smaller.
