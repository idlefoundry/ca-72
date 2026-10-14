# Standalone packaging resources

`icon.svg` is original vector artwork by Idle Foundry, GPL-3.0-or-later. The PNG,
ICO and ICNS are rendered from it and carry the same licence. No external image
or additional embedded font is used. Nimbus Sans is used only during rendering.

`CA-72.desktop` launches the Linux app. The installer substitutes the absolute
user installation path where needed; session audio/MIDI configuration remains
in the user's application data directory.

`entitlements.plist` permits optional audio capture in a hardened macOS app.
The app's Info.plist also contains a microphone purpose string. Output-only use
does not request capture permission.
