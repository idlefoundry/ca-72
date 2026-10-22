#!/usr/bin/env python3
"""Emit the fixed-pulse edge-delay reduction from the SPICE reference JSON.

The table contains delays versus buffer slew rate, not audio samples, harmonic targets,
output EQ, or a waveform lookup. DC thresholds are separately solved from Q11/Q12's
physical circuit at setup. Factory and provisional reference components remain separate.
"""
import argparse, json, math
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('reference',type=Path);p.add_argument('--output',type=Path);a=p.parse_args()
d=json.loads(a.reference.read_text());points=d['points'] if isinstance(d,dict) else d;blocks=[]
for tag in ['factory','reference']:
 rows=[sorted([x for x in points if x['tag']==tag and x['width_bias_nominal_v']==w],key=lambda x:x['buffer_slope_v_per_s']) for w in [0,-1.5,-2.5]]
 assert all(len(r)==7 for r in rows)
 ar=lambda data:'[\n'+',\n'.join('    ['+', '.join(f'{v:.12e}' for v in r)+']' for r in data)+'\n]'
 blocks.append(f'const {tag.upper()}: Timing = Timing {{ log_slope:'+ar([[math.log(x['buffer_slope_v_per_s']) for x in r] for r in rows])+',fall_delay:'+ar([[x['fall_delay_s'] for x in r] for r in rows])+',rise_delay:['+', '.join(f"{r[0]['rise_delay_s']:.12e}" for r in rows)+'] };')
s='// Generated from circuits/boards/reference/pulse-timing.json; see pulse_reference.py.\nuse super::{PulseCircuit, Timing};\n'+'\n'.join(blocks)+'\npub(super) fn timing(circuit: PulseCircuit) -> Option<Timing> { if circuit == PulseCircuit::reference() { Some(REFERENCE) } else if circuit == PulseCircuit::default() { Some(FACTORY) } else { None } }\n'
if a.output:a.output.write_text(s)
else:print(s,end='')
