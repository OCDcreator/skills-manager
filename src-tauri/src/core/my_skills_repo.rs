use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::Serialize;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, OnceLock};
use std::thread;
use walkdir::WalkDir;

use super::{
    content_hash, git_backup, git_fetcher, installer, skill_metadata,
    skill_store::{SkillRecord, SkillStore, SkillTargetRecord},
    sync_engine,
};

pub const WORKSPACE_PATH_SETTING_KEY: &str = "my_skills_workspace_path";
const MY_SKILLS_REPO_SLUG: &str = "ocdcreator/my-skills";
const MY_SKILLS_REPO_URL: &str = "https://github.com/OCDcreator/my-skills";
static ANSI_ESCAPE_RE: OnceLock<Regex> = OnceLock::new();

#[derive(Debug, Clone, Serialize)]
pub struct MySkillsWorkspaceStatus {
    pub available: bool,
    pub configured: bool,
    pub path: Option<String>,
    pub is_repo: bool,
    pub branch: Option<String>,
    pub remote_url: Option<String>,
    pub has_changes: bool,
    pub managed_skill_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MySkillsWorkspaceActionResult {
    pub action: String,
    pub path: String,
    pub refreshed_skills: usize,
    pub status: String,
    pub detail: Option<String>,
    pub branch: Option<String>,
    pub remote_url: Option<String>,
    pub has_changes: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MySkillsWorkspaceLinkImportResult {
    pub path: String,
    pub source_url: String,
    pub runner: String,
    pub status: String,
    pub detail: Option<String>,
    pub refreshed_skills: usize,
    pub imported_skills: usize,
    pub skipped_skills: usize,
    pub imported_names: Vec<String>,
    pub errors: Vec<String>,
    pub branch: Option<String>,
    pub remote_url: Option<String>,
    pub has_changes: bool,
}

#[derive(Debug, Clone)]
pub struct LinkImportPreparation {
    pub source_url: String,
    pub workspace: PathBuf,
    before_paths: BTreeSet<String>,
    before_status: git_backup::GitBackupStatus,
    before_revision: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LinkImportProcessOutput {
    pub success: bool,
    pub text: String,
    pub exit_label: String,
}

#[derive(Debug, Clone)]
pub struct MySkillsTerminalLaunch {
    pub program: String,
    pub args: Vec<String>,
    pub command: String,
    pub command_preview: String,
    pub startup_banner: String,
    pub path: String,
    pub source_url: String,
    pub path_env: Option<OsString>,
}

#[derive(Debug, Clone)]
pub struct LinkImportOutputLine {
    pub stream: &'static str,
    pub line: String,
}

pub type LinkImportOutputHandler = Arc<dyn Fn(LinkImportOutputLine) + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MySkillsWorkspaceAction {
    Pull,
    Push,
    Update,
}

impl MySkillsWorkspaceAction {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "pull" => Some(Self::Pull),
            "push" => Some(Self::Push),
            "update" => Some(Self::Update),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pull => "pull",
            Self::Push => "push",
            Self::Update => "update",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MySkillsWorkspaceSource {
    pub workspace_root: PathBuf,
    pub skill_dir: PathBuf,
    pub revision: String,
}

pub fn workspace_status(store: &SkillStore) -> Result<MySkillsWorkspaceStatus> {
    let (path, configured) = resolve_workspace_path_with_origin(store);
    let managed_skill_count = linked_skill_count(store)?;

    let Some(path) = path else {
        return Ok(MySkillsWorkspaceStatus {
            available: false,
            configured,
            path: None,
            is_repo: false,
            branch: None,
            remote_url: None,
            has_changes: false,
            managed_skill_count,
        });
    };

    let status = git_backup::get_status(&path).unwrap_or(git_backup::GitBackupStatus {
        is_repo: false,
        remote_url: None,
        branch: None,
        has_changes: false,
        ahead: 0,
        behind: 0,
        last_commit: None,
        last_commit_time: None,
        current_snapshot_tag: None,
        restored_from_tag: None,
    });

    Ok(MySkillsWorkspaceStatus {
        available: true,
        configured,
        path: Some(path.to_string_lossy().to_string()),
        is_repo: status.is_repo,
        branch: status.branch,
        remote_url: status.remote_url,
        has_changes: status.has_changes,
        managed_skill_count,
    })
}

pub fn run_workspace_action(
    store: &SkillStore,
    action: MySkillsWorkspaceAction,
) -> Result<MySkillsWorkspaceActionResult> {
    let Some(workspace) = resolve_workspace_path(store) else {
        bail!("My Skills workspace path is not configured");
    };

    ensure_workspace_root(&workspace)?;
    let output = execute_workspace_script(&workspace, action)?;
    let report = parse_workspace_script_output(&output);
    if !output.status.success()
        || matches!(
            report.status,
            WorkspaceScriptStatus::Cancelled | WorkspaceScriptStatus::Error
        )
    {
        bail!(
            "My Skills {} failed: {}",
            action.as_str(),
            report.detail.unwrap_or_else(|| summarize_output(&output))
        );
    }

    let sync_summary = sync_workspace_after_change(store, &workspace)?;
    let status = git_backup::get_status(&workspace)?;

    Ok(MySkillsWorkspaceActionResult {
        action: action.as_str().to_string(),
        path: workspace.to_string_lossy().to_string(),
        refreshed_skills: sync_summary.refreshed_skills,
        status: report.status.as_str().to_string(),
        detail: report.detail,
        branch: status.branch,
        remote_url: status.remote_url,
        has_changes: status.has_changes,
    })
}

pub fn run_link_import(
    store: &SkillStore,
    source_url: &str,
    output_handler: Option<LinkImportOutputHandler>,
) -> Result<MySkillsWorkspaceLinkImportResult> {
    let prepared = prepare_link_import(store, source_url)?;
    let output = execute_workspace_link_import(&prepared, output_handler)?;
    let process_output = LinkImportProcessOutput {
        success: output.status.success(),
        text: combined_output_text(&output),
        exit_label: format!("process exited with {}", output.status),
    };
    finalize_link_import(store, &prepared, &process_output)
}

pub fn prepare_link_import(store: &SkillStore, source_url: &str) -> Result<LinkImportPreparation> {
    let source_url = source_url.trim();
    if source_url.is_empty() {
        bail!("Source URL is required");
    }

    let Some(workspace) = resolve_workspace_path(store) else {
        bail!("My Skills workspace path is not configured");
    };

    ensure_workspace_root(&workspace)?;

    Ok(LinkImportPreparation {
        source_url: source_url.to_string(),
        before_paths: workspace_skill_path_set(&workspace),
        before_status: git_backup::get_status(&workspace)?,
        before_revision: git_fetcher::get_head_revision(&workspace).ok(),
        workspace,
    })
}

pub fn finalize_link_import(
    store: &SkillStore,
    prepared: &LinkImportPreparation,
    output: &LinkImportProcessOutput,
) -> Result<MySkillsWorkspaceLinkImportResult> {
    let report = parse_agent_import_text(&output.text, output.success, &output.exit_label);

    if !report.saw_result_marker {
        bail!(
            "My Skills link import failed: OpenCode finished without the required RESULT line. Output: {}",
            summarize_text(&output.text, &output.exit_label)
        );
    }

    if !output.success || matches!(report.status, AgentImportStatus::Error) {
        bail!(
            "My Skills link import failed: {}",
            report
                .detail
                .clone()
                .unwrap_or_else(|| summarize_text(&output.text, &output.exit_label))
        );
    }

    let after_paths = workspace_skill_path_set(&prepared.workspace);
    let status = git_backup::get_status(&prepared.workspace)?;
    let after_revision = git_fetcher::get_head_revision(&prepared.workspace).ok();
    validate_link_import_result(
        &report,
        &prepared.before_status,
        prepared.before_revision.as_deref(),
        &prepared.before_paths,
        &status,
        after_revision.as_deref(),
        &after_paths,
    )?;

    let sync_summary = sync_workspace_after_change(store, &prepared.workspace)?;
    let import_summary = sync_summary.import_summary;

    Ok(MySkillsWorkspaceLinkImportResult {
        path: prepared.workspace.to_string_lossy().to_string(),
        source_url: prepared.source_url.clone(),
        runner: "OpenCode".to_string(),
        status: report.status.as_str().to_string(),
        detail: report.detail,
        refreshed_skills: sync_summary.refreshed_skills,
        imported_skills: import_summary.imported,
        skipped_skills: import_summary.skipped,
        imported_names: import_summary.imported_names,
        errors: import_summary.errors,
        branch: status.branch,
        remote_url: status.remote_url,
        has_changes: status.has_changes,
    })
}

pub fn build_link_import_terminal_launch(
    prepared: &LinkImportPreparation,
) -> Result<MySkillsTerminalLaunch> {
    let invocation = build_opencode_invocation(&prepared.workspace, &prepared.source_url);
    let command = build_terminal_startup_script(&invocation);
    let command_preview = invocation.command_preview();

    Ok(MySkillsTerminalLaunch {
        program: invocation.program,
        args: invocation.args,
        command,
        command_preview: command_preview.clone(),
        startup_banner: build_terminal_startup_banner(prepared, &command_preview),
        path: prepared.workspace.to_string_lossy().to_string(),
        source_url: prepared.source_url.clone(),
        path_env: invocation.path_override,
    })
}

pub fn resolve_workspace_source(
    store: &SkillStore,
    skill: &SkillRecord,
) -> Result<Option<MySkillsWorkspaceSource>> {
    if !is_my_skills_skill(skill) {
        return Ok(None);
    }

    let Some(subpath) = skill.source_subpath.as_deref() else {
        return Ok(None);
    };
    let Some(workspace_root) = resolve_workspace_path(store) else {
        return Ok(None);
    };
    ensure_workspace_root(&workspace_root)?;

    let skill_dir = workspace_root.join(subpath);
    if !skill_dir.is_dir() || !skill_metadata::is_valid_skill_dir(&skill_dir) {
        return Ok(None);
    }

    let revision = git_fetcher::get_head_revision(&workspace_root)
        .with_context(|| format!("Failed to read My Skills revision from {}", workspace_root.display()))?;

    Ok(Some(MySkillsWorkspaceSource {
        workspace_root,
        skill_dir,
        revision,
    }))
}

pub(crate) fn update_skill_from_workspace_if_available(
    store: &SkillStore,
    skill: &SkillRecord,
) -> Result<Option<bool>> {
    let Some(source) = resolve_workspace_source(store, skill)? else {
        return Ok(None);
    };

    store.update_skill_update_status(&skill.id, "updating")?;
    let update_result = (|| -> Result<bool> {
        let content_changed =
            sync_skill_from_workspace(store, skill, &source.skill_dir, &source.revision)?;
        import_missing_workspace_skills(store, &source.workspace_root)?;
        Ok(content_changed)
    })();

    match update_result {
        Ok(content_changed) => Ok(Some(content_changed)),
        Err(err) => {
            let message = err.to_string();
            let _ = store.update_skill_check_state(
                &skill.id,
                Some(&source.revision),
                "error",
                Some(&message),
            );
            Err(err)
        }
    }
}

pub(crate) fn check_workspace_update_if_available(
    store: &SkillStore,
    skill: &SkillRecord,
) -> Result<bool> {
    let Some(source) = resolve_workspace_source(store, skill)? else {
        return Ok(false);
    };

    import_missing_workspace_skills(store, &source.workspace_root)?;
    let local_hash = content_hash::hash_directory(&source.skill_dir)?;
    let content_changed = skill.content_hash.as_deref() != Some(local_hash.as_str());

    store.update_skill_source_metadata(
        &skill.id,
        skill.source_ref_resolved.as_deref(),
        skill.source_subpath.as_deref(),
        skill.source_branch.as_deref(),
        Some(&source.revision),
    )?;

    store.update_skill_check_state(
        &skill.id,
        Some(&source.revision),
        if content_changed {
            "update_available"
        } else {
            "up_to_date"
        },
        None,
    )?;

    Ok(true)
}

pub fn is_my_skills_collection_root(path: &Path, repo_url: Option<&str>) -> bool {
    if !repo_url.is_some_and(is_my_skills_repo_url) {
        return false;
    }

    if looks_like_workspace_root(path) {
        return true;
    }

    let Some(parent) = path.parent() else {
        return false;
    };
    if !looks_like_workspace_root(parent) {
        return false;
    }

    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("custom") | Some("external")
    )
}

pub fn collect_workspace_skill_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    for folder in ["custom", "external"] {
        let base = if root.file_name().and_then(|name| name.to_str()) == Some(folder) {
            root.to_path_buf()
        } else {
            root.join(folder)
        };
        if !base.is_dir() {
            continue;
        }

        for entry in WalkDir::new(&base)
            .min_depth(1)
            .max_depth(8)
            .into_iter()
            .filter_entry(|entry| entry.file_name().to_string_lossy() != ".git")
            .flatten()
        {
            if entry.file_type().is_dir() && skill_metadata::is_valid_skill_dir(entry.path()) {
                dirs.push(entry.path().to_path_buf());
            }
        }
    }

    dirs.sort_by_key(|dir| path_key(root, dir).unwrap_or_else(|| dir.to_string_lossy().to_string()));
    dirs
}

pub fn path_key(root: &Path, skill_dir: &Path) -> Option<String> {
    let relative = skill_dir.strip_prefix(root).ok()?;
    Some(relative.to_string_lossy().replace('\\', "/"))
}

pub fn is_my_skills_repo_url(value: &str) -> bool {
    normalize_repo_identity(value)
        .map(|normalized| normalized == MY_SKILLS_REPO_SLUG)
        .unwrap_or(false)
}

pub fn is_my_skills_skill(skill: &SkillRecord) -> bool {
    skill
        .source_ref_resolved
        .as_deref()
        .is_some_and(is_my_skills_repo_url)
        || skill
            .source_ref
            .as_deref()
            .is_some_and(is_my_skills_repo_url)
}

pub fn resolve_workspace_path(store: &SkillStore) -> Option<PathBuf> {
    resolve_workspace_path_with_origin(store).0
}

fn resolve_workspace_path_with_origin(store: &SkillStore) -> (Option<PathBuf>, bool) {
    if let Some(raw) = store
        .get_setting(WORKSPACE_PATH_SETTING_KEY)
        .ok()
        .flatten()
        .filter(|value| !value.trim().is_empty())
    {
        let candidate = expand_home_path(raw.trim());
        if ensure_workspace_root(&candidate).is_ok() {
            return (Some(candidate), true);
        }
        return (None, true);
    }

    let detected = auto_detect_workspace_path();
    (detected, false)
}

fn auto_detect_workspace_path() -> Option<PathBuf> {
    if let Ok(value) = std::env::var("MY_SKILLS_REPO_PATH") {
        let candidate = expand_home_path(&value);
        if ensure_workspace_root(&candidate).is_ok() {
            return Some(candidate);
        }
    }

    let mut candidates = Vec::new();
    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join("Desktop").join("Write").join("custom-project").join("my-skills"));
        candidates.push(home.join("Projects").join("my-skills"));
        candidates.push(home.join("my-skills"));
    }

