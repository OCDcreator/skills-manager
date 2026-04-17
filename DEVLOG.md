# Fork Devlog

This file records changes that are intentionally different from upstream
`xingkongliang/skills-manager`. Keep it updated whenever this fork adds,
removes, or reworks behavior that should survive future upstream syncs.

## Upstream Sync Checklist

1. `git fetch upstream`
2. Sync `main` from `upstream/main` by merge or rebase.
3. Review the entries in **Fork Deltas** before resolving conflicts.
4. Keep fork-only changes that are still useful; drop entries that upstream has replaced.
5. Add a new dated entry for every fork-specific change.

## Fork Deltas

| Date | Branch / Commit | Area | Keep? | Notes |
| --- | --- | --- | --- | --- |
| 2026-04-16 | `4976bbf` | Fork docs | Yes | Added `AGENTS.md` and clarified that this repo is a fork of `xingkongliang/skills-manager`. |
| 2026-04-16 | `92e4776` | Release updater | Yes | Pointed updater/release config at the fork-owned release channel. Re-check if upstream changes updater config. |
| 2026-04-16 | `feat/my-skills-specialization` | `OCDcreator/my-skills` integration | Yes | Special-cases only `OCDcreator/my-skills`: recursive root import, relative-path preview keys, local workspace status, workspace `pull` / `update` / `push` actions, grouped tag ordering, and per-scenario agent summary in `My Skills`. |

## Design Rules

- Keep generic Git repositories on upstream behavior unless a change is broadly useful.
- Keep `OCDcreator/my-skills` behavior behind explicit repo identity detection.
- Prefer new modules and narrow hook points over broad rewrites.
- When upstream changes Git import/update code, re-test:
  - importing `https://github.com/OCDcreator/my-skills`
  - duplicate skill names under different paths
  - workspace `pull`, `update`, and `push`
  - normal non-`my-skills` Git imports
