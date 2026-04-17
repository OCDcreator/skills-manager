# Autopilot Lane Map

> **Preset**: `Maintainability / Refactor`
> **Current `[NEXT]`**: none; preset queue complete after `R3 - Checkpoint after first refactor batch`

## Current priority

- Stop the current preset queue unless a maintainer manually adds another bounded slice
- Keep the queue bounded and repo-specific
- Keep the repo-source sibling-skill refresh fix stable in the downstream enhancement flow
- Reduce one maintainability hotspot at a time
- Keep configured validation commands green

## Suggested entrypoints

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
- `src-tauri/src/core/my_skills_terminal.rs`
- `src-tauri/src/core/my_skills_repo.rs`

## Validation baseline

- Lint: `npx eslint src/views/MySkills.tsx src/lib/tauri.ts src/components/MySkillsTerminalPanel.tsx` (source: `manual downstream override`)
- Typecheck: `npx tsc -b --pretty false` (source: `CLI override`)
- Full test: `cargo test --manifest-path src-tauri/Cargo.toml` (source: `CLI override`)
- Build: `npm run build` (source: `CLI override`)
- Vulture: not inferred

## Boundaries

- Do not refactor outside the queued slice
- Do not add a new `[NEXT]` item automatically after the R3 checkpoint
- Do not turn maintainability work into a broad rewrite
- Keep upstream-owned structure recognizable; optimize downstream enhancement seams instead
- Thin large downstream files with cohesive extraction, not wrapper fragmentation
- Do not spend rounds fixing unrelated upstream lint debt unless that debt directly blocks the queued downstream slice
- Keep `automation/runtime/` ignored and machine-local state out of committed files