    candidates
        .into_iter()
        .find(|candidate| ensure_workspace_root(candidate).is_ok())
}

fn expand_home_path(value: &str) -> PathBuf {
    if value == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from(value));
    }
    if let Some(rest) = value.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(value)
}

fn ensure_workspace_root(path: &Path) -> Result<()> {
    if !looks_like_workspace_root(path) {
        bail!("{} is not a valid My Skills workspace", path.display());
    }
    Ok(())
}

fn looks_like_workspace_root(path: &Path) -> bool {
    path.is_dir()
        && path.join("custom").is_dir()
        && path.join("external").is_dir()
        && (path.join("update.bat").is_file() || path.join("update.sh").is_file())
        && (path.join("push.bat").is_file() || path.join("push.sh").is_file())
        && (path.join("pull.bat").is_file() || path.join("pull.sh").is_file())
}

fn normalize_repo_identity(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.contains("://") || trimmed.starts_with("git@") {
        let parsed = git_fetcher::parse_git_source(trimmed);
        return github_slug_from_url(&parsed.clone_url);
    }

    github_slug_from_url(trimmed)
}

fn github_slug_from_url(url: &str) -> Option<String> {
    let normalized = url
        .trim()
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .to_ascii_lowercase();

    if let Some(rest) = normalized.strip_prefix("https://github.com/") {
        return Some(rest.to_string());
    }
    if let Some(rest) = normalized.strip_prefix("http://github.com/") {
        return Some(rest.to_string());
    }
    if let Some(rest) = normalized.strip_prefix("git@github.com:") {
        return Some(rest.to_string());
    }
    if normalized.matches('/').count() == 1 && !normalized.contains('\\') {
        return Some(normalized);
    }

    None
}

