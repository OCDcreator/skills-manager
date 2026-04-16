# Skills Manager

## Repo Identity
- This repo is a fork. The upstream project is `xingkongliang/skills-manager`; local `origin` may point at the fork instead of the upstream canonical repo.

## Structure
- Desktop app = React/Vite frontend in `src/` plus Tauri/Rust backend in `src-tauri/`.
- Frontend entry is `src/main.tsx`; routes and shell live in `src/App.tsx`; shared app loading/state lives in `src/context/AppContext.tsx`.
- Frontend should talk to Rust through `src/lib/tauri.ts`. When adding a Tauri command, update the Rust command implementation, `src-tauri/src/lib.rs` `generate_handler!`, and the TS wrapper/types together.
- Keep business logic in `src-tauri/src/core/*.rs`; `src-tauri/src/commands/*.rs` are the thin Tauri command layer.
- Persistent app data lives under `~/.skills-manager` (`skills/`, `scenarios/`, `cache/`, `logs/`, `skills-manager.db`). Startup migrates the legacy `~/.agent-skills` path.

## Commands
- `npm install`
- `npm run tauri:dev` is the normal desktop dev loop; it uses `src-tauri/tauri.dev.conf.json`.
- `npm run dev` runs the frontend only.
- `npm run build` runs TypeScript build plus Vite build; `tauri build` calls this via `beforeBuildCommand`.
- `npm run lint`
- `cargo check --manifest-path src-tauri/Cargo.toml` is the fastest Rust validation.
- `cargo test --manifest-path src-tauri/Cargo.toml <test_name> -- --exact` is the focused test path. There is no frontend test runner configured.

## Repo Gotchas
- User-facing text is i18n-backed. When adding or changing UI copy, update `src/i18n/en.json`, `src/i18n/zh.json`, and usually `src/i18n/zh-TW.json`.
- `npm run release:prepare -- <patch|minor|major|x.y.z> [--dry-run]` updates `package.json`, `src-tauri/tauri.conf.json`, `CHANGELOG.md`, `CHANGELOG-zh.md`, and only the version text in `src/i18n/en.json` plus `src/i18n/zh.json`. `src/i18n/zh-TW.json` is not auto-bumped.
- Release builds come from Git tags matching `v*`; the workflow extracts the current version section from both changelog files.
- Built-in tool definitions live in `src-tauri/src/core/tool_adapters.rs`. If you add or change one, also check project scanning in `src-tauri/src/commands/projects.rs`; multiple agents can intentionally share the same `relative_skills_dir`.
- In `tool_adapters`, custom tools and absolute path overrides intentionally count as `installed`; do not “fix” that without a product decision.
- Close/tray behavior is event-driven: Rust emits `window-close-requested`, `tray-scenario-switched`, and `app-files-changed`; the frontend listeners live in `src/components/CloseActionGuard.tsx` and `src/context/AppContext.tsx`.
- There is no formatter config in the repo. Match the surrounding file style instead of introducing one.
