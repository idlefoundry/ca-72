#!/usr/bin/env python3
"""Build a portable Live 12 comparison project using ordinary saved Live Set structures.

The base set must contain empty MIDI/audio tracks; the clip set supplies clip structures.
No plug-in state, original music, recordings, or port assignments are copied from either.
"""

import argparse
import copy
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "docs/comparison-presets"
SET_NAME = "CA-72 vs Model D - 24 Comparisons.als"
ARRANGER = "./DeviceChain/MainSequencer/ClipTimeable/ArrangerAutomation/Events"
AUDIO_ARRANGER = "./DeviceChain/MainSequencer/Sample/ArrangerAutomation/Events"
GLOBAL_ID_TAGS = {"Pointee", "VolumeModulationTarget", "TranspositionModulationTarget"}
# Live's controller slots reserve 0 for pitch bend and 1 for channel pressure.
# CC numbers begin at slot 2, so CC1 is slot 3, not slot 1.
PITCH_BEND_TARGET = "ControllerTargets.0"
CC1_TARGET = "ControllerTargets.3"


def read_set(path):
    data = path.read_bytes()
    if data[:2] == b"\x1f\x8b":
        data = gzip.decompress(data)
    root = ET.fromstring(data)
    if root.tag != "Ableton" or not root.get("Creator", "").startswith("Ableton Live 12."):
        raise ValueError("A native Live 12 Set is required")
    return root


def value(node, path, content):
    target = node.find(path)
    if target is None:
        raise ValueError(f"Missing native field: {node.tag}/{path}")
    if isinstance(content, bool):
        content = str(content).lower()
    target.set("Value", str(content))


def global_id(node):
    return (node.tag in GLOBAL_ID_TAGS or node.tag.endswith("Target")
            or node.tag.startswith("ControllerTargets.")) and "Id" in node.attrib


def envelope(clip, index, target, amount):
    env = ET.SubElement(clip.find("./Envelopes/Envelopes"), "ClipEnvelope", Id=str(index))
    dest = ET.SubElement(env, "EnvelopeTarget")
    ET.SubElement(dest, "PointeeId", Value=target)
    automation = ET.SubElement(env, "Automation")
    events = ET.SubElement(automation, "Events")
    for i, time in enumerate([-63072000, 0]):
        ET.SubElement(events, "FloatEvent", Id=str(i), Time=str(time), Value=str(amount))
    transform = ET.SubElement(automation, "AutomationTransformViewState")
    ET.SubElement(transform, "IsTransformPending", Value="false")
    ET.SubElement(transform, "TimeAndValueTransforms")
    ET.SubElement(ET.SubElement(env, "LoopSlot"), "Value")
    scroll = ET.SubElement(env, "ScrollerTimePreserver")
    ET.SubElement(scroll, "LeftTime", Value="0")
    ET.SubElement(scroll, "RightTime", Value="0")


def clip_common(template, clip_id, name, start, length, source_end, colour, annotation):
    clip = copy.deepcopy(template)
    clip.set("Id", str(clip_id))
    clip.set("Time", str(start))
    for path, content in {
        "CurrentStart": start, "CurrentEnd": start + length,
        "Name": name, "Annotation": annotation, "Color": colour,
        "Disabled": False, "Legato": False, "TakeId": 0, "IsInKey": False,
        "./GrooveSettings/GrooveId": -1, "./FollowAction/FollowActionEnabled": False,
        "./Loop/LoopStart": 0, "./Loop/LoopEnd": source_end,
        "./Loop/StartRelative": 0, "./Loop/LoopOn": False,
        "./Loop/OutMarker": source_end, "./Loop/HiddenLoopStart": 0,
        "./Loop/HiddenLoopEnd": source_end,
        "./ScrollerTimePreserver/LeftTime": 0,
        "./ScrollerTimePreserver/RightTime": source_end,
        "./TimeSelection/AnchorTime": 0, "./TimeSelection/OtherTime": 0,
    }.items():
        value(clip, path, content)
    clip.find("./Envelopes/Envelopes").clear()
    return clip


