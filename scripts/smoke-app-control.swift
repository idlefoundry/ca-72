import AppKit
import ApplicationServices
import CoreGraphics
import Foundation

let pid = pid_t(CommandLine.arguments[1])!
let action = CommandLine.arguments[2]
if action == "inspect" {
    let windows = (CGWindowListCopyWindowInfo(.optionAll, kCGNullWindowID) as? [[String: Any]] ?? [])
        .filter { ($0[kCGWindowOwnerPID as String] as? Int) == Int(pid)
            && ($0[kCGWindowLayer as String] as? Int) == 0 }
    let result = windows.compactMap { window -> [String: Any]? in
        guard let bounds = window[kCGWindowBounds as String] as? [String: Any],
              let width = bounds["Width"] as? Double,
              let height = bounds["Height"] as? Double,
              width > 100, height > 100 else { return nil }
        return ["pid": Int(pid), "id": window[kCGWindowNumber as String] ?? 0,
                "bounds": bounds]
    }
    print(String(data: try! JSONSerialization.data(withJSONObject: result), encoding: .utf8)!)
    exit(result.isEmpty ? 2 : 0)
}
guard action == "close" else { exit(2) }
// Press the real window close button; application termination can bypass the
// baseview window-close handler and the runtime's final session save.
let app = AXUIElementCreateApplication(pid)
var value: CFTypeRef?
let queried = AXUIElementCopyAttributeValue(app, kAXWindowsAttribute as CFString, &value)
guard queried == .success, let windows = value as? [AXUIElement], let window = windows.first else {
    print("{\"stage\":\"windows\",\"ax_error\":\(queried.rawValue),\"trusted\":\(AXIsProcessTrusted())}")
    exit(2)
}
value = nil
let found = AXUIElementCopyAttributeValue(window, kAXCloseButtonAttribute as CFString, &value)
guard found == .success, let button = value else {
    print("{\"stage\":\"close_button\",\"ax_error\":\(found.rawValue)}")
    exit(2)
}
let pressed = AXUIElementPerformAction(unsafeBitCast(button, to: AXUIElement.self), kAXPressAction as CFString)
print("{\"method\":\"AXCloseButton\",\"ax_error\":\(pressed.rawValue),\"pid\":\(pid)}")
exit(pressed == .success ? 0 : 2)
