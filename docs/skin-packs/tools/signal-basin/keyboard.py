#!/usr/bin/env python3
"""Signal Basin: writes the pack's art/keyboard.svg, its drawing over the keyboard view's case
(the pack's face laid there: `keyboard_case = "face"`), lined up with the keyboard's places (the
CA-72's `keyboard.rs`). Units are the strip's: 3108 by 850, y from the rail's top."""
import sys, os, math, random

out = sys.argv[1]
W, H = 3108, 850
AREA = (370, 116, 3080, 844)            # the keyboard view's case
BED = (402, 185, 3050, 817)             # its key bed (its own dark stroke round it)
RANGE = (1069, 124, 348, 50)            # the OCTAVE range's window
CASE = (378, 118, 3074, 840)

def rr(x0, y0, x1, y1, r):
    return (f"M{x0+r},{y0}H{x1-r}A{r},{r} 0 0 1 {x1},{y0+r}V{y1-r}A{r},{r} 0 0 1 {x1-r},{y1}"
            f"H{x0+r}A{r},{r} 0 0 1 {x0},{y1-r}V{y0+r}A{r},{r} 0 0 1 {x0+r},{y0}Z")

svg = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">',
       "<title>SIGNAL BASIN's keyboard: its keys in a copper-rimmed bed, in a teal case edged in copper</title>",
       '<defs>',
       '<linearGradient id="rim" x1="0" y1="0" x2="0.25" y2="1">'
       '<stop offset="0" stop-color="#c08a60"/><stop offset="0.5" stop-color="#8a5a3a"/>'
       '<stop offset="1" stop-color="#4e3020"/></linearGradient>',
       '<radialGradient id="head" cx="0.34" cy="0.3" r="0.8">'
       '<stop offset="0" stop-color="#9a7452"/><stop offset="0.25" stop-color="#5a3e2a"/>'
       '<stop offset="1" stop-color="#2a1c12"/></radialGradient>',
       '<radialGradient id="drop"><stop offset="0.55" stop-color="#000" stop-opacity="0.5"/>'
       '<stop offset="1" stop-color="#000" stop-opacity="0"/></radialGradient>',
       '<linearGradient id="slip" x1="0" y1="0" x2="0" y2="1">'
       '<stop offset="0" stop-color="#5a3e2c"/><stop offset="1" stop-color="#2a1c12"/></linearGradient>',
       '<radialGradient id="grime"><stop offset="0" stop-color="#050302" stop-opacity="0.34"/>'
       '<stop offset="1" stop-color="#050302" stop-opacity="0"/></radialGradient>',
       '<radialGradient id="scuff"><stop offset="0" stop-color="#9fc6c4" stop-opacity="0.07"/>'
       '<stop offset="1" stop-color="#9fc6c4" stop-opacity="0"/></radialGradient>',
       '<g id="sc">'
       '<circle r="16" cx="0.9" cy="2.2" fill="url(#drop)"/>'
       '<circle r="12.5" fill="url(#head)" stroke="#140d08" stroke-width="1"/>'
       '<path d="M-7.4,-1.5H-1.5V-7.4H1.5V-1.5H7.4V1.5H1.5V7.4H-1.5V1.5H-7.4Z" fill="#140d08"/>'
       '<path d="M-7.4,1.5H-1.5V7.4M1.5,1.5H7.4" fill="none" stroke="#9a7452" stroke-opacity="0.5" stroke-width="0.8"/>'
       '</g>',
       '</defs>']
rnd = random.Random(74)
wear = random.Random(1974)
x0, y0, x1, y1 = AREA
# The gap round the case darker; the case's teal a shade deeper, scuffed.
svg.append(f'<path fill-rule="evenodd" fill="#020809" fill-opacity="0.45" d="M{x0},{y0}H{x1}V{y1}H{x0}Z{rr(*CASE, 10)}"/>')
svg.append(f'<path d="{rr(*CASE, 10)}" fill="#04161a" fill-opacity="0.18"/>')
cx0, cy0, cx1, cy1 = CASE
for _ in range(14):
    gx = wear.uniform(cx0 + 20, cx1 - 20)
    gy = wear.choice([wear.uniform(cy0 + 5, cy0 + 50), wear.uniform(cy1 - 40, cy1 - 5)])
    svg.append(f'<ellipse cx="{gx:.1f}" cy="{gy:.1f}" rx="{wear.uniform(20, 70):.1f}" ry="{wear.uniform(6, 16):.1f}" fill="url(#scuff)"/>')
