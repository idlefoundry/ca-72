#!/usr/bin/env python3
"""Signal Basin: writes the pack's layout.toml and art/panel.svg from one description of the
panel, so that the drawing's panels, rules and arrows sit where the layout's controls are.
Units are the layout's: the face 0..3108 by 0..795, the strip above it -130..0."""
import sys, os, math

out = sys.argv[1]
os.makedirs(os.path.join(out, "art"), exist_ok=True)

# The concept's arrangement: from the left, the controllers' teal column, the sources' (the
# external input and noise, which the concept puts in a fourth row: the panel's height has
# room for three rows of dials), three oscillator rows feeding a copper bus, the bus feeding
# the FILTER and the two contours' panels, then OUTPUT's and POWER's teal columns.
PANEL_Y = [(18, 279), (299, 528), (548, 777)]    # the three rows' modules, 20 apart
R = [176, 425, 674]                              # their knobs' centres
LEG = 106                                        # a legend's height above its knob
TALL = (6, 789)                                  # a bay
INNER = (18, 777)                                # a module the height of the central bay
X = {"ctl": (30, 340), "bay": (348, 2597), "src": (360, 676), "osc": (698, 1724),
     "mod": (1834, 2585), "out": (2605, 2959), "pwr": (2967, 3078)}
BUS = 1769                                       # the bus, in the channel between them
FEED = 687                                       # the sources' feeder, in the gap beside them

controls, texts, lines = [], [], []

def knob(param, x, y, big=False):
    c = {"param": param, "at": [x, y]}
    if big:
        c["big"] = True
    controls.append(c)

def rocker(param, x, y, on="right", size=None):
    if size is None:
        size = [108, 50] if on == "right" else [54, 100]
    controls.append({"param": param, "at": [x, y], "size": size, "colour": "blue", "on": on})

def text(t, x, y, size=22):
    e = {"text": t, "at": [x, y]}
    if size:
        e["size"] = size
    texts.append(e)

# CONTROLLERS.
for (p, t), y in zip([("tune", "TUNE"), ("glide", "GLIDE"), ("mod_mix", "MODULATION MIX")], R):
    knob(p, 138, y)
    text(t, 138, y - LEG)
mid = [(R[0] + R[1]) / 2, (R[1] + R[2]) / 2]
rocker("osc_mod", 290, mid[0], on="top")
text("OSC.", 290, mid[0] - 84, 17); text("MOD.", 290, mid[0] - 64, 17)
rocker("osc3_control", 290, mid[1], on="top")
text("OSC. 3", 290, mid[1] - 84, 17); text("CONTROL", 290, mid[1] - 64, 17)

# SOURCES: each a knob and its ON switch, as the concept's fourth row, stood on end.
SK, SR = 474, 634
knob("ext_volume", SK, R[0]); rocker("ext_on", SR, R[0], on="top")
text("EXTERNAL INPUT", SK, R[0] - LEG); text("ON", SR, R[0] - LEG)
knob("noise_volume", SK, R[1]); rocker("noise_on", SR, R[1], on="top")
text("NOISE", SK, R[1] - LEG); text("ON", SR, R[1] - LEG)
rocker("noise_type", SK, R[2], on="bottom")
text("WHITE", SK, R[2] - 76); text("PINK", SK, R[2] + 76)
text("OVERLOAD", SR, R[2] - 48)

# OSCILLATORS: one row each, in the concept's order.
COL = {"range": 819, "wave": 1059, "freq": 1299, "vol": 1529, "on": 1665}
for i, y in enumerate(R):
    n = i + 1
    knob(f"osc{n}_range", COL["range"], y)
    knob(f"osc{n}_waveform", COL["wave"], y)
    if n > 1:
        # Lowered so that its legend clears its dial on the row's line.
        knob(f"osc{n}_frequency", COL["freq"], y + 12, big=True)
    knob(f"osc{n}_volume", COL["vol"], y)
    rocker(f"osc{n}_on", COL["on"], y, on="top")
    for t, k in [("RANGE", "range"), ("WAVEFORM", "wave"), ("VOLUME", "vol"), ("ON", "on")]:
        text(t, COL[k], y - LEG)
    if n > 1:
        text("FREQUENCY", COL["freq"], y - LEG)