fn linked_skill_count(store: &SkillStore) -> Result<usize> {
    Ok(store
        .get_all_skills()?
        .into_iter()
        .filter(is_my_skills_skill)
        .count())
}

fn execute_workspace_script(workspace: &Path, action: MySkillsWorkspaceAction) -> Result<Output> {
    let script_path = script_path_for_action(workspace, action)
        .with_context(|| format!("Missing My Skills {} script", action.as_str()))?;
    let needs_confirmation = matches!(action, MySkillsWorkspaceAction::Pull | MySkillsWorkspaceAction::Update);

    let mut command = script_command(&script_path);
    command
        .current_dir(workspace)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().with_context(|| {
        format!(
            "Failed to start My Skills {} script at {}",
            action.as_str(),
            script_path.display()
        )
    })?;

    if let Some(mut stdin) = child.stdin.take() {
        let input = if needs_confirmation {
            b"Y\r\n\r\n\r\n".as_slice()
        } else {
            b"\r\n\r\n".as_slice()
        };
        stdin.write_all(input)?;
        stdin.flush()?;
    }

    child
        .wait_with_output()
        .with_context(|| format!("Failed to finish My Skills {} script", action.as_str()))
}

fn execute_workspace_link_import(
    prepared: &LinkImportPreparation,
    output_handler: Option<LinkImportOutputHandler>,
) -> Result<Output> {
    let invocation = build_opencode_invocation(&prepared.workspace, &prepared.source_url);
    let mut command = opencode_command(&invocation);
    command
        .current_dir(&prepared.workspace)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("NO_COLOR", "1");

    let mut child = command.spawn().with_context(|| {
        format!(
            "Failed to start OpenCode in {}. Make sure `opencode` is installed and configured.",
            prepared.workspace.display()
        )
    })?;

    let stdout_handle = child
        .stdout
        .take()
        .map(|stdout| spawn_output_reader(stdout, "stdout", output_handler.clone()));
    let stderr_handle = child
        .stderr
        .take()
        .map(|stderr| spawn_output_reader(stderr, "stderr", output_handler));

    let status = child
        .wait()
        .with_context(|| {
            format!(
                "Failed to finish OpenCode import in {}",
                prepared.workspace.display()
            )
        })?;
    let stdout = join_output_reader(stdout_handle);
    let stderr = join_output_reader(stderr_handle);

    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

#[derive(Debug, Clone)]
struct OpencodeInvocation {
    program: String,
    args: Vec<String>,
    path_override: Option<OsString>,
}

impl OpencodeInvocation {
    fn command_preview(&self) -> String {
        let mut parts = Vec::with_capacity(self.args.len() + 1);
        parts.push(shell_quote_for_display(&self.program));
        for arg in self.args.iter().take(self.args.len().saturating_sub(1)) {
            parts.push(shell_quote_for_display(arg));
        }
        parts.push("<prompt>".to_string());
        parts.join(" ")
    }

    fn command_for_copy(&self) -> String {
        let mut parts = Vec::with_capacity(self.args.len() + 1);
        parts.push(shell_quote_for_display(&self.program));
        for arg in &self.args {
            parts.push(shell_quote_for_display(arg));
        }
        parts.join(" ")
    }
}

fn build_opencode_invocation(workspace: &Path, source_url: &str) -> OpencodeInvocation {
    let prompt = build_workspace_link_import_prompt(source_url);
    let title = format!("Import {} into my-skills", truncate_title(source_url));
    let mut args = vec![
        "run".to_string(),
        "--dir".to_string(),
        workspace.to_string_lossy().to_string(),
        "--dangerously-skip-permissions".to_string(),
        "--title".to_string(),
        title.clone(),
        prompt.clone(),
    ];

    let program = resolved_opencode_program()
        .unwrap_or_else(|| "opencode".to_string());
    let path_override = build_opencode_path_override();
    args.shrink_to_fit();

    OpencodeInvocation {
        program,
        args,
        path_override,
    }
}

fn spawn_output_reader<R>(
    reader: R,
    stream: &'static str,
    output_handler: Option<LinkImportOutputHandler>,
) -> thread::JoinHandle<Vec<u8>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        let mut collected = Vec::new();

        loop {
            let mut chunk = Vec::new();
            match reader.read_until(b'\n', &mut chunk) {
                Ok(0) => break,
                Ok(_) => {
                    collected.extend_from_slice(&chunk);
                    if let Some(handler) = output_handler.as_ref() {
                        let line = String::from_utf8_lossy(&chunk)
                            .trim_end_matches(['\r', '\n'])
                            .to_string();
                        if !line.trim().is_empty() {
                            handler(LinkImportOutputLine { stream, line });
                        }
                    }
                }
                Err(_) => break,
            }
        }

        collected
    })
}