# The case's rim, bevelled as the panel's, its screws in its top corners (the key bed reaches
# its foot), grime round them.
d = rr(*CASE, 10)
svg.append(f'<path d="{rr(cx0 + 3, cy0 + 3, cx1 - 3, cy1 - 3, 8)}" fill="none" stroke="#000" stroke-opacity="0.45" stroke-width="2.5"/>')
svg.append(f'<path d="{d}" fill="none" stroke="url(#rim)" stroke-width="2.8"/>')
dash = ",".join(f"{wear.uniform(15, 80):.0f},{wear.uniform(5, 40):.0f}" for _ in range(12))
svg.append(f'<path d="{rr(cx0 + 1.4, cy0 + 1.4, cx1 - 1.4, cy1 - 1.4, 8.8)}" fill="none" stroke="#d6a07a" stroke-opacity="0.35" stroke-width="0.8" stroke-dasharray="{dash}"/>')
for sx, sy in [(cx0 + 22, cy0 + 30), (cx1 - 22, cy0 + 30)]:
    svg.append(f'<circle cx="{sx}" cy="{sy}" r="26" fill="url(#grime)"/>')
    svg.append(f'<use href="#sc" transform="translate({sx} {sy}) rotate({rnd.uniform(0, 90):.0f})"/>')
# The key bed's copper rim, just outside its own dark edge, and the shade it is let into.
bx0, by0, bx1, by1 = BED
svg.append(f'<path d="{rr(bx0 - 9, by0 - 9, bx1 + 9, by1 + 9, 12)}" fill="#000" fill-opacity="0.35"/>')
svg.append(f'<path d="{rr(bx0 - 7, by0 - 7, bx1 + 7, by1 + 7, 11)}" fill="none" stroke="url(#rim)" stroke-width="3"/>')
# The range's window: a thin copper bezel round it.
rx, ry, rw, rh = RANGE
svg.append(f'<path d="{rr(rx - 9, ry - 5, rx + rw + 9, ry + rh + 5, 8)}" fill="#0a0e10" fill-opacity="0.85"/>')
svg.append(f'<path d="{rr(rx - 8, ry - 4, rx + rw + 8, ry + rh + 4, 7)}" fill="none" stroke="url(#rim)" stroke-width="3.2"/>')
svg.append(f'<path d="M{rx - 6},{ry + rh + 2}V{ry - 2}H{rx + rw + 6}" fill="none" stroke="#b88258" stroke-opacity="0.55" stroke-width="1"/>')
# The key slip under the bed: a strip of the case's wood, its top edged in copper, a small
# screw at each end.
sy0, sy1 = by1 + 10, cy1 - 3
svg.append(f'<rect x="{bx0 - 6}" y="{sy0}" width="{bx1 - bx0 + 12}" height="{sy1 - sy0}" rx="3" fill="url(#slip)"/>')
for _ in range(5):
    gy = sy0 + rnd.uniform(1.5, sy1 - sy0 - 1.5)
    svg.append(f'<rect x="{bx0 - 6}" y="{gy:.1f}" width="{bx1 - bx0 + 12}" height="{rnd.uniform(0.4, 0.9):.1f}" fill="#1a0f08" fill-opacity="{rnd.uniform(0.3, 0.5):.2f}"/>')
svg.append(f'<rect x="{bx0 - 6}" y="{sy0}" width="{bx1 - bx0 + 12}" height="1.4" fill="#b88258" fill-opacity="0.7"/>')
for sx in (bx0 + 6, bx1 - 6):
    svg.append(f'<use href="#sc" transform="translate({sx} {(sy0 + sy1) / 2}) scale(0.42) rotate({rnd.uniform(0, 90):.0f})"/>')
svg.append("</svg>")
os.makedirs(os.path.join(out, "art"), exist_ok=True)
with open(os.path.join(out, "art", "keyboard.svg"), "w") as f:
    f.write("\n".join(svg) + "\n")
print("keyboard drawn:", len(svg), "elements")
