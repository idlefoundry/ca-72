#!/usr/bin/env python3
"""Check the listening triplets and resolve the 32-sample pulse into its harmonics.

Uses Python's standard library. Run after both zipper measurements, with their CSVs and
48 WAVs in one folder: python3 docs/lfo-stress/measure-wavs.py <folder>.
The harmonic figures are envelope power ratios, not audio noise levels or hearing thresholds.
"""

import csv
import math
from pathlib import Path
import sys
import wave


def db(amplitude):
    return 20 * math.log10(max(amplitude, 1e-300))


def read(path):
    with wave.open(str(path), "rb") as wav:
        assert (wav.getframerate(), wav.getnchannels(), wav.getsampwidth(), wav.getnframes()) == (
            48000, 2, 3, 384000
        ), path
        data = wav.readframes(wav.getnframes())
    samples = [int.from_bytes(data[i:i + 3], "little", signed=True) / 8388607
               for i in range(0, len(data), 3)]
    peak = max(map(abs, samples))
    assert 0 < peak < 1, path
    mono = [(l + r) / 2 for l, r in zip(samples[::2], samples[1::2])]
    return db(peak), mono[28800:350400]


def harmonics(x):
    # The same third difference, four-sample envelope and Hann window as lfo_stress.rs.
    d = [abs(x[i] - 3 * x[i - 1] + 3 * x[i - 2] - x[i - 3])
         for i in range(3, len(x))]
    e = [sum(d[i:i + 4]) / len(d[i:i + 4]) for i in range(0, len(d), 4)]
    mean = sum(e) / len(e)
    e = [(v - mean) * (0.5 - 0.5 * math.cos(2 * math.pi * i / len(e)))
         for i, v in enumerate(e)]
    n = 1 << (len(e) - 1).bit_length()
    result = []
    for harmonic in range(1, 4):
        centre = round(harmonic * 1500 * n / 12000)
        powers = []
        for k in range(centre - 2, centre + 3):
            # Goertzel gives the FFT bin without calculating the unused bins.
            coefficient = 2 * math.cos(2 * math.pi * k / n)
            a, b = 0.0, 0.0
            for v in e:
                a, b = v + coefficient * a - b, a
            powers.append(max(0.0, a * a + b * b - coefficient * a * b))
        result.append(max(powers))
    return result


def main(folder):
    rows = {}
    for label in ("before", "after"):
        with (folder / f"metrics-{label}.csv").open() as file:
            for row in csv.DictReader(file):
                rows[(row["preset"], label)] = row
    names = sorted({name for name, label in rows})
    assert len(names) == 16, f"Expected sixteen presets, found {len(names)}"
    output = []
    worst_agreement = 0
    for name in names:
        stem = name.removeprefix("LFO TEST ").replace(" ", "-") + "-HI"
        readings = {label: read(folder / f"{stem}-{label}.wav")
                    for label in ("before", "after", "reference-every-sample")}
        assert abs(max(p for p, x in readings.values()) + 14) < 0.001, name
        gains = []
        for label in ("before", "after"):
            row = rows[(name, label)]
            gains.append(readings[label][0] - float(row["peak_dbfs"]))
            assert row["not_finite"] == "0" and row["clipped"] == "0", name
        gains.append(readings["reference-every-sample"][0]
                     - float(rows[(name, "before")]["peak_reference_dbfs"]))
        assert max(gains) - min(gains) < 0.012, (name, gains)
        spectra = {label: harmonics(x) for label, (p, x) in readings.items()}
        reference = spectra["reference-every-sample"]
        for label in ("before", "after"):
            ratios = [10 * math.log10(max(p, 1e-300) / max(r, 1e-300))
                      for p, r in zip(spectra[label], reference)]
            summed = 10 * math.log10(sum(spectra[label]) / sum(reference))
            row = rows[(name, label)]
            rust = float(row["pulse_db"]) - float(row["pulse_floor_db"])
            disagreement = abs(summed - rust)
            worst_agreement = max(worst_agreement, disagreement)
            assert disagreement < 0.15, (name, label, summed, rust)
            output.append([name, label, *[f"{v:.2f}" for v in ratios],
                           f"{summed:.2f}", f"{readings[label][0]:.4f}",
                           f"{readings['reference-every-sample'][0]:.4f}"])
    with (folder / "wav-metrics.csv").open("w", newline="") as file:
        writer = csv.writer(file, lineterminator="\n")
        writer.writerow(["preset", "label", "pulse_1500_db_vs_reference",
                         "pulse_3000_db_vs_reference", "pulse_4500_db_vs_reference",
                         "pulse_sum_db_vs_reference", "wav_peak_dbfs",
                         "reference_wav_peak_dbfs"])
        writer.writerows(output)
    print(f"48 WAVs checked: stereo, 48 kHz, 24-bit, 8 s; common gain in each triplet;")
    print(f"each triplet peaks at -14 dBFS. Harmonic sums agree with the Rust measurement")
    print(f"within {worst_agreement:.4f} dB (24-bit WAVs against unquantised renders).")


if __name__ == "__main__":
    main(Path(sys.argv[1]))