# MODIFIERS: the FILTER's panel, titled as the concept's, and the two contours'.
FX = [1950, 2145, 2340]
text("FILTER", 2145, PANEL_Y[0][0] + 5, 21)
for p, t, x in zip(["cutoff", "emphasis", "contour_amount"], ["CUTOFF", "EMPHASIS", "CONTOUR AMOUNT"], FX):
    knob(p, x, R[0]); text(t, x, R[0] - LEG)
for pre, y in [("filter", R[1]), ("loudness", R[2])]:
    for p, x in zip(["attack", "decay", "sustain"], FX):
        knob(f"{pre}_{p}", x, y); text(p.upper(), x, y - LEG)
RX = 2508
text("FILTER", RX, 52, 17); text("MODE", RX, 70, 17)
rocker("filter_mode", RX, 110)
text("FILTER MOD.", RX, 168, 17)
rocker("filter_mod", RX, 210)
text("KEYBOARD", RX, R[1] - LEG, 17)
rocker("keyboard_control_1", RX, R[1] - 50); text("1", RX - 67, R[1] - 50)
rocker("keyboard_control_2", RX, R[1] + 40); text("2", RX - 67, R[1] + 40)

# OUTPUT.
OK_, ORK = 2721, 2891
knob("volume", OK_, R[0]); text("VOLUME", OK_, R[0] - LEG)
rocker("main_output", ORK, R[0]); text("MAIN", ORK, R[0] - LEG - 9, 17); text("OUTPUT", ORK, R[0] - LEG + 9, 17)
knob("feedback", OK_, R[1] + 12); text("FEEDBACK", OK_, R[1] + 12 - LEG)
rocker("a440", ORK, R[1] + 12); text("A–440", ORK, R[1] + 12 - LEG)
PH = ((X["out"][0] + X["out"][1]) / 2, 652)
text("PHONES", PH[0], PH[1] - 58)

# QUALITY and POWER.
PX = 3022
controls.append({"param": "quality", "at": [PX, 266], "size": [50, 80.83333333333333], "across": False})
for t, x, y, s in [("QUALITY", PX, 168, None), ("ULTRA", PX, 216, 15), ("HI", PX - 40, 266, 15),
                   ("LO", PX, 318, 15), ("POWER", PX, 465, None), ("ON", PX, 500, None)]:
    text(t, x, y, s)

def fmt(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, str):
        return '"' + v + '"'
    if isinstance(v, list):
        return "[" + ", ".join(fmt(x) for x in v) + "]"
    if isinstance(v, float) and v.is_integer():
        return str(int(v))
    return repr(v)

with open(os.path.join(out, "layout.toml"), "w") as f:
    f.write(f"""# SIGNAL BASIN's panel: every source in a row of its own, all flowing into one copper bus and on
# to the filter and contours, as a signal does. Dedicated to the public domain under CC0 1.0 (LICENSE.txt). The format: the
# CA-72's docs/skin-packs.md. Units are the drawing's: the face 0 to 3108 by 0 to 795, the
# strip above it -130 to 0.

format = 1
controls = 1

[fixtures]
power = [{PX}, 580]
lamp = [{PX}, 426]
overload = [{SR}, {R[2]}]
ultra = [{PX}, 92]
plate = [2788, -106]
phones = [{PH[0]}, {PH[1]}, 34]
screws = []
""")
    for c in controls:
        f.write("\n[[control]]\n" + "".join(f"{k} = {fmt(v)}\n" for k, v in c.items()))
    for t in texts:
        f.write("\n[[text]]\n" + "".join(f"{k} = {fmt(v)}\n" for k, v in t.items()))

# The drawing: 3108 by 925, the face's y + 130.
Y = 130
teal = [(X[k][0], TALL[0], X[k][1], TALL[1]) for k in ("ctl", "bay", "out", "pwr")]
bay = teal[1]
inset = [(X["osc"][0], a, X["osc"][1], b) for a, b in PANEL_Y] + [(X["src"][0], INNER[0], X["src"][1], INNER[1])]
inset += [(X["mod"][0], a, X["mod"][1], b) for a, b in PANEL_Y]
panels = teal + inset

