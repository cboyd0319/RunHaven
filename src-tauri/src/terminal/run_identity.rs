use runhaven_core::runtime::plans::AgentRunPlan;
use runhaven_core::ui_contracts::{project_label, strip_terminal_controls};
use serde::Serialize;

/// Rust-owned identity returned after an interactive run opens. The WebView
/// uses it to bind the terminal to History without deriving a host path label.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunTerminalOpenResponse {
    pub run_id: String,
    pub label: String,
}

pub(super) fn run_terminal_sidebar_label(plan: &AgentRunPlan) -> String {
    let mut agent = strip_terminal_controls(&plan.profile_name)
        .chars()
        .collect::<Vec<_>>();
    if let Some(first) = agent.first_mut() {
        first.make_ascii_uppercase();
    }
    let agent = if agent.is_empty() {
        "-".to_string()
    } else {
        agent.into_iter().collect()
    };
    let display_workspace = plan
        .worktree
        .as_ref()
        .map(|worktree| &worktree.source_workspace)
        .unwrap_or(&plan.workspace);
    let project = project_label(&display_workspace.to_string_lossy());
    format!("{agent} · {project}")
}