fn join_output_reader(handle: Option<thread::JoinHandle<Vec<u8>>>) -> Vec<u8> {
    handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default()
}

fn opencode_command(invocation: &OpencodeInvocation) -> Command {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        let mut command = Command::new("cmd");
        command.creation_flags(0x08000000);
        command.arg("/C").arg(&invocation.program);
        for arg in &invocation.args {
            command.arg(arg);
        }
        inject_opencode_search_path(&mut command, invocation.path_override.clone());
        command
    }

    #[cfg(not(target_os = "windows"))]
    {
        let mut command = Command::new(&invocation.program);
        for arg in &invocation.args {
            command.arg(arg);
        }
        inject_opencode_search_path(&mut command, invocation.path_override.clone());
        command
    }
}

fn inject_opencode_search_path(command: &mut Command, path_override: Option<OsString>) {
    if let Some(joined) = path_override {
        command.env("PATH", joined);
    }
}

fn build_opencode_path_override() -> Option<OsString> {
    let extra_paths = opencode_search_paths();
    if extra_paths.is_empty() {
        return None;
    }

    let existing = std::env::var_os("PATH").unwrap_or_default();
    let mut paths: Vec<PathBuf> = std::env::split_paths(&existing).collect();
    let mut changed = false;

    for path in extra_paths {
        if path.is_dir() && !paths.iter().any(|existing_path| existing_path == &path) {
            paths.insert(0, path);
            changed = true;
        }
    }

    if !changed {
        return None;
    }

    std::env::join_paths(paths).ok()
}

fn opencode_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            paths.push(PathBuf::from(appdata).join("npm"));
        }
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            paths.push(PathBuf::from(local_app_data).join("pnpm"));
        }
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".bun").join("bin"));
            paths.push(home.join("scoop").join("shims"));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".bun").join("bin"));
            paths.push(home.join(".local").join("bin"));
        }
    }

    paths
}

fn resolved_opencode_program() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        return find_windows_opencode_program().map(|path| path.to_string_lossy().to_string());
    }

    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

#[cfg(target_os = "windows")]
fn find_windows_opencode_program() -> Option<PathBuf> {
    find_windows_opencode_program_in_dirs(&opencode_search_paths())
}

#[cfg(target_os = "windows")]
fn find_windows_opencode_program_in_dirs(paths: &[PathBuf]) -> Option<PathBuf> {
    let candidates = ["opencode.cmd", "opencode.exe", "opencode.bat"];
    for dir in paths {
        for candidate in candidates {
            let full_path = dir.join(candidate);
            if full_path.is_file() {
                return Some(full_path);
            }
        }
    }
    None
}

fn script_path_for_action(workspace: &Path, action: MySkillsWorkspaceAction) -> Option<PathBuf> {
    let base = action.as_str();
    let bat = workspace.join(format!("{base}.bat"));
    if cfg!(target_os = "windows") && bat.is_file() {
        return Some(bat);
    }

    let sh = workspace.join(format!("{base}.sh"));
    if sh.is_file() {
        return Some(sh);
    }

    if bat.is_file() {
        Some(bat)
    } else {
        None
    }
}

fn script_command(script_path: &Path) -> Command {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        let mut command = Command::new("cmd");
        command.creation_flags(0x08000000);
        command.arg("/C").arg(script_path);
        command
    }

    #[cfg(not(target_os = "windows"))]
    {
        if script_path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("sh"))
        {
            let mut command = Command::new("bash");
            command.arg(script_path);
            command
        } else {
            let mut command = Command::new(script_path);
            command
        }
    }
}

fn combined_output_text(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        stdout
    } else if stdout.is_empty() {
        stderr
    } else {
        format!("{stdout}\n{stderr}")
    }
}

fn summarize_output(output: &Output) -> String {
    summarize_text(
        &combined_output_text(output),
        &format!("process exited with {}", output.status),
    )
}

fn summarize_text(text: &str, empty_fallback: &str) -> String {
    let sanitized = strip_ansi_from_text(text).trim().to_string();
    let truncated = sanitized.chars().take(1200).collect::<String>();
    if sanitized.chars().count() > truncated.chars().count() {
        format!("{truncated}...")
    } else if truncated.is_empty() {
        empty_fallback.to_string()
    } else {
        truncated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkspaceScriptStatus {
    Success,
    NoChanges,
    Partial,
    Cancelled,
    Error,
}

impl WorkspaceScriptStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::NoChanges => "no_changes",
            Self::Partial => "partial",
            Self::Cancelled => "cancelled",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone)]
