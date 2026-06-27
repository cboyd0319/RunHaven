use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::plans::{
    AgentRunPlan, AuthScope, NetworkMode, RunOptions, WorkspaceScope, build_run_plan,
    default_network_mode,
};
use crate::profiles::AgentProfile;
use crate::shell;

pub(crate) const CONFIRM_PHRASE: &str = "run";

#[derive(Debug)]
pub(crate) struct LauncherState {
    pub(crate) workspace: PathBuf,
    pub(crate) workspace_picker: WorkspacePicker,
    pub(crate) review: Option<PlanReview>,
    pub(crate) plan_error: Option<String>,
    pub(crate) confirm_input: String,
}

impl LauncherState {
    pub(crate) fn new(initial_workspace: PathBuf) -> Self {
        let workspace = canonicalize_or_self(initial_workspace);
        Self {
            workspace: workspace.clone(),
            workspace_picker: WorkspacePicker::new(workspace),
            review: None,
            plan_error: None,
            confirm_input: String::new(),
        }
    }

    pub(crate) fn open_workspace_picker(&mut self) {
        self.workspace_picker = WorkspacePicker::new(self.workspace.clone());
    }

    pub(crate) fn confirm_workspace_selection(&mut self) -> Result<()> {
        self.workspace = self.workspace_picker.selected_path()?;
        self.review = None;
        self.plan_error = None;
        self.confirm_input.clear();
        Ok(())
    }

    pub(crate) fn build_review(&mut self, profile: &AgentProfile) {
        match build_plan_review(profile, &self.workspace) {
            Ok(review) => {
                self.review = Some(review);
                self.plan_error = None;
            }
            Err(error) => {
                self.review = None;
                self.plan_error = Some(error.to_string());
            }
        }
        self.confirm_input.clear();
    }

    pub(crate) fn confirm_ready(&self) -> bool {
        self.review.as_ref().is_some_and(|review| {
            !review.requires_typed_confirm || self.confirm_input == CONFIRM_PHRASE
        })
    }

