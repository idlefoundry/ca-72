#!/usr/bin/env python3
"""Signal Basin: writes the pack's art/drawer.svg, its drawing over the presets' drawer's face
(under its displays' glass, its keys and its print), lined up with the drawer's places (the
CA-72's `presets/drawer.rs`). Units are the drawer's: 3108 by 734, y from the rail's foot (the
strip's y less its rail's 116)."""
import sys, os, random

out = sys.argv[1]
W, H = 3108, 734
# The displays (left, top, width, height): FIND's search and tags, PRESETS' list, PRESET's.
FOOT = H - 27
SEARCH = (63, 51, 687, 47)
TAGS = (63, 110, 687, FOOT - 110)
LIST = (814, 51, 1476, FOOT - 51)
STATUS = (2354, 244, 678, FOOT - 244)
# The bays: FIND's, PRESETS', PRESET's, each round its displays (and PRESET's keys), the
# strip's gutter between; their foot below the displays' bezels, the case's wood under it.
TOP, BASE = 10, 724
bays = [(30, TOP, 780, BASE), (784, TOP, 2320, BASE), (2324, TOP, 3078, BASE)]

def rr(x0, y0, x1, y1, r):
    return (f"M{x0+r},{y0}H{x1-r}A{r},{r} 0 0 1 {x1},{y0+r}V{y1-r}A{r},{r} 0 0 1 {x1-r},{y1}"
            f"H{x0+r}A{r},{r} 0 0 1 {x0},{y1-r}V{y0+r}A{r},{r} 0 0 1 {x0+r},{y0}Z")

svg = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">',
       "<title>SIGNAL BASIN's presets drawer: FIND, PRESETS and PRESET in teal bays edged in copper, their displays in copper-rimmed black glass</title>",
       '<defs>',
       '<linearGradient id="rim" x1="0" y1="0" x2="0.25" y2="1">'
       '<stop offset="0" stop-color="#c08a60"/><stop offset="0.5" stop-color="#8a5a3a"/>'
       '<stop offset="1" stop-color="#4e3020"/></linearGradient>',
       '<linearGradient id="cheek" x1="0" y1="0" x2="1" y2="0">'
       '<stop offset="0" stop-color="#2c1e15"/><stop offset="0.45" stop-color="#4e3628"/>'
       '<stop offset="1" stop-color="#241810"/></linearGradient>',
       '<linearGradient id="footwood" x1="0" y1="0" x2="0" y2="1">'
       '<stop offset="0" stop-color="#4e3628"/><stop offset="1" stop-color="#2a1c12"/></linearGradient>',
       '<radialGradient id="head" cx="0.34" cy="0.3" r="0.8">'
       '<stop offset="0" stop-color="#9a7452"/><stop offset="0.25" stop-color="#5a3e2a"/>'
       '<stop offset="1" stop-color="#2a1c12"/></radialGradient>',
       '<radialGradient id="drop"><stop offset="0.55" stop-color="#000" stop-opacity="0.5"/>'
       '<stop offset="1" stop-color="#000" stop-opacity="0"/></radialGradient>',
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
rnd = random.Random(75)
wear = random.Random(1975)
# The face between and round the bays a shade darker; the rail's foot a dark seam.
holes = "".join(rr(*b, 10) for b in bays)
svg.append(f'<path fill-rule="evenodd" fill="#020809" fill-opacity="0.45" d="M0,0H{W}V{H}H0Z{holes}"/>')
svg.append(f'<rect x="0" y="-3" width="{W}" height="5" fill="#000" fill-opacity="0.55"/>')
# The case's wood at the drawer's ends and a slim lip at its foot, as the strip's.
for x0, w in ((0, 26), (3080, 28)):
    svg.append(f'<rect x="{x0}" y="0" width="{w}" height="{H}" fill="url(#cheek)"/>')
    for _ in range(9):
        gx = x0 + rnd.uniform(1.5, w - 1.5)
        svg.append(f'<rect x="{gx:.1f}" y="0" width="{rnd.uniform(0.4, 1.2):.1f}" height="{H}" fill="#1a0f08" fill-opacity="{rnd.uniform(0.25, 0.55):.2f}"/>')
    svg.append(f'<rect x="{x0 + (w if x0 == 0 else -3)}" y="0" width="3" height="{H}" fill="#000" fill-opacity="0.55"/>')
LIP = H - 8
svg.append(f'<rect x="0" y="{LIP}" width="{W}" height="{H - LIP}" fill="url(#footwood)"/>')
for _ in range(3):
    gy = LIP + rnd.uniform(1.0, H - LIP - 1.0)
    svg.append(f'<rect x="0" y="{gy:.1f}" width="{W}" height="{rnd.uniform(0.4, 0.9):.1f}" fill="#1a0f08" fill-opacity="{rnd.uniform(0.25, 0.5):.2f}"/>')