def audio_clip(template, clip_id, name, start, seconds, relative, colour, annotation, output):
    # Audio geometry is in seconds inside an unwarped clip; Arrangement positions are beats.
    clip = clip_common(template, clip_id, name, start, seconds * 2, seconds, colour, annotation)
    for path, content in {
        "IsWarped": False, "Sync": False, "HiQ": True, "Fade": False,
        "PitchCoarse": 0, "PitchFine": 0, "SampleVolume": 1,
        "IsSongTempoLeader": False, "MarkersGenerated": False,
        "./Fades/FadeInLength": 0, "./Fades/FadeOutLength": 0,
        "./Fades/ClipFadesAreInitialized": True,
        "./Fades/IsDefaultFadeIn": False, "./Fades/IsDefaultFadeOut": False,
        "./SampleRef/DefaultDuration": round(seconds * 48000),
        "./SampleRef/DefaultSampleRate": 48000,
        "./SampleRef/SamplesToAutoWarp": 0,
        "./SampleRef/LastModDate": 0,
        "./SampleRef/FileRef/RelativePathType": 3,
        "./SampleRef/FileRef/RelativePath": relative,
        "./SampleRef/FileRef/Path": "",
        "./SampleRef/FileRef/OriginalFileSize": (output / relative).stat().st_size,
        "./SampleRef/FileRef/OriginalCrc": 0,
        "./SampleRef/FileRef/LivePackName": "",
        "./SampleRef/FileRef/LivePackId": "",
    }.items():
        value(clip, path, content)
    # Live requires marker arrays even when Warp is off. These map the original speed
    # at 120 BPM; they do not enable stretching or alter the waveform.
    for tag in ["WarpMarkers", "SavedWarpMarkersForStretched"]:
        markers = clip.find(tag)
        markers.clear()
        ET.SubElement(markers, "WarpMarker", Id="0", SecTime="0", BeatTime="0")
        ET.SubElement(markers, "WarpMarker", Id="1", SecTime=str(seconds), BeatTime=str(seconds * 2))
    clip.find("./Onsets/UserOnsets").clear()
    value(clip, "./Onsets/HasUserOnsets", False)
    clip.find("./SampleRef/SourceContext").clear()
    return clip