def rr(x0, y0, x1, y1, r):
    y0 += Y; y1 += Y
    return (f"M{x0+r},{y0}H{x1-r}A{r},{r} 0 0 1 {x1},{y0+r}V{y1-r}A{r},{r} 0 0 1 {x1-r},{y1}"
            f"H{x0+r}A{r},{r} 0 0 1 {x0},{y1-r}V{y0+r}A{r},{r} 0 0 1 {x0+r},{y0}Z")

COPPER = "#a8724c"
svg = ['<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 3108 925" width="3108" height="925">',
       "<title>SIGNAL BASIN: black panels let into weathered teal, edged in copper, the signal's way drawn between them</title>",
       '<defs>',
       # A rim lit from the upper left: bright copper above, dark below.
       '<linearGradient id="rim" x1="0" y1="0" x2="0.25" y2="1">'
       '<stop offset="0" stop-color="#c08a60"/><stop offset="0.5" stop-color="#8a5a3a"/>'
       '<stop offset="1" stop-color="#4e3020"/></linearGradient>',
       '<linearGradient id="sheen" x1="0" y1="0" x2="0" y2="1">'
       '<stop offset="0" stop-color="#ffffff" stop-opacity="0.035"/>'
       '<stop offset="0.6" stop-color="#ffffff" stop-opacity="0"/>'
       '<stop offset="1" stop-color="#000000" stop-opacity="0.12"/></linearGradient>',
       '<radialGradient id="vignette" cx="0.5" cy="0.55" r="0.75">'
       '<stop offset="0.55" stop-color="#000" stop-opacity="0"/>'
       '<stop offset="1" stop-color="#000" stop-opacity="0.32"/></radialGradient>',
       '<linearGradient id="cheek" x1="0" y1="0" x2="1" y2="0">'
       '<stop offset="0" stop-color="#2c1e15"/><stop offset="0.45" stop-color="#4e3628"/>'
       '<stop offset="1" stop-color="#241810"/></linearGradient>',
       '<radialGradient id="head" cx="0.34" cy="0.3" r="0.8">'
       '<stop offset="0" stop-color="#9a7452"/><stop offset="0.25" stop-color="#5a3e2a"/>'
       '<stop offset="1" stop-color="#2a1c12"/></radialGradient>',
       # A screw: a domed dark bronze pan head, its cross recess cut dark and lit along one edge.
       '<radialGradient id="drop"><stop offset="0.55" stop-color="#000" stop-opacity="0.5"/>'
       '<stop offset="1" stop-color="#000" stop-opacity="0"/></radialGradient>',
       '<g id="sc">'
       '<circle r="16" cx="0.9" cy="2.2" fill="url(#drop)"/>'
       '<circle r="12.5" fill="url(#head)" stroke="#140d08" stroke-width="1"/>'
       '<path d="M-7.4,-1.5H-1.5V-7.4H1.5V-1.5H7.4V1.5H1.5V7.4H-1.5V1.5H-7.4Z" fill="#140d08"/>'
       '<path d="M-7.4,1.5H-1.5V7.4M1.5,1.5H7.4" fill="none" stroke="#9a7452" stroke-opacity="0.5" stroke-width="0.8"/>'
       '</g>',
       # A knob's shadow on the panel (it does not turn).
       '<radialGradient id="kshadow"><stop offset="0.62" stop-color="#000" stop-opacity="0.5"/>'
       '<stop offset="1" stop-color="#000" stop-opacity="0"/></radialGradient>',
       '</defs>']
# Wood at the face's two ends, as the case's cheeks.
import random
rnd = random.Random(72)
for x0 in (0, 3082):
    svg.append(f'<rect x="{x0}" y="{Y}" width="26" height="795" fill="url(#cheek)"/>')
    for _ in range(9):
        gx = x0 + rnd.uniform(1.5, 24.5)
        svg.append(f'<rect x="{gx:.1f}" y="{Y}" width="{rnd.uniform(0.4, 1.2):.1f}" height="795" fill="#1a0f08" fill-opacity="{rnd.uniform(0.25, 0.55):.2f}"/>')
    svg.append(f'<rect x="{x0 + (26 if x0 == 0 else -3)}" y="{Y}" width="3" height="795" fill="#000" fill-opacity="0.55"/>')
