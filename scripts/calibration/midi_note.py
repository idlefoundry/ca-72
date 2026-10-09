#!/usr/bin/env python3
"""Hold a MIDI note on the reference (channel 1, from the interface's MIDI OUT) for a take
that needs its keyboard's voltage, releasing the one held before:

    midi_note.py NOTE      (NOTE 0 releases every note)
"""

import json
import os
import sys
import time

from midi_capture import open_midi, send, libc

STATE = os.path.expanduser("~/.ca72-midi-note")


def main():
    note = int(sys.argv[1])
    port, dest = open_midi()
    now = libc.mach_absolute_time()
    if os.path.exists(STATE):
        with open(STATE) as f:
            held = json.load(f).get("note")
        if held:
            send(port, dest, 0x80, held, 0, now)
    if note:
        time.sleep(0.05)
        send(port, dest, 0x90, note, 100, libc.mach_absolute_time())
    with open(STATE, "w") as f:
        json.dump({"note": note or None}, f)
    time.sleep(0.05)
    print(f"holding {note}" if note else "released")


if __name__ == "__main__":
    main()
