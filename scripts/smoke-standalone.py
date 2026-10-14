"""Run only in a disposable OS runner, against a hash-verified CI product artifact."""
import ctypes as C
import ctypes.util
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import time

SOURCE = "e43d39cc0b941f3a6635a50bf75cc6fc45ff07bd"
ROOT = Path("native-smoke").resolve()
ROOT.mkdir(exist_ok=True)
SYSTEM = platform.system()
ARTIFACT_NAME = {"Darwin": "CA-72-macOS", "Windows": "CA-72-Windows", "Linux": "CA-72-Linux"}[SYSTEM]
report = {"source": SOURCE, "system": SYSTEM, "hardware_access": False, "complete": False}


def save():
    (ROOT / "proof.json").write_text(json.dumps(report, indent=2) + "\n")


def download():
    import zipfile
    endpoint = "repos/idlefoundry/ca-72/actions/runs/38644759309"
    metadata = json.loads(subprocess.check_output(["gh", "api", endpoint]))
    assert metadata["head_sha"] == SOURCE
    inventory = json.loads(subprocess.check_output(["gh", "api", endpoint + "/artifacts"]))
    matches = [a for a in inventory["artifacts"] if a["name"] == ARTIFACT_NAME and not a["expired"]]
    assert len(matches) == 1, "exactly one initial-run artifact required"
    artifact = matches[0]
    archive = ROOT / "candidate.zip"
    with archive.open("wb") as target:
        subprocess.run(["gh", "api", f"repos/idlefoundry/ca-72/actions/artifacts/{artifact['id']}/zip"],
                       stdout=target, check=True, timeout=180)
    digest = "sha256:" + hashlib.sha256(archive.read_bytes()).hexdigest()
    assert digest == artifact["digest"]
    report["artifact"] = {"id": artifact["id"], "name": artifact["name"], "digest": digest}
    dest = ROOT / "product"
    dest.mkdir()
    with zipfile.ZipFile(archive) as zipped:
        assert all((dest / item.filename).resolve().is_relative_to(dest) for item in zipped.infolist())
        zipped.extractall(dest)
    return dest


def windows_for(pid):
    user = C.windll.user32
    callback = C.WINFUNCTYPE(C.c_bool, C.c_void_p, C.c_ssize_t)
    result = []

    @callback
    def visit(window, _):
        owner = C.c_ulong()
        user.GetWindowThreadProcessId(C.c_void_p(window), C.byref(owner))
        if owner.value == pid and user.GetWindowTextLengthW(C.c_void_p(window)) > 0:
            result.append({"id": window, "pid": pid})
        return True

    user.EnumWindows(visit, 0)
    return result


def x11():
    lib = C.CDLL(ctypes.util.find_library("X11"))
    lib.XOpenDisplay.argtypes = [C.c_char_p]
    lib.XOpenDisplay.restype = C.c_void_p
    display = lib.XOpenDisplay(None)
    assert display, "isolated Xvfb display required"
    lib.XDefaultRootWindow.argtypes = [C.c_void_p]
    lib.XDefaultRootWindow.restype = C.c_ulong
    lib.XQueryTree.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_ulong), C.POINTER(C.c_ulong),
                              C.POINTER(C.POINTER(C.c_ulong)), C.POINTER(C.c_uint)]
    lib.XFetchName.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_void_p)]
    lib.XFree.argtypes = [C.c_void_p]
    lib.XCloseDisplay.argtypes = [C.c_void_p]
    return lib, display


def inspect(pid):
    if SYSTEM == "Windows":
        return windows_for(pid)
    if SYSTEM == "Darwin":
        result = subprocess.run([str(ROOT / "app-control"), str(pid), "inspect"],
                                capture_output=True, text=True, timeout=10)
        assert result.returncode in (0, 2), result.stderr
        return json.loads(result.stdout)
    # The workflow supplies a fresh Xvfb server containing only this test app.
    lib, display = x11()
    try:
        root, parent = C.c_ulong(), C.c_ulong()
        children = C.POINTER(C.c_ulong)()
        count = C.c_uint()
        assert lib.XQueryTree(display, lib.XDefaultRootWindow(display), C.byref(root),
                              C.byref(parent), C.byref(children), C.byref(count))
        result = []
        for i in range(count.value):
            name = C.c_void_p()
            if lib.XFetchName(display, children[i], C.byref(name)) and name.value:
                title = C.string_at(name).decode(errors="replace")
                lib.XFree(name)
                if title == "CA-72":
                    result.append({"id": children[i], "title": title, "pid": pid})
        if children:
            lib.XFree(children)
        return result
    finally:
        lib.XCloseDisplay(display)