    pub(crate) fn launch_plan(&self) -> Option<AgentRunPlan> {
        self.review.as_ref().map(|review| review.plan.clone())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PlanReview {
    pub(crate) plan: AgentRunPlan,
    pub(crate) cli_command: String,
    pub(crate) requires_typed_confirm: bool,
}

pub(crate) fn build_plan_review(profile: &AgentProfile, workspace: &Path) -> Result<PlanReview> {
    let network = default_network_mode(profile);
    let plan = build_run_plan(RunOptions {
        profile: profile.clone(),
        workspace: workspace.to_path_buf(),
        agent_args: Vec::new(),
        image: None,
        cpus: "4".to_string(),
        memory: "4g".to_string(),
        network,
        workspace_scope: WorkspaceScope::Current,
        session: None,
        auth_scope: AuthScope::Agent,
        read_only_workspace: false,
        ssh: false,
        env: Vec::new(),
        user: "agent".to_string(),
        interactive: true,
        tty: true,
        allow_sensitive_workspace: false,
        allow_root_user: false,
        provider_hosts: Vec::new(),
        api_key_broker_env: None,
        worktree: None,
        run_id: None,
    })?;
    let cli_command = cli_command_for_plan(profile.name, &plan, network);
    let requires_typed_confirm = !plan.security_notices.is_empty();

    Ok(PlanReview {
        plan,
        cli_command,
        requires_typed_confirm,
    })
}

fn cli_command_for_plan(agent: &str, plan: &AgentRunPlan, network: NetworkMode) -> String {
    shell::join(&[
        "runhaven".to_string(),
        "run".to_string(),
        agent.to_string(),
        "--workspace".to_string(),
        plan.workspace.display().to_string(),
        "--workspace-scope".to_string(),
        WorkspaceScope::Current.as_str().to_string(),
        "--auth-scope".to_string(),
        AuthScope::Agent.as_str().to_string(),
        "--network".to_string(),
        network.as_str().to_string(),
        "--tty".to_string(),
        "always".to_string(),
    ])
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorkspaceCandidate {
    pub(crate) label: String,
    pub(crate) detail: String,
    pub(crate) path: PathBuf,
}

#[derive(Debug)]
pub(crate) struct WorkspacePicker {
    root: PathBuf,
    query: String,
    candidates: Vec<WorkspaceCandidate>,
    matches: Vec<usize>,
    selected: usize,
}

impl WorkspacePicker {
    pub(crate) fn new(root: PathBuf) -> Self {
        let root = canonicalize_or_self(root);
        let candidates = workspace_candidates(&root);
        let mut picker = Self {
            root,
            query: String::new(),
            candidates,
            matches: Vec::new(),
            selected: 0,
        };
        picker.refresh_matches();
        picker
    }

    pub(crate) fn query(&self) -> &str {
        &self.query
    }

    pub(crate) fn push_query_char(&mut self, ch: char) {
        if ch.is_control() {
            return;
        }
        self.query.push(ch);
        self.refresh_matches();
    }

    pub(crate) fn pop_query_char(&mut self) {
        self.query.pop();
        self.refresh_matches();
    }

    pub(crate) fn select_next(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        self.selected = (self.selected + 1).min(self.matches.len() - 1);
    }

    pub(crate) fn select_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub(crate) fn visible_candidates(&self) -> impl Iterator<Item = (usize, &WorkspaceCandidate)> {
        self.matches
            .iter()
            .enumerate()
            .filter_map(|(visible_index, candidate_index)| {
                self.candidates
                    .get(*candidate_index)
                    .map(|candidate| (visible_index, candidate))
            })
    }

    pub(crate) fn selected_visible_index(&self) -> usize {
        self.selected
    }

    pub(crate) fn selected_path(&self) -> Result<PathBuf> {
        if let Some(candidate) = self.selected_candidate() {
            return Ok(candidate.path.clone());
        }
        let typed = self.typed_path();
        if typed.is_dir() {
            return typed.canonicalize().map_err(|error| {
                anyhow::anyhow!("could not resolve {}: {error}", typed.display())
            });
        }
        bail!(
            "workspace path does not exist or is not a directory: {}",
            typed.display()
        )
    }

    fn selected_candidate(&self) -> Option<&WorkspaceCandidate> {
        self.matches
            .get(self.selected)
            .and_then(|index| self.candidates.get(*index))
    }

    fn typed_path(&self) -> PathBuf {
        expand_workspace_query(&self.query, &self.root)
    }

    fn refresh_matches(&mut self) {
        let query = self.query.trim();
        let mut scored = self
            .candidates
            .iter()
            .enumerate()
            .filter_map(|(index, candidate)| {
                fuzzy_score(candidate, query).map(|score| (score, candidate.label.clone(), index))
            })
            .collect::<Vec<_>>();
        scored.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
        self.matches = scored.into_iter().map(|(_, _, index)| index).collect();
        self.selected = self.selected.min(self.matches.len().saturating_sub(1));
    }
}

fn workspace_candidates(root: &Path) -> Vec<WorkspaceCandidate> {
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();
    push_candidate(&mut candidates, &mut seen, "Current folder", root);

    if let Some(parent) = root.parent() {
        push_candidate(&mut candidates, &mut seen, "Parent folder", parent);
    }

    if let Ok(entries) = fs::read_dir(root) {
        let mut children = entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let file_type = entry.file_type().ok()?;
                file_type.is_dir().then_some(entry)
            })
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(is_project_candidate_name)
            })
            .collect::<Vec<_>>();
        children.sort_by_key(|entry| entry.file_name().to_string_lossy().to_ascii_lowercase());

        for child in children {
            let label = child.file_name().to_string_lossy().to_string();
            push_candidate(&mut candidates, &mut seen, &label, &child.path());
        }
    }

    candidates
}

fn push_candidate(
    candidates: &mut Vec<WorkspaceCandidate>,
    seen: &mut HashSet<PathBuf>,
    label: &str,
    path: &Path,
) {
    let path = canonicalize_or_self(path.to_path_buf());
    if !seen.insert(path.clone()) {
        return;
    }
    candidates.push(WorkspaceCandidate {
        label: label.to_string(),
        detail: path.display().to_string(),
        path,
    });
}

