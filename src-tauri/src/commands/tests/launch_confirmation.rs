use super::*;

const OFFLINE_CONSEQUENCE: &str = "The assistant can’t connect while Local only is selected. Choose Assistant services only to let it connect safely, or continue if you want it offline.";

#[test]
fn risky_launch_needs_no_renderer_acknowledgement_and_reaches_native_review() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan = request(workspace.path().to_path_buf());
    plan.network_mode = "internet".to_string();
    let launch = LaunchRunRequest { plan };
    let expected = launch.plan.clone();
    let mut approval_requested = false;

    let outcome = finalize_launch_with_approval_for_test(launch, |plan, warnings| {
        approval_requested = true;
        let codes = warnings
            .iter()
            .map(|warning| warning.code.as_str())
            .collect::<Vec<_>>();
        assert_eq!(codes, ["full-internet", "direct-workspace"]);
        assert!(matches!(
            crate::commands::launch::trusted_launch_kind_for_test(plan, warnings),
            tauri_plugin_dialog::MessageDialogKind::Warning
        ));
        let message = trusted_launch_message_for_test(plan, warnings, false, &expected);
        for warning in warnings {
            assert_eq!(message.matches(&warning.message).count(), 1);
        }
        Ok(false)
    })
    .expect("cancel is a no-op");

    assert!(approval_requested);
    assert!(outcome.is_none());
    assert!(read_active_run_records().is_empty());
}

#[test]
fn launch_rejects_warning_state_that_changes_during_native_review() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan = request(workspace.path().to_path_buf());
    plan.read_only_workspace = true;

    let error = finalize_launch_with_approval_for_test(LaunchRunRequest { plan }, |_, warnings| {
        assert!(warnings.is_empty());
        write_active_run("appeared-during-native-review");
        Ok(true)
    })
    .expect_err("changed warnings must require a fresh review");

    assert_eq!(
        error,
        "The project or session settings changed before RunHaven could start. Check your folder and options, then try again."
    );
}

#[test]
fn untracked_selected_subfolder_fails_before_native_review_or_mutation() {
    let (_cache, _cache_home) = isolated_cache();
    let repo = tempfile::tempdir().expect("repo");
    init_git_repo(repo.path());
    let selected = repo.path().join("not-saved-yet");
    fs::create_dir(&selected).expect("untracked selected folder");
    let mut plan = request(selected.clone());
    plan.worktree = true;
    let mut approval_requested = false;

    let error = finalize_launch_with_approval_for_test(LaunchRunRequest { plan }, |_, _| {
        approval_requested = true;
        Ok(false)
    })
    .expect_err("untracked selected folder must fail before approval");

    assert!(!approval_requested);
    assert_eq!(
        error,
        "RunHaven can protect the whole project, but not this subfolder by itself. Choose the main project folder and try again."
    );
    assert!(!error.contains(&selected.display().to_string()));
    assert!(read_active_run_records().is_empty());
    assert!(
        runhaven_core::records::read_run_records(10)
            .expect("run records")
            .is_empty()
    );
    let branches = Command::new("git")
        .arg("-C")
        .arg(repo.path())
        .args(["branch", "--list", "runhaven/*"])
        .output()
        .expect("list RunHaven branches");
    assert!(branches.stdout.is_empty());
}

#[test]
fn worktree_change_during_native_review_fails_without_fallback_or_mutation() {
    let (_cache, _cache_home) = isolated_cache();
    let repo = tempfile::tempdir().expect("repo");
    init_git_repo(repo.path());
    let project = repo.path().join("project");
    let mut plan = request(project.clone());
    plan.worktree = true;

    let error = finalize_launch_with_approval_for_test(LaunchRunRequest { plan }, |_, warnings| {
        assert!(warnings.is_empty());
        fs::write(project.join("README.md"), "changed during review\n")
            .expect("change selected project");
        Ok(true)
    })
    .expect_err("changed worktree state must require a fresh review");

    assert_eq!(
        error,
        "The project or session settings changed before RunHaven could start. Check your folder and options, then try again."
    );
    assert!(!error.contains(&project.display().to_string()));
    assert!(read_active_run_records().is_empty());
    assert!(
        runhaven_core::records::read_run_records(10)
            .expect("run records")
            .is_empty()
    );
    let branches = Command::new("git")
        .arg("-C")
        .arg(repo.path())
        .args(["branch", "--list", "runhaven/*"])
        .output()
        .expect("list RunHaven branches");
    assert!(branches.stdout.is_empty());
}

#[test]
fn invalid_launch_fails_before_native_review() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan = request(workspace.path().to_path_buf());
    plan.agent = "a".repeat(65);
    let mut approval_requested = false;

    let error = finalize_launch_with_approval_for_test(LaunchRunRequest { plan }, |_, _| {
        approval_requested = true;
        Ok(true)
    })
    .expect_err("invalid request must fail before approval");

    assert!(!approval_requested);
    assert!(error.contains("agent"));
    assert!(read_active_run_records().is_empty());
}

