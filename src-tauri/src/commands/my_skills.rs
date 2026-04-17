use crate::core::{
    error::AppError,
    my_skills_terminal::{self, MySkillsTerminalManager, MySkillsTerminalState},
    my_skills_repo::{
        self, MySkillsWorkspaceAction, MySkillsWorkspaceActionResult,
        MySkillsWorkspaceLinkImportResult, MySkillsWorkspaceStatus,
    },
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

#[tauri::command]
pub async fn run_my_skills_link_import(
    source_url: String,
    store: State<'_, Arc<SkillStore>>,
    app_handle: tauri::AppHandle,
) -> Result<MySkillsWorkspaceLinkImportResult, AppError> {
    let store = store.inner().clone();

    tokio::task::spawn_blocking(move || {
        use tauri::Emitter;

        let output_handler: my_skills_repo::LinkImportOutputHandler = Arc::new(move |line| {
            app_handle
                .emit(
                    "my-skills-link-import-output",
                    serde_json::json!({
                        "stream": line.stream,
                        "line": line.line,
                    }),
                )
                .ok();
        });

        my_skills_repo::run_link_import(&store, &source_url, Some(output_handler))
            .map_err(AppError::internal)
    })
    .await?
}

#[tauri::command]
pub async fn start_my_skills_link_import_terminal(
    source_url: String,
    store: State<'_, Arc<SkillStore>>,
    terminal_manager: State<'_, Arc<MySkillsTerminalManager>>,
    app_handle: tauri::AppHandle,
) -> Result<MySkillsTerminalState, AppError> {
    let store = store.inner().clone();
    let terminal_manager = terminal_manager.inner().clone();
    tokio::task::spawn_blocking(move || {
        terminal_manager
            .start_link_import(app_handle, store, &source_url)
            .map_err(my_skills_terminal::classify_terminal_error)
    })
    .await?
}

#[tauri::command]
pub fn write_my_skills_terminal_input(
    session_id: String,
    input: String,
    terminal_manager: State<'_, Arc<MySkillsTerminalManager>>,
) -> Result<(), AppError> {
    terminal_manager
        .inner()
        .write_input(&session_id, &input)
        .map_err(my_skills_terminal::classify_terminal_error)
}

#[tauri::command]
pub fn resize_my_skills_terminal(
    session_id: String,
    cols: u16,
    rows: u16,
    terminal_manager: State<'_, Arc<MySkillsTerminalManager>>,
) -> Result<(), AppError> {
    terminal_manager
        .inner()
        .resize(&session_id, cols, rows)
        .map_err(my_skills_terminal::classify_terminal_error)
}

#[tauri::command]
pub fn interrupt_my_skills_terminal(
    session_id: String,
    terminal_manager: State<'_, Arc<MySkillsTerminalManager>>,
) -> Result<(), AppError> {
    terminal_manager
        .inner()
        .interrupt(&session_id)
        .map_err(my_skills_terminal::classify_terminal_error)
}

#[tauri::command]
pub fn close_my_skills_terminal(
    session_id: String,
    terminal_manager: State<'_, Arc<MySkillsTerminalManager>>,
) -> Result<(), AppError> {
    terminal_manager
        .inner()
        .close(&session_id)
        .map_err(my_skills_terminal::classify_terminal_error)
}

#[tauri::command]
pub fn get_my_skills_terminal_status(
    terminal_manager: State<'_, Arc<MySkillsTerminalManager>>,
) -> Result<Option<MySkillsTerminalState>, AppError> {
    Ok(terminal_manager.inner().get_status())
}
