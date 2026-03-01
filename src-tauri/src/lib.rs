use std::sync::Arc;

mod commands;
mod core;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Ensure central repo exists
    core::central_repo::ensure_central_repo().expect("Failed to create central repo");

    // Initialize database
    let db_path = core::central_repo::db_path();
    let store = Arc::new(
        core::skill_store::SkillStore::new(&db_path).expect("Failed to initialize database"),
    );

    tauri::Builder::default()
        .manage(store)
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Tools
            commands::tools::get_tool_status,
            // Skills
            commands::skills::get_managed_skills,
            commands::skills::get_skills_for_scenario,
            commands::skills::get_skill_document,
            commands::skills::delete_managed_skill,
            commands::skills::install_local,
            commands::skills::install_git,
            commands::skills::install_from_skillssh,
            // Sync
            commands::sync::sync_skill_to_tool,
            commands::sync::unsync_skill_from_tool,
            // Scan
            commands::scan::scan_local_skills,
            commands::scan::import_existing_skill,
            commands::scan::import_all_discovered,
            // Browse
            commands::browse::fetch_leaderboard,
            commands::browse::search_skillssh,
            // Settings
            commands::settings::get_settings,
            commands::settings::set_settings,
            // Scenarios
            commands::scenarios::get_scenarios,
            commands::scenarios::get_active_scenario,
            commands::scenarios::create_scenario,
            commands::scenarios::update_scenario,
            commands::scenarios::delete_scenario,
            commands::scenarios::switch_scenario,
            commands::scenarios::add_skill_to_scenario,
            commands::scenarios::remove_skill_from_scenario,
            commands::scenarios::reorder_scenarios,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
