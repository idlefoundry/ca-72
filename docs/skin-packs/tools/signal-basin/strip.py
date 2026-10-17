#!/usr/bin/env python3
"""Signal Basin: writes the pack's art/strip.svg, its drawing of the plug-in's strip (the rail and
the sections under the panel), lined up with the strip's places (`cargo run -p ca72-panel
--example strip_places`). Units are the strip's: 3108 by 850, y from the rail's top."""
import sys, os, math, random

out = sys.argv[1]
W, H = 3108, 850
RAIL = 116
TOP, FOOT = RAIL + 12, H - 24          # the bays' top and foot
GAP = 8
# The bays: the left hand's, VOICES's, STEREO's, OUTPUT's (the strip's sections, a gap between).
SECTIONS = [(0, 370), (370, 976), (976, 2370), (2370, 3080)]
bays = []
for i, (a, b) in enumerate(SECTIONS):
    x0 = (32 if i == 0 else a + GAP / 2)
    x1 = (3076 if i == 3 else b - GAP / 2)
    bays.append((x0, TOP, x1, FOOT))
# What the strip draws in its bays (from strip_places): the banks' recesses, the readouts'
# windows, the voices' display.
BANKS = [(430, 187, 486, 82), (1253, 187, 330, 82), (1763, 187, 330, 82), (2554, 187, 174, 82)]
READOUTS = [(701, 410), (701, 634), (1271, 410), (2131, 410), (2753, 186), (2753, 410), (2753, 634), (1701, 410)]
RW, RH = 156, 84
DISPLAY = (1059, 614, 1228, 124)
ROWS = [228, 452, 676]

def rr(x0, y0, x1, y1, r):
    return (f"M{x0+r},{y0}H{x1-r}A{r},{r} 0 0 1 {x1},{y0+r}V{y1-r}A{r},{r} 0 0 1 {x1-r},{y1}"
            f"H{x0+r}A{r},{r} 0 0 1 {x0},{y1-r}V{y0+r}A{r},{r} 0 0 1 {x0+r},{y0}Z")

COPPER = "#a8724c"
svg = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">',
       "<title>SIGNAL BASIN's strip: teal bays edged in copper under the rail, the readouts in black glass, the signal's way into OUTPUT</title>",
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
rnd = random.Random(73)
wear = random.Random(1973)
# The face between and round the bays a shade darker; the rail's foot a dark seam.
holes = "".join(rr(*b, 10) for b in bays)
svg.append(f'<path fill-rule="evenodd" fill="#020809" fill-opacity="0.45" d="M0,{RAIL}H{W}V{H}H0Z{holes}"/>')
svg.append(f'<rect x="0" y="{RAIL - 3}" width="{W}" height="5" fill="#000" fill-opacity="0.55"/>')
# The case's wood at the strip's ends and foot, as the panel's.
for x0 in (0, 3082):
    svg.append(f'<rect x="{x0}" y="{RAIL}" width="26" height="{H - RAIL}" fill="url(#cheek)"/>')
    for _ in range(9):
        gx = x0 + rnd.uniform(1.5, 24.5)
        svg.append(f'<rect x="{gx:.1f}" y="{RAIL}" width="{rnd.uniform(0.4, 1.2):.1f}" height="{H - RAIL}" fill="#1a0f08" fill-opacity="{rnd.uniform(0.25, 0.55):.2f}"/>')
    svg.append(f'<rect x="{x0 + (26 if x0 == 0 else -3)}" y="{RAIL}" width="3" height="{H - RAIL}" fill="#000" fill-opacity="0.55"/>')
svg.append(f'<rect x="0" y="{H - 20}" width="{W}" height="20" fill="url(#footwood)"/>')
for _ in range(7):
    gy = H - 20 + rnd.uniform(1.5, 18.5)
    svg.append(f'<rect x="0" y="{gy:.1f}" width="{W}" height="{rnd.uniform(0.4, 1.1):.1f}" fill="#1a0f08" fill-opacity="{rnd.uniform(0.25, 0.5):.2f}"/>')
