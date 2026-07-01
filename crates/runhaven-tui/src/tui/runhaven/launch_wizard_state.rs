use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crossterm::event::KeyCode;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;
use runhaven_core::ui_contracts::AgentCatalogItemData;
use runhaven_core::ui_contracts::LaunchPlanData;

use crate::key_hint;
use crate::style::danger_style;
use crate::style::muted_but_readable_style;
use crate::style::safe_style;
use crate::style::warning_style;

use super::super::service::AgentLaunchPreview;
use super::super::service::LaunchPreviewError;
use super::super::service::PreparedLaunch;
use super::super::service::WorkspaceLaunchPreview;
use super::CONFIRM_PHRASE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WorkspaceDecisionVm {
    pub(super) label: String,
    pub(super) description: String,
    pub(super) workspace: PathBuf,
    pub(super) decisions: Arc<Vec<AgentDecisionVm>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AgentDecisionVm {
    pub(super) agent: AgentCatalogItemData,
    pub(super) plan: Result<PreparedLaunch, LaunchPreviewError>,
    pub(super) status_label: String,
    pub(super) auth_scope_label: String,
    pub(super) auth_label: String,
    pub(super) network_label: String,
    pub(super) boundary_label: String,
}

impl From<AgentLaunchPreview> for AgentDecisionVm {
    fn from(preview: AgentLaunchPreview) -> Self {
        let status_label = match &preview.plan {
            Ok(launch) if launch.data.confirm_required => "review".to_string(),
            Ok(_) => "ready".to_string(),
            Err(_) => "blocked".to_string(),
        };
        let auth_scope_label = preview
            .plan
            .as_ref()
            .map(|launch| launch.data.auth_scope.clone())
            .unwrap_or_else(|_| "unknown".to_string());
        let auth_label = match preview.agent.sign_in.as_str() {
            "n/a" => "no sign-in".to_string(),
            sign_in => format!("{sign_in}, {auth_scope_label} state"),
        };
        let network_label = preview.plan.as_ref().map_or_else(
            |_| network_mode_label(&preview.agent.default_network).to_string(),
            |launch| network_label(&launch.data),
        );

        Self {
            agent: preview.agent,
            plan: preview.plan,
            status_label,
            auth_scope_label,
            auth_label,
            network_label,
            boundary_label: "/workspace only".to_string(),
        }
    }
}

impl From<WorkspaceLaunchPreview> for WorkspaceDecisionVm {
    fn from(choice: WorkspaceLaunchPreview) -> Self {
        Self {
            label: choice.label,
            description: choice.description,
            workspace: choice.payload.workspace,
            decisions: Arc::new(
                choice
                    .payload
                    .previews
                    .into_iter()
                    .map(AgentDecisionVm::from)
                    .collect(),
            ),
        }
    }
}

impl AgentDecisionVm {
    pub(super) fn network_style(&self) -> Style {
        if self.network_label.contains("internet") {
            warning_style()
        } else {
            safe_style()
        }
    }

    pub(super) fn status_style(&self) -> Style {
        match self.status_label.as_str() {
            "ready" => safe_style(),
            "review" => warning_style(),
            _ => danger_style(),
        }
    }

    pub(super) fn first_step_network_label(&self) -> &'static str {
        match self.network_label.as_str() {
            "provider allowlist" => "Provider only",
            "local only" => "Local only",
            "internet unrestricted" => "Unrestricted internet",
            _ => "Custom network",
        }
    }
}

pub(super) fn selected_decision<'a>(
    decisions: &'a [AgentDecisionVm],
    selected_idx: &AtomicUsize,
) -> Option<&'a AgentDecisionVm> {
    let selected = selected_idx.load(Ordering::Relaxed);
    decisions.get(selected).or_else(|| decisions.first())
}

pub(super) fn selected_workspace<'a>(
    workspaces: &'a [WorkspaceDecisionVm],
    selected_workspace_idx: &AtomicUsize,
) -> Option<&'a WorkspaceDecisionVm> {
    let selected = selected_workspace_idx.load(Ordering::Relaxed);
    workspaces.get(selected).or_else(|| workspaces.first())
}

pub(super) fn label_value(
    label: &'static str,
    value: impl Into<String>,
    value_style: Style,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<12}"), muted_but_readable_style()),
        Span::styled(value.into(), value_style),
    ])
}

pub(super) fn paragraph(lines: Vec<Line<'static>>) -> Paragraph<'static> {
    Paragraph::new(lines).wrap(Wrap { trim: true })
}

pub(super) fn review_footer_line() -> Line<'static> {
    Line::from(vec![
        key_hint::plain(KeyCode::Char('b')).into(),
        Span::raw(" or "),
        key_hint::plain(KeyCode::Esc).into(),
        Span::raw(" goes back. "),
        key_hint::plain(KeyCode::Enter).into(),
        Span::raw(" opens confirmation. q quits."),
    ])
}

pub(super) fn confirm_footer_line(text_field_active: bool) -> Line<'static> {
    let mut line = Line::from(vec![
        key_hint::plain(KeyCode::Esc).into(),
        Span::raw(" goes back. "),
        key_hint::plain(KeyCode::Enter).into(),
        Span::raw(" confirms."),
    ]);
    if !text_field_active {
        line.push_span(" ");
        line.push_span(key_hint::plain(KeyCode::Char('q')));
        line.push_span(" quits.");
    }
    line
}

pub(super) fn confirmation_matches(value: &str) -> bool {
    value.trim().eq_ignore_ascii_case(CONFIRM_PHRASE)
}

fn network_label(plan: &LaunchPlanData) -> String {
    network_mode_label(&plan.network.mode).to_string()
}

fn network_mode_label(mode: &str) -> &'static str {
    match mode {
        "provider" => "provider allowlist",
        "internal" => "local only",
        "internet" => "internet unrestricted",
        _ => "custom network",
    }
}

pub(super) fn worktree_label(plan: &LaunchPlanData) -> String {
    plan.worktree
        .as_ref()
        .map(|worktree| format!("on, branch {}", worktree.branch))
        .unwrap_or_else(|| "off".to_string())
}

pub(super) fn workspace_title(workspace: &Path) -> String {
    workspace
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("workspace")
        .to_string()
}
