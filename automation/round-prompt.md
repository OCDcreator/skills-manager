# Repository Autopilot Round — Maintainability / Refactor

You are running one unattended repository autopilot round inside the `skills-manager` repository.

Read these files first, in order:
- `AGENTS.md` if it exists
- `docs/status/autopilot-master-plan.md`
- `docs/status/autopilot-round-roadmap.md`
- `docs/status/autopilot-lane-map.md`
- `{{last_phase_doc}}`

Mission:
- Continue the maintainability / refactor program toward smaller, easier-to-own modules.
- Execute exactly one queued refactor slice: the first item marked `[NEXT]` in `docs/status/autopilot-round-roadmap.md`.
- Do not freestyle outside the queue.
- Do not start another round.

Round metadata:
- Attempt number: `{{round_attempt}}`
- Next phase number: `{{next_phase_number}}`
- New phase doc path: `{{next_phase_doc}}`
- Current branch: `{{current_branch}}`
- Last successful phase doc: `{{last_phase_doc}}`
- Last commit: `{{last_commit_sha}}`
- Previous summary: `{{last_summary}}`
- Focus hint: `{{focus_hint}}`
- Objective: `{{objective}}`
- Platform note: `{{platform_note}}`
- Runner kind: `{{runner_kind}}`
- Runner model: `{{runner_model}}`

Configured validation commands:
- Lint: `{{lint_command}}`
- Typecheck: `{{typecheck_command}}`
- Full test: `{{full_test_command}}`
- Build: `{{build_command}}`
- Vulture: `{{vulture_command}}`

Required workflow:
1. Use the plan tool before making substantive changes.
2. Read the current `[NEXT]` queue item and restate its lane, goal, constraints, and acceptance criteria in your plan.
3. Start from the roadmap and lane-map entrypoints before broad searching.
4. Read only the code and docs needed for this one slice.
5. Make the smallest meaningful maintainability refactor that satisfies the queue item and preserves behavior.
6. Prefer reducing direct ownership and import/assembly surface over moving code into new thin wrappers.
7. Treat the upstream `xingkongliang/skills-manager` architecture as stable: prefer isolating, thinning, and documenting downstream enhancement code rather than reshaping upstream-owned subsystems.
8. Thin large downstream files by extracting cohesive enhancement-owned units, not by scattering tiny wrapper files or fragment-only pass-through modules.
9. Update only directly related docs when the module boundary materially changes.
10. Run targeted tests first when code or tests change and a targeted test command pattern is configured.
11. `Run every configured validation command below on successful rounds.`
12. When a validation command is blank, do not invent a substitute; record the gap in the phase doc instead.
13. When Vulture is configured, use it as the dead-code observability command when ownership cleanup or unused code is relevant; record the finding count or any gap in the phase doc.
14. Update `docs/status/autopilot-round-roadmap.md` on success: mark the executed `[NEXT]` item as `[DONE]`, promote the next `[QUEUED]` item to `[NEXT]`, and keep later items `[QUEUED]`.
15. Write the round summary to `{{next_phase_doc}}`. Include scope, files changed, validation commands, Vulture findings when configured, the lane advanced, the completed roadmap queue item, and the next recommended slice.
16. Commit successful rounds as `{{commit_prefix}}: round {{round_attempt}} - <short subject>`.
17. If validation fails, attempt one focused repair. If it still fails, revert the round, do not commit, and return `failure`.
18. If the queued objective is already complete, avoid unnecessary edits, update the roadmap accordingly, and return `goal_complete`.

Response contract:
- Your final response must be valid JSON matching the provided output schema.
- Use actual repo-relative paths in `phase_doc_path` and `changed_files`.
- Set `status` to one of `success`, `failure`, or `goal_complete`.
- On `success`, `commit_sha` and `commit_message` must be non-null.
- On `failure`, `blocking_reason` must explain why the round stopped.
- Include every command you ran in `commands_run`, and list the validation commands in `tests_run`.