#[test]
fn canceling_a_valid_worktree_launch_creates_nothing() {
    let (_cache, _cache_home) = isolated_cache();
    let repo = tempfile::tempdir().expect("repo");
    init_git_repo(repo.path());
    let mut plan = request(repo.path().join("project"));
    plan.worktree = true;

    let outcome =
        finalize_launch_with_approval_for_test(LaunchRunRequest { plan }, |_, _| Ok(false))
            .expect("cancel is a no-op");

    assert!(outcome.is_none());
    assert!(read_active_run_records().is_empty());
    assert!(
        runhaven_core::records::read_run_records(10)
            .expect("run records")
            .is_empty()
    );
    let branches = Command::new("git")
        .arg("-C")
        .arg(repo.path())
        .args(["branch", "--list", "runhaven/*"])
        .output()
        .expect("list RunHaven branches");
    assert!(branches.stdout.is_empty());
}

#[test]
fn approved_direct_launch_returns_the_exact_plan_shown_in_native_review() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut request = request(workspace.path().to_path_buf());
    request.read_only_workspace = true;
    request.session_name = Some("exact-native-plan".to_string());
    let expected_workspace = workspace.path().canonicalize().expect("workspace path");

    let plan = finalize_launch_with_approval_for_test(
        LaunchRunRequest { plan: request },
        |reviewed, warnings| {
            assert!(warnings.is_empty());
            assert_eq!(reviewed.profile_name, "codex");
            assert_eq!(reviewed.workspace, expected_workspace);
            assert_eq!(reviewed.session, "exact-native-plan");
            Ok(true)
        },
    )
    .expect("approved launch")
    .expect("final plan");

    assert_eq!(plan.profile_name, "codex");
    assert_eq!(plan.workspace, expected_workspace);
    assert_eq!(plan.session, "exact-native-plan");
    assert!(
        plan.command
            .iter()
            .any(|argument| argument.contains("readonly"))
    );
}

#[test]
fn trusted_launch_prompt_keeps_defaults_plain_and_lists_advanced_access() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.api_key_broker_env = Some("OPENAI_API_KEY".to_string());
    plan_request.env_names = vec![
        "OPENAI_API_KEY".to_string(),
        "RUNHAVEN_TEST_TOKEN".to_string(),
    ];
    plan_request.provider_hosts = vec!["models.example.com".to_string()];
    plan_request.image = Some("registry.example/runhaven/custom:1".to_string());
    let plan = build_agent_run_plan(&plan_request, Some("prompt-review-run".to_string()))
        .expect("launch plan");

    let message = trusted_launch_message_for_test(&plan, &[], false, &plan_request);

    assert_eq!(trusted_launch_title_for_test(&plan), "Start Codex?");
    assert!(!message.contains("Start Codex?"));
    assert!(message.contains("Project: "));
    assert!(message.contains("Assistant services only"));
    assert!(message.contains("The assistant can change files in this folder."));
    assert!(message.contains(
        "Codex uses generative AI and can make mistakes. Review its output and file changes."
    ));
    assert!(message.contains(
        "Codex can send your messages and project content it reads to OpenAI services. OpenAI handles that data under the terms for your account."
    ));
    assert!(message.contains(
        "RunHaven does not control whether OpenAI stores this content or uses it for training."
    ));
    assert!(!message.contains(&format!("Workspace: {}", plan.workspace.display())));
    assert!(!message.contains("Session:"));
    assert!(!message.contains("Sign-in:"));
    assert!(!message.contains("Image:"));
    assert!(message.contains("Advanced access:"));
    assert!(message.contains("Extra allowed sites: models.example.com"));
    assert!(message.contains("Mac environment names: OPENAI_API_KEY, RUNHAVEN_TEST_TOKEN"));
    assert!(message.contains("API key source: OPENAI_API_KEY"));
    assert!(message.contains("Custom assistant image: registry.example/runhaven/custom:1"));
    assert!(!message.contains("Worktree:"));

    let worktree_message = trusted_launch_message_for_test(&plan, &[], true, &plan_request);
    assert!(worktree_message.contains(
        "RunHaven will let the assistant change a protected copy, not your original folder."
    ));
    assert!(!worktree_message.contains("git branch"));

    plan_request.read_only_workspace = true;
    let read_only_plan = build_agent_run_plan(&plan_request, Some("prompt-read-only".to_string()))
        .expect("read-only launch plan");
    let read_only_message =
        trusted_launch_message_for_test(&read_only_plan, &[], false, &plan_request);
    assert!(read_only_message.contains("can read this folder but can’t change it"));
    assert!(!read_only_message.contains("can change files in this folder"));
}