def close(pid, window):
    if SYSTEM == "Darwin":
        result = subprocess.run([str(ROOT / "app-control"), str(pid), "close"],
                                capture_output=True, text=True, timeout=10)
        assert result.returncode == 0, result.stdout + result.stderr
        return json.loads(result.stdout)
    if SYSTEM == "Windows":
        # Re-read exact ownership before sending WM_CLOSE to the captured root window.
        assert window in windows_for(pid)
        assert C.windll.user32.PostMessageW(C.c_void_p(window["id"]), 0x10, 0, 0)
        return {"method": "WM_CLOSE", "window": window}
    lib, display = x11()
    try:
        class Data(C.Union):
            _fields_ = [("longs", C.c_long * 5), ("bytes", C.c_char * 20)]
        class Client(C.Structure):
            _fields_ = [("type", C.c_int), ("serial", C.c_ulong), ("sent", C.c_int),
                        ("display", C.c_void_p), ("window", C.c_ulong), ("message", C.c_ulong),
                        ("format", C.c_int), ("data", Data)]
        class Event(C.Union):
            _fields_ = [("client", Client), ("padding", C.c_long * 24)]
        lib.XInternAtom.argtypes = [C.c_void_p, C.c_char_p, C.c_int]
        lib.XInternAtom.restype = C.c_ulong
        lib.XSendEvent.argtypes = [C.c_void_p, C.c_ulong, C.c_int, C.c_long, C.POINTER(Event)]
        lib.XFlush.argtypes = [C.c_void_p]
        event = Event()
        event.client.type, event.client.sent, event.client.format = 33, 1, 32
        event.client.display, event.client.window = display, window["id"]
        event.client.message = lib.XInternAtom(display, b"WM_PROTOCOLS", 0)
        event.client.data.longs[0] = lib.XInternAtom(display, b"WM_DELETE_WINDOW", 0)
        assert lib.XSendEvent(display, window["id"], 0, 0, C.byref(event))
        lib.XFlush(display)
        return {"method": "WM_DELETE_WINDOW", "window": window}
    finally:
        lib.XCloseDisplay(display)


def session():
    data = json.loads((ROOT / "session/session.json").read_text())
    assert data["format"] == 1 and data["app_id"] == "com.idlefoundry.ca-72"
    plugin = json.loads(bytes(data["plugin_state"]))
    assert plugin["version"] == "0.2.2"
    return data, plugin


try:
    assert os.environ.get("GITHUB_ACTIONS") == "true", "disposable CI runners only"
    product = download()
    binary = product / {"Darwin": "CA-72.app/Contents/MacOS/ca72-standalone",
                        "Windows": "CA-72.exe", "Linux": "ca72-standalone"}[SYSTEM]
    binary.chmod(0o755)
    report["binary_sha256"] = hashlib.sha256(binary.read_bytes()).hexdigest()
    report["cli"] = []
    for args, expected in [(["--help"], 0), (["--unknown-smoke-argument"], 2), (["--state-dir"], 2)]:
        result = subprocess.run([str(binary), *args], capture_output=True, text=True, timeout=15)
        report["cli"].append({"args": args, "exit_code": result.returncode,
                              "stdout": result.stdout, "stderr": result.stderr})
        assert result.returncode == expected
    if SYSTEM == "Darwin":
        subprocess.run(["swiftc", "scripts/smoke-app-control.swift", "-o", str(ROOT / "app-control")],
                       check=True, timeout=120)
    report["launches"] = []
    for number in range(2):
        with (ROOT / f"launch-{number}.log").open("w") as log:
            command = [str(binary), "--no-devices", "--state-dir", str(ROOT / "session")]
            proc = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
            launch = {"pid": proc.pid, "command": command, "forced_cleanup": False}
            report["launches"].append(launch)
            save()
            try:
                deadline = time.monotonic() + 30
                windows = []
                while time.monotonic() < deadline:
                    assert proc.poll() is None, "app exited before opening its window"
                    windows = inspect(proc.pid)
                    if windows and (ROOT / "session/session.json").exists():
                        break
                    time.sleep(0.25)
                assert windows and (ROOT / "session/session.json").exists(), "window/session startup timed out"
                launch["windows"] = windows
                data, plugin = session()
                if number == 1:
                    assert plugin["params"]["tune"] == {"f32": 0.125}, "nondefault session was not restored"
                started = time.monotonic()
                launch["close"] = close(proc.pid, windows[0])
                launch["exit_code"] = proc.wait(timeout=8)
                launch["close_seconds"] = time.monotonic() - started
                assert launch["exit_code"] == 0 and launch["close_seconds"] < 5
                data, plugin = session()
                if number == 0:
                    assert set(plugin["params"]["tune"]) == {"f32"}
                    plugin["params"]["tune"] = {"f32": 0.125}
                    data["plugin_state"] = list(json.dumps(plugin, separators=(",", ":")).encode())
                    # The runtime's SessionFile serializer drops unknown fields. Requiring
                    # this marker to disappear proves the second run actually rewrote the
                    # file rather than merely leaving our seeded value untouched.
                    data["native_smoke_marker"] = "must disappear on app save"
                    (ROOT / "session/session.json").write_text(json.dumps(data))
                else:
                    assert "native_smoke_marker" not in data, "app did not replace the seeded session"
                    assert plugin["params"]["tune"] == {"f32": 0.125}, "restored value was not re-saved"
            finally:
                if proc.poll() is None:
                    launch["forced_cleanup"] = True
                    # This exact Popen child belongs to this isolated runner/test only.
                    launch["cleanup_identity"] = {"pid": proc.pid, "command": command}
                    proc.terminate()
                    try:
                        proc.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        proc.kill()
                        proc.wait(timeout=5)
                save()
    report["complete"] = True
except Exception as error:
    report["error"] = repr(error)
    raise
finally:
    save()
