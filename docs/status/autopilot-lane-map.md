# Autopilot Lane Map

> **Preset**: `Maintainability / Refactor`
> **Current `[NEXT]`**: `R1 - First maintainability / refactor slice`

## Current priority

- Keep the queue bounded and repo-specific
- Reduce one maintainability hotspot at a time
- Keep configured validation commands green

## Suggested entrypoints

- `AGENTS.md`
- `automation/round-prompt.md`
- `docs/status/`
- `src/views/MySkills.tsx`
- `src/components/MySkillsTerminalPanel.tsx`
- `src/lib/tauri.ts`
- `src-tauri/src/core/my_skills_terminal.rs`
- `src-tauri/src/commands/skills.rs`
- `src-tauri/src/core/my_skills_repo.rs`

## Validation baseline

- Lint: `npm run lint` (source: `CLI override`)
- Typecheck: `npx tsc -b --pretty false` (source: `CLI override`)
- Full test: `cargo test --manifest-path src-tauri/Cargo.toml` (source: `CLI override`)
- Build: `npm run build` (source: `CLI override`)
- Vulture: not inferred

## Boundaries

- Do not refactor outside the queued slice
- Do not turn maintainability work into a broad rewrite
- Keep upstream-owned structure recognizable; optimize downstream enhancement seams instead
- Thin large downstream files with cohesive extraction, not wrapper fragmentation
- Keep `automation/runtime/` ignored and machine-local state out of committed files
