use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::State;
use walkdir::WalkDir;

use crate::core::{git_fetcher, installer, skill_store::SkillStore};

#[derive(Debug, Serialize)]
pub struct ManagedSkillDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub source_type: String,
    pub source_ref: Option<String>,
    pub central_path: String,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub status: String,
    pub targets: Vec<TargetDto>,
    pub scenario_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TargetDto {
    pub id: String,
    pub skill_id: String,
    pub tool: String,
    pub target_path: String,
    pub mode: String,
    pub status: String,
    pub synced_at: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SkillDocumentDto {
    pub skill_id: String,
    pub filename: String,
    pub content: String,
    pub central_path: String,
}

#[tauri::command]
pub fn get_managed_skills(store: State<'_, Arc<SkillStore>>) -> Result<Vec<ManagedSkillDto>, String> {
    let skills = store.get_all_skills().map_err(|e| e.to_string())?;
    let all_targets = store.get_all_targets().map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for skill in skills {
        let targets: Vec<TargetDto> = all_targets
            .iter()
            .filter(|t| t.skill_id == skill.id)
            .map(|t| TargetDto {
                id: t.id.clone(),
                skill_id: t.skill_id.clone(),
                tool: t.tool.clone(),
                target_path: t.target_path.clone(),
                mode: t.mode.clone(),
                status: t.status.clone(),
                synced_at: t.synced_at,
            })
            .collect();

        let scenario_ids = store
            .get_scenarios_for_skill(&skill.id)
            .unwrap_or_default();

        result.push(ManagedSkillDto {
            id: skill.id,
            name: skill.name,
            description: skill.description,
            source_type: skill.source_type,
            source_ref: skill.source_ref,
            central_path: skill.central_path,
            enabled: skill.enabled,
            created_at: skill.created_at,
            updated_at: skill.updated_at,
            status: skill.status,
            targets,
            scenario_ids,
        });
    }
    Ok(result)
}

#[tauri::command]
pub fn get_skills_for_scenario(
    scenario_id: String,
    store: State<'_, Arc<SkillStore>>,
) -> Result<Vec<ManagedSkillDto>, String> {
    let skills = store
        .get_skills_for_scenario(&scenario_id)
        .map_err(|e| e.to_string())?;
    let all_targets = store.get_all_targets().map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for skill in skills {
        let targets: Vec<TargetDto> = all_targets
            .iter()
            .filter(|t| t.skill_id == skill.id)
            .map(|t| TargetDto {
                id: t.id.clone(),
                skill_id: t.skill_id.clone(),
                tool: t.tool.clone(),
                target_path: t.target_path.clone(),
                mode: t.mode.clone(),
                status: t.status.clone(),
                synced_at: t.synced_at,
            })
            .collect();

        let scenario_ids = store
            .get_scenarios_for_skill(&skill.id)
            .unwrap_or_default();

        result.push(ManagedSkillDto {
            id: skill.id,
            name: skill.name,
            description: skill.description,
            source_type: skill.source_type,
            source_ref: skill.source_ref,
            central_path: skill.central_path,
            enabled: skill.enabled,
            created_at: skill.created_at,
            updated_at: skill.updated_at,
            status: skill.status,
            targets,
            scenario_ids,
        });
    }
    Ok(result)
}

#[tauri::command]
pub fn get_skill_document(
    skill_id: String,
    store: State<'_, Arc<SkillStore>>,
) -> Result<SkillDocumentDto, String> {
    let skill = store
        .get_skill_by_id(&skill_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Skill not found".to_string())?;

    let central = PathBuf::from(&skill.central_path);
    let candidates = [
        "SKILL.md",
        "skill.md",
        "CLAUDE.md",
        "claude.md",
        "README.md",
        "readme.md",
    ];

    // Direct file check
    for name in &candidates {
        let path = central.join(name);
        if path.exists() {
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            return Ok(SkillDocumentDto {
                skill_id,
                filename: name.to_string(),
                content,
                central_path: skill.central_path,
            });
        }
    }

    // Recursive search (max depth 4)
    for entry in WalkDir::new(&central).max_depth(4) {
        if let Ok(e) = entry {
            let fname = e.file_name().to_string_lossy();
            if candidates.contains(&fname.as_ref()) {
                let content = std::fs::read_to_string(e.path()).map_err(|e| e.to_string())?;
                return Ok(SkillDocumentDto {
                    skill_id,
                    filename: fname.to_string(),
                    content,
                    central_path: skill.central_path,
                });
            }
        }
    }

    Err("No documentation file found".to_string())
}

#[tauri::command]
pub fn delete_managed_skill(
    skill_id: String,
    store: State<'_, Arc<SkillStore>>,
) -> Result<(), String> {
    let skill = store
        .get_skill_by_id(&skill_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Skill not found".to_string())?;

    // Remove all sync targets
    let targets = store
        .get_targets_for_skill(&skill_id)
        .map_err(|e| e.to_string())?;
    for target in &targets {
        let target_path = PathBuf::from(&target.target_path);
        crate::core::sync_engine::remove_target(&target_path).ok();
    }

    // Remove central directory
    let central = PathBuf::from(&skill.central_path);
    if central.exists() {
        std::fs::remove_dir_all(&central).ok();
    }

    // Delete from database (cascades to targets and scenario_skills)
    store.delete_skill(&skill_id).map_err(|e| e.to_string())?;

    Ok(())
}

fn store_installed_skill(
    store: &SkillStore,
    result: &installer::InstallResult,
    source_type: &str,
    source_ref: Option<&str>,
    active_scenario_id: Option<&str>,
) -> Result<String, String> {
    let now = chrono::Utc::now().timestamp_millis();
    let id = uuid::Uuid::new_v4().to_string();

    let record = crate::core::skill_store::SkillRecord {
        id: id.clone(),
        name: result.name.clone(),
        description: result.description.clone(),
        source_type: source_type.to_string(),
        source_ref: source_ref.map(|s| s.to_string()),
        source_revision: None,
        central_path: result.central_path.to_string_lossy().to_string(),
        content_hash: Some(result.content_hash.clone()),
        enabled: true,
        created_at: now,
        updated_at: now,
        status: "ok".to_string(),
    };

    store.insert_skill(&record).map_err(|e| e.to_string())?;

    // Auto-add to active scenario if provided
    if let Some(scenario_id) = active_scenario_id {
        store
            .add_skill_to_scenario(scenario_id, &id)
            .map_err(|e| e.to_string())?;
    }

    Ok(id)
}

#[tauri::command]
pub fn install_local(
    source_path: String,
    name: Option<String>,
    store: State<'_, Arc<SkillStore>>,
) -> Result<(), String> {
    let path = PathBuf::from(&source_path);
    let result = installer::install_from_local(&path, name.as_deref()).map_err(|e| e.to_string())?;

    let active = store.get_active_scenario_id().ok().flatten();
    store_installed_skill(&store, &result, "local", Some(&source_path), active.as_deref())?;

    Ok(())
}

#[tauri::command]
pub fn install_git(
    repo_url: String,
    name: Option<String>,
    store: State<'_, Arc<SkillStore>>,
) -> Result<(), String> {
    let temp_dir = git_fetcher::clone_repo(&repo_url).map_err(|e| e.to_string())?;

    // Check for subpath in URL
    let subpath = git_fetcher::extract_subpath(&repo_url);
    let skill_dir = if let Some(ref sub) = subpath {
        let p = temp_dir.join(sub);
        if p.exists() {
            p
        } else {
            git_fetcher::find_skill_dir(&temp_dir, None).map_err(|e| e.to_string())?
        }
    } else {
        git_fetcher::find_skill_dir(&temp_dir, None).map_err(|e| e.to_string())?
    };

    let result =
        installer::install_from_git_dir(&skill_dir, name.as_deref()).map_err(|e| e.to_string())?;

    let active = store.get_active_scenario_id().ok().flatten();
    store_installed_skill(&store, &result, "git", Some(&repo_url), active.as_deref())?;

    git_fetcher::cleanup_temp(&temp_dir);
    Ok(())
}

#[tauri::command]
pub fn install_from_skillssh(
    source: String,
    skill_id: String,
    store: State<'_, Arc<SkillStore>>,
) -> Result<(), String> {
    let repo_url = format!("https://github.com/{}.git", source);
    let temp_dir = git_fetcher::clone_repo(&repo_url).map_err(|e| e.to_string())?;

    let skill_dir =
        git_fetcher::find_skill_dir(&temp_dir, Some(&skill_id)).map_err(|e| e.to_string())?;

    let result =
        installer::install_from_git_dir(&skill_dir, Some(&skill_id)).map_err(|e| e.to_string())?;

    let source_ref = format!("{}/{}", source, skill_id);
    let active = store.get_active_scenario_id().ok().flatten();
    store_installed_skill(
        &store,
        &result,
        "skillssh",
        Some(&source_ref),
        active.as_deref(),
    )?;

    git_fetcher::cleanup_temp(&temp_dir);
    Ok(())
}
