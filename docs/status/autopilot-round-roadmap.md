# Autopilot Round Roadmap

## Queue

### [DONE] R1 - Fix repo-source sibling skill refresh after upstream adds new SKILL.md directories

- **Lane**: Repo-source expansion sync
- **Goal**: When a repo-backed installed skill's upstream source repo gains additional sibling `SKILL.md` directories, make those newly available skills discoverable/importable in Skills Manager without requiring manual reinstall or missing them after “检查全部”.
- **Priority entrypoints**:
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
- **Constraints**:
  - Stay inside one bounded slice
  - Do not create thin wrappers that only rename pass-through ownership
  - Preserve upstream-owned architecture; refactor downstream enhancement seams first
  - Thin files by extracting cohesive owned logic, not by creating fragmented micro-files
  - Preserve existing runtime behavior
- **Acceptance**:
  - A repo-backed source with newly added sibling skill directories surfaces new importable skills or sync targets in-product
  - The behavior is documented in the phase doc, including how discovery/update now works
  - The phase doc records scope, changed files, and validation results
  - Every configured validation command passes

### [DONE] R2 - Follow-up maintainability / refactor slice

- **Lane**: Maintainability / ownership reduction
- **Goal**: After the sibling-skill refresh bug is fixed, continue with the next bounded downstream maintainability slice while staying within the same validation baseline.
- **Constraints**:
  - Build on the prior phase doc instead of starting a new free-form lane
  - Keep behavior unchanged outside the queued slice
- **Acceptance**:
  - Another queued slice lands with validations green

### [NEXT] R3 - Checkpoint after first refactor batch

- **Lane**: Checkpoint
- **Goal**: Review R1-R2, document what ownership actually moved, and decide whether the preset queue should stop or be manually extended.
- **Constraints**:
  - Do not extend the queue automatically beyond R3
  - Focus on documentation and metrics, not new refactors
- **Acceptance**:
  - The phase doc captures wins, remaining hotspots, and a clear stop/continue recommendation

## Current state

- The current `[NEXT]` is `R3 - Checkpoint after first refactor batch`.
- Successful rounds must keep the queue synchronized with the phase docs.
