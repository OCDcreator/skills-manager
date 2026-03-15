# CLAUDE.md

## 项目概述

Skills Manager v2 — AI Agent Skills 桌面管理工具，基于 Tauri 2 + React + TypeScript 构建。

## 技术栈

- **前端**: React 19 + TypeScript + Vite
- **后端**: Tauri 2 (Rust)
- **数据库**: SQLite (rusqlite)
- **构建/发布**: GitHub Actions 跨平台自动构建

## 发版流程

使用本地脚本一键升版（自动更新 `package.json`、`src-tauri/tauri.conf.json`、`src/i18n/en.json`、`src/i18n/zh.json`、`CHANGELOG.md`），再手动填写 CHANGELOG 后提交：

```bash
npm run release:prepare -- minor   # 或 patch / major / x.y.z
# 编辑 CHANGELOG.md 填写具体变更内容
git add CHANGELOG.md package.json src-tauri/tauri.conf.json src/i18n/en.json src/i18n/zh.json
git commit -m "chore: bump version to x.y.z"
git tag vx.y.z
git push origin main --tags
```

也可通过 GitHub Actions 手动触发 `Prepare Release` 工作流（自动完成全流程，CHANGELOG 为空模板）。

## CHANGELOG 规范

- 面向用户，描述可感知的功能变更和 bug 修复，不写代码/技术细节
- 使用英文，覆盖上一个 tag 以来的全部变更
- 遵循 [Keep a Changelog](https://keepachangelog.com/) 格式

## 关键路径

| 文件/目录 | 说明 |
|-----------|------|
| `src/` | React 前端源码 |
| `src-tauri/` | Rust 后端源码 |
| `src-tauri/tauri.conf.json` | Tauri 配置（版本号、窗口、打包） |
| `.github/workflows/release.yml` | CI 跨平台构建 workflow |
| `CHANGELOG.md` | 版本变更记录 |