# The face weathered: darker towards its edges.
svg.append(f'<rect x="10" y="{Y}" width="3088" height="795" fill="url(#vignette)"/>')
for x0, y0, x1, y1 in [q for q in panels if q != bay]:
    for k, o in ((5, 0.2), (2, 0.3)):
        svg.append(f'<path fill-rule="evenodd" d="{rr(x0 + k * 0.6, y0 + k, x1 + k * 0.6, y1 + k, 10 + k)}{rr(x0, y0, x1, y1, 10)}" fill="#000" fill-opacity="{o}"/>')
for x0, y0, x1, y1 in teal:
    svg.append(f'<path d="{rr(x0, y0, x1, y1, 10)}" fill="#04161a" fill-opacity="0.18"/>')
for x0, y0, x1, y1 in inset:
    # A panel let into the face: near-black with a little blue, darker at its foot.
    d = rr(x0, y0, x1, y1, 10)
    svg.append(f'<path d="{d}" fill="#0e1417" fill-opacity="0.95"/>')
    svg.append(f'<path d="{d}" fill="url(#sheen)"/>')
svg.insert(svg.index("</defs>"),
    '<radialGradient id="grime"><stop offset="0" stop-color="#050302" stop-opacity="0.34"/>'
    '<stop offset="1" stop-color="#050302" stop-opacity="0"/></radialGradient>'
    '<radialGradient id="scuff"><stop offset="0" stop-color="#9fc6c4" stop-opacity="0.07"/>'
    '<stop offset="1" stop-color="#9fc6c4" stop-opacity="0"/></radialGradient>')
wear = random.Random(1972)
for x0, y0, x1, y1 in [q for q in panels if q != bay]:
    # Grime in a band outside the rim, heavier at the panel's foot and its corners.
    for k, o in ((20, 0.07), (8, 0.11)):
        svg.append(f'<path fill-rule="evenodd" d="{rr(x0 - k, y0 - k, x1 + k, y1 + k, 10 + k)}{rr(x0, y0, x1, y1, 10)}" fill="#050302" fill-opacity="{o}"/>')
    edge = 2 * ((x1 - x0) + (y1 - y0))
    for _ in range(int(edge / 120)):
        t = wear.random()
        side = wear.random()
        if side < 0.5:
            gx, gy = x0 + t * (x1 - x0), y1 + wear.uniform(-2, 10)
        elif side < 0.6:
            gx, gy = x0 + t * (x1 - x0), y0 - wear.uniform(-2, 8)
        elif side < 0.8:
            gx, gy = x0 - wear.uniform(-2, 8), y0 + t * (y1 - y0)
        else:
            gx, gy = x1 + wear.uniform(-2, 8), y0 + t * (y1 - y0)
        rx, ry = wear.uniform(14, 46), wear.uniform(6, 16)
        if side >= 0.6:
            rx, ry = ry, rx
        svg.append(f'<ellipse cx="{gx:.1f}" cy="{gy + Y:.1f}" rx="{rx:.1f}" ry="{ry:.1f}" fill="url(#grime)"/>')
for x0, y0, x1, y1 in teal:
    # Lighter worn patches on the teal, mostly near its edges, where hands go.
    for _ in range(int((x1 - x0) * (y1 - y0) / 40000) + 2):
        gx = wear.uniform(x0 + 20, x1 - 20)
        gy = wear.choice([wear.uniform(y0 + 10, y0 + 80), wear.uniform(y1 - 80, y1 - 10), wear.uniform(y0, y1)])
        svg.append(f'<ellipse cx="{gx:.1f}" cy="{gy + Y:.1f}" rx="{wear.uniform(20, 70):.1f}" ry="{wear.uniform(10, 34):.1f}" fill="url(#scuff)"/>')