def build(args):
    global PACK, SET_NAME
    PACK = args.pack.resolve()
    SET_NAME = args.set_name
    if Path(SET_NAME).name != SET_NAME or not SET_NAME.endswith(".als"):
        raise ValueError("Set name must be a plain .als filename")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    (output / "Ableton Project Info").mkdir(exist_ok=True)
    root = read_set(args.base_set)
    live = root.find("LiveSet")
    templates = read_set(args.clip_set)
    midi_template = templates.find(".//MidiClip")
    audio_template = templates.find(".//AudioClip")
    if midi_template is None or audio_template is None:
        raise ValueError("The clip Set needs one ordinary MIDI and one audio clip")
    originals = live.find("Tracks")
    midi_track = copy.deepcopy(originals.find("MidiTrack"))
    audio_track = copy.deepcopy(originals.find("AudioTrack"))
    if midi_track is None or audio_track is None:
        raise ValueError("The base needs a MIDI and an audio track")
    if midi_track.findall(".//MidiClip") or audio_track.findall(".//AudioClip"):
        raise ValueError("Base tracks must be empty")
    originals.clear()
    next_id = 100000

    def track(template, track_id, name, colour, audible, armed=False):
        nonlocal next_id
        node = copy.deepcopy(template)
        node.set("Id", str(track_id))
        mapping = {}
        for element in node.iter():
            if global_id(element):
                old = element.get("Id")
                mapping[old] = str(next_id)
                element.set("Id", str(next_id))
                next_id += 1
        for element in node.iter("PointeeId"):
            if element.get("Value") in mapping:
                element.set("Value", mapping[element.get("Value")])
        for path, content in {
            "./Name/EffectiveName": name, "./Name/UserName": name,
            "Color": colour, "./DeviceChain/Mixer/Speaker/Manual": audible,
            "./DeviceChain/Mixer/Volume/Manual": 1,
            "./DeviceChain/Mixer/Pan/Manual": 0,
            "./DeviceChain/Mixer/SoloSink": False,
            "./DeviceChain/MainSequencer/MonitoringEnum": 2,
            "./DeviceChain/MainSequencer/Recorder/IsArmed": armed,
            "./DeviceChain/MainSequencer/Recorder/TakeCounter": 0,
            "IsContentSelectedInDocument": armed,
        }.items():
            value(node, path, content)
        node.find("./DeviceChain/Mixer/Sends").clear()
        node.find("./DeviceChain/DeviceChain/Devices").clear()
        node.find("./AutomationEnvelopes/Envelopes").clear()
        if node.tag == "AudioTrack":
            value(node, "./DeviceChain/MainSequencer/KeepRecordMonitoringLatency", False)
            # Template mono input 1 is a placeholder; the owner selects the actual input.
            value(node, "./DeviceChain/AudioInputRouting/Target", "AudioIn/External/M0")
            value(node, "./DeviceChain/AudioInputRouting/LowerDisplayString", "1")
        originals.append(node)
        return node

    midi = track(midi_track, 12, "MODEL D MIDI - choose output, Ch 1", 19, True)
    listen = track(audio_track, 8, "CA-72 LISTEN - unmute to audition", 16, False)
    record = track(audio_track, 14, "MODEL D RECORD - choose mono input", 1, True, True)
    raw = track(audio_track, 15, "CA-72 RAW - fixed preset volume", 24, False)
    probe = track(audio_track, 16, "EXT PROBE 23 - choose dedicated output", 26, False)
    value(probe, "./DeviceChain/AudioOutputRouting/Target", "AudioOut/None")
    value(probe, "./DeviceChain/AudioOutputRouting/UpperDisplayString", "No Output")
    value(probe, "./DeviceChain/AudioOutputRouting/LowerDisplayString", "")

    for element in live.findall("./MainTrack/DeviceChain/Mixer/Sends") + live.findall("./PreHearTrack/DeviceChain/Mixer/Sends"):
        element.clear()
    live.find("SendsPre").clear()
    live.find("SignalModulations").clear()
    live.find("GroovePool").clear()
    value(live, "./MainTrack/DeviceChain/Mixer/Tempo/Manual", 120)
    value(live, "ResetNonautomatedMidiControllersOnClipStarts", False)
    value(live, "SelectedDocumentViewInMainWindow", 0)
    value(live, "HighlightedTrackIndex", 2)
    value(live, "ShowVideoWindow", False)
    value(live, "ViewStateMainWindowClipDetailOpen", False)
    value(live, "ViewStateMainWindowDeviceDetailOpen", False)
    value(live, "./SequencerNavigator/BeatTimeHelper/CurrentZoom", 1.2)
    value(live, "./Transport/CurrentTime", 0)
    value(live, "./Transport/LoopStart", 4)
    value(live, "./Transport/LoopLength", 9.6)
    value(live, "./Transport/LoopOn", False)
    value(live, "./Transport/PunchIn", False)
    value(live, "./Transport/PunchOut", False)
    value(live, "./TimeSelection/AnchorTime", 4)
    value(live, "./TimeSelection/OtherTime", 13.6)

    locators = live.find("./Locators/Locators")
    locators.clear()
    phrases = json.loads((PACK / "phrases.json").read_text())["presets"]
    if len(phrases) != 24:
        raise ValueError("Expected 24 comparison examples")
    (output / "Samples/Imported/CA-72 Listening").mkdir(parents=True, exist_ok=True)
    (output / "Samples/Imported/CA-72 Raw").mkdir(parents=True, exist_ok=True)
    (output / "Samples/Imported/External Probe").mkdir(parents=True, exist_ok=True)
    schedule = []
    for patch in phrases:
        n = patch["number"]
        section = (n - 1) * 64
        start = section + 4
        seconds = patch["seconds"]
        name = patch["name"][4:]
        annotation = (patch["purpose"] + "\n" + patch["hardware"]
                      + "\nListen: " + patch["listen"])
        if patch.get("cutoff_sweep"):
            annotation += "\nCutoff time/dial: " + "; ".join(f"{t:g}s/{v:+g}" for t, v in patch["cutoff_sweep"])
        colour = 26 if patch.get("cutoff_sweep") else (22 if n == 13 else 19)
        clip = clip_common(midi_template, n - 1, name, start, seconds * 2, seconds * 2, colour, annotation)
        keys = clip.find("./Notes/KeyTracks")
        keys.clear()
        key_map = {}
        for note_id, (at, held, key) in enumerate(patch["notes"], 1):
            if key not in key_map:
                row = ET.SubElement(keys, "KeyTrack", Id=str(len(key_map)))
                key_map[key] = ET.SubElement(row, "Notes")
                ET.SubElement(row, "MidiKey", Value=str(key))
            ET.SubElement(key_map[key], "MidiNoteEvent", Time=str(at * 2),
                          Duration=str(held * 2), Velocity="100", OffVelocity="0",
                          Probability="1", IsEnabled="true", NoteId=str(note_id))
        clip.find("./Notes/PerNoteEventStore/EventLists").clear()
        clip.find("./Notes/NoteProbabilityGroups").clear()
        value(clip, "./Notes/NoteIdGenerator/NextId", len(patch["notes"]) + 1)
        value(clip, "./Notes/ProbabilityGroupIdGenerator/NextId", 1)
        value(clip, "NoteEditorFoldOutZoom", -1)
        value(clip, "NoteEditorFoldOutScroll", 0)
        value(clip, "NoteEditorFoldInZoom", -1)
        value(clip, "NoteEditorFoldInScroll", 0)
        controllers = midi.find("./DeviceChain/MainSequencer/MidiControllers")
        envelope(clip, 0, controllers.find(PITCH_BEND_TARGET).get("Id"), 0)
        if patch["cc1"] is not None:
            envelope(clip, 1, controllers.find(CC1_TARGET).get("Id"), patch["cc1"])
        midi.find(ARRANGER).append(clip)
        for kind, folder, lane, channel_colour in [
            ("listening", "CA-72 Listening", listen, 16),
            ("raw", "CA-72 Raw", raw, 24),
        ]:
            relative = f"Samples/Imported/{folder}/{patch['name']}.wav"
            shutil.copy2(PACK / "audio" / kind / (patch["name"] + ".wav"), output / relative)
            lane.find(AUDIO_ARRANGER).append(audio_clip(audio_template, n - 1, name, start,
                seconds, relative, channel_colour, annotation, output))
        locator = ET.SubElement(locators, "Locator", Id=str(n - 1))
        for key, content in [("LomId", 0), ("Time", section), ("Name", name),
                             ("Annotation", annotation), ("IsSongStart", False)]:
            ET.SubElement(locator, key, Value=str(content).lower() if isinstance(content, bool) else str(content))
        schedule.append({"number": n, "name": name, "locator_bar": section // 4 + 1,
                         "clip_bar": start // 4 + 1, "start_seconds": start / 2,
                         "seconds": seconds, "end_seconds": start / 2 + seconds,
                         "notes": patch["notes"], "cutoff_sweep": patch.get("cutoff_sweep")})
        if n == 23:
            relative = "Samples/Imported/External Probe/external-input.wav"
            shutil.copy2(PACK / "external-input.wav", output / relative)
            probe.find(AUDIO_ARRANGER).append(audio_clip(audio_template, 0, "23 EXTERNAL INPUT PROBE",
                start, seconds, relative, 26, "Route to the Model D external input, using a separate physical output. Match actual input voltage as described in Guide.", output))
    value(live, "NextPointeeId", next_id)
    global_ids = [e.get("Id") for e in root.iter() if global_id(e)]
    if len(global_ids) != len(set(global_ids)):
        raise ValueError("Duplicate global automation IDs")
    if record.find(AUDIO_ARRANGER) is None or len(record.find(AUDIO_ARRANGER)):
        raise ValueError("Recording lane must be empty")
    guide = output / "Guide"
    guide.mkdir(exist_ok=True)
    for folder in ["presets", "midi", "panels"]:
        shutil.copytree(PACK / folder, guide / folder, dirs_exist_ok=True)
    for file in ["bank.toml", "phrases.json", "measurements.json", "external-input.wav"]:
        shutil.copy2(PACK / file, guide / file)
    text = (PACK / "README.md").read_text()
    text = text.replace("audio/listening/", "../Samples/Imported/CA-72%20Listening/")
    text = text.replace("audio/raw/", "../Samples/Imported/CA-72%20Raw/")
    (guide / "README.md").write_text(text)
    (output / "Comparison Schedule.json").write_text(json.dumps(schedule, indent=2) + "\n")
    (output / "START HERE.txt").write_text("""CA-72 / MODEL D COMPARISON RECORDING PROJECT

Open CA-72 vs Model D - 24 Comparisons.als in Live 12.
120 BPM, 4/4. Keep the tempo unchanged; all audio clips have Warp and fades off.
07 and 10 are 10 seconds; the other 22 clips are 4.8 seconds. All MIDI is monophonic.

1. On MODEL D MIDI, choose your hardware MIDI output and channel 1.
2. On MODEL D RECORD, choose the connected mono audio input. This track is armed,
   Monitor Off, at 0 dB. Choose 48 kHz in Live's audio settings. Use direct monitoring
   on your interface if you want to hear the hardware while recording.
3. Select the numbered locator, set the hardware patch using Guide/README.md or the
   matching panel image in Guide/panels, and record with Arrangement Record. The
   matching MIDI and references start at the next bar, giving two seconds of lead-in.
   Stop after that example and change the patch before the next take. Each section
   occupies 16 bars; do not expect the Model D to load presets or change knobs itself.
4. CA-72 LISTEN holds the existing constant-gain listening copies. CA-72 RAW holds
   the unchanged raw render at its preset volume. Both begin muted for recording.
   Unmute one reference lane to audition it; mute the other to avoid summing copies.
   Compare with the dry MODEL D RECORD take. Apply only constant downstream gain to
   listening copies; keep raw recordings, without limiting or per-note normalisation.

The capture lane is intentionally empty. No hardware takes have been recorded.
All assets are collected in Samples/Imported; no CA-72 plug-in installation is needed.
Clip and locator annotations contain the patch purpose and hardware instructions.
Comparison Schedule.json lists every locator, clip start and exact duration.

SWEEPS 07 / 10: Turn cutoff manually -5 to +5 and back. Relative to clip start:
0.00/-5, 0.40/-5, 2.40/0, 4.40/+5, 5.40/+5, 7.40/0, 9.40/-5.
The reference audio contains this sweep; MIDI sends the held note and depth settings.
There is no automatic physical cutoff control. Sweep cues are in clip annotations.

WHITE LOTUS 13: Before the take, clear MIDI modulation with CC 1 = 0, then set the
hardware MOD DEPTH knob to match the factory sound. The White Lotus performance clip
has no CC 1 envelope. Automatic non-automated MIDI-controller resets at clip start are
disabled in this Set so starting this clip does not overwrite the matched depth.
Leave the wheel/depth controller untouched during the take. See the full patch guide.

EXTERNAL INPUT 23: The EXT PROBE 23 track is muted and has No Output. Choose a separate
physical output feeding the hardware's external input, then unmute the probe for that
take only. Measure the input voltage: the source is 0.1 peak, representing 0.5 V peak
in CA-72. The probe starts exactly with the other example 23 clips.

FEEDBACK 24: Patch the PHONES return according to the full guide before that take.
""")
    if SET_NAME != "CA-72 vs Model D - 24 Comparisons.als":
        instructions = output / "START HERE.txt"
        text = instructions.read_text().replace("CA-72 vs Model D - 24 Comparisons.als", SET_NAME)
        if any(p.get("sweep_note_policy") == "repeating_phrase" for p in phrases):
            text = text.replace("MIDI sends the held note and depth settings.",
                                "MIDI sends the repeating musical phrase and depth settings.")
        instructions.write_text(text)
    ET.indent(root)
    xml = ET.tostring(root, encoding="utf-8", xml_declaration=True)
    if b"/Users/" in xml or b"Holst" in xml or b"Swing 16ths" in xml:
        raise ValueError("Source-project content leaked into the portable Set")
    with (output / SET_NAME).open("wb") as destination:
        with gzip.GzipFile(filename="", mode="wb", fileobj=destination, mtime=0) as compressed:
            compressed.write(xml)
    manifest = {"format": 1, "set": SET_NAME, "examples": 24, "tempo": 120,
                "clip_start_lead_in_seconds": 2, "tracks": 5,
                "hardware_recordings_included": False,
                "source_phrase_sha256": hashlib.sha256((PACK / "phrases.json").read_bytes()).hexdigest(),
                "files": {p.relative_to(output).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                          for p in sorted(output.rglob("*"))
                          if p.is_file() and p.name != "PROJECT MANIFEST.json"}}
    (output / "PROJECT MANIFEST.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"{output}: 24 MIDI clips, 48 reference clips, 1 probe, 24 locators, empty armed mono capture lane")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base-set", type=Path, required=True)
    parser.add_argument("--clip-set", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--pack", type=Path, default=PACK)
    parser.add_argument("--set-name", default=SET_NAME)
    build(parser.parse_args())