struct WorkspaceScriptReport {
    status: WorkspaceScriptStatus,
    detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentImportStatus {
    Success,
    NoChanges,
    Partial,
    Error,
}

impl AgentImportStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::NoChanges => "no_changes",
            Self::Partial => "partial",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone)]
struct AgentImportReport {
    status: AgentImportStatus,
    detail: Option<String>,
    saw_result_marker: bool,
}

fn parse_workspace_script_output(output: &Output) -> WorkspaceScriptReport {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut status = if output.status.success() {
        WorkspaceScriptStatus::Success
    } else {
        WorkspaceScriptStatus::Error
    };
    let mut detail = None;

    for line in stdout.lines().chain(stderr.lines()) {
        let trimmed = line.trim();
        if let Some(value) = trimmed
            .strip_prefix("结果：")
            .or_else(|| trimmed.strip_prefix("Result:"))
        {
            status = classify_workspace_script_status(value.trim(), status);
        }
        if let Some(value) = trimmed
            .strip_prefix("说明：")
            .or_else(|| trimmed.strip_prefix("Detail:"))
        {
            let value = value.trim();
            if !value.is_empty() {
                detail = Some(value.to_string());
            }
        }
    }

    WorkspaceScriptReport { status, detail }
}

#[cfg(test)]
fn parse_agent_import_output(output: &Output) -> AgentImportReport {
    parse_agent_import_text(
        &combined_output_text(output),
        output.status.success(),
        &format!("process exited with {}", output.status),
    )
}

fn parse_agent_import_text(
    text: &str,
    success: bool,
    empty_fallback: &str,
) -> AgentImportReport {
    let mut status = if success {
        AgentImportStatus::Success
    } else {
        AgentImportStatus::Error
    };
    let mut detail = None;
    let mut saw_result_marker = false;

    for line in text.lines() {
        let sanitized = strip_ansi_from_text(line);
        let trimmed = sanitized.trim();
        if let Some(value) = trimmed.strip_prefix("RESULT:") {
            saw_result_marker = true;
            status = classify_agent_import_status(value.trim(), status);
        }
        if let Some(value) = trimmed.strip_prefix("DETAIL:") {
            let value = value.trim();
            if !value.is_empty() {
                detail = Some(value.to_string());
            }
        }
    }

    if detail.is_none() {
        let summary = summarize_text(text, empty_fallback);
        if !summary.is_empty() {
            detail = Some(summary);
        }
    }

    AgentImportReport {
        status,
        detail,
        saw_result_marker,
    }
}

fn validate_link_import_result(
    report: &AgentImportReport,
    before_status: &git_backup::GitBackupStatus,
    before_revision: Option<&str>,
    before_paths: &BTreeSet<String>,
    after_status: &git_backup::GitBackupStatus,
    after_revision: Option<&str>,
    after_paths: &BTreeSet<String>,
) -> Result<()> {
    let head_changed = before_revision != after_revision;
    let skill_paths_changed = before_paths != after_paths;
    let dirty_state_changed = before_status.has_changes != after_status.has_changes;
    let visible_change = head_changed || skill_paths_changed || dirty_state_changed;

    if matches!(report.status, AgentImportStatus::Success | AgentImportStatus::Partial)
        && !visible_change
    {
        bail!(
            "OpenCode reported success but my-skills did not change. The import was not committed or no files were added."
        );
    }

    if !before_status.has_changes && after_status.has_changes {
        bail!(
            "OpenCode left my-skills with uncommitted changes. Please inspect the workspace before importing."
        );
    }

    if after_status.ahead > before_status.ahead {
        bail!(
            "OpenCode created local commit(s) but did not push them to origin. Please push my-skills before importing."
        );
    }

    if matches!(report.status, AgentImportStatus::Success | AgentImportStatus::Partial)
        && (head_changed || skill_paths_changed)
        && after_status.remote_url.is_none()
    {
        bail!("OpenCode updated my-skills, but no origin remote is configured, so push cannot be verified.");
    }

    Ok(())
}

fn classify_workspace_script_status(
    value: &str,
    fallback: WorkspaceScriptStatus,
) -> WorkspaceScriptStatus {
    let normalized = value.trim().to_ascii_lowercase();

    if normalized.contains("partial") || normalized.contains("部分成功") {
        return WorkspaceScriptStatus::Partial;
    }
    if normalized.contains("cancel") || normalized.contains("取消") {
        return WorkspaceScriptStatus::Cancelled;
    }
    if normalized.contains("no changes")
        || normalized.contains("没有新变化")
        || normalized.contains("没有变更")
    {
        return WorkspaceScriptStatus::NoChanges;
    }
    if normalized.contains("failed")
        || normalized.contains("error")
        || normalized.contains("执行失败")
        || normalized.contains("失败")
    {
        return WorkspaceScriptStatus::Error;
    }
    if normalized.contains("success") || normalized.contains("成功") {
        return WorkspaceScriptStatus::Success;
    }

    fallback
}

fn classify_agent_import_status(value: &str, fallback: AgentImportStatus) -> AgentImportStatus {
    let normalized = value.trim().to_ascii_lowercase();

    if normalized.contains("no changes") || normalized.contains("no_changes") {
        return AgentImportStatus::NoChanges;
    }
    if normalized.contains("partial") {
        return AgentImportStatus::Partial;
    }
    if normalized.contains("error") || normalized.contains("failed") {
        return AgentImportStatus::Error;
    }
    if normalized.contains("success") {
        return AgentImportStatus::Success;
    }

    fallback
}

fn refresh_managed_skills_from_workspace(store: &SkillStore, workspace: &Path) -> Result<usize> {
    let revision = git_fetcher::get_head_revision(workspace)
        .with_context(|| format!("Failed to resolve My Skills revision from {}", workspace.display()))?;
    let skills = store.get_all_skills()?;
    let mut refreshed = 0usize;

    for skill in skills.into_iter().filter(is_my_skills_skill) {
        let Some(subpath) = skill.source_subpath.as_deref() else {
            continue;
        };

        let local_dir = workspace.join(subpath);
        if !local_dir.is_dir() || !skill_metadata::is_valid_skill_dir(&local_dir) {
            continue;
        }

        sync_skill_from_workspace(store, &skill, &local_dir, &revision)?;
        refreshed += 1;
    }

    Ok(refreshed)
}

