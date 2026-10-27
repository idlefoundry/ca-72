# Production pulse alias coverage

`alias::circuit` uses the active production comparator and physical width network.
`alias::circuit_ideal` retains the explicit legacy ideal-voltage bench. The CLI defaults
to the production path with `alias --model circuit`; `--model circuit-ideal` selects
the legacy path. The measured signal and its 8x harmonic reference use the same path.

Both paths retain the original alias budgets at 44.1, 48 and 96 kHz, A4 and A7,
for square, wide and narrow rectangles, sawtooth and triangle. HI uses 1x and ULTRA
4x. The metric integrates nonharmonic power from 20 Hz to 20 kHz relative to the
fundamental, using 131072 samples and the existing seven-term window/harmonic masks.
A separate routing regression confirms production and legacy benches render distinct
pulse audio, preventing a passing legacy gate from masquerading as production coverage.

The standard-pulse branch validation measured worst sawtooth/rectangle totals of
−80.824 dB at HI and −114.296 dB at ULTRA, against unchanged −80/−105 dB limits.
The HI margin is 0.824 dB; it is not the old presumed 3 dB margin. Legacy totals were
−84.251/−114.765 dB. Triangle totals in that pre-integration tree were
−61.849/−73.403 dB against −58/−70 dB limits. Integration with other shaper changes
must rerun these gates on its actual source commit.

The moving-width regression now explicitly enables the physical network and retains
the legacy case. It tests both following and nonfollowing behavior at all three sample
rates and 1x/4x/8x. After moving from square to narrow at 30% of the cycle, following
must switch within 100 microseconds after subtracting known output latency;
nonfollowing must wait beyond 7 ms. The physical cases measured approximately
0–31 microseconds (sampled, with latency removed) and 8.8 ms respectively. This checks comparator event behavior;
it does not claim a dynamic-modulation alias measurement or hardware sound acceptance.
