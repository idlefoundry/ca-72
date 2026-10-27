# Working on the CA-72 as an agent

These rules are for coding agents (Claude, Codex and others) working on the owner's
machines: the Mac, the Omarchy desktop, the Omarchy laptop and the Windows PC. Several agents
often work at once, on several machines. The rules keep their work from being lost or
colliding, and keep the disks from filling. How to build, test and record decisions is in
CONTRIBUTING.md and the README.

## GitHub is the only meeting place

- Start from GitHub: `git fetch origin`, and branch from `origin/<base>`. Don't fetch, pull or
  copy work from another machine's checkout, and don't add remotes that point at one.
- Push your branch to `origin` before you stop, finished or not, and push again whenever you
  commit something worth keeping. Work that is only on one machine is work that can be lost.
- Never push to `main` directly or force-push a branch you didn't create; `main` changes
  through pull requests the owner merges.
- Don't delete branches, local or on GitHub; the owner prunes them.

## One worktree per task

- Do your task in its own worktree, beside the main checkout:
  `git worktree add <code folder>/ca-72-w-<topic> -b <topic> origin/<base>`. The code folder
  is `~/Code` on the Mac and the desktop, `~/Projects/Code` on the laptop and
  `C:\Users\idlewile\Code` on Windows.
- Don't work in the main checkout (`<code folder>/ca-72`): the owner or another agent may be
  using it, and switching its branch pulls the ground from under them.
- Don't put worktrees or clones anywhere else (`~/.cache`, `/tmp`, `~/Documents`, `~/Music`).
  A worktree your tool makes for itself (Codex's `~/.codex/worktrees`, Claude Code's
  `.claude/worktrees`) is fine, under the same rules.
- When the task's branch is pushed and the work is merged or set aside, remove the worktree
  with `git worktree remove` (the branch stays). Before removing one, check that it has no
  uncommitted changes and that nothing is running in it.

## Big files go to the lab share

- Renders, measurements, captures, recordings, bundles and other large output go to the shared
  lab storage, in `/mnt/lab/artifacts/ca-72/<topic>/` on Linux or `~/lab/artifacts/ca-72/<topic>/`
  on the Mac, not into the repository or the code folder. Check the share is mounted
  (`df` shows the NAS) before writing to it.
- `target/` is disposable: a worktree's build output may be deleted once it has been idle for
  a few days. Keep nothing there you need.
- Check free disk space before a long build or a large capture (`df -h`). The Mac's disk is
  the tightest.