svg.append(f'<rect x="0" y="{LIP - 2}" width="{W}" height="2.5" fill="#000" fill-opacity="0.55"/>')
# Each bay: a soft shadow, its teal a shade deeper, scuffs and grime round its rim.
for x0, y0, x1, y1 in bays:
    for k, o in ((5, 0.2), (2, 0.3)):
        svg.append(f'<path fill-rule="evenodd" d="{rr(x0 + k * 0.6, y0 + k, x1 + k * 0.6, y1 + k, 10 + k)}{rr(x0, y0, x1, y1, 10)}" fill="#000" fill-opacity="{o}"/>')
    svg.append(f'<path d="{rr(x0, y0, x1, y1, 10)}" fill="#04161a" fill-opacity="0.18"/>')
    for _ in range(int((x1 - x0) * (y1 - y0) / 45000) + 2):
        gx = wear.uniform(x0 + 20, x1 - 20)
        gy = wear.choice([wear.uniform(y0 + 6, y0 + 40), wear.uniform(y1 - 40, y1 - 6), wear.uniform(y0, y1)])
        svg.append(f'<ellipse cx="{gx:.1f}" cy="{gy:.1f}" rx="{wear.uniform(20, 70):.1f}" ry="{wear.uniform(8, 26):.1f}" fill="url(#scuff)"/>')
    for k, o in ((14, 0.07), (6, 0.11)):
        svg.append(f'<path fill-rule="evenodd" d="{rr(x0 - k, y0 - k, x1 + k, y1 + k, 10 + k)}{rr(x0, y0, x1, y1, 10)}" fill="#050302" fill-opacity="{o}"/>')
# PRESET's keys: a dark recess behind the eight, no copper (the keys carry the group).
KX0, KX1, KY0, KY1 = 2422 - 68 - 14, 2965 + 68 + 14, 99 - 26 - 12, 197 + 26 + 14
svg.append(f'<path d="{rr(KX0, KY0, KX1, KY1, 10)}" fill="#0c1214" fill-opacity="0.55"/>')
svg.append(f'<path d="{rr(KX0 + 1.5, KY0 + 1.5, KX1 - 1.5, KY1 - 1.5, 9)}" fill="none" stroke="#000" stroke-opacity="0.45" stroke-width="2"/>')
# Each display: a copper rim round its bezel, lit above and shaded below; FIND's two (their
# bezels meet) in one frame, a copper bar between them.
FIND = (SEARCH[0], SEARCH[1], SEARCH[2], TAGS[1] + TAGS[3] - SEARCH[1])
for x, y, w, h in (FIND, LIST, STATUS):
    P = 9
    svg.append(f'<path d="{rr(x - P + 1.5, y - P + 2.5, x + w + P + 1.5, y + h + P + 2.5, 10)}" fill="none" stroke="#000" stroke-opacity="0.4" stroke-width="3"/>')
    svg.append(f'<path d="{rr(x - P, y - P, x + w + P, y + h + P, 10)}" fill="none" stroke="url(#rim)" stroke-width="3"/>')
    svg.append(f'<path d="M{x - P + 3},{y + h + P - 2}V{y - P + 3}H{x + w + P - 3}" fill="none" stroke="#b88258" stroke-opacity="0.5" stroke-width="1"/>')
bar = (SEARCH[1] + SEARCH[3] + TAGS[1]) / 2
svg.append(f'<rect x="{SEARCH[0] - 9}" y="{bar - 2.5}" width="{SEARCH[2] + 18}" height="5" fill="url(#rim)"/>')
svg.append(f'<rect x="{SEARCH[0] - 9}" y="{bar + 2.5}" width="{SEARCH[2] + 18}" height="1.2" fill="#000" fill-opacity="0.5"/>')
# The bays' rims, bevelled as the panel's, a small screw in each corner (clear of the
# displays, which reach near the bays' edges).
for x0, y0, x1, y1 in bays:
    d = rr(x0, y0, x1, y1, 10)
    svg.append(f'<path d="{rr(x0 + 3, y0 + 3, x1 - 3, y1 - 3, 8)}" fill="none" stroke="#000" stroke-opacity="0.45" stroke-width="2.5"/>')
    svg.append(f'<path d="{rr(x0 - 1.8, y0 - 1.8, x1 + 1.8, y1 + 1.8, 11.5)}" fill="none" stroke="#000" stroke-opacity="0.35" stroke-width="1.2"/>')
    svg.append(f'<path d="{d}" fill="none" stroke="url(#rim)" stroke-width="2.8"/>')
    dash = ",".join(f"{wear.uniform(15, 80):.0f},{wear.uniform(5, 40):.0f}" for _ in range(12))
    svg.append(f'<path d="{rr(x0 + 1.4, y0 + 1.4, x1 - 1.4, y1 - 1.4, 8.8)}" fill="none" stroke="#d6a07a" stroke-opacity="0.35" stroke-width="0.8" stroke-dasharray="{dash}"/>')
    for sx, sy in [(x0 + 13, y0 + 13), (x1 - 13, y0 + 13), (x0 + 13, y1 - 13), (x1 - 13, y1 - 13)]:
        svg.append(f'<circle cx="{sx}" cy="{sy}" r="{wear.uniform(14, 18):.1f}" fill="url(#grime)"/>')
        svg.append(f'<use href="#sc" transform="translate({sx} {sy}) scale(0.6) rotate({rnd.uniform(0, 90):.0f})"/>')
svg.append("</svg>")
os.makedirs(os.path.join(out, "art"), exist_ok=True)
with open(os.path.join(out, "art", "drawer.svg"), "w") as f:
    f.write("\n".join(svg) + "\n")
print("drawer drawn:", len(svg), "elements")
