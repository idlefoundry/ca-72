"""LO source-transfer approximation to SPICE DC only; no hardware/audio fitting."""
from pathlib import Path
import numpy as np,json,re,hashlib,sys
ROOT=Path(__file__).resolve().parent.parent;tablefile=Path(sys.argv[1]) if len(sys.argv)>1 else ROOT/'crates/ca72/src/tables.rs';text=tablefile.read_text();v=np.array([float(z) for z in re.search(r'pub const TRI:.*?= \[(.*?)\];',text,re.S)[1].split(',') if z.strip()]);x=-4.6+np.arange(len(v))*.01;high=2.0346*.012771+3.9098;low=2.0346*(-3.970812)+3.9098;span=high-low;valid=(x>=low)&(x<=high);p=(high-x[valid])/span;y=v[valid];order=np.argsort(p);p=p[order];y=y[order]
def hinge(z,w):return np.where(z<0,0,np.where(z<w,z*z/(2*w),z-w/2))
def evalfit(q,ret=False):
 if any(q[::2]<.3) or any(q[::2]>.65) or any(q[1::2]<.0005) or any(q[1::2]>.2):return (1e3,None) if ret else 1e3
 A=np.stack([np.ones(len(p)),p,p*p]+[hinge(p-c,w) for c,w in zip(q[::2],q[1::2])],axis=1);coef=np.linalg.lstsq(A,y,rcond=None)[0]
 if any(coef[3:]<0):return (1e2,None) if ret else 1e2
 loss=float(np.mean((A@coef-y)**2));return (loss,coef) if ret else loss
def nm(start):
 n=len(start);simplex=np.array([start]+[start+np.eye(n)[i]*.003 for i in range(n)]);values=np.array([evalfit(z) for z in simplex])
 for iteration in range(1600):
  order=np.argsort(values);simplex=simplex[order];values=values[order]
  if max(np.ptp(simplex,axis=0))<1e-9:break
  center=np.mean(simplex[:-1],axis=0);r=2*center-simplex[-1];fr=evalfit(r)
  if fr<values[0]:
   e=center+2*(r-center);fe=evalfit(e);simplex[-1],values[-1]=(e,fe) if fe<fr else (r,fr)
  elif fr<values[-2]:simplex[-1],values[-1]=r,fr
  else:
   c=center+.5*((r if fr<values[-1] else simplex[-1])-center);fc=evalfit(c)
   if fc<min(fr,values[-1]):simplex[-1],values[-1]=c,fc
   else:
    simplex[1:]=simplex[0]+.5*(simplex[1:]-simplex[0]);values[1:]=[evalfit(z) for z in simplex[1:]]
 return simplex[np.argmin(values)]
best=None
for start in [np.array([.475,.015,.47,.05]),np.array([.48,.025,.46,.04]),np.array([.47,.02,.48,.05]),np.array([.475,.03,.50,.08])]:
 z=nm(start);loss,coef=evalfit(z,True)
 if best is None or loss<best[0]:best=(loss,z,coef)
loss,q,coef=best
# Sort the finite-knee pieces for stable source serialization.
o=np.argsort(q[::2]);knees=[{'start':float(q[2*i]),'width':float(q[2*i+1]),'slope_change_volts_per_cycle':float(coef[3+i])} for i in o]
def model(p):return coef[0]+coef[1]*p+coef[2]*p*p+sum(k['slope_change_volts_per_cycle']*hinge(p-k['start'],k['width']) for k in knees)
mean=coef[0]+coef[1]/2+coef[2]/3+sum(k['slope_change_volts_per_cycle']*((1-k['start'])**2/2-k['width']*(1-k['start'])/2+k['width']**2/6) for k in knees);scale=.5*span*4.7/9;pp=np.linspace(0,1,65536,endpoint=False);yy=model(pp);actual=np.interp(high-span*pp,x,v);ff=np.fft.rfft(yy)/len(yy)*2;dd=np.fft.rfft(actual)/len(actual)*2;error=20*np.log10(abs(ff[1:65])/abs(dd[1:65]));r=model(p)-y;out={'method':'Two finite knees plus broad quadratic base/emitter curvature; approximation fitted only to891point ngspice-derived DCsource table using variable-projectionleast-squares anddeterministicNelderMead. No hardware audio or harmonictargetparticipation. Derivedsamephysicalslow-buffer endpoints asHI Core/Shaper; lightphase remainslinear asitspriorapproximation. Eachfinitekneeslopechange admits existingaveragedBLAMP/filletcorrection, resetstep+resetcorner separately, no outputfilter or lookupofmeasuredspectra.','table_sha256':hashlib.sha256(tablefile.read_bytes()).hexdigest(),'buffer_range_volts':[low,high],'parameters':{'v0_volts':float(coef[0]),'start_slope_volts_per_cycle':float(coef[1]),'curvature_volts_per_cycle_squared':float(coef[2]),'knees':knees,'period_mean_volts':float(mean),'saw_half_swing_volts':float(scale),'reset_step_volts':float(model(np.array([0]))[0]-model(np.array([1]))[0]),'reset_slope_change_volts_per_cycle':float(-2*coef[2]-sum(k['slope_change_volts_per_cycle'] for k in knees)),'normalized_source_peak':float(max(abs(yy-mean))/scale)},'dc_rms_error_volts':float(np.sqrt(loss)),'dc_max_error_volts':float(max(abs(r))),'h1_error_db':float(error[0]),'h2_h24_rms_error_db':float(np.sqrt(np.mean(error[1:24]**2))),'harmonic_error_db':error.tolist()};
if len(sys.argv)>2:Path(sys.argv[2]).write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({k:v for k,v in out.items() if k!='harmonic_error_db'},indent=2));print('H2–24:',[round(v,3) for v in error[1:24]])
