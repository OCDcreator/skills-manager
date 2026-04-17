# Autopilot Baseline: Phase 0

> **Status**: [BASELINE]
> **Preset**: `Maintainability / Refactor`
> **Repository**: `skills-manager`

## Objective

- Improve maintainability of our downstream enhancement layer relative to upstream xingkongliang/skills-manager, thin future enhancement files without creating fragmented architecture, and avoid restructuring upstream-owned code; prefer isolating and maintaining only our added enhancement code.

## Seeded entrypoints

- `AGENTS.md`
- `README.md`
- `docs/`
- `src/`
- `src/views/MySkills.tsx`
- `src-tauri/src/commands/skills.rs`
- `src-tauri/src/core/my_skills_repo.rs`
- `src/views/InstallSkills.tsx`

## Inferred validation commands

- Lint: `npm run lint` (source: `CLI override`)
- Typecheck: `npx tsc -b --pretty false` (source: `CLI override`)
- Full test: `cargo test --manifest-path src-tauri/Cargo.toml` (source: `CLI override`)
- Build: `npm run build` (source: `CLI override`)
- Vulture: not inferred

## Notes

- This phase-0 document is the scaffold baseline.
- The first unattended round should write `docs/status/autopilot-phase-1.md`.
