"""Diagnostic-only execution budget probe around the exact unmodified589 Audio Unit."""
import json
import os
import platform
from pathlib import Path
import signal
import subprocess
import threading
import time

assert os.environ.get("GITHUB_ACTIONS") == "true" and os.environ.get("RUNNER_ENVIRONMENT") == "github-hosted"
assert platform.system() == "Darwin", "disposable GitHub macOS runner only"

OUT = Path("diagnostic")
PRODUCT = "589e5cca08c66d4c8e64c0a337b0b875ee785344"


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
        trace = OUT.resolve() / name
        trace.mkdir()
        env = dict(os.environ, CA72_AU_TRACE_DIR=str(trace))
        proc = subprocess.Popen(args, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
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
                if now - started > 240:
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
registry_actions = []
for pid, (_, _, command) in rows.items():
    if Path(command.split()[0]).name != "AudioComponentRegistrar":
        continue
    current = subprocess.run(["ps", "-p", str(pid), "-o", "uid=,args="],
                             text=True, capture_output=True)
    identity = current.stdout.strip().split(None, 1)
    if len(identity) == 2 and identity[1] == command:
        if int(identity[0]) != os.getuid():
            registry_actions.append({"pid": pid, "command": command, "action": "skip-system-owner"})
            continue
        registry_actions.append({"pid": pid, "command": command, "action": "refresh-own-registrar"})
        try:
            os.kill(pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
(OUT / "registry-refresh.json").write_text(json.dumps(registry_actions, indent=2) + "\n")
results = [run_probe("direct-stress", ["auval", "-strict", "-stress", "20", "-v", "aumu", "CA72", "IdlF"])]
if results[0]["reader_stopped"]:
    results.append(run_probe("nested-verbose", [
        validator, "--strictness-level", "10", "--validate-in-process", "--timeout-ms", "180000",
        "--random-seed", "0x8dcb47", "--verbose", "--validate", component,
    ], nested=True))
(OUT / "result.json").write_text(json.dumps({
    "source": PRODUCT, "diagnostic_wrapper": False, "au_only_timeout_ms": 180000, "original_timeout_ms": 30000,
    "harness_sha": os.environ.get("GITHUB_SHA"), "runner_arch": platform.machine(),
    "artifact_id": 11932543293, "original_run": 38646776111,
    "strictness": 10, "pluginval_idle_timeout_ms": 180000, "results": results,
}, indent=2) + "\n")
raise SystemExit(0 if len(results) == 2 and results[1]["nested_phase_seen"] and all(
    r["exit_code"] == 0 and not r["outer_timeout"] and r["reader_stopped"] for r in results) else 1)
