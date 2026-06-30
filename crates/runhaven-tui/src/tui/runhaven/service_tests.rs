use super::*;
use runhaven_core::support::paths::{
    ensure_private_parent, override_cache_root_for_tests, runs_log_path,
};
use std::io::Write;

fn preview<'a>(payload: &'a LaunchPreviewPayload, name: &str) -> &'a AgentLaunchPreview {
    payload
        .previews
        .iter()
        .find(|preview| preview.agent.name == name)
        .unwrap_or_else(|| panic!("missing {name} preview"))
}

fn launch<'a>(preview: &'a AgentLaunchPreview, name: &str) -> &'a PreparedLaunch {
    preview
        .plan
        .as_ref()
        .unwrap_or_else(|error| panic!("{name} plan failed: {error}"))
}

fn plan<'a>(preview: &'a AgentLaunchPreview, name: &str) -> &'a LaunchPlanData {
    &launch(preview, name).data
}

fn run_git(args: &[&str], cwd: &Path) {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_history_payload_omits_workspace_paths() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    ensure_private_parent(&runs_log_path()).expect("runs log parent");
    let mut file = std::fs::File::create(runs_log_path()).expect("runs log file");
    writeln!(
        file,
        "{}",
        serde_json::json!({
            "timestamp": "2026-06-30T01:00:00Z",
            "started_at": "2026-06-30T00:00:00Z",
            "finished_at": "2026-06-30T01:00:00Z",
            "run_id": "run-\u{1b}123",
            "profile": "codex",
            "workspace": "/Users/c/secret/project",
            "workspace_scope": "current",
            "state_volume": "runhaven-codex-shared-home",
            "session": "none",
            "network": "provider",
            "status": "succeeded",
            "return_code": 0,
            "provider_policy": {"allowed": 3, "denied": 1},
            "auth_broker": {"allowed": 2, "denied": 0},
            "cleanup": {"provider_network": "removed"},
            "git": {"available": "false", "reason": "not-a-git-worktree"}
        })
    )
    .expect("write run record");

    let history = RunHavenTuiService::new()
        .run_history_payload(10)
        .expect("run history");
    let encoded = serde_json::to_string(&history).expect("serialize history");

    assert_eq!(history.runs.len(), 1);
    assert_eq!(history.runs[0].run_id, "run-123");
    assert_eq!(history.runs[0].return_code, Some(0));
    assert_eq!(history.runs[0].provider_denied, 1);
    assert!(!encoded.contains("/Users/c/secret/project"));
    assert!(!encoded.contains("\\u001b"));
}

#[test]
fn launch_preview_payload_maps_profiles_to_plan_payloads_by_name() {
    let workspace = tempfile::tempdir().expect("workspace");
    let payload = RunHavenTuiService::new().launch_preview_payload(workspace.path());

    assert_eq!(payload.workspace, workspace.path());
    assert!(payload.previews.len() >= 5);
    for name in [
        "antigravity",
        "claude",
        "codex",
        "copilot",
        "gemini",
        "shell",
    ] {
        let preview = preview(&payload, name);
        let plan = plan(preview, name);
        assert_eq!(preview.agent.name, name);
        assert_eq!(plan.profile_name, name);
        assert_eq!(
            plan.command,
            launch(preview, name).executable.shell_command()
        );
        assert!(!preview.agent.description.trim().is_empty());
        assert!(!preview.agent.image.trim().is_empty());
        assert!(!plan.command.trim().is_empty());
    }
}

#[test]
fn prepared_launch_display_data_is_derived_from_executable_plan() {
    let workspace = tempfile::tempdir().expect("workspace");
    let payload = RunHavenTuiService::new().launch_preview_payload(workspace.path());

    for preview in payload.previews {
        let launch = preview.plan.expect("launch should be prepared");
        assert_eq!(
            launch.data,
            LaunchPlanData::from(&launch.executable),
            "{} display data must stay derived from the executable plan",
            preview.agent.name
        );
    }
}

