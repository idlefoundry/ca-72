"""Disposable CI probe of the exact c16 artifact; never a replacement validation gate."""
import json
import os
from pathlib import Path
import signal
import subprocess
import threading
import time

OUT = Path("diagnostic")


def processes():
    raw = subprocess.check_output(["ps", "-axo", "pid=,ppid=,pgid=,args="], text=True)
    rows = {}
    for line in raw.splitlines():
        parts = line.strip().split(None, 3)
        if len(parts) == 4:
            rows[int(parts[0])] = (int(parts[1]), int(parts[2]), parts[3])
    return raw, rows


def cleanup_probe(name, leader):
    actions = []
    for sig in (signal.SIGTERM, signal.SIGKILL):
        raw, rows = processes()
        (OUT / f"{name}-cleanup-{sig.name}-processes.txt").write_text(raw)
        for pid, (_, group, command) in rows.items():
            if group != leader:
                continue
            # A new session gives this probe its own group. Re-read each exact PID's
            # command and group immediately before signalling; never kill by name/group.
            current = subprocess.run(["ps", "-p", str(pid), "-o", "pgid=,args="],
                                     text=True, capture_output=True)
            parts = current.stdout.strip().split(None, 1)
            if len(parts) != 2 or int(parts[0]) != leader or parts[1] != command:
                continue
            actions.append({"pid": pid, "command": command, "signal": sig.name})
            try:
                os.kill(pid, sig)
            except ProcessLookupError:
                pass
        if sig == signal.SIGTERM:
            time.sleep(1)
    (OUT / f"{name}-cleanup.json").write_text(json.dumps(actions, indent=2) + "\n")


def run_probe(name, args, nested=False):
    started = time.monotonic()
    phase = threading.Event()
    samplers = []
    samples = set()
    with (OUT / f"{name}.log").open("w") as log:
        proc = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                text=True, bufsize=1, start_new_session=True)

        def read_output():
            for line in proc.stdout:
                stamped = f"+{time.monotonic() - started:.3f}s {line}"
                log.write(stamped)
                log.flush()
                print(stamped, end="", flush=True)
                if "Starting tests in: pluginval / auval" in line:
                    phase.set()

        reader = threading.Thread(target=read_output, daemon=True)
        reader.start()
        phase_start = started if not nested else None
        timed_out = False
        try:
            while proc.poll() is None:
                now = time.monotonic()
                if nested and phase.is_set() and phase_start is None:
                    phase_start = now
                for delay in (10, 22):
                    if phase_start is None or now - phase_start < delay or delay in samples:
                        continue
                    samples.add(delay)
                    raw, rows = processes()
                    (OUT / f"{name}-{delay}-processes.txt").write_text(raw)
                    for pid, (_, group, command) in rows.items():
                        if group != proc.pid or (pid != proc.pid and "auval" not in command):
                            continue
                        # sample is read-only; exact identity and parentage are saved above.
                        samplers.append(subprocess.Popen(
                            ["sample", str(pid), "2", "1", "-file",
                             str(OUT / f"{name}-{delay}-pid-{pid}-sample.txt")],
                            stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT))
                if now - started > 180:
                    timed_out = True
                    (OUT / f"{name}-timeout-processes.txt").write_text(processes()[0])
                    break
                time.sleep(0.25)
        finally:
            cleanup_probe(name, proc.pid)
            code = proc.wait(timeout=5)
            reader.join(timeout=5)
        sampler_results = []
        for sampler in samplers:
            try:
                sample_code = sampler.wait(timeout=10)
            except subprocess.TimeoutExpired:
                # Exact Popen identity; save the command before stopping a stuck sampler.
                (OUT / f"{name}-sampler-{sampler.pid}-processes.txt").write_text(processes()[0])
                sampler.terminate()
                try:
                    sample_code = sampler.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    sampler.kill()
                    sample_code = sampler.wait(timeout=3)
            sampler_results.append({"pid": sampler.pid, "exit_code": sample_code})
    result = {"command": args, "exit_code": code, "outer_timeout": timed_out,
              "elapsed_seconds": time.monotonic() - started,
              "nested_phase_seen": phase.is_set(), "sample_delays": sorted(samples),
              "samplers": sampler_results, "reader_stopped": not reader.is_alive()}
    (OUT / f"{name}-result.json").write_text(json.dumps(result, indent=2) + "\n")
    return result


component = str(Path.home() / "Library/Audio/Plug-Ins/Components/CA-72.component")
validator = str(OUT.resolve() / "validator/pluginval.app/Contents/MacOS/pluginval")
# Match validate.sh's registry refresh, limited to exact observed registrar PIDs on
# this disposable runner. Read each command again immediately before signalling.
raw, rows = processes()
(OUT / "registry-refresh-processes.txt").write_text(raw)
for pid, (_, _, command) in rows.items():
    if Path(command.split()[0]).name != "AudioComponentRegistrar":
        continue
    current = subprocess.run(["ps", "-p", str(pid), "-o", "args="],
                             text=True, capture_output=True)
    if current.stdout.strip() == command:
        try:
            os.kill(pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
results = [run_probe("direct-stress", ["auval", "-strict", "-stress", "20", "-v", "aumu", "CA72", "IdlF"])]
if results[0]["reader_stopped"]:
    results.append(run_probe("nested-verbose", [
        validator, "--strictness-level", "10", "--validate-in-process", "--timeout-ms", "30000",
        "--random-seed", "0x6cc460f", "--verbose", "--validate", component,
    ], nested=True))
(OUT / "result.json").write_text(json.dumps({
    "source": "c16e18825b0674789088350334621f1161a28535",
    "artifact_id": 11928323615, "original_run": 38633302925,
    "strictness": 10, "pluginval_idle_timeout_ms": 30000, "results": results,
}, indent=2) + "\n")
raise SystemExit(0 if len(results) == 2 and results[1]["nested_phase_seen"] and all(
    r["exit_code"] == 0 and not r["outer_timeout"] and r["reader_stopped"] for r in results) else 1)
