#!/usr/bin/env python3
"""A stand-in for RTKit (org.freedesktop.RealtimeKit1) on a private bus, for testing the
standalone's RTKit fallback where no machine has RTKit (decisions.md R-RTKIT). Not shipped.

Run it on a private session bus and point the standalone's system bus at that bus:

    dbus-run-session -- sh -c '
        python3 scripts/fake-rtkit.py &
        sleep 1
        DBUS_SYSTEM_BUS_ADDRESS=$DBUS_SESSION_BUS_ADDRESS \\
            prlimit --rtprio=0:98 target/bundled/ca72-standalone'

It answers as RTKit 0.13 does as it comes (MaxRealtimePriority 20, MinNiceLevel -15,
RTTimeUSecMax 200000), checks what RTKit checks (the thread is the process's, the process's
hard RLIMIT_RTTIME is at most RTTimeUSecMax, the priority within the most), and makes the thread
SCHED_RR | SCHED_RESET_ON_FORK. RTKit does that with CAP_SYS_NICE; this, without it, raises the
process's soft RLIMIT_RTPRIO to its hard one for the moment (hence `prlimit --rtprio=0:98`: no
allowance the process can use itself, a hard limit for the stand-in to lend), then puts it back.

FAKE_RTKIT=refuse refuses every thread as RTKit does when its limits are reached
(org.freedesktop.DBus.Error.AccessDenied); FAKE_RTKIT=refuse-after:N grants N threads first.
Every call is logged on standard error.
"""

import os
import resource
import sys

import dbus
import dbus.service
from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib

NAME = "org.freedesktop.RealtimeKit1"
PATH = "/org/freedesktop/RealtimeKit1"
PROPS = {
    "MaxRealtimePriority": dbus.Int32(20),
    "MinNiceLevel": dbus.Int32(-15),
    "RTTimeUSecMax": dbus.Int64(200000),
}
MODE = os.environ.get("FAKE_RTKIT", "")


def log(*a):
    print("fake-rtkit:", *a, file=sys.stderr, flush=True)


class Denied(dbus.DBusException):
    _dbus_error_name = "org.freedesktop.DBus.Error.AccessDenied"


class RealtimeKit(dbus.service.Object):
    def __init__(self, bus):
        super().__init__(bus, PATH)
        self.bus = bus
        self.granted = 0

    @dbus.service.method(
        "org.freedesktop.DBus.Properties", in_signature="ss", out_signature="v"
    )
    def Get(self, interface, name):
        log("Get", name)
        return PROPS[name]

    @dbus.service.method(
        "org.freedesktop.DBus.Properties", in_signature="s", out_signature="a{sv}"
    )
    def GetAll(self, interface):
        return PROPS

    @dbus.service.method(NAME, in_signature="tu", sender_keyword="sender")
    def MakeThreadRealtime(self, thread, priority, sender=None):
        pid = self.bus.get_unix_process_id(sender)
        self.make(pid, thread, priority)

    @dbus.service.method(NAME, in_signature="ttu")
    def MakeThreadRealtimeWithPID(self, process, thread, priority):
        self.make(int(process), thread, priority)

    def make(self, pid, tid, priority):
        log(f"MakeThreadRealtime pid {pid} tid {tid} priority {priority}")
        if MODE == "refuse" or (
            MODE.startswith("refuse-after:") and self.granted >= int(MODE.split(":")[1])
        ):
            log("  refused (FAKE_RTKIT)")
            raise Denied("Operation not permitted")
        if not 1 <= priority <= int(PROPS["MaxRealtimePriority"]):
            log("  refused: priority out of range")
            raise Denied("Invalid argument")
        if not os.path.exists(f"/proc/{pid}/task/{tid}"):
            log("  refused: not a thread of the process")
            raise Denied("No such process")
        with open(f"/proc/{pid}/limits") as f:
            line = next(l for l in f if l.startswith("Max realtime timeout"))
        hard = line.split()[4]
        if hard == "unlimited" or int(hard) > int(PROPS["RTTimeUSecMax"]):
            log(f"  refused: RLIMIT_RTTIME too high ({line.strip()})")
            raise Denied("Operation not permitted")
        soft, hard_prio = resource.prlimit(pid, resource.RLIMIT_RTPRIO)
        try:
            resource.prlimit(pid, resource.RLIMIT_RTPRIO, (hard_prio, hard_prio))
            os.sched_setscheduler(
                tid,
                os.SCHED_RR | os.SCHED_RESET_ON_FORK,
                os.sched_param(priority),
            )
        except OSError as e:
            log(f"  refused: {e}")
            raise Denied(str(e))
        finally:
            resource.prlimit(pid, resource.RLIMIT_RTPRIO, (soft, hard_prio))
        self.granted += 1
        log(f"  SCHED_RR {priority} ({self.granted} granted)")


def main():
    DBusGMainLoop(set_as_default=True)
    bus = dbus.SessionBus()
    name = dbus.service.BusName(NAME, bus)
    RealtimeKit(bus)
    log("on", os.environ.get("DBUS_SESSION_BUS_ADDRESS"), "mode", MODE or "grant")
    GLib.MainLoop().run()


if __name__ == "__main__":
    main()