for x0, y0, x1, y1 in inset:
    # A few faint scratches on each module.
    for _ in range(int((x1 - x0) * (y1 - y0) / 28000) + 3):
        sx, sy = wear.uniform(x0 + 25, x1 - 25), wear.uniform(y0 + 25, y1 - 25)
        a = wear.uniform(-0.5, 0.5) + (0 if wear.random() < 0.7 else 1.57)
        L = wear.uniform(18, 90)
        ex, ey = sx + L * math.cos(a), sy + L * math.sin(a)
        mx, my = (sx + ex) / 2 + wear.uniform(-4, 4), (sy + ey) / 2 + wear.uniform(-4, 4)
        svg.append(f'<path d="M{sx:.1f},{sy + Y:.1f} Q{mx:.1f},{my + Y:.1f} {ex:.1f},{ey + Y:.1f}" fill="none" stroke="#e8f0ee" stroke-opacity="{wear.uniform(0.035, 0.07):.3f}" stroke-width="{wear.uniform(0.5, 1.1):.1f}" stroke-linecap="round"/>')
for c in controls:
    if "size" in c:
        continue
    r = 65 if c.get("big") else 54
    if c["param"].endswith(("range", "waveform")):
        r = 50
    x, y = c["at"]
    svg.append(f'<circle cx="{x + 3}" cy="{y + Y + 5}" r="{r + 9}" fill="url(#kshadow)"/>')
def under_a_control(x, y, r=16):
    """Whether a screw at x, y would lie under a control (or its knob's shadow) or a legend."""
    for t in texts:
        tx, ty = t["at"]
        size = t.get("size", 22)
        hw = len(t["text"]) * size * 0.3 + 4
        hh = size * 0.45 + 3
        if abs(x - tx) < hw + r and abs(y - ty) < hh + r:
            return True
    for c in controls:
        cx, cy = c["at"]
        if "size" in c:
            w, h = c["size"]
            if abs(x - cx) < w / 2 + r and abs(y - cy) < h / 2 + r:
                return True
        elif math.hypot(x - cx, y - cy) < (65 if c.get("big") else 54) + r:
            return True
    return False

svg.insert(svg.index("</defs>"),
    '<linearGradient id="frame" x1="0" y1="0" x2="0.25" y2="1">'
    '<stop offset="0" stop-color="#7e563a"/><stop offset="1" stop-color="#4a3020"/></linearGradient>')
for x0, y0, x1, y1 in panels:
    d = rr(x0, y0, x1, y1, 10)
    if (x0, y0, x1, y1) == bay:
        svg.append(f'<path d="{d}" fill="none" stroke="url(#frame)" stroke-width="1.6"/>')
        continue
    # Its shadow inside the rim, then the rim, bevelled: bright above, dark below.
    svg.append(f'<path d="{rr(x0 + 3, y0 + 3, x1 - 3, y1 - 3, 8)}" fill="none" stroke="#000" stroke-opacity="0.45" stroke-width="2.5"/>')
    svg.append(f'<path d="{rr(x0 - 1.8, y0 - 1.8, x1 + 1.8, y1 + 1.8, 11.5)}" fill="none" stroke="#000" stroke-opacity="0.35" stroke-width="1.2"/>')
    svg.append(f'<path d="{d}" fill="none" stroke="url(#rim)" stroke-width="2.8"/>')
    dash = ",".join(f"{wear.uniform(15, 80):.0f},{wear.uniform(5, 40):.0f}" for _ in range(12))
    svg.append(f'<path d="{rr(x0 + 1.4, y0 + 1.4, x1 - 1.4, y1 - 1.4, 8.8)}" fill="none" stroke="#d6a07a" stroke-opacity="0.35" stroke-width="0.8" stroke-dasharray="{dash}"/>')
    for _ in range(int(2 * ((x1 - x0) + (y1 - y0)) / 160)):
        t = wear.random()
        side = wear.randrange(4)
        tx, ty = [(x0 + t * (x1 - x0), y0), (x0 + t * (x1 - x0), y1), (x0, y0 + t * (y1 - y0)), (x1, y0 + t * (y1 - y0))][side]
        svg.append(f'<ellipse cx="{tx:.1f}" cy="{ty + Y:.1f}" rx="{wear.uniform(3, 12) if wear.random() < 0.7 else wear.uniform(10, 20):.1f}" ry="{wear.uniform(1.3, 2.2):.1f}" transform="rotate({0 if side < 2 else 90} {tx:.1f} {ty + Y:.1f})" fill="#2a1a10" fill-opacity="{wear.uniform(0.35, 0.45) + 0.2:.2f}"/>')
    if (x0, y0, x1, y1) == bay:
        continue
    k = 22 if (x0, y0, x1, y1) in teal else 16
    corners = [(x0 + k, y0 + k, 1, 1), (x1 - k, y0 + k, -1, 1), (x0 + k, y1 - k, 1, -1), (x1 - k, y1 - k, -1, -1)]
    if x0 == X["ctl"][0]:
        corners[2] = (x0 + 16, y1 - 16, 1, -1)
    for cx, cy, dx, dy in corners:
        # Moved along its edges, away from the corner, until it clears what is printed there.
        for step in range(0, 9):
            sx, sy = (cx, cy + dy * step * 10) if step % 2 == 0 else (cx + dx * step * 10, cy)
            if not under_a_control(sx, sy):
                break
        else:
            continue
        svg.append(f'<circle cx="{sx}" cy="{sy + Y}" r="{wear.uniform(22, 30):.1f}" fill="url(#grime)"/>')
        svg.append(f'<use href="#sc" transform="translate({sx} {sy + Y}) rotate({rnd.uniform(0, 90):.0f})"/>')

