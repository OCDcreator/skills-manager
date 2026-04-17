# Autopilot Baseline: Phase 0

> **Status**: [BASELINE]
> **Preset**: `Maintainability / Refactor`
> **Repository**: `skills-manager`

## Objective

- Improve maintainability of our downstream enhancement layer relative to upstream xingkongliang/skills-manager, thin future enhancement files without creating fragmented architecture, and avoid restructuring upstream-owned code; prefer isolating and maintaining only our added enhancement code.

## Seeded entrypoints

- `AGENTS.md`
- `automation/round-prompt.md`
- `docs/status/`
- `src/views/MySkills.tsx`
- `src/components/MySkillsTerminalPanel.tsx`
- `src/lib/tauri.ts`
- `src-tauri/src/commands/skills.rs`
- `src-tauri/src/core/git_fetcher.rs`
- `src-tauri/src/core/installer.rs`
- `src-tauri/src/core/skill_store.rs`
- `src-tauri/src/core/my_skills_repo.rs`

## Inferred validation commands

- Lint: `npx eslint src/views/MySkills.tsx src/lib/tauri.ts src/components/MySkillsTerminalPanel.tsx` (source: `manual downstream override`)
- Typecheck: `npx tsc -b --pretty false` (source: `CLI override`)
- Full test: `cargo test --manifest-path src-tauri/Cargo.toml` (source: `CLI override`)
- Build: `npm run build` (source: `CLI override`)
- Vulture: not inferred

## Notes

- This phase-0 document is the scaffold baseline.
- Global `npm run lint` is currently blocked by pre-existing upstream-owned `SkillDetailPanel.tsx` rules; unattended rounds should validate the downstream enhancement UI seam until that upstream baseline is intentionally addressed.
- The first unattended round should write `docs/status/autopilot-phase-1.md`.