fn is_project_candidate_name(name: &str) -> bool {
    !name.starts_with('.')
        && !matches!(
            name,
            "node_modules" | "target" | "dist" | "build" | "__pycache__"
        )
}

fn fuzzy_score(candidate: &WorkspaceCandidate, query: &str) -> Option<usize> {
    if query.is_empty() {
        return Some(0);
    }
    let query = query.to_ascii_lowercase();
    let label = candidate.label.to_ascii_lowercase();
    if let Some(index) = label.find(&query) {
        return Some(index);
    }
    let path_like_query = query.contains('/')
        || query.contains('\\')
        || query.starts_with('.')
        || query.starts_with('~');
    if path_like_query {
        let detail = candidate.detail.to_ascii_lowercase();
        if let Some(index) = detail.find(&query) {
            return Some(100 + index);
        }
        return subsequence_score(&detail, &query);
    }
    subsequence_score(&label, &query)
}

fn subsequence_score(haystack: &str, needle: &str) -> Option<usize> {
    let mut score = 200;
    let mut chars = needle.chars();
    let mut wanted = chars.next()?;
    for (index, ch) in haystack.chars().enumerate() {
        if ch == wanted {
            score += index;
            if let Some(next) = chars.next() {
                wanted = next;
            } else {
                return Some(score);
            }
        }
    }
    None
}

fn expand_workspace_query(query: &str, root: &Path) -> PathBuf {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return root.to_path_buf();
    }
    if trimmed == "~" {
        return home_dir().unwrap_or_else(|| root.to_path_buf());
    }
    if let Some(rest) = trimmed.strip_prefix("~/")
        && let Some(home) = home_dir()
    {
        return home.join(rest);
    }
    let path = PathBuf::from(trimmed);
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn canonicalize_or_self(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::profiles::get_profile;

    use super::*;

    #[test]
    fn picker_filters_child_directories_with_fuzzy_query() {
        let root = tempdir().unwrap();
        fs::create_dir(root.path().join("alpha-project")).unwrap();
        fs::create_dir(root.path().join("beta")).unwrap();

        let mut picker = WorkspacePicker::new(root.path().to_path_buf());
        picker.push_query_char('a');
        picker.push_query_char('p');

        let labels = picker
            .visible_candidates()
            .map(|(_, candidate)| candidate.label.as_str())
            .collect::<Vec<_>>();
        assert!(labels.contains(&"alpha-project"));
        assert!(!labels.contains(&"beta"));
    }

    #[test]
    fn typed_relative_path_can_select_nested_directory() {
        let root = tempdir().unwrap();
        fs::create_dir(root.path().join("packages")).unwrap();
        fs::create_dir(root.path().join("packages/app")).unwrap();

        let mut picker = WorkspacePicker::new(root.path().to_path_buf());
        for ch in "packages/app".chars() {
            picker.push_query_char(ch);
        }

        assert_eq!(
            picker.selected_path().unwrap(),
            root.path().join("packages/app").canonicalize().unwrap()
        );
    }

    #[test]
    fn plan_review_uses_shared_planner_defaults() {
        let workspace = tempdir().unwrap();
        let profile = get_profile("codex").unwrap();

        let review = build_plan_review(&profile, workspace.path()).unwrap();

        assert_eq!(
            review.plan.workspace,
            workspace.path().canonicalize().unwrap()
        );
        assert_eq!(review.plan.network_mode, NetworkMode::Provider);
        assert!(
            review
                .plan
                .provider_allowed_hosts
                .contains(&"api.openai.com".to_string())
        );
        assert!(review.cli_command.contains("runhaven run codex"));
        assert!(review.cli_command.contains("--workspace"));
        assert!(!review.requires_typed_confirm);
    }

    #[test]
    fn lower_security_plan_requires_typed_confirmation() {
        let workspace = tempdir().unwrap();
        let profile = get_profile("shell").unwrap();

        let review = build_plan_review(&profile, workspace.path()).unwrap();

        assert_eq!(review.plan.network_mode, NetworkMode::Internet);
        assert!(!review.plan.security_notices.is_empty());
        assert!(review.requires_typed_confirm);
    }
}