pub(crate) struct WorkspaceImportSummary {
    imported: usize,
    skipped: usize,
    imported_names: Vec<String>,
    errors: Vec<String>,
}

struct WorkspaceSyncSummary {
    refreshed_skills: usize,
    import_summary: WorkspaceImportSummary,
}

struct WorkspaceInstallMetadata {
    source_type: String,
    source_ref: Option<String>,
    source_ref_resolved: Option<String>,
    source_subpath: Option<String>,
    source_branch: Option<String>,
    source_revision: Option<String>,
    remote_revision: Option<String>,
    update_status: String,
}

fn sync_workspace_after_change(store: &SkillStore, workspace: &Path) -> Result<WorkspaceSyncSummary> {
    Ok(WorkspaceSyncSummary {
        refreshed_skills: refresh_managed_skills_from_workspace(store, workspace)?,
        import_summary: import_missing_workspace_skills(store, workspace)?,
    })
}

fn import_missing_workspace_skills(
    store: &SkillStore,
    workspace: &Path,
) -> Result<WorkspaceImportSummary> {
    let revision = git_fetcher::get_head_revision(workspace)
        .with_context(|| format!("Failed to resolve My Skills revision from {}", workspace.display()))?;
    let active = store.get_active_scenario_id().ok().flatten();
    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut imported_names = Vec::new();
    let mut errors = Vec::new();

    for (dir, subpath) in collect_untracked_workspace_skill_dirs(store, workspace)? {
        let name = skill_metadata::infer_skill_name(&dir);
        match installer::install_from_local(&dir, Some(&name)) {
            Ok(result) => {
                let metadata = WorkspaceInstallMetadata {
                    source_type: "git".to_string(),
                    source_ref: Some(MY_SKILLS_REPO_URL.to_string()),
                    source_ref_resolved: Some(MY_SKILLS_REPO_URL.to_string()),
                    source_subpath: Some(subpath),
                    source_branch: None,
                    source_revision: Some(revision.clone()),
                    remote_revision: Some(revision.clone()),
                    update_status: "up_to_date".to_string(),
                };

                match store_workspace_skill(store, &result, &metadata, active.as_deref()) {
                    Ok(ImportStoreResult::Inserted(name)) => {
                        imported += 1;
                        imported_names.push(name);
                    }
                    Ok(ImportStoreResult::Skipped) => {
                        skipped += 1;
                    }
                    Err(err) => errors.push(format!("{name}: {err}")),
                }
            }
            Err(err) => errors.push(format!("{name}: {err}")),
        }
    }

    Ok(WorkspaceImportSummary {
        imported,
        skipped,
        imported_names,
        errors,
    })
}

fn tracked_workspace_skill_paths(store: &SkillStore) -> Result<BTreeSet<String>> {
    Ok(store
        .get_all_skills()?
        .into_iter()
        .filter(is_my_skills_skill)
        .filter_map(|skill| skill.source_subpath)
        .collect())
}

fn collect_untracked_workspace_skill_dirs(
    store: &SkillStore,
    workspace: &Path,
) -> Result<Vec<(PathBuf, String)>> {
    let tracked_paths = tracked_workspace_skill_paths(store)?;

    Ok(collect_workspace_skill_dirs(workspace)
        .into_iter()
        .filter_map(|dir| {
            let subpath = path_key(workspace, &dir)?;
            (!tracked_paths.contains(&subpath)).then_some((dir, subpath))
        })
        .collect())
}

enum ImportStoreResult {
    Inserted(String),
    Skipped,
}

fn store_workspace_skill(
    store: &SkillStore,
    result: &installer::InstallResult,
    metadata: &WorkspaceInstallMetadata,
    active_scenario_id: Option<&str>,
) -> Result<ImportStoreResult> {
    let prospective_path = PathBuf::from(&result.central_path);
    let prospective_path_str = prospective_path.to_string_lossy().to_string();

    if store
        .get_skill_by_central_path(&prospective_path_str)?
        .is_some()
    {
        return Ok(ImportStoreResult::Skipped);
    }

    let now = chrono::Utc::now().timestamp_millis();
    let id = uuid::Uuid::new_v4().to_string();
    let record = SkillRecord {
        id: id.clone(),
        name: result.name.clone(),
        description: result.description.clone(),
        source_type: metadata.source_type.clone(),
        source_ref: metadata.source_ref.clone(),
        source_ref_resolved: metadata.source_ref_resolved.clone(),
        source_subpath: metadata.source_subpath.clone(),
        source_branch: metadata.source_branch.clone(),
        source_revision: metadata.source_revision.clone(),
        remote_revision: metadata.remote_revision.clone(),
        central_path: prospective_path_str,
        content_hash: Some(result.content_hash.clone()),
        enabled: true,
        created_at: now,
        updated_at: now,
        status: "ok".to_string(),
        update_status: metadata.update_status.clone(),
        last_checked_at: Some(now),
        last_check_error: None,
    };

    store.insert_skill(&record)?;
    if let Some(scenario_id) = active_scenario_id {
        store.add_skill_to_scenario(scenario_id, &id)?;
    }

    Ok(ImportStoreResult::Inserted(result.name.clone()))
}

fn workspace_skill_path_set(workspace: &Path) -> BTreeSet<String> {
    collect_workspace_skill_dirs(workspace)
        .into_iter()
        .filter_map(|dir| path_key(workspace, &dir))
        .collect()
}

fn build_terminal_startup_script(invocation: &OpencodeInvocation) -> String {
    invocation.command_for_copy()
}

fn build_terminal_startup_banner(prepared: &LinkImportPreparation, command_preview: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        format!(
            "PS {}> {}\r\n\r\n",
            prepared.workspace.to_string_lossy(),
            command_preview
        )
    }

    #[cfg(not(target_os = "windows"))]
    {
        format!(
            "{}$ {}\n\n",
            prepared.workspace.to_string_lossy(),
            command_preview
        )
    }
}

fn strip_ansi_from_text(text: &str) -> String {
    ANSI_ESCAPE_RE
        .get_or_init(|| {
            Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]")
                .expect("ANSI escape regex must compile")
        })
        .replace_all(text, "")
        .into_owned()
}

fn shell_quote_for_display(value: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        ps_single_quote(value)
    }

    #[cfg(not(target_os = "windows"))]
    {
        sh_single_quote(value)
    }
}