#[test]
fn launch_preview_payload_uses_default_network_and_agent_auth_scope() {
    let workspace = tempfile::tempdir().expect("workspace");
    let payload = RunHavenTuiService::new().launch_preview_payload(workspace.path());

    let shell = preview(&payload, "shell");
    let shell_plan = plan(shell, "shell");
    assert_eq!(shell.agent.default_network, "internet");
    assert_eq!(shell_plan.network.mode, "internet");
    assert_eq!(shell_plan.auth_scope, "agent");
    assert!(shell_plan.confirm_required);

    let codex = preview(&payload, "codex");
    let codex_plan = plan(codex, "codex");
    assert_eq!(codex.agent.default_network, "provider");
    assert_eq!(codex_plan.network.mode, "provider");
    assert_eq!(codex_plan.auth_scope, "agent");
    assert!(!codex_plan.boundary.not_shared.is_empty());
}

#[test]
fn launch_preview_payload_applies_policy_overrides_to_executable_plan() {
    let workspace = tempfile::tempdir().expect("workspace");
    let payload = RunHavenTuiService::new().launch_preview_payload_with_policy(
        workspace.path(),
        LaunchPolicySelection {
            network: NetworkPolicySelection::Fixed(NetworkMode::Internal),
            auth_scope: AuthScope::Project,
        },
    );

    let codex = preview(&payload, "codex");
    let codex_launch = launch(codex, "codex");

    assert_eq!(codex_launch.data.network.mode, "internal");
    assert_eq!(codex_launch.executable.network_mode, NetworkMode::Internal);
    assert_eq!(codex_launch.data.auth_scope, "project");
    assert_eq!(codex_launch.executable.auth_scope, AuthScope::Project);
    assert_eq!(
        codex_launch.data,
        LaunchPlanData::from(&codex_launch.executable),
        "display policy data must stay derived from executable plan"
    );
}

#[test]
fn provider_policy_override_fails_closed_for_profiles_without_provider_hosts() {
    let workspace = tempfile::tempdir().expect("workspace");
    let payload = RunHavenTuiService::new().launch_preview_payload_with_policy(
        workspace.path(),
        LaunchPolicySelection {
            network: NetworkPolicySelection::Fixed(NetworkMode::Provider),
            auth_scope: AuthScope::Agent,
        },
    );

    let shell = preview(&payload, "shell");
    let error = shell.plan.as_ref().expect_err("shell provider plan");

    assert!(error.detail().contains("provider hosts are required"));
}

#[test]
fn launch_preview_payload_maps_auth_and_provider_metadata() {
    let workspace = tempfile::tempdir().expect("workspace");
    let payload = RunHavenTuiService::new().launch_preview_payload(workspace.path());

    let shell = preview(&payload, "shell");
    assert_eq!(shell.agent.sign_in, "n/a");
    assert_eq!(shell.agent.broker, "n/a");
    assert_eq!(shell.agent.provider_host_count, 0);

    let codex = preview(&payload, "codex");
    let codex_plan = plan(codex, "codex");
    assert_eq!(codex.agent.sign_in, "runhaven login");
    assert_eq!(codex.agent.broker, "yes");
    assert_eq!(codex.agent.provider_host_count, 3);
    assert_eq!(codex_plan.network.mode, "provider");
    assert_eq!(
        codex_plan.network.provider_allowed_hosts,
        ["api.openai.com", "chatgpt.com", "auth.openai.com"]
    );
    assert!(!codex_plan.confirm_required);

    let claude = preview(&payload, "claude");
    let claude_plan = plan(claude, "claude");
    assert_eq!(claude.agent.sign_in, "runhaven login");
    assert_eq!(claude.agent.broker, "yes");
    assert_eq!(claude_plan.network.mode, "provider");
    assert!(
        claude_plan
            .network
            .provider_allowed_hosts
            .contains(&"api.anthropic.com".to_string())
    );
}

#[test]
fn launch_preview_payload_surfaces_internet_confirmation_for_shell() {
    let workspace = tempfile::tempdir().expect("workspace");
    let payload = RunHavenTuiService::new().launch_preview_payload(workspace.path());

    let shell = preview(&payload, "shell");
    let shell_plan = plan(shell, "shell");

    assert_eq!(shell_plan.network.mode, "internet");
    assert!(shell_plan.confirm_required);
    assert!(shell_plan.safety_notes.iter().any(|note| {
        note.contains("Unrestricted internet egress") && note.contains("Use --network provider")
    }));
}

