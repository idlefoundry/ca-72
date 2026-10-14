# AU nested validation investigation

Resumed by the owner2026-10-14; installation behavior stays unchanged. Product589e5cc,
failed normal CI38646776111, nested seed0x8dcb47. Earlier c16 probes are precedent,
not proof of the current cause. Required acceptance is an identified cause, a failing
regression before its repair, unchanged strict validator settings passing, and full
three-platform CI on the repaired product. Do not merge a diagnostic-only build.

First experiment: rebuild only the AU C++ wrapper with bounded, preallocated shared
memory tracing around the exact final CI CLAP binary (artifact11932543293, verified
archive SHA256f17d0d0cf16980d36981c32a55d30ca09acc2ad5517bcbf9e2a502dbc94c847f).
This keeps all Rust DSP bytes unchanged. Run direct strict stress20 and pluginval1.0.4
strictness10/nested stress20 with30000ms inactivity limit and original failed seed.

Trace Initialize/activation, per-instance parameter snapshots, AU/CLAP render entry
and exit, frame count/maximum/rate/sample position, parameter and MIDI changes,
state-load boundaries, resets and unlogged-event counts. Setup/prefault file mappings
in Initialize; callback writer only reserves a fixed slot and publishes scalar fields.
No callback file calls/allocations/locks, no stdout progress heartbeat. Never replace
or suppress the watchdog. Decoder rejects missing traces and exhausted capacity.

Inspect completed frames and request durations before selecting a repair. If one
CLAP call remains active without enough internal evidence, extend only that boundary
with bounded DSP chunk/state tracing in a separate experiment. Stop after three
inconclusive experiments under the circuit-plugin skill and report remaining options.

Source review identified missing AU-to-CLAP Reset forwarding. The diagnostic override
only records Reset and delegates to the existing base behavior. A causal fix must
serialize reset against render, discard stale queued MIDI and preserve musical
parameters; prove it with a focused host regression before claiming the timeout fixed.

Evidence is stored under au-render-trace artifacts in each diagnostic Actions run;
exact input hashes, raw trace, decoded summaries, parent/child samples and result
codes stay together. No owner hardware, installed plug-ins or DAW sessions are used.

## Evidence-driven execution-budget hypothesis

Run38653356837 records1513 complete nested CLAP calls consuming31.315s. The
matching direct stress prefix uses different randomized controls and spends1.480s
in1746 completed calls. No oversized block or dropped trace record occurred.
The nested workload uses ULTRA/unison and retained doubled tails with A-440 on;
its172 expensive512-frame blocks consume22.545s. Linux offline replay of captured
parameter/note/lifecycle records reproduces12.808s processing; forwarding Reset
changes that to12.716s, so missing Reset is not established as the timeout cause.
Replay omits state blob/seed and sidechain samples, and is not native AU validation.

The original30000ms requirement above was this investigation's proposed gate, not
an owner's restriction. The evidence now justifies one diagnostic with a finite
AU-only180000ms output timeout, original failed pluginval seed, strictness10,
nested strict stress20 and all editor tests. Use the original589 AU wrapper and
embedded CLAP bytes with no trace build or DSP changes; outer240s bound per case.
Require real auval and pluginval exit0 and complete logs before considering a
production harness budget correction. A failure does not authorize another increase.
Do not claim the original30s gate passed or a DSP performance defect was repaired.
Installer behavior remains unchanged.
