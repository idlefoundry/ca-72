# SIGNAL BASIN's tools

*Dedicated to the public domain under CC0 1.0, as the pack is.*

The scripts that made the SIGNAL BASIN pack (`../../signal-basin`), kept outside it because a
pack holds no code. Python 3 with Pillow and NumPy.

- `build.py <pack>`: writes the pack's `layout.toml` and `art/panel.svg` from one description
  of the panel (its wear is seeded, so it writes the same files each time).
- `strip.py <pack>`, `keyboard.py <pack>`, `drawer.py <pack>`: write `art/strip.svg`,
  `art/keyboard.svg` and `art/drawer.svg`, the drawings over the strip, the keyboard view's case
  and the presets' drawer, lined up with the plug-in's places (seeded, as `build.py`).
- `process.py <generated> <pack> [slot...]`: cuts each generated picture out and sizes it for
  its slot.
- `lamp.py <pack>/pictures/rocker-blue.png`: moves the rocker's lamp to its flat half.
- `grade.py <pack> [slot...]`: grades the wood, knobs, face and plate towards the concept.
- `crops.py <concept> <render> <folder>`: the same thirds of a concept and a render, for a
  reviewer to compare.

Order: `process.py`, `lamp.py`, `grade.py`, then `build.py`. The generated originals, the
concept picture's crops and the prompts are on the lab's share (`ca-72/look/signal-basin`);
the prompts are in the pack's README too.