def arrow(pts, head=True, w=2.6):
    d = "M" + " L".join(f"{x},{y + Y}" for x, y in pts)
    svg.append(f'<path d="{d}" fill="none" stroke="#000" stroke-opacity="0.5" stroke-width="{w + 1.6}" stroke-linejoin="round" transform="translate(0.8 1.2)"/>')
    svg.append(f'<path d="{d}" fill="none" stroke="{COPPER}" stroke-width="{w}" stroke-linejoin="round"/>')
    if head:
        (ax, ay), (bx, by) = pts[-2], pts[-1]
        a = math.atan2(by - ay, bx - ax)
        L, W = 20, 9
        tip = (bx, by)
        l = (bx - L * math.cos(a) + W * math.sin(a), by - L * math.sin(a) - W * math.cos(a))
        r = (bx - L * math.cos(a) - W * math.sin(a), by - L * math.sin(a) + W * math.cos(a))
        svg.append('<polygon points="' + " ".join(f"{x:.1f},{y + Y:.1f}" for x, y in (tip, l, r)) + f'" fill="{COPPER}"/>')

# Each oscillator out through its module's rim into the bus; the bus into the FILTER's and
# each contour's module; the sources along the gaps between the oscillators' rows into the bus,
# as the concept's fourth row.
def grommet(x, y):
    svg.append(f'<circle cx="{x}" cy="{y + Y}" r="6.5" fill="#2a1a10" stroke="#a8724c" stroke-width="2"/>')
for y in R:
    arrow([(COL["on"] + 29, y), (BUS - 9, y)])
    grommet(X["osc"][1], y)
OUT_Y = [(a + b) / 2 for a, b in PANEL_Y]
gaps = [(PANEL_Y[0][1] + PANEL_Y[1][0]) / 2, (PANEL_Y[1][1] + PANEL_Y[2][0]) / 2]
arrow([(BUS, min(OUT_Y)), (BUS, max(R + gaps))], head=False, w=3.2)
for y in OUT_Y:
    arrow([(BUS, y), (X["mod"][0] - 20, y)])
for y, g in zip(R[:2], gaps):
    arrow([(SR + 29, y), (FEED, y), (FEED, g), (BUS - 9, g)])
    grommet(X["src"][1], y)
fx0, fx1 = 2145 - 72, 2145 + 72
fy = PANEL_Y[0][0]
# (Clear of the bay's frame above it by a sliver of teal.)
svg.append(f'<path d="{rr(fx0 + 1, fy - 6, fx1 + 1, fy + 19, 5)}" fill="#000" fill-opacity="0.4"/>')
svg.append(f'<path d="{rr(fx0, fy - 7, fx1, fy + 17, 5)}" fill="#0b0f11" stroke="url(#rim)" stroke-width="2.4"/>')
svg.append(f'<path d="{rr(fx0 + 3, fy - 4, fx1 - 3, fy + 14, 3)}" fill="none" stroke="#000" stroke-opacity="0.5" stroke-width="1.2"/>')
svg.append("</svg>")
with open(os.path.join(out, "art", "panel.svg"), "w") as f:
    f.write("\n".join(svg) + "\n")
print(len(controls), "controls,", len(texts), "legends")