#[cfg(not(target_os = "windows"))]
fn sh_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn ps_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(not(target_os = "windows"))]
fn unique_heredoc_delimiter(prompt: &str) -> String {
    let mut delimiter = "__OPENCODE_PROMPT__".to_string();
    while prompt.lines().any(|line| line == delimiter) {
        delimiter.push('_');
    }
    delimiter
}

fn build_workspace_link_import_prompt(source_url: &str) -> String {
    format!(
        "你在 my-skills 仓库根目录工作，目标是把这个来源链接纳入仓库：{source_url}\n\n\
请严格按下面流程执行：\n\
1. 先阅读 AGENTS.md 和 README.md，再检查 git status。\n\
2. 如果工作区干净，则先同步最新 origin/main；如果不干净，不要做 destructive reset，保留现有修改并在此基础上继续。\n\
3. 判断这个链接应该作为哪一类内容接入：\n\
   - 上游仓库里存在真实 Skill 目录（包含 SKILL.md）=> 作为 external skill source\n\
   - 主要是参考资料/文档，没有真实 Skill 目录 => 作为 external reference source\n\
   - 如果更适合整理成自定义 Skill => 放到 custom/\n\
4. 严格遵守 AGENTS.md 中的规则更新所有必须文件，尤其是 README.md、update.sh、update.bat，以及存在时的 SKILLS.md。\n\
5. 如果新增的是 external source/reference source，确保 external/<name>/ 中的镜像内容已经落地；必要时运行 update 脚本验证。\n\
6. 不要改动无关文件；保持 Windows 和 shell 脚本计数一致。\n\
7. 完成后自行提交到 main 并 push 到 origin。\n\
8. 最终必须用下面两行收尾：\n\
RESULT: success 或 no_changes 或 partial 或 error\n\
DETAIL: 用一句话说明你做了什么。\n"
    )
}

fn truncate_title(value: &str) -> String {
    let truncated: String = value.chars().take(60).collect();
    if value.chars().count() > truncated.chars().count() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

fn sync_skill_from_workspace(
    store: &SkillStore,
    skill: &SkillRecord,
    local_dir: &Path,
    revision: &str,
) -> Result<bool> {
    let new_hash = content_hash::hash_directory(local_dir)?;
    let content_changed = skill.content_hash.as_deref() != Some(new_hash.as_str());

    if content_changed {
        let staged_path = staged_path_for(&skill.central_path);
        let install_result =
            installer::install_from_local_to_destination(local_dir, Some(&skill.name), &staged_path)?;
        swap_skill_directory(&staged_path, Path::new(&skill.central_path))?;

        store.update_skill_after_reinstall(
            &skill.id,
            &skill.name,
            install_result.description.as_deref(),
            &skill.source_type,
            skill.source_ref.as_deref(),
            skill.source_ref_resolved.as_deref(),
            skill.source_subpath.as_deref(),
            skill.source_branch.as_deref(),
            Some(revision),
            Some(revision),
            Some(&install_result.content_hash),
            "up_to_date",
        )?;
        resync_copy_targets(store, &skill.id)?;
        return Ok(true);
    }

    store.update_skill_source_metadata(
        &skill.id,
        skill.source_ref_resolved.as_deref(),
        skill.source_subpath.as_deref(),
        skill.source_branch.as_deref(),
        Some(revision),
    )?;
    store.update_skill_check_state(&skill.id, Some(revision), "up_to_date", None)?;
    resync_copy_targets(store, &skill.id)?;

    Ok(false)
}

fn staged_path_for(central_path: &str) -> PathBuf {
    let path = PathBuf::from(central_path);
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "skill".to_string());
    path.with_file_name(format!(".{file_name}.staged-{}", uuid::Uuid::new_v4()))
}

fn swap_skill_directory(staged_path: &Path, current_path: &Path) -> Result<()> {
    let backup_path = current_path.with_file_name(format!(
        ".{}.backup-{}",
        current_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "skill".to_string()),
        uuid::Uuid::new_v4()
    ));

    remove_path_if_exists(&backup_path)?;
    if current_path.exists() {
        std::fs::rename(current_path, &backup_path).with_context(|| {
            format!(
                "Failed to move existing skill directory {} to backup",
                current_path.display()
            )
        })?;
    }

    if let Err(err) = std::fs::rename(staged_path, current_path) {
        let _ = remove_path_if_exists(current_path);
        if backup_path.exists() {
            let _ = std::fs::rename(&backup_path, current_path);
        }
        return Err(err).with_context(|| {
            format!(
                "Failed to move staged skill {} into place",
                staged_path.display()
            )
        });
    }

    remove_path_if_exists(&backup_path)?;
    Ok(())
}