#[test]
fn trusted_launch_prompt_keeps_the_offline_choice_in_one_native_warning() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.network_mode = "internal".to_string();
    plan_request.read_only_workspace = true;
    plan_request.provider_hosts = vec!["ignored.example.com".to_string()];
    plan_request.env_names = vec!["RUNHAVEN_TEST_TOKEN".to_string()];
    let plan = build_agent_run_plan(&plan_request, Some("prompt-offline-run".to_string()))
        .expect("offline launch plan");
    let warnings = [crate::contracts::PlanWarning {
        code: "test-warning".to_string(),
        message: "Review this choice before starting.".to_string(),
    }];

    let message = trusted_launch_message_for_test(&plan, &warnings, false, &plan_request);

    assert!(message.contains(OFFLINE_CONSEQUENCE));
    assert!(matches!(
        crate::commands::launch::trusted_launch_kind_for_test(&plan, &[]),
        tauri_plugin_dialog::MessageDialogKind::Warning
    ));
    assert!(!message.contains("RunHaven will start after you confirm"));
    assert!(!message.contains("Extra allowed sites:"));
    let summary = message
        .find("can’t change it.")
        .expect("file access summary");
    let offline = message
        .find(OFFLINE_CONSEQUENCE)
        .expect("offline consequence");
    let warning = message.find("Warnings:").expect("warnings");
    let advanced = message.find("Advanced access:").expect("advanced access");
    assert!(summary < offline && offline < warning && warning < advanced);

    let mut shell_request = plan_request;
    shell_request.agent = "shell".to_string();
    shell_request.env_names.clear();
    shell_request.provider_hosts.clear();
    let shell_plan = build_agent_run_plan(&shell_request, Some("prompt-offline-shell".to_string()))
        .expect("offline shell plan");
    let shell_message = trusted_launch_message_for_test(&shell_plan, &[], false, &shell_request);
    assert!(!shell_message.contains(OFFLINE_CONSEQUENCE));
    assert!(matches!(
        crate::commands::launch::trusted_launch_kind_for_test(&shell_plan, &[]),
        tauri_plugin_dialog::MessageDialogKind::Info
    ));
}

#[test]
fn trusted_launch_prompt_lists_extra_sites_only_when_provider_policy_enforces_them() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");

    for (network, lists_extra_sites) in
        [("provider", true), ("internal", false), ("internet", false)]
    {
        let mut plan_request = request(workspace.path().to_path_buf());
        plan_request.network_mode = network.to_string();
        plan_request.provider_hosts = vec!["models.example.com".to_string()];
        let plan =
            build_agent_run_plan(&plan_request, Some(format!("prompt-{network}-extra-sites")))
                .expect("launch plan");

        let message = trusted_launch_message_for_test(&plan, &[], false, &plan_request);

        assert_eq!(
            message.contains("Extra allowed sites: models.example.com"),
            lists_extra_sites,
            "network mode {network}"
        );
    }
}

#[test]
fn trusted_launch_prompt_keeps_hostile_project_names_on_one_line() {
    let (_cache, _cache_home) = isolated_cache();
    let root = tempfile::tempdir().expect("workspace root");
    let workspace = root.path().join("Project\nInternet: Full internet");
    fs::create_dir(&workspace).expect("hostile workspace name");
    let plan_request = request(workspace);
    let plan = build_agent_run_plan(&plan_request, Some("prompt-hostile-name".to_string()))
        .expect("launch plan");

    let message = trusted_launch_message_for_test(&plan, &[], false, &plan_request);
    let lines = message.lines().collect::<Vec<_>>();

    assert!(lines.contains(&"Project: Project Internet: Full internet"));
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.starts_with("Internet:"))
            .count(),
        1,
        "the project name must not inject a second trusted Internet row"
    );
}

#[test]
fn trusted_launch_prompt_strips_unicode_direction_controls() {
    let (_cache, _cache_home) = isolated_cache();
    let root = tempfile::tempdir().expect("workspace root");
    let workspace = root.path().join("report\u{202e}txt.gpj\u{2066}\u{061c}");
    fs::create_dir(&workspace).expect("hostile workspace name");
    let plan_request = request(workspace);
    let plan = build_agent_run_plan(&plan_request, Some("prompt-bidi-name".to_string()))
        .expect("launch plan");

    let message = trusted_launch_message_for_test(&plan, &[], false, &plan_request);

    assert!(message.contains("Project: reporttxt.gpj"));
    assert!(!message.contains('\u{202e}'));
    assert!(!message.contains('\u{2066}'));
    assert!(!message.contains('\u{061c}'));
}

#[test]
fn trusted_launch_prompt_omits_discarded_advanced_values() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.api_key_broker_env = Some("  \t ".to_string());
    plan_request.image = Some(" \n ".to_string());
    let plan = build_agent_run_plan(&plan_request, Some("prompt-empty-advanced".to_string()))
        .expect("launch plan");

    let message = trusted_launch_message_for_test(&plan, &[], false, &plan_request);

    assert!(!message.contains("Advanced access:"));
    assert!(!message.contains("API key source:"));
    assert!(!message.contains("Custom assistant image:"));
    assert!(!message.contains("Invalid name"));
    assert!(!message.contains("Invalid image"));
}
