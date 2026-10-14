#!/usr/bin/env bash
# The Windows ZIP for installing by hand, target/packages/CA-72-<version>-Windows-x86_64.zip,
# with its checksum beside it (.zip.sha256): the VST3 and the CLAP as CI built them for
# Windows (its CA-72-Windows artifact), unchanged, with README.txt, LICENSE.txt and
# THIRD-PARTY-NOTICES.txt (also in the VST3's Resources) and FILE-SHA256SUMS.txt
# (decisions.md R40). CI's release job makes it for each tag; by hand:
#
#   scripts/windows-zip.sh <folder holding CA-72.vst3 and CA-72.clap>
#
# The README names the commit, and in CI the run; CA72_WINDOWS_ARTIFACT_ID and
# CA72_WINDOWS_ARTIFACT_SHA256, when set, the artifact and its ZIP's checksum. Needs zip and
# sha256sum.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
bundles="$(cd "$1" && pwd)"
version="$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -n 1)"
commit="$(git -C "$root" rev-parse HEAD)"
repo=idlefoundry/ca-72
tag="v$version"
name="CA-72-$version-Windows-x86_64"
work="$root/target/windows-zip"
rm -rf "$work"
mkdir -p "$work/$name" "$root/target/packages"
cd "$work/$name"

binary=CA-72.vst3/Contents/x86_64-win/CA-72.vst3
cp "$bundles/CA-72.clap" CA-72.clap
cp -R "$bundles/CA-72.vst3" CA-72.vst3
[ -f "$binary" ] || { echo "windows-zip: no $binary in $bundles" >&2; exit 1; }
mkdir -p CA-72.vst3/Contents/Resources
# The CLAP and the VST3 are one library (nih-plug exports both entry points).
cmp -s CA-72.clap "$binary" || { echo "windows-zip: the CLAP and the VST3 differ" >&2; exit 1; }
cp "$root/LICENSE" LICENSE.txt
cp "$root/THIRD-PARTY-NOTICES.txt" THIRD-PARTY-NOTICES.txt
cp LICENSE.txt THIRD-PARTY-NOTICES.txt CA-72.vst3/Contents/Resources/

provenance="Source commit: $commit"
if [ -n "${GITHUB_RUN_ID:-}" ]; then
    provenance+=$'\n'"CI run that built it:"$'\n'"https://github.com/$repo/actions/runs/$GITHUB_RUN_ID"
fi
if [ -n "${CA72_WINDOWS_ARTIFACT_ID:-}" ]; then
    provenance+=$'\n'"Binary artifact: CA-72-Windows, artifact ID $CA72_WINDOWS_ARTIFACT_ID"
fi
if [ -n "${CA72_WINDOWS_ARTIFACT_SHA256:-}" ]; then
    provenance+=$'\n'"Original artifact ZIP SHA-256:"$'\n'"$CA72_WINDOWS_ARTIFACT_SHA256"
fi
provenance+=$'\n'"Each plug-in binary SHA-256:"$'\n'"$(sha256sum CA-72.clap | cut -d' ' -f1)"

title="CA-72 $version - Windows x64 manual installation"
cat > README.txt <<EOF
$title
$(printf '%*s' ${#title} '' | tr ' ' '=')

VST3 and CLAP plug-ins for a compatible 64-bit audio host on Windows 10 or 11,
without the installer. These are the unchanged binaries of the $tag release
build, from the same CI run as CA-72-$version-Windows-setup.exe. No standalone
application is included.

Install
-------
1. Close your audio host before installing or updating.
2. Extract this ZIP. Do not load plug-ins directly from the ZIP.
3. Copy the whole CA-72.vst3 folder to:
   C:\\Program Files\\Common Files\\VST3\\
   Keep its internal Contents\\x86_64-win\\CA-72.vst3 structure intact.
4. If your host supports CLAP, copy CA-72.clap to:
   C:\\Program Files\\Common Files\\CLAP\\
   You may install only the format you use.
5. Start your host and rescan its plug-ins. CA-72 appears as an instrument
   by Idle Foundry.

Windows may ask for administrator permission when copying to Program Files.
An alternative folder works only if your host supports and scans that folder
for the chosen plug-in format. Keep just one installed copy of each format in
the folders your host scans, to avoid loading an older duplicate.

Update
------
Close the host. Replace the old CA-72.vst3 folder as a whole with the new one;
do not merge it and leave old files behind. Replace CA-72.clap if installed.
Then reopen and rescan the host. Keep your presets folder unchanged.

The plug-in's DOWNLOAD button opens the Windows installer. The manual-install
ZIP of each version is on the release page:
https://github.com/$repo/releases

Requirements and data
---------------------
This build statically links the C runtime and has no Visual C++
Redistributable dependency. Its imported DLLs are Windows system libraries.
Factory presets and fonts are built into the plug-ins; no separate sound or
font files need installing. Keep LICENSE.txt and THIRD-PARTY-NOTICES.txt with
your copy. Copies of both are also inside the VST3 bundle's Resources folder.

This is an installer-free plug-in package, not a standalone synthesizer or a
self-contained portable profile. Your presets and their library settings
remain in:
%APPDATA%\\Idle Foundry\\CA-72\\Presets

Uninstall
---------
Close the host and remove the copied CA-72.vst3 folder and CA-72.clap file.
Your presets remain. Manual installation creates no Windows Apps entry.
If you previously used the installer, its Apps entry may still show the old
version, and its uninstaller can remove plug-ins later replaced manually.

Licence and source
------------------
Copyright (C) 2026 Idle Foundry Ltd.
GNU General Public License, version 3 or later. See LICENSE.txt.
Third-party licences are in THIRD-PARTY-NOTICES.txt.
Source and build instructions:
https://github.com/$repo/tree/$tag
Git dependency sources accompany the release:
https://github.com/$repo/releases/tag/$tag

Build provenance
----------------
$provenance

This archive adds installation instructions and licence files to those
unmodified binaries. FILE-SHA256SUMS.txt lists every other file in this ZIP.
EOF

find . -type f ! -name FILE-SHA256SUMS.txt | sed 's|^\./||' | LC_ALL=C sort | xargs sha256sum > FILE-SHA256SUMS.txt
zip="$root/target/packages/$name.zip"
rm -f "$zip" "$zip.sha256"
find . -type f | sed 's|^\./||' | LC_ALL=C sort | zip -q -X "$zip" -@
(cd "$root/target/packages" && sha256sum "$name.zip" > "$name.zip.sha256")
echo "Created $zip"
