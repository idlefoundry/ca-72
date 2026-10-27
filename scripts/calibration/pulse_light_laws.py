#!/usr/bin/env python3
"""Reduce the full circuit MIX source to LO pulse duty/level laws; no hardware fit."""
import json,numpy as np
from scipy.optimize import minimize_scalar
from pathlib import Path
import argparse
parser=argparse.ArgumentParser();parser.add_argument('input',type=Path);parser.add_argument('--output',type=Path);args=parser.parse_args();x=json.load(open(args.input));out=[]
for a in x:
 if a['wave']=='Sawtooth':continue
 saw=next(b for b in x if b['tag']==a['tag'] and b['midi']==a['midi'] and b['wave']=='Sawtooth')
 h=np.arange(2,21);amp=np.array(a['bus_harmonics']);sh=np.array(saw['bus_harmonics']);q=(amp[h-1]/amp[0])/(sh[h-1]/sh[0])
 def loss(d):
  target=np.abs(np.sin(np.pi*h*d))/np.sin(np.pi*d)
  return np.mean((np.log(np.maximum(target,1e-5))-np.log(np.maximum(q,1e-5)))**2)
 bound={'Square':(.46,.4999),'WideRectangle':(.27,.33),'NarrowRectangle':(.15,.21)}[a['wave']]
 grid=np.linspace(*bound,10001);ii=np.argmin([loss(d) for d in grid]);fit=minimize_scalar(loss,bounds=(grid[max(0,ii-3)],grid[min(len(grid)-1,ii+3)]),method='bounded',options={'xatol':1e-12});d=fit.x
 # Light's SOURCE_OHMS uses one effective rectangle source resistance.
 pot=25000.;track=.8453;series=33000.
 def channel(r):return track/(r+(1-track)*pot+track*pot*r/series+track*(1-track)*pot*pot/series) # use actual codebelow instead
 # Norton source into virtual-ground: p / (Rsrc*(1+p*pot/series) + pot*p*(1-p) + series?)
 # Direct total load seen by source: top+(bottom||series), output bus source current vp/series.
 def channel(r):
  rt=pot*(1-track);rb=pot*track;parallel=rb*series/(rb+series)
  return parallel/(r+rt+parallel)/series
 level=(amp[0]/sh[0])/(2*np.sin(np.pi*d))*channel(2246.)/channel(3500.)
 out.append(dict(tag=a['tag'],midi=a['midi'],wave=a['wave'],hz=a['hz'],duty=float(d),level=float(level),objective=float(fit.fun)))
for a in out:print(a)
if args.output:args.output.write_text(json.dumps(out,indent=2)+'\n')
