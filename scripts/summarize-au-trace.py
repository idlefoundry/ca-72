"""Decode diagnostic-only fixed records after the probe has stopped its processes."""
import json
import heapq
from collections import deque
from pathlib import Path
import struct

names = {1: 'initialize', 2: 'activate-enter', 3: 'activate-exit', 4: 'render-enter',
         5: 'render-exit', 6: 'clap-enter', 7: 'clap-exit', 10: 'parameter-snapshot',
         11: 'parameter-event', 12: 'note-event', 13: 'reset', 14: 'state-load-enter', 15: 'state-load-exit',
         16: 'midi-event', 17: 'unlogged-event'}

def number(bits):
    return struct.unpack('<d', struct.pack('<Q', bits))[0]

summaries = []
for path in Path('diagnostic').glob('*/au-render-*.bin'):
    raw = path.read_bytes()
    magic, version, capacity, size, numer, denom, pid, ppid, count, dropped = struct.unpack_from('<10Q', raw)
    assert magic == 0x4341373254524143 and version == 1 and size == 128
    assert len(raw) == 4096 + capacity * size
    start, last, committed = None, None, 0
    active, durations, counts, params, recent = {}, [], {}, {}, deque(maxlen=100)
    for index in range(min(count, capacity)):
        r = struct.unpack_from('<16Q', raw, 4096 + index * size)
        if r[0] != index + 1:
            continue  # stopped process may leave an unfinished record
        committed += 1
        _, ticks, kind, obj, thread, *values = r
        if start is None:
            start = ticks
        last = ticks
        seconds = (ticks - start) * numer / denom / 1e9
        counts[names.get(kind, str(kind))] = counts.get(names.get(kind, str(kind)), 0) + 1
        entry = {'t': seconds, 'kind': names.get(kind, kind), 'object': obj, 'thread': thread,
                 'values': values}
        if kind == 4:
            entry.update(frames=values[0], maximum=values[1], position=number(values[2]),
                         flags=values[3], rate=number(values[4]), initialized=values[5])
        if kind in (4, 6):
            # Values are replaced, never mutated, so this shallow copy preserves
            # the exact prior state without duplicating every value dictionary.
            entry['parameters_before'] = dict(params.get(str(obj), {}))
            entry['parameter_events'] = []
            active[(kind, obj, thread)] = entry
        if kind in (5, 7):
            opened = active.pop((kind - 1, obj, thread), None)
            if opened:
                duration = seconds - opened['t']
                candidate = (duration, index, {'entry': opened, 'seconds': duration})
                if len(durations) < 30:
                    heapq.heappush(durations, candidate)
                elif duration > durations[0][0]:
                    heapq.heapreplace(durations, candidate)
        if kind == 10:
            name = struct.pack('<4Q', *values[2:6]).split(b'\0')[0].decode(errors='replace')
            params.setdefault(str(obj), {})[str(values[0])] = {'name': name, 'value': number(values[1])}
            entry.update(name=name, value=number(values[1]))
        if kind == 11:
            controls = params.setdefault(str(obj), {})
            parameter = dict(controls.get(str(values[0]), {'name': 'unknown'}))
            parameter['value'] = number(values[1])
            controls[str(values[0])] = parameter
            entry.update(parameter=parameter, sample_offset=values[2])
            for (event_kind, owner, _), opened in active.items():
                if event_kind == 6 and owner == obj:
                    opened['parameter_events'].append({'id':values[0], 'value':parameter['value'],
                                                       'sample_offset':values[2]})
        recent.append(entry)
    assert committed, 'trace must contain committed records'
    summary = {'path': str(path), 'pid': pid, 'ppid': ppid, 'allocated': capacity,
               'attempted': count, 'committed': committed, 'dropped': dropped,
               'seconds_observed': (last-start)*numer/denom/1e9, 'counts': counts,
               'slowest': [d[2] for d in sorted(durations, reverse=True)],
               'unfinished': list(active.values()), 'parameters': params, 'last_records': list(recent)}
    path.with_suffix('.json').write_text(json.dumps(summary, indent=2) + '\n')
    summaries.append(summary)
Path('diagnostic/trace-summary.json').write_text(json.dumps(summaries, indent=2) + '\n')
print(json.dumps([{k:s[k] for k in ['path','pid','ppid','committed','dropped','counts','unfinished']} for s in summaries], indent=2))
assert summaries, 'no trace files produced'
assert all(s['dropped'] == 0 for s in summaries), 'trace capacity exhausted; evidence incomplete'