#[test]
fn launch_preview_payload_uses_agent_shared_state_volume() {
    let workspace = tempfile::tempdir().expect("workspace");
    let payload = RunHavenTuiService::new().launch_preview_payload(workspace.path());

    for name in ["codex", "shell"] {
        let preview = preview(&payload, name);
        let plan = plan(preview, name);

        assert_eq!(plan.auth_scope, "agent");
        assert_eq!(plan.state_volume, format!("runhaven-{name}-shared-home"));
        assert_eq!(
            plan.boundary.mounted_state_volume,
            format!("runhaven-{name}-shared-home -> /home/agent")
        );
    }
}

#[test]
fn launch_preview_payload_preserves_nested_git_workspace_note() {
    let repo = tempfile::tempdir().expect("repo");
    run_git(&["init", "-q"], repo.path());
    let nested = repo.path().join("nested");
    std::fs::create_dir(&nested).expect("nested workspace");

    let payload = RunHavenTuiService::new().launch_preview_payload(&nested);
    let codex = preview(&payload, "codex");
    let codex_plan = plan(codex, "codex");
    let note = codex_plan
        .workspace_scope_note
        .as_ref()
        .expect("workspace scope note");

    assert!(note.contains("selected workspace is inside git repository root"));
    assert!(note.contains("RunHaven mounts only the selected directory"));
}

#[test]
fn launch_workspace_choices_offer_current_and_git_root_for_nested_repo() {
    let repo = tempfile::tempdir().expect("repo");
    run_git(&["init", "-q"], repo.path());
    let nested = repo.path().join("nested");
    std::fs::create_dir(&nested).expect("nested workspace");

    let choices = RunHavenTuiService::new().launch_workspace_choices(&nested);

    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].label, "Current directory");
    assert_eq!(choices[0].payload.workspace, nested);
    assert_eq!(choices[1].label, "Git repository root");
    assert_eq!(
        choices[1].payload.workspace,
        repo.path().canonicalize().expect("canonical repo")
    );
    assert!(
        choices[1]
            .description
            .contains("Mount the full repository instead of only the nested folder")
    );

    let current_codex = launch(preview(&choices[0].payload, "codex"), "codex");
    let root_codex = launch(preview(&choices[1].payload, "codex"), "codex");
    assert_eq!(
        current_codex.executable.workspace,
        choices[0]
            .payload
            .workspace
            .canonicalize()
            .expect("canonical current workspace")
    );
    assert_eq!(
        root_codex.executable.workspace,
        choices[1]
            .payload
            .workspace
            .canonicalize()
            .expect("canonical root workspace")
    );
    assert_ne!(
        current_codex.executable.workspace,
        root_codex.executable.workspace
    );
}

#[test]
fn launch_preview_payload_keeps_plan_errors_per_agent() {
    let root = tempfile::tempdir().expect("root");
    let missing_workspace = root.path().join("missing-workspace");
    let payload = RunHavenTuiService::new().launch_preview_payload(&missing_workspace);

    assert_eq!(payload.workspace, missing_workspace);
    assert!(payload.previews.len() >= 5);
    assert!(payload.previews.iter().all(|preview| preview.plan.is_err()));
    let codex = preview(&payload, "codex");
    assert_eq!(
        codex.agent.description,
        "OpenAI Codex CLI with workspace-write sandboxing inside the container."
    );
    assert_eq!(codex.agent.sign_in, "runhaven login");
    assert_eq!(codex.agent.default_network, "provider");
    assert!(
        payload
            .previews
            .iter()
            .all(|preview| preview.plan.as_ref().err().is_some_and(|error| {
                error.reason() == "Plan could not be built."
                    && (error.detail().contains("could not resolve workspace path")
                        || error.detail().contains("workspace does not exist"))
            }))
    );
}

