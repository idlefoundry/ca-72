# The CA-72 skin pack agent skill

*Dedicated to the public domain under CC0 1.0 (`LICENSE.txt`), like the format it teaches
(`docs/skin-packs.md`) and the starter pack (`docs/skin-packs/starter`).*

**Status: a draft, not published.** It follows the skin-pack prototype on the `skin-packs`
branch, which may still change before its first release.

`ca72-skin-pack/` is an agent skill: instructions that teach a coding agent (Claude Code,
Codex and others) to make a CA-72 skin pack with someone, from the CC0 starter, checking it
with `check_pack` after every change, recording its licence and provenance honestly, and
delivering it as a pack file (`.ifskin`). It leaves signing to the marketplace.

```
ca72-skin-pack/
  SKILL.md                  the steps and the rules (read first)
  reference/layout.md       coordinates, the 42 controls, click areas, fixtures, print
  reference/drawing.md      the SVG subset, the drawing's offset, picture slots, fonts
  reference/provenance.md   the manifest, licences, provenance, ids and updates, signatures,
                            pack files, declarations, what never goes in
  reference/findings.md     each check finding and what to do about it
```

## Using it

The agent needs a clone of the CA-72's repository (for the starter and `check_pack`) and a
Rust toolchain.

- **Claude Code:** copy `ca72-skin-pack/` into `~/.claude/skills/` (for you) or a project's
  `.claude/skills/` (for everyone working in it). Ask for "a CA-72 skin pack".
- **Codex:** copy `ca72-skin-pack/` into Codex's skills folder (`~/.codex/skills/`), or point
  the agent at `SKILL.md` from an `AGENTS.md`.
- **Other agents, or a chat without a shell:** give it `SKILL.md` and the reference files. If
  it cannot run commands, run `check_pack` yourself and give it the output and both previews.

A pack made with the skill is the user's, under the licence they choose. The skill records
what the agent made as generated (`origin = "generated"`, with the tool), because whether
such work is protected, and so licensable, differs between countries.
