# What counting the instances across processes by locked seat files costs: 64 seat files, 8 of
# them locked by another process (as 8 instances elsewhere), the seats probed with non-blocking
# exclusive locks (a free one unlocked at once); and that a process's locks go when it is
# killed (the crash-safe release).
import fcntl, os, subprocess, sys, tempfile, time
d = tempfile.mkdtemp(prefix="ca72-seats-")
paths = [os.path.join(d, f"seat-{k}") for k in range(64)]
for p in paths:
    open(p, "w").close()
holder = subprocess.Popen([sys.executable, "-I", "-c", f"""
import fcntl, sys, time
fs = [open(p, "r+") for p in {paths[:8]!r}]
for f in fs: fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
print("held", flush=True); time.sleep(60)
"""], stdout=subprocess.PIPE, text=True)
assert holder.stdout.readline().strip() == "held"
fds = [os.open(p, os.O_RDWR) for p in paths]
def probe():
    held = 0
    for fd in fds:
        try:
            fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
            fcntl.flock(fd, fcntl.LOCK_UN)
        except BlockingIOError:
            held += 1
    return held
assert probe() == 8
n = 2000
t = time.perf_counter()
for _ in range(n):
    probe()
us = (time.perf_counter() - t) / n * 1e6
holder.kill(); holder.wait()
after = probe()
print(f"{sys.platform}: probing 64 seats (8 held elsewhere): {us:.1f} us; after the holder was killed, held: {after}")
