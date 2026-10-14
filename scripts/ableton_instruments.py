#!/usr/bin/env python3
"""Make 24 editable MIDI tracks with fully recalled CA-72 VST3 presets."""

import argparse
import copy
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import xml.etree.ElementTree as ET

from ableton_comparison import (
    ARRANGER, AUDIO_ARRANGER, CC1_TARGET, PITCH_BEND_TARGET,
    audio_clip, clip_common, envelope, global_id, read_set, value,
)

SET_NAME = "CA-72 Musical Presets - 24 Instances.als"


def build(args):
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    (out / "Ableton Project Info").mkdir()
    root = read_set(args.base_set)
    live = root.find("LiveSet")
    clips = read_set(args.clip_set)
    midi_clip = clips.find(".//MidiClip")
    source_clip = clips.find(".//AudioClip")
    device = ET.parse(args.plugin_device).getroot()
    assert device.tag == "PluginDevice"
    phrases = json.loads((args.pack / "phrases.json").read_text())["presets"]
    states = json.loads(args.states.read_text())["presets"]
    assert len(phrases) == len(states) == 24
    tracks = live.find("Tracks")
    midi_template = copy.deepcopy(tracks.find("MidiTrack"))
    audio_template = copy.deepcopy(tracks.find("AudioTrack"))
    tracks.clear()
    next_id = 100000

    def remap(node):
        nonlocal next_id
        mapping = {}
        for e in node.iter():
            if global_id(e):
                old = e.get("Id")
                mapping[old] = str(next_id)
                e.set("Id", str(next_id))
                next_id += 1
        for e in node.iter("PointeeId"):
            if e.get("Value") in mapping:
                e.set("Value", mapping[e.get("Value")])
        return node

    def track(template, number, name, colour):
        t = remap(copy.deepcopy(template))
        t.set("Id", str(number))
        for path, content in {
            "./Name/EffectiveName": name, "./Name/UserName": name,
            "Color": colour, "IsContentSelectedInDocument": False,
            "./DeviceChain/Mixer/Speaker/Manual": True,
            "./DeviceChain/Mixer/Volume/Manual": 1,
            "./DeviceChain/Mixer/Pan/Manual": 0,
            "./DeviceChain/Mixer/SoloSink": False,
            "./DeviceChain/MainSequencer/MonitoringEnum": 2,
            "./DeviceChain/MainSequencer/Recorder/IsArmed": False,
            "./DeviceChain/MainSequencer/Recorder/TakeCounter": 0,
        }.items():
            value(t, path, content)
        t.find("./DeviceChain/Mixer/Sends").clear()
        t.find("./AutomationEnvelopes/Envelopes").clear()
        t.find("./DeviceChain/DeviceChain/Devices").clear()
        t.find(ARRANGER if t.tag == "MidiTrack" else AUDIO_ARRANGER).clear()
        for events in t.findall(".//ClipSlot/ClipSlot/Value"):
            events.clear()
        if t.tag == "MidiTrack":
            for path, text in {
                "./DeviceChain/MidiInputRouting/Target": "MidiIn/None",
                "./DeviceChain/MidiInputRouting/UpperDisplayString": "No Input",
                "./DeviceChain/MidiInputRouting/LowerDisplayString": "",
            }.items():
                value(t, path, text)
        tracks.append(t)
        return t

    def instrument(state):
        d = remap(copy.deepcopy(device))
        d.set("Id", "0")
        for path, text in {
            "UserName": state["name"], "Annotation": "Embedded complete CA-72 preset; editable MIDI.",
            "IsExpanded": True, "LastSelectedTimeableIndex": 0,
            "LastSelectedClipEnvelopeIndex": 0,
            "./PluginDesc/Vst3PluginInfo/IsPlaceholderDevice": False,
            "./PluginDesc/Vst3PluginInfo/WinPosX": -1,
            "./PluginDesc/Vst3PluginInfo/WinPosY": -1,
            "./SideChain/OnOff/Manual": False,
        }.items():
            value(d, path, text)
        d.find("./SourceContext/Value").clear()
        for tag in ["LastPresetRef", "LockedScripts"]:
            d.find(tag).clear()
        preset = d.find("./PluginDesc/Vst3PluginInfo/Preset/Vst3Preset")
        value(preset, "Name", state["name"])
        preset.find("ControllerState").text = None
        preset.find("PresetRef").clear()
        preset.find("ParameterSettings").clear()
        processor = json.dumps(state["state"], separators=(",", ":"), ensure_ascii=False).encode()
        preset.find("ProcessorState").text = processor.hex().upper()
        params = {str(p["vst3_id"]): p for p in state["parameters"]}
        listed = d.find("ParameterList")
        seen = set()
        for p in list(listed):
            key = p.find("ParameterId").get("Value")
            if key == "-1":
                listed.remove(p)
                continue
            if key not in params:
                raise ValueError(f"Unknown template parameter {key}")
            value(p, "./ParameterValue/Manual", params[key]["plain"])
            value(p, "ParameterName", params[key]["name"])
            seen.add(key)
        # Live represents the VST3 bypass parameter with device On/IsOn rather than a row.
        bypass = next(p for p in state["parameters"] if p["id"] == "bypass")
        value(d, "./On/Manual", bypass["plain"] < 0.5)
        value(preset, "IsOn", bypass["plain"] < 0.5)
        if seen != params.keys() - {str(bypass["vst3_id"])}:
            raise ValueError("The native parameter list differs from the actual VST3 interface")
        return d

    schedule = []
    cursor = 0
    for p, s in zip(phrases, states):
        assert p["name"] == s["name"]
        n = p["number"]
        colour = 26 if p.get("cutoff_sweep") else (22 if n == 13 else 19)
        start = cursor + 4  # Two-second lead before the first musical phrase.
        slot = 32 if p["seconds"] == 10 else 16
        name = p["name"][4:]
        t = track(midi_template, n + 10, name, colour)
        d = instrument(s)
        t.find("./DeviceChain/DeviceChain/Devices").append(d)
        annotation = p["musical_intent"] + "\n" + p["listen"]
        c = clip_common(midi_clip, 0, name, start, p["seconds"] * 2,
                        p["seconds"] * 2, colour, annotation)
        keys = c.find("./Notes/KeyTracks")
        keys.clear()
        key_map = {}
        for note_id, (at, held, key) in enumerate(p["notes"], 1):
            if key not in key_map:
                row = ET.SubElement(keys, "KeyTrack", Id=str(len(key_map)))
                key_map[key] = ET.SubElement(row, "Notes")
                ET.SubElement(row, "MidiKey", Value=str(key))
            ET.SubElement(key_map[key], "MidiNoteEvent", Time=str(at * 2),
                          Duration=str(held * 2), Velocity="100", OffVelocity="0",
                          Probability="1", IsEnabled="true", NoteId=str(note_id))
        c.find("./Notes/PerNoteEventStore/EventLists").clear()
        c.find("./Notes/NoteProbabilityGroups").clear()
        value(c, "./Notes/NoteIdGenerator/NextId", len(p["notes"]) + 1)
        value(c, "./Notes/ProbabilityGroupIdGenerator/NextId", 1)
        for path, amount in {
            "NoteEditorFoldOutZoom": -1, "NoteEditorFoldOutScroll": 0,
            "NoteEditorFoldInZoom": -1, "NoteEditorFoldInScroll": 0,
        }.items():
            value(c, path, amount)
        controllers = t.find("./DeviceChain/MainSequencer/MidiControllers")
        envelope(c, 0, controllers.find(PITCH_BEND_TARGET).get("Id"), 0)
        if p["cc1"] is not None:
            envelope(c, 1, controllers.find(CC1_TARGET).get("Id"), p["cc1"])
        if p.get("cutoff_sweep"):
            cutoff = next(q for q in d.findall("./ParameterList/PluginFloatParameter")
                          if q.find("ParameterId").get("Value") == "798419949")
            target = cutoff.find("./ParameterValue/AutomationTarget").get("Id")
            envelope(c, 2, target, -5)
            events = c.findall("./Envelopes/Envelopes/ClipEnvelope")[-1].find("./Automation/Events")
            events.clear()
            for event_id, (at, amount) in enumerate([(-31536000, -5), *p["cutoff_sweep"]]):
                ET.SubElement(events, "FloatEvent", Id=str(event_id), Time=str(at * 2), Value=str(amount))
        t.find(ARRANGER).append(c)
        schedule.append({"number": n, "name": name, "track_id": n + 10,
                         "start_beats": start, "start_seconds": start / 2,
                         "clip_seconds": p["seconds"], "allocated_beats": slot,
                         "notes": p["notes"], "cutoff_sweep": p.get("cutoff_sweep")})
        cursor += slot

    # An existing test source excites preset 23; its output is heard only through CA-72.
    probe = track(audio_template, 99, "SOURCE 23 - external input test tone", 26)
    value(probe, "./DeviceChain/AudioOutputRouting/Target", "AudioOut/None")
    value(probe, "./DeviceChain/AudioOutputRouting/UpperDisplayString", "No Output")
    value(probe, "./DeviceChain/AudioOutputRouting/LowerDisplayString", "")
    relative = "Samples/Imported/external-input.wav"
    (out / "Samples/Imported").mkdir(parents=True)
    shutil.copy2(args.pack / "external-input.wav", out / relative)
    start = schedule[22]["start_beats"]
    source = audio_clip(source_clip, 0, "EXISTING EXTERNAL TEST SOURCE", start, 4.8,
                        relative, 26, "Existing 220 Hz harmonic probe, routed only to preset 23.", out)
    value(source, "CurrentEnd", start + 16)
    value(source, "./Loop/LoopOn", True)
    probe.find(AUDIO_ARRANGER).append(source)
    input_device = tracks.findall("MidiTrack")[22].find("./DeviceChain/DeviceChain/Devices/PluginDevice")
    for path, text in {
        "./SideChain/OnOff/Manual": True,
        "./SideChain/RoutedInput/Routable/Target": "AudioIn/Track.99/PreFxOut",
        "./SideChain/RoutedInput/Routable/UpperDisplayString": "SOURCE 23 - external input test tone",
        "./SideChain/RoutedInput/Routable/LowerDisplayString": "Pre FX",
        "./SideChain/RoutedInput/Volume/Manual": 1,
        "./SideChain/DryWet/Manual": 1,
    }.items():
        value(input_device, path, text)

    for e in live.findall("./MainTrack/DeviceChain/Mixer/Sends") + live.findall("./PreHearTrack/DeviceChain/Mixer/Sends"):
        e.clear()
    for tag in ["SendsPre", "SignalModulations", "GroovePool"]:
        live.find(tag).clear()
    locators = live.find("./Locators/Locators")
    locators.clear()
    for row in schedule:
        l = ET.SubElement(locators, "Locator", Id=str(row["number"] - 1))
        for key, text in [("LomId", 0), ("Time", row["start_beats"]), ("Name", row["name"]),
                          ("Annotation", "One CA-72 instance with embedded preset and editable MIDI."),
                          ("IsSongStart", False)]:
            ET.SubElement(l, key, Value=str(text).lower() if isinstance(text, bool) else str(text))
    for path, text in {
        "./MainTrack/DeviceChain/Mixer/Tempo/Manual": 120,
        "ResetNonautomatedMidiControllersOnClipStarts": False,
        "SelectedDocumentViewInMainWindow": 0, "HighlightedTrackIndex": 0,
        "ViewStateMainWindowClipDetailOpen": True,
        "ViewStateMainWindowDeviceDetailOpen": True,
        "./SequencerNavigator/BeatTimeHelper/CurrentZoom": 1.2,
        "./Transport/CurrentTime": 0, "./Transport/LoopStart": 4,
        "./Transport/LoopLength": cursor, "./Transport/LoopOn": False,
        "./Transport/PunchIn": False, "./Transport/PunchOut": False,
        "./TimeSelection/AnchorTime": 4, "./TimeSelection/OtherTime": 13.6,
        "NextPointeeId": next_id,
    }.items():
        value(live, path, text)
    ids = [e.get("Id") for e in root.iter() if global_id(e)]
    assert len(ids) == len(set(ids))
    targets = set(ids)
    assert all(e.get("Value") in targets for e in root.findall(".//EnvelopeTarget/PointeeId"))
    ET.indent(root)
    xml = ET.tostring(root, encoding="utf-8", xml_declaration=True)
    assert b"/Users/" not in xml and b"Holst" not in xml
    with (out / SET_NAME).open("wb") as f:
        with gzip.GzipFile(filename="", mode="wb", fileobj=f, mtime=0) as g:
            g.write(xml)
    (out / "Arrangement.json").write_text(json.dumps(schedule, indent=2) + "\n")
    (out / "Embedded Preset States.json").write_text(args.states.read_text())
    shutil.copytree(args.pack / "presets", out / "Presets")
    (out / "START HERE.txt").write_text(
        "CA-72 Musical Presets - 24 Instances\n\n"
        "Open CA-72 Musical Presets - 24 Instances.als after restarting Live to load the current VST3.\n"
        "24 MIDI tracks each contain one CA-72 VST3 and a complete embedded preset.\n"
        "The Round 2 notes are editable. Notes, rhythms and original preset levels are preserved.\n"
        "Tempo is 120 BPM; clips are staggered with room between their tails.\n"
        "Use the numbered locators to jump between examples. Double-click a MIDI clip to edit it.\n"
        "07 and 10 contain editable full-range cutoff automation; their sweep clips last ten seconds.\n"
        "The other clips last 4.8 seconds. Every CA-72 is monophonic and set to ULTRA.\n"
        "All tracks start unarmed with MIDI input disabled, so live keys cannot trigger all instances.\n\n"
        "SOURCE 23 is one utility audio track, feeding the existing harmonic test-tone file\n"
        "to preset 23's sidechain. It has No Output, so its dry sound does not enter the mix.\n"
        "No synthesizer audio was rendered or frozen. The MIDI on track 23 also remains editable.\n\n"
        "Presets are embedded; importing the included Presets folder is optional.\n"
        "This is a new Set; the earlier hardware-recording projects remain available.\n")
    manifest = {"format": 1, "set": SET_NAME, "ca72_instances": 24, "midi_tracks": 24,
                "existing_source_audio_tracks": 1, "synth_audio_rendered": False,
                "source_phrase_sha256": hashlib.sha256((args.pack / "phrases.json").read_bytes()).hexdigest(),
                "files": {p.relative_to(out).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                          for p in sorted(out.rglob("*")) if p.is_file()}}
    (out / "PROJECT MANIFEST.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"{out}: 24 CA-72 VST3 instances, 24 staggered editable MIDI clips, 2 cutoff sweeps")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base-set", type=Path, required=True)
    parser.add_argument("--clip-set", type=Path, required=True)
    parser.add_argument("--plugin-device", type=Path, required=True)
    parser.add_argument("--states", type=Path, required=True)
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    build(parser.parse_args())
