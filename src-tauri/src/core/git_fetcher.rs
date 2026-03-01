use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn clone_repo(url: &str) -> Result<PathBuf> {
    let normalized = normalize_url(url);
    let temp_dir = std::env::temp_dir().join(format!("skills-manager-clone-{}", uuid::Uuid::new_v4()));

    // Try system git first (faster, supports SSH)
    let status = Command::new("git")
        .args(["clone", "--depth", "1", &normalized, &temp_dir.to_string_lossy()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    if let Ok(s) = status {
        if s.success() {
            return Ok(temp_dir);
        }
    }

    // Fallback to git2
    git2::Repository::clone(&normalized, &temp_dir)
        .with_context(|| format!("Failed to clone {}", normalized))?;

    Ok(temp_dir)
}

pub fn extract_subpath(url: &str) -> Option<String> {
    // Parse GitHub tree URLs: https://github.com/user/repo/tree/main/skills/my-skill
    let re = regex::Regex::new(r"github\.com/[^/]+/[^/]+/tree/[^/]+/(.+)").ok()?;
    re.captures(url).map(|c| c[1].to_string())
}

fn normalize_url(url: &str) -> String {
    let trimmed = url.trim();

    // Already a full URL
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("git@")
    {
        // Strip tree path for cloning
        if let Some(idx) = trimmed.find("/tree/") {
            return trimmed[..idx].to_string() + ".git";
        }
        return trimmed.to_string();
    }

    // Shorthand: user/repo
    if trimmed.contains('/') && !trimmed.contains(' ') {
        return format!("https://github.com/{}.git", trimmed);
    }

    trimmed.to_string()
}

pub fn find_skill_dir(repo_dir: &Path, skill_id: Option<&str>) -> Result<PathBuf> {
    // If skill_id provided, look for it specifically
    if let Some(id) = skill_id {
        let direct = repo_dir.join(id);
        if direct.exists() && direct.is_dir() {
            return Ok(direct);
        }

        let in_skills = repo_dir.join("skills").join(id);
        if in_skills.exists() && in_skills.is_dir() {
            return Ok(in_skills);
        }

        // Recursive search
        for entry in walkdir::WalkDir::new(repo_dir).max_depth(3) {
            if let Ok(e) = entry {
                if e.file_type().is_dir() && e.file_name().to_string_lossy() == id {
                    return Ok(e.path().to_path_buf());
                }
            }
        }
    }

    // Check if root is a skill
    let has_skill_md = ["SKILL.md", "skill.md", "CLAUDE.md"]
        .iter()
        .any(|f| repo_dir.join(f).exists());
    if has_skill_md {
        return Ok(repo_dir.to_path_buf());
    }

    // Check skills/ subdirectory
    let skills_subdir = repo_dir.join("skills");
    if skills_subdir.is_dir() {
        return Ok(skills_subdir);
    }

    let skill_subdir = repo_dir.join("skill");
    if skill_subdir.is_dir() {
        return Ok(skill_subdir);
    }

    // Default to root
    Ok(repo_dir.to_path_buf())
}

pub fn cleanup_temp(path: &Path) {
    let _ = std::fs::remove_dir_all(path);
}
