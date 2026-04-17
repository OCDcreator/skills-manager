# Autopilot Phase 1

> **Status**: [SUCCESS]
> **Round**: `3`
> **Lane advanced**: `Repo-source expansion sync`
> **Completed roadmap item**: `R1 - Fix repo-source sibling skill refresh after upstream adds new SKILL.md directories`

## Scope

- Consolidated downstream My Skills workspace post-sync handling into a shared `sync_workspace_after_change` flow so workspace actions and link-import finalization refresh linked skills and then import any newly discovered sibling `SKILL.md` directories from the same repo.
- Switched missing-sibling detection from the transient `before_paths` snapshot to tracked repo subpaths already owned in `SkillStore`, which lets downstream repo-backed workspace checks discover newly added sibling skills without requiring reinstall.
- Extended workspace-backed `check_skill_update` and `update_skill` flows to trigger missing-sibling discovery before returning, so “检查全部” now surfaces newly added importable skills from the linked My Skills repo.
- Added a focused Rust test covering the untracked-workspace-skill detection logic.

## Files changed

- `src-tauri/src/core/my_skills_repo.rs`
- `src-tauri/src/commands/skills.rs`
- `docs/status/autopilot-lane-map.md`
- `docs/status/autopilot-round-roadmap.md`
- `docs/status/autopilot-phase-1.md`

## Discovery/update behavior

- For linked My Skills repo sources, workspace sync now compares current workspace skill directories against tracked `source_subpath` entries in `SkillStore`.
- Existing linked skills still refresh in place from the workspace revision.
- Any sibling workspace skill directory not already tracked is installed into the managed catalog and becomes available in-product after workspace actions, link-import completion, or workspace-backed update checks.

## Validation

- Focused test: `cargo test --manifest-path src-tauri/Cargo.toml core::my_skills_repo::tests::collects_only_untracked_workspace_skill_dirs -- --exact`
- Lint: `npx eslint src/views/MySkills.tsx src/lib/tauri.ts src/components/MySkillsTerminalPanel.tsx`
- Typecheck: `npx tsc -b --pretty false`
- Full test: `cargo test --manifest-path src-tauri/Cargo.toml`
- Build: `npm run build`

## Vulture

- Not configured for this repository autopilot preset, so no dead-code observability run was available this round.

## Next recommended slice

- Advance to `R2 - Follow-up maintainability / refactor slice`, with the next bounded focus on trimming the remaining downstream command/update ownership surface around the My Skills enhancement seam without adding pass-through wrappers.
