use crate::core::{
    error::AppError,
    my_skills_repo::{self, MySkillsWorkspaceAction, MySkillsWorkspaceActionResult, MySkillsWorkspaceStatus},
    skill_store::SkillStore,
};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_my_skills_workspace_status(
    store: State<'_, Arc<SkillStore>>,
) -> Result<MySkillsWorkspaceStatus, AppError> {
    let store = store.inner().clone();
    tokio::task::spawn_blocking(move || {
        my_skills_repo::workspace_status(&store).map_err(AppError::internal)
    })
    .await?
}

#[tauri::command]
pub async fn run_my_skills_workspace_action(
    action: String,
    store: State<'_, Arc<SkillStore>>,
) -> Result<MySkillsWorkspaceActionResult, AppError> {
    let action = MySkillsWorkspaceAction::from_str(&action)
        .ok_or_else(|| AppError::invalid_input("Unsupported My Skills action"))?;
    let store = store.inner().clone();

    tokio::task::spawn_blocking(move || {
        my_skills_repo::run_workspace_action(&store, action).map_err(AppError::classify_git_error)
    })
    .await?
}
