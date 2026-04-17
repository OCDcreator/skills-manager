# Autopilot Master Plan

> **Status**: [ACTIVE]
> **Preset**: `Maintainability / Refactor`
> **Repository**: `skills-manager`

## Overall objective

- Improve maintainability of our downstream enhancement layer relative to upstream xingkongliang/skills-manager, thin future enhancement files without creating fragmented architecture, and avoid restructuring upstream-owned code; prefer isolating and maintaining only our added enhancement code.
- Prefer queue-driven ownership reduction over free-form cleanup
- Keep configured validation commands green after every successful round

## Priority lanes

- **P1. Thick-owner reduction**: shrink modules or entrypoints that still concentrate too much ownership
- **P2. Validation friction**: clean up validation-heavy hotspots only when they unblock or stabilize P1 work
- **P3. Boundary hygiene**: keep docs, queue, and validation instructions aligned with the current architecture

## Guardrails

- Follow the first `[NEXT]` item in `docs/status/autopilot-round-roadmap.md`
- Do not expand the queue automatically beyond the preset checkpoint item
- Do not create new thin wrappers unless they isolate a genuinely reused or risky dependency
- Do not reshape upstream-owned architecture when the maintainability target is only a downstream enhancement seam
- Favor consolidating enhancement logic into clearer owned modules over sprinkling micro-files
- Do not change product behavior while chasing maintainability wins