fn remove_path_if_exists(path: &Path) -> Result<()> {
    if path.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

fn resync_copy_targets(store: &SkillStore, skill_id: &str) -> Result<()> {
    let skill = store
        .get_skill_by_id(skill_id)?
        .ok_or_else(|| anyhow::anyhow!("Skill not found"))?;
    let source = PathBuf::from(&skill.central_path);
    let targets = store.get_targets_for_skill(skill_id)?;

    for target in targets {
        if target.mode != "copy" {
            continue;
        }

        sync_engine::sync_skill(
            &source,
            Path::new(&target.target_path),
            sync_engine::SyncMode::Copy,
        )?;

        let updated_target = SkillTargetRecord {
            synced_at: Some(chrono::Utc::now().timestamp_millis()),
            status: "ok".to_string(),
            last_error: None,
            ..target
        };
        store.insert_target(&updated_target)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_skill(dir: &Path, name: &str) {
        let skill_dir = dir.join(name);
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), format!("---\nname: {name}\n---\n")).unwrap();
    }

    fn make_store() -> (tempfile::TempDir, SkillStore) {
        let tmp = tempdir().unwrap();
        let db_path = tmp.path().join("skills-manager.db");
        let store = SkillStore::new(&db_path).unwrap();
        (tmp, store)
    }

    fn insert_my_skills_record(store: &SkillStore, root: &Path, subpath: &str, name: &str) {
        let now = chrono::Utc::now().timestamp_millis();
        let record = SkillRecord {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: None,
            source_type: "git".to_string(),
            source_ref: Some(MY_SKILLS_REPO_URL.to_string()),
            source_ref_resolved: Some(MY_SKILLS_REPO_URL.to_string()),
            source_subpath: Some(subpath.to_string()),
            source_branch: None,
            source_revision: Some("rev-1".to_string()),
            remote_revision: Some("rev-1".to_string()),
            central_path: root.join(name).to_string_lossy().to_string(),
            content_hash: None,
            enabled: true,
            created_at: now,
            updated_at: now,
            status: "ok".to_string(),
            update_status: "up_to_date".to_string(),
            last_checked_at: Some(now),
            last_check_error: None,
        };

        store.insert_skill(&record).unwrap();
    }

    #[test]
    fn detects_my_skills_repo_urls() {
        assert!(is_my_skills_repo_url("https://github.com/OCDcreator/my-skills"));
        assert!(is_my_skills_repo_url("https://github.com/OCDcreator/my-skills.git"));
        assert!(is_my_skills_repo_url("git@github.com:OCDcreator/my-skills.git"));
        assert!(is_my_skills_repo_url(
            "https://github.com/OCDcreator/my-skills/tree/main/custom"
        ));
        assert!(!is_my_skills_repo_url("https://github.com/OCDcreator/skills-manager"));
    }

    #[test]
    fn collects_nested_workspace_skills() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("custom").join("group")).unwrap();
        std::fs::create_dir_all(tmp.path().join("external").join("vendor")).unwrap();
        std::fs::write(tmp.path().join("update.sh"), "").unwrap();
        std::fs::write(tmp.path().join("push.sh"), "").unwrap();
        std::fs::write(tmp.path().join("pull.sh"), "").unwrap();

        create_skill(&tmp.path().join("custom"), "skill-a");
        create_skill(&tmp.path().join("custom").join("group"), "skill-b");
        create_skill(&tmp.path().join("external").join("vendor"), "skill-c");

        let paths = collect_workspace_skill_dirs(tmp.path());
        let keys: Vec<String> = paths
            .iter()
            .filter_map(|path| path_key(tmp.path(), path))
            .collect();

        assert_eq!(
            keys,
            vec![
                "custom/group/skill-b".to_string(),
                "custom/skill-a".to_string(),
                "external/vendor/skill-c".to_string()
            ]
        );
    }

    #[test]
    fn detects_workspace_root_structure() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("custom")).unwrap();
        std::fs::create_dir_all(tmp.path().join("external")).unwrap();
        std::fs::write(tmp.path().join("update.sh"), "").unwrap();
        std::fs::write(tmp.path().join("push.sh"), "").unwrap();
        std::fs::write(tmp.path().join("pull.sh"), "").unwrap();

        assert!(looks_like_workspace_root(tmp.path()));
        assert!(is_my_skills_collection_root(
            tmp.path(),
            Some("https://github.com/OCDcreator/my-skills")
        ));
    }

    #[test]
    fn collects_only_untracked_workspace_skill_dirs() {
        let workspace = tempdir().unwrap();
        std::fs::create_dir_all(workspace.path().join("custom")).unwrap();
        std::fs::create_dir_all(workspace.path().join("external")).unwrap();
        std::fs::write(workspace.path().join("update.sh"), "").unwrap();
        std::fs::write(workspace.path().join("push.sh"), "").unwrap();
        std::fs::write(workspace.path().join("pull.sh"), "").unwrap();
        create_skill(&workspace.path().join("custom"), "tracked-skill");
        create_skill(&workspace.path().join("external"), "new-skill");

        let (_store_dir, store) = make_store();
        insert_my_skills_record(&store, workspace.path(), "custom/tracked-skill", "tracked-skill");

        let pending = collect_untracked_workspace_skill_dirs(&store, workspace.path()).unwrap();
        let pending_paths: Vec<String> = pending.into_iter().map(|(_, subpath)| subpath).collect();

        assert_eq!(pending_paths, vec!["external/new-skill".to_string()]);
    }

    #[test]
    fn parses_workspace_script_statuses() {
        let success = Output {
            status: success_status(),
            stdout: "结果：更新并推送成功\n说明：外部资源已同步，变更已提交并推送。\n".into(),
            stderr: Vec::new(),
        };
        let partial = Output {
            status: success_status(),
            stdout: "结果：部分成功，需要查看警告\n说明：已提交并推送可用更新，但有部分来源下载失败。\n".into(),
            stderr: Vec::new(),
        };
        let no_changes = Output {
            status: success_status(),
            stdout: "结果：没有变更，不需要推送\n说明：没有检测到新的变更，所以这次无需提交和推送。\n".into(),
            stderr: Vec::new(),
        };

        let success_report = parse_workspace_script_output(&success);
        let partial_report = parse_workspace_script_output(&partial);
        let no_changes_report = parse_workspace_script_output(&no_changes);

        assert_eq!(success_report.status, WorkspaceScriptStatus::Success);
        assert_eq!(partial_report.status, WorkspaceScriptStatus::Partial);
        assert_eq!(no_changes_report.status, WorkspaceScriptStatus::NoChanges);
        assert_eq!(
            no_changes_report.detail.as_deref(),
            Some("没有检测到新的变更，所以这次无需提交和推送。")
        );
    }

    #[test]
    fn parses_agent_import_result_marker() {
        let output = Output {
            status: success_status(),
            stdout: "RESULT: success\nDETAIL: 已导入并推送。\n".into(),
            stderr: Vec::new(),
        };

        let report = parse_agent_import_output(&output);

        assert!(report.saw_result_marker);
        assert_eq!(report.status, AgentImportStatus::Success);
        assert_eq!(report.detail.as_deref(), Some("已导入并推送。"));
    }

    #[test]
    fn detects_missing_agent_import_result_marker() {
        let output = Output {
            status: success_status(),
            stdout: "OpenCode says it is done.\n".into(),
            stderr: Vec::new(),
        };

        let report = parse_agent_import_output(&output);

        assert!(!report.saw_result_marker);
        assert_eq!(report.status, AgentImportStatus::Success);
        assert_eq!(report.detail.as_deref(), Some("OpenCode says it is done."));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn finds_windows_opencode_wrapper_in_common_dirs() {
        let tmp = tempdir().unwrap();
        let npm_dir = tmp.path().join("npm");
        std::fs::create_dir_all(&npm_dir).unwrap();
        let wrapper = npm_dir.join("opencode.cmd");
        std::fs::write(&wrapper, "@echo off\r\n").unwrap();

        let found = find_windows_opencode_program_in_dirs(&[npm_dir]);

        assert_eq!(found.as_deref(), Some(wrapper.as_path()));
    }

    #[cfg(target_os = "windows")]
    fn success_status() -> std::process::ExitStatus {
        use std::os::windows::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(0)
    }

    #[cfg(unix)]
    fn success_status() -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(0)
    }
}