svg.append(f'<rect x="0" y="{H - 22}" width="{W}" height="3" fill="#000" fill-opacity="0.55"/>')
# Each bay: a soft shadow, its teal a shade deeper, grime round its rim.
for x0, y0, x1, y1 in bays:
    for k, o in ((5, 0.2), (2, 0.3)):
        svg.append(f'<path fill-rule="evenodd" d="{rr(x0 + k * 0.6, y0 + k, x1 + k * 0.6, y1 + k, 10 + k)}{rr(x0, y0, x1, y1, 10)}" fill="#000" fill-opacity="{o}"/>')
    svg.append(f'<path d="{rr(x0, y0, x1, y1, 10)}" fill="#04161a" fill-opacity="0.18"/>')
    for _ in range(int((x1 - x0) * (y1 - y0) / 45000) + 2):
        gx = wear.uniform(x0 + 20, x1 - 20)
        gy = wear.choice([wear.uniform(y0 + 10, y0 + 70), wear.uniform(y1 - 70, y1 - 10), wear.uniform(y0, y1)])
        svg.append(f'<ellipse cx="{gx:.1f}" cy="{gy:.1f}" rx="{wear.uniform(20, 70):.1f}" ry="{wear.uniform(10, 34):.1f}" fill="url(#scuff)"/>')
    for k, o in ((14, 0.07), (6, 0.11)):
        svg.append(f'<path fill-rule="evenodd" d="{rr(x0 - k, y0 - k, x1 + k, y1 + k, 10 + k)}{rr(x0, y0, x1, y1, 10)}" fill="#050302" fill-opacity="{o}"/>')
# A dark recess behind each bank of tabs, a black window round each readout and the voices'
# display, each edged in copper.
def inset(x0, y0, x1, y1, r, pad):
    d = rr(x0 - pad, y0 - pad, x1 + pad, y1 + pad, r)
    svg.append(f'<path d="{rr(x0 - pad + 1.5, y0 - pad + 2.5, x1 + pad + 1.5, y1 + pad + 2.5, r)}" fill="#000" fill-opacity="0.35"/>')
    svg.append(f'<path d="{d}" fill="#0b1012" fill-opacity="0.92"/>')
    svg.append(f'<path d="{rr(x0 - pad + 2.5, y0 - pad + 2.5, x1 + pad - 2.5, y1 + pad - 2.5, max(r - 2, 2))}" fill="none" stroke="#000" stroke-opacity="0.5" stroke-width="2"/>')
    svg.append(f'<path d="{d}" fill="none" stroke="url(#rim)" stroke-width="2.4"/>')
# The banks' recesses faint, no copper: the tabs carry the group.
for x, y, w, h in BANKS:
    svg.append(f'<path d="{rr(x - 4, y - 4, x + w + 4, y + h + 4, 9)}" fill="#0c1214" fill-opacity="0.6"/>')
    svg.append(f'<path d="{rr(x - 2.5, y - 2.5, x + w + 2.5, y + h + 2.5, 8)}" fill="none" stroke="#000" stroke-opacity="0.45" stroke-width="2"/>')
for x, y in READOUTS:
    inset(x, y, x + RW, y + RH, 7, 6)
# The voices' display: a framed instrument window, its copper bezel lit above and shaded
# below, a screw in each corner.
x, y, w, h = DISPLAY
P = 12
svg.append(f'<path d="{rr(x - P + 2, y - P + 3, x + w + P + 2, y + h + P + 3, 12)}" fill="#000" fill-opacity="0.4"/>')
svg.append(f'<path d="{rr(x - P, y - P, x + w + P, y + h + P, 12)}" fill="#0a0e10"/>')
svg.append(f'<path d="{rr(x - P + 2, y - P + 2, x + w + P - 2, y + h + P - 2, 10)}" fill="none" stroke="url(#rim)" stroke-width="4"/>')
svg.append(f'<path d="M{x - P + 4},{y + h + P - 4}H{x + w + P - 4}V{y - P + 4}" fill="none" stroke="#3a2418" stroke-opacity="0.8" stroke-width="1.2"/>')
svg.append(f'<path d="M{x - P + 4},{y + h + P - 4}V{y - P + 4}H{x + w + P - 4}" fill="none" stroke="#b88258" stroke-opacity="0.6" stroke-width="1.2"/>')
svg.append(f'<path d="{rr(x - 4, y - 4, x + w + 4, y + h + 4, 7)}" fill="none" stroke="#000" stroke-opacity="0.6" stroke-width="2.5"/>')
for sx, sy in [(x - P + 6, y - P + 6), (x + w + P - 6, y - P + 6), (x - P + 6, y + h + P - 6), (x + w + P - 6, y + h + P - 6)]:
    svg.append(f'<use href="#sc" transform="translate({sx} {sy}) scale(0.38) rotate({rnd.uniform(0, 90):.0f})"/>')
