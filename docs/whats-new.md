# What's New

The bundled highlights live in `docs/releases/notes.toml`. They are short, user-facing
sentences condensed from the matching `docs/releases/<version>.md` files, with American
spelling because they appear on screen. The Markdown notes remain the detailed release
record. Only 0.2.3 onwards have Markdown files in this checkout; no changes are invented
for 0.2.0 to 0.2.2.

For each release:

1. Update its Markdown notes, including the current draft before publication.
2. Add or revise its entry in `notes.toml`. Keep three lists: `features`,
   `performance_improvements`, `bug_fixes`, in that order. An empty list is valid; an
   entirely empty release is not. Keep entries in increasing semantic-version order.
3. Set the workspace version as usual. The current build must have a matching entry.
4. Run `cargo test -p ca72-plugin --lib whats_new` and the touched crates' usual checks.
   CI runs `bundled_notes_match_the_build_and_the_maintained_sources` on each platform
   before bundling. It checks the workspace/build version, the current entry, unique
   ordered versions, non-empty highlights and coverage of all maintained Markdown releases
   from 0.2.0 onwards. Review the wording against the Markdown: this test cannot verify
   that a summary faithfully describes a change.

The automatic view groups releases after the previous shown version through the installed
one, by Features, Performance improvements, Bug fixes. It lists exactly the versions
included; combined entries name their release so that an older performance or compatibility
claim is not mistaken for a claim about the installed build. Missing or unsafe previous
versions use the current release's highlights without claiming an upgrade history. Semantic
version precedence includes pre-releases and ignores build metadata. A downgrade shows
nothing automatically and keeps the higher version's history.

`CA-72/settings.toml` in the shared data folder keeps `last_version_shown`. The existing
`settings.toml.lock` serialises the read, comparison and atomic replacement across all
instances, formats, hosts and the standalone. This claim happens only in a window frame
once the editor is visible, never in construction, activation, processing or an ordinary
hidden draw. A successful claim marks the version shown immediately, rather than waiting
for dismissal, so two open editors cannot both display it. A crash after the claim can
therefore consume that automatic showing; Settings can always reopen the current notes.
An unreadable or unwritable settings file suppresses the automatic showing and is left
alone. There is no network request and no audio-thread work.

SETTINGS > WHAT'S NEW always opens the current release's highlights. It neither claims nor
rewrites history. Opening or dismissing notes changes no sound parameter, preset or saved
project field. The temporary view is in `ca72-panel/src/whats_new.rs`; selection and history
are in `ca72-plugin/src/whats_new.rs` and `settings.rs`. The view fits the actual viewport,
including compact views, with scrolling, page buttons and keyboard navigation. Escape,
Enter or CLOSE dismisses it. Its final appearance awaits the owner's choice between the
AUDIO & MIDI style and a drawer on the panel's face (R-WHATS-NEW).
