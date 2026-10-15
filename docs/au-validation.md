# Audio Unit validation budget

The AU runs pluginval 1.0.4 at strictness 10, including editors and its nested
`auval -strict -stress 20`. `scripts/validate.sh` gives this invocation a finite
180000 ms output inactivity timeout. VST3, CLAP, direct native/Rosetta auval and
all checks retain their existing settings. No heartbeat, skipped test, quality
reduction or DSP change is used. The installer is unchanged.

## Why the default timeout failed

In normal CI [38646776111](https://github.com/idlefoundry/ca-72/actions/runs/38646776111),
source 589e5cc passed direct native/Rosetta auval but failed nested AU validation
when pluginval's default 30-second output inactivity watchdog fired.

[Trace run 38653356837](https://github.com/idlefoundry/ca-72/actions/runs/38653356837)
rebuilt only the AU wrapper around the same CLAP bytes. It recorded 1,513
completed CLAP calls consuming 31.315 seconds before termination, without an
oversized block or dropped trace record. A ULTRA/unison state with A-440 consumed 22.545 seconds in 172 blocks.
Offline replay additionally showed retained doubled voice tails. Requests and notes
continued advancing. The direct run used different randomized controls, including
bypass, and its shorter duration is not a controlled comparison. Offline Linux
replay took 12.808 seconds; forwarding AU Reset took 12.716 seconds. That replay
omits state blobs/seed and sidechain samples and is not native AU acceptance.
Missing AU-to-CLAP Reset forwarding remains a separate lifecycle finding.

Apple's `auval -h` defines stress duration as simulated seconds of audio. Twenty
seconds can take more than 30 wall-clock seconds under a costly valid state.
[pluginval's AU test](https://github.com/Tracktion/pluginval/blob/v1.0.4/Source/tests/BasicTests.cpp#L610)
forwards nested output through 2048-byte reads before updating its output watchdog.
[Verbose logging](https://github.com/Tracktion/pluginval/blob/v1.0.4/Source/PluginTests.cpp#L67)
updates that watchdog even with verbose display off. Completed render progress
therefore need not create an output update during the stress interval.

The original investigation proposed keeping 30 seconds; that was an agent-authored
constraint, superseded after evidence and independent review. The 180-second inactivity
allowance applies throughout the AU invocation; it is not a total wall-clock
bound or a new real-time CPU budget. It does not establish that every state meets an audio deadline.

## Acceptance and limitations

[Bounded diagnostic 38654571401](https://github.com/idlefoundry/ca-72/actions/runs/38654571401)
uses the original, uninstrumented 589e5cc AU from artifact 11932543293 of run
38646776111. Archive SHA256:
`f17d0d0cf16980d36981c32a55d30ca09acc2ad5517bcbf9e2a502dbc94c847f`.
Pinned pluginval, failed parent seed `0x8dcb47`, strictness 10, stress 20 and all
GUI tests remain enabled. Its independent outer bound is 240 seconds per case;
cleanup identifies only its own exact child processes on a disposable runner.
Direct strict stress and nested auval exited zero; pluginval completed every test
and reported SUCCESS. This new auval draw completed faster than the failure;
it does not replay the exact failing randomized state. The original failure is
preserved and is never reported as passing the default 30-second gate.

Production acceptance additionally requires full CI on the final commit on
macOS, Windows and Linux, including actual AU success. A future timeout or test
failure remains a failure; this change does not authorize automatic retries,
further timeout increases or waivers. No owner audio hardware or open DAW is used.

NAS evidence: `/mnt/lab/artifacts/ca-72/au-render-investigation/`, including raw
logs, trace bytes, input hashes, replay sources and summaries for each run.
