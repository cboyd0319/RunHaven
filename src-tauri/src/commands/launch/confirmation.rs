use runhaven_core::runtime::plans::{AgentRunPlan, NetworkMode};
use runhaven_core::runtime::profiles::get_profile;
use runhaven_core::ui_contracts::strip_terminal_controls;
use tauri::AppHandle;
use tauri_plugin_dialog::MessageDialogKind;

use crate::commands::dialog::confirm_native;
use crate::contracts::{PlanWarning, RunPlanRequest};

const OFFLINE_ASSISTANT_CONSEQUENCE: &str = "The assistant can’t connect while Local only is selected. Choose Assistant services only to let it connect safely, or continue if you want it offline.";

pub(super) async fn trusted_launch_confirmed(
    app: AppHandle,
    plan: &AgentRunPlan,
    warnings: &[PlanWarning],
    worktree: bool,
    request: &RunPlanRequest,
) -> Result<bool, String> {
    let message = trusted_launch_message(plan, warnings, worktree, request);
    let kind = trusted_launch_kind(plan, warnings);
    confirm_native(app, &trusted_launch_title(plan), "Start", kind, message).await
}

fn trusted_launch_kind(plan: &AgentRunPlan, warnings: &[PlanWarning]) -> MessageDialogKind {
    if warnings.is_empty() && !offline_agent_warning_applies(plan) {
        MessageDialogKind::Info
    } else {
        MessageDialogKind::Warning
    }
}

fn offline_agent_warning_applies(plan: &AgentRunPlan) -> bool {
    plan.network_mode == NetworkMode::Internal
        && get_profile(&plan.profile_name).is_ok_and(|profile| !profile.provider_hosts.is_empty())
}

fn trusted_launch_title(plan: &AgentRunPlan) -> String {
    format!("Start {}?", trusted_agent_label(&plan.profile_name))
}

fn trusted_agent_label(profile: &str) -> &str {
    match profile {
        "claude" => "Claude",
        "codex" => "Codex",
        "antigravity" => "Antigravity",
        "copilot" => "Copilot",
        other => other,
    }
}

fn trusted_launch_message(
    plan: &AgentRunPlan,
    warnings: &[PlanWarning],
    worktree: bool,
    request: &RunPlanRequest,
) -> String {
    let project = plan
        .workspace
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(|name| trusted_dialog_value(name, "Selected folder"))
        .unwrap_or_else(|| "Selected folder".to_string());
    let network = match plan.network_mode {
        NetworkMode::Provider => "Assistant services only",
        NetworkMode::Internal => "Local only",
        NetworkMode::Internet => "Full internet",
    };
    let file_access = if request.read_only_workspace {
        if worktree {
            "The assistant can read the protected copy but can’t change it."
        } else {
            "The assistant can read this folder but can’t change it."
        }
    } else if worktree {
        "RunHaven will let the assistant change a protected copy, not your original folder."
    } else {
        "The assistant can change files in this folder."
    };
    let mut message = format!("Project: {project}\nInternet: {network}\n{file_access}");
    if offline_agent_warning_applies(plan) {
        message.push_str("\n\n");
        message.push_str(OFFLINE_ASSISTANT_CONSEQUENCE);
    }
    if !warnings.is_empty() {
        message.push_str("\n\nWarnings:");
        for warning in warnings {
            message.push_str("\n- ");
            message.push_str(&warning.message);
        }
    }
    let mut advanced_access = Vec::new();
    if plan.network_mode == NetworkMode::Provider && !request.provider_hosts.is_empty() {
        advanced_access.push(format!(
            "Extra allowed sites: {}",
            trusted_dialog_list(&request.provider_hosts)
        ));
    }
    if !request.env_names.is_empty() {
        advanced_access.push(format!(
            "Mac environment names: {}",
            trusted_dialog_list(&request.env_names)
        ));
    }
    if let Some(name) = plan.api_key_broker_env_name() {
        advanced_access.push(format!(
            "API key source: {}",
            trusted_dialog_value(name, "Invalid name")
        ));
    }
    if request
        .image
        .as_deref()
        .is_some_and(|image| !image.trim().is_empty())
    {
        advanced_access.push(format!(
            "Custom assistant image: {}",
            trusted_dialog_value(&plan.image, "Invalid image")
        ));
    }
    if !advanced_access.is_empty() {
        message.push_str("\n\nAdvanced access:");
        for access in advanced_access {
            message.push_str("\n- ");
            message.push_str(&access);
        }
    }
    message
}

fn trusted_dialog_list(values: &[String]) -> String {
    values
        .iter()
        .map(|value| trusted_dialog_value(value, "Invalid value"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Keep untrusted filesystem and request labels on one short line in the
/// trusted native sheet. macOS permits newlines in filenames, so using the raw
/// basename would let a project impersonate an Internet or warning row.
fn trusted_dialog_value(value: &str, fallback: &str) -> String {
    let controls_as_spaces = value
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect::<String>();
    let one_line = strip_terminal_controls(&controls_as_spaces)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if one_line.is_empty() {
        return fallback.to_string();
    }
    one_line.chars().take(120).collect()
}

#[cfg(test)]
pub(crate) fn trusted_launch_message_for_test(
    plan: &AgentRunPlan,
    warnings: &[PlanWarning],
    worktree: bool,
    request: &RunPlanRequest,
) -> String {
    trusted_launch_message(plan, warnings, worktree, request)
}

#[cfg(test)]
pub(crate) fn trusted_launch_title_for_test(plan: &AgentRunPlan) -> String {
    trusted_launch_title(plan)
}

#[cfg(test)]
pub(crate) fn trusted_launch_kind_for_test(
    plan: &AgentRunPlan,
    warnings: &[PlanWarning],
) -> MessageDialogKind {
    trusted_launch_kind(plan, warnings)
}
