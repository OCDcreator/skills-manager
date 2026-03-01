use std::sync::Arc;
use tauri::State;

use crate::core::skill_store::SkillStore;

#[tauri::command]
pub fn get_settings(key: String, store: State<'_, Arc<SkillStore>>) -> Result<Option<String>, String> {
    store.get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_settings(
    key: String,
    value: String,
    store: State<'_, Arc<SkillStore>>,
) -> Result<(), String> {
    store.set_setting(&key, &value).map_err(|e| e.to_string())
}
