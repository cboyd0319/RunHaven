//! Path-redacted desktop contracts for saved assistant/project setups.

use std::fs;
use std::path::Path;

use runhaven_core::records::saved_setups::{SavedSetupRecord, read_saved_setups, save_saved_setup};
use runhaven_core::runtime::plans::default_network_mode;
use runhaven_core::runtime::profiles::get_profile;
use runhaven_core::support::validators::validate_run_id;
use runhaven_core::ui_contracts::{project_label, strip_terminal_controls};
use serde::Serialize;

use crate::contracts::RunPlanRequest;

const SAVED_SETUP_UNAVAILABLE: &str = "This saved setup is unavailable.";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SavedSetupSummary {
    pub id: String,
    pub assistant: String,
    pub project: String,
    pub name: String,
    pub available: bool,
    pub pinned: bool,
}

pub(crate) fn save_setup(assistant: &str, project_path: &Path) -> Result<String, String> {
    save_saved_setup(assistant, project_path)
        .map(|record| record.id)
        .map_err(|_| "RunHaven could not save this setup.".to_string())
}

#[tauri::command]
pub(crate) fn pin_saved_setup(assistant: String, project_path: String) -> Result<String, String> {
    save_and_pin_setup(&assistant, Path::new(&project_path))
}

fn save_and_pin_setup(assistant: &str, project_path: &Path) -> Result<String, String> {
    let id = save_setup(assistant, project_path)?;
    update_saved_setup_pin(&id, true)?;
    Ok(id)
}

#[tauri::command]
pub(crate) fn get_saved_setups() -> Result<Vec<SavedSetupSummary>, String> {
    let summaries = read_saved_setups()
        .map_err(|_| "RunHaven could not read saved setups.".to_string())?
        .into_iter()
        .map(saved_setup_summary)
        .collect();
    Ok(summaries)
}

#[tauri::command]
pub(crate) fn select_saved_setup(id: String) -> Result<RunPlanRequest, String> {
    selected_setup_request(&id)
}

fn selected_setup_request(id: &str) -> Result<RunPlanRequest, String> {
    let record = saved_setup(id)?;
    let workspace =
        available_project_path(&record).ok_or_else(|| SAVED_SETUP_UNAVAILABLE.to_string())?;
    let profile =
        get_profile(&record.assistant).map_err(|_| SAVED_SETUP_UNAVAILABLE.to_string())?;
    Ok(RunPlanRequest {
        agent: record.assistant,
        workspace_path: workspace.display().to_string(),
        network_mode: default_network_mode(&profile).as_str().to_string(),
        workspace_scope: "current".to_string(),
        auth_scope: "agent".to_string(),
        session_name: None,
        read_only_workspace: false,
        cpus: "4".to_string(),
        memory: "4g".to_string(),
        provider_hosts: Vec::new(),
        env_names: Vec::new(),
        api_key_broker_env: None,
        image: None,
        allow_sensitive_workspace: false,
        allow_root_user: false,
        user: "agent".to_string(),
        worktree: true,
    })
}

#[tauri::command]
pub(crate) fn set_saved_setup_pinned(id: String, pinned: bool) -> Result<(), String> {
    update_saved_setup_pin(&id, pinned)
}

fn update_saved_setup_pin(id: &str, pinned: bool) -> Result<(), String> {
    validate_run_id(id).map_err(|_| "RunHaven could not update this saved setup.".to_string())?;
    runhaven_core::records::saved_setups::set_saved_setup_pinned(id, pinned)
        .map_err(|_| "RunHaven could not update this saved setup.".to_string())
}

fn saved_setup(id: &str) -> Result<SavedSetupRecord, String> {
    validate_run_id(id).map_err(|_| SAVED_SETUP_UNAVAILABLE.to_string())?;
    read_saved_setups()
        .map_err(|_| SAVED_SETUP_UNAVAILABLE.to_string())?
        .into_iter()
        .find(|record| record.id == id)
        .ok_or_else(|| SAVED_SETUP_UNAVAILABLE.to_string())
}

fn saved_setup_summary(record: SavedSetupRecord) -> SavedSetupSummary {
    let project = project_label(&record.project_path.display().to_string());
    let assistant_label = assistant_label(&record.assistant);
    let available = available_project_path(&record).is_some();
    SavedSetupSummary {
        id: record.id,
        assistant: strip_terminal_controls(&record.assistant),
        name: format!("{assistant_label} · {project}"),
        available,
        project,
        pinned: record.pinned,
    }
}

fn available_project_path(record: &SavedSetupRecord) -> Option<std::path::PathBuf> {
    let current = fs::canonicalize(&record.project_path).ok()?;
    (current == record.project_path && current.is_dir()).then_some(current)
}

fn assistant_label(assistant: &str) -> String {
    let mut chars = strip_terminal_controls(assistant)
        .chars()
        .collect::<Vec<_>>();
    let Some(first) = chars.first_mut() else {
        return "-".to_string();
    };
    first.make_ascii_uppercase();
    chars.into_iter().collect()
}

#[cfg(test)]
use serde_json::Value;

#[cfg(test)]
pub(crate) fn save_saved_setup_for_test(
    assistant: &str,
    project_path: &Path,
) -> Result<String, String> {
    save_setup(assistant, project_path)
}

#[cfg(test)]
pub(crate) fn pin_saved_setup_for_test(
    assistant: &str,
    project_path: &Path,
) -> Result<String, String> {
    save_and_pin_setup(assistant, project_path)
}

#[cfg(test)]
pub(crate) fn list_saved_setups_for_test() -> Result<Vec<Value>, String> {
    get_saved_setups()?
        .into_iter()
        .map(|summary| serde_json::to_value(summary).map_err(|error| error.to_string()))
        .collect()
}

#[cfg(test)]
pub(crate) fn select_saved_setup_for_test(id: &str) -> Result<RunPlanRequest, String> {
    selected_setup_request(id)
}

#[cfg(test)]
pub(crate) fn set_saved_setup_pinned_for_test(id: &str, pinned: bool) -> Result<(), String> {
    update_saved_setup_pin(id, pinned)
}