# The preset's name's window on the rail: a copper rim round its bezel, a screw at each end.
nx, ny, nw, nh = 262, 30, 2200, 50
svg.append(f'<path d="{rr(nx - 11, ny - 11, nx + nw + 11, ny + nh + 11, 9)}" fill="#0a0e10" fill-opacity="0.85"/>')
svg.append(f'<path d="{rr(nx - 10, ny - 10, nx + nw + 10, ny + nh + 10, 8)}" fill="none" stroke="url(#rim)" stroke-width="2.6"/>')
for sx in (nx - 2, nx + nw + 2):
    pass
# The bays' rims, bevelled as the panel's, and their screws.
for x0, y0, x1, y1 in bays:
    d = rr(x0, y0, x1, y1, 10)
    svg.append(f'<path d="{rr(x0 + 3, y0 + 3, x1 - 3, y1 - 3, 8)}" fill="none" stroke="#000" stroke-opacity="0.45" stroke-width="2.5"/>')
    svg.append(f'<path d="{rr(x0 - 1.8, y0 - 1.8, x1 + 1.8, y1 + 1.8, 11.5)}" fill="none" stroke="#000" stroke-opacity="0.35" stroke-width="1.2"/>')
    svg.append(f'<path d="{d}" fill="none" stroke="url(#rim)" stroke-width="2.8"/>')
    dash = ",".join(f"{wear.uniform(15, 80):.0f},{wear.uniform(5, 40):.0f}" for _ in range(12))
    svg.append(f'<path d="{rr(x0 + 1.4, y0 + 1.4, x1 - 1.4, y1 - 1.4, 8.8)}" fill="none" stroke="#d6a07a" stroke-opacity="0.35" stroke-width="0.8" stroke-dasharray="{dash}"/>')
    for sx, sy in [(x0 + 22, y0 + 22), (x1 - 22, y0 + 22), (x0 + 22, y1 - 22), (x1 - 22, y1 - 22)]:
        svg.append(f'<circle cx="{sx}" cy="{sy}" r="{wear.uniform(22, 30):.1f}" fill="url(#grime)"/>')
        svg.append(f'<use href="#sc" transform="translate({sx} {sy}) rotate({rnd.uniform(0, 90):.0f})"/>')

def arrow(pts, head=True, w=2.6):
    d = "M" + " L".join(f"{x},{y}" for x, y in pts)
    svg.append(f'<path d="{d}" fill="none" stroke="#000" stroke-opacity="0.5" stroke-width="{w + 1.6}" stroke-linejoin="round" transform="translate(0.8 1.2)"/>')
    svg.append(f'<path d="{d}" fill="none" stroke="{COPPER}" stroke-width="{w}" stroke-linejoin="round"/>')
    if head:
        (ax, ay), (bx, by) = pts[-2], pts[-1]
        a = math.atan2(by - ay, bx - ax)
        L, Wd = 20, 9
        l = (bx - L * math.cos(a) + Wd * math.sin(a), by - L * math.sin(a) - Wd * math.cos(a))
        r = (bx - L * math.cos(a) - Wd * math.sin(a), by - L * math.sin(a) + Wd * math.cos(a))
        svg.append('<polygon points="' + " ".join(f"{x:.1f},{y:.1f}" for x, y in ((bx, by), l, r)) + f'" fill="{COPPER}"/>')

# The signal's way on into OUTPUT: down its left side, into AUTO GAIN, DRIVE and LEVEL.
LX = 2418
arrow([(LX, TOP + 40), (LX, ROWS[2])], head=False, w=3.2)
for y in ROWS:
    arrow([(LX, y), (2524, y)])
svg.append("</svg>")
os.makedirs(os.path.join(out, "art"), exist_ok=True)
with open(os.path.join(out, "art", "strip.svg"), "w") as f:
    f.write("\n".join(svg) + "\n")
print("strip drawn:", len(svg), "elements")