#[test]
fn active_runs_payload_omits_workspace_paths() {
    use runhaven_core::runtime::active::write_active_run_payload;
    use runhaven_core::support::paths::override_cache_root_for_tests;

    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    write_active_run_payload(
        "run-123",
        serde_json::json!({
            "timestamp": "2026-06-29T00:00:00Z",
            "run_id": "run-123",
            "profile": "codex",
            "workspace": "/Users/c/secret/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-project-run",
            "state_volume": "runhaven-codex-shared-home",
            "session": "none"
        }),
    )
    .expect("active marker");

    let payload = RunHavenTuiService::new().active_runs_payload();

    assert_eq!(payload.runs.len(), 1);
    assert_eq!(payload.runs[0].run_id, "run-123");
    let serialized = serde_json::to_string(&payload).expect("serialize");
    assert!(!serialized.contains("/Users/c/secret/project"));
}

#[test]
fn diagnostics_payload_maps_secret_free_log_metadata() {
    use runhaven_core::support::paths::auth_broker_log_path;
    use runhaven_core::support::paths::egress_policy_log_path;
    use runhaven_core::support::paths::ensure_private_parent;
    use runhaven_core::support::paths::override_cache_root_for_tests;
    use std::io::Write;

    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    ensure_private_parent(&egress_policy_log_path()).expect("egress parent");
    ensure_private_parent(&auth_broker_log_path()).expect("auth parent");
    {
        let mut file = std::fs::File::create(egress_policy_log_path()).expect("egress log file");
        writeln!(
            file,
            "{}",
            serde_json::json!({
                "timestamp": "2026-06-29T00:00:00Z",
                "profile": "codex",
                "decision": "denied",
                "host": "example.com",
                "port": 443,
                "count": 1,
                "reason": "not-in-allowlist",
                "matched_rule": "",
                "run_id": "run-123",
                "workspace": "/Users/c/secret/project"
            })
        )
        .expect("egress write");
    }
    {
        let mut file = std::fs::File::create(auth_broker_log_path()).expect("auth log file");
        writeln!(
            file,
            "{}",
            serde_json::json!({
                "timestamp": "2026-06-29T00:00:00Z",
                "profile": "codex",
                "broker": "api-key",
                "decision": "allowed",
                "method": "POST",
                "path": "/v1/responses?token=secret#fragment",
                "upstream_status": 200,
                "count": 1,
                "reason": "-",
                "run_id": "run-123",
                "authorization": "Bearer secret"
            })
        )
        .expect("auth write");
    }

    let payload = RunHavenTuiService::new()
        .diagnostics_payload(10)
        .expect("diagnostics");

    assert_eq!(payload.egress_log[0].host, "example.com");
    assert_eq!(payload.auth_log[0].path, "/v1/responses");
    assert_eq!(payload.auth_log[0].upstream_status, Some(200));
    let serialized = serde_json::to_string(&payload).expect("serialize");
    assert!(!serialized.contains("/Users/c/secret/project"));
    assert!(!serialized.contains("Bearer secret"));
    assert!(!serialized.contains("token=secret"));
}

#[tokio::test]
async fn log_snapshot_rejects_invalid_line_count_before_backend_lookup() {
    let error = RunHavenTuiService::new()
        .handle_request(ClientRequest::RunHavenRunLogSnapshot {
            request_id: 42,
            run_id: "not-a-real-run".to_string(),
            lines: 0,
            confirm_sensitive_output: true,
        })
        .await
        .expect_err("invalid line count should fail validation");

    match error {
        RunHavenServiceError::Validation { method, message } => {
            assert_eq!(method, "runhaven/run/logSnapshot");
            assert!(message.contains("between 1 and 500"));
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[tokio::test]
async fn log_snapshot_requires_sensitive_output_confirmation_before_backend_lookup() {
    let error = RunHavenTuiService::new()
        .handle_request(ClientRequest::RunHavenRunLogSnapshot {
            request_id: 42,
            run_id: "not-a-real-run".to_string(),
            lines: 100,
            confirm_sensitive_output: false,
        })
        .await
        .expect_err("missing confirmation should fail validation");

    match error {
        RunHavenServiceError::Validation { method, message } => {
            assert_eq!(method, "runhaven/run/logSnapshot");
            assert!(message.contains("Confirm raw log viewing"));
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}
